//! Window placements that depend on where the window already is, plus the
//! extra fixed tiles (fourths, three-fourths, sixths).
//!
//! Every adjustment resolves the window's current frame and its display's
//! visible frame, computes new bounds with [`adjusted_bounds`], and applies
//! them through [`super::actions::set_window_bounds`], so it is verified and
//! can be undone like any other window change.

use anyhow::Result;

use super::types::Bounds;

/// Which screen edge to move a window against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEdge {
    Left,
    Right,
    Top,
    Bottom,
}

/// A placement computed from the display and, for some, the window itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAdjustment {
    /// One of four equal columns, 0 = leftmost.
    Fourth(u8),
    /// The left (`first`) or right three quarters.
    ThreeFourths { first: bool },
    /// One of six cells: three columns by two rows. `column` 0..3, `top`.
    Sixth { column: u8, top: bool },
    /// Full display height, keeping x and width.
    MaximizeHeight,
    /// Full display width, keeping y and height.
    MaximizeWidth,
    /// Against an edge, keeping the size and the other axis.
    MoveToEdge(WindowEdge),
    /// A comfortable centered size: 60% of the display, at most 1024×900.
    ReasonableSize,
    /// Grow by [`STEP`] on every side, within the display.
    MakeLarger,
    /// Shrink by [`STEP`] on every side, no smaller than a quarter of the display.
    MakeSmaller,
}

/// How much Make Larger / Make Smaller change each side, in points.
pub const STEP: i32 = 32;

fn clamp_into(frame: Bounds, mut bounds: Bounds) -> Bounds {
    bounds.width = bounds.width.min(frame.width).max(1);
    bounds.height = bounds.height.min(frame.height).max(1);
    let max_x = frame.x + (frame.width - bounds.width) as i32;
    let max_y = frame.y + (frame.height - bounds.height) as i32;
    bounds.x = bounds.x.clamp(frame.x, max_x);
    bounds.y = bounds.y.clamp(frame.y, max_y);
    bounds
}

/// Column `index` of `count` equal columns, with exact edges.
fn column(frame: Bounds, index: u32, span: u32, count: u32) -> (i32, u32) {
    let edge =
        |i: u32| frame.x + ((u64::from(frame.width) * u64::from(i)) / u64::from(count)) as i32;
    let left = edge(index);
    let right = edge(index + span);
    (left, (right - left) as u32)
}

fn row(frame: Bounds, index: u32, count: u32) -> (i32, u32) {
    let edge =
        |i: u32| frame.y + ((u64::from(frame.height) * u64::from(i)) / u64::from(count)) as i32;
    let top = edge(index);
    (top, (edge(index + 1) - top) as u32)
}

/// Where `adjustment` puts a window at `current` on a display whose visible
/// frame is `frame`.
pub fn adjusted_bounds(frame: Bounds, current: Bounds, adjustment: WindowAdjustment) -> Bounds {
    match adjustment {
        WindowAdjustment::Fourth(index) => {
            let (x, width) = column(frame, u32::from(index.min(3)), 1, 4);
            Bounds::new(x, frame.y, width, frame.height)
        }
        WindowAdjustment::ThreeFourths { first } => {
            let (x, width) = column(frame, if first { 0 } else { 1 }, 3, 4);
            Bounds::new(x, frame.y, width, frame.height)
        }
        WindowAdjustment::Sixth { column: index, top } => {
            let (x, width) = column(frame, u32::from(index.min(2)), 1, 3);
            let (y, height) = row(frame, if top { 0 } else { 1 }, 2);
            Bounds::new(x, y, width, height)
        }
        WindowAdjustment::MaximizeHeight => clamp_into(
            frame,
            Bounds::new(current.x, frame.y, current.width, frame.height),
        ),
        WindowAdjustment::MaximizeWidth => clamp_into(
            frame,
            Bounds::new(frame.x, current.y, frame.width, current.height),
        ),
        WindowAdjustment::MoveToEdge(edge) => {
            let sized = clamp_into(frame, current);
            let right = frame.x + (frame.width - sized.width) as i32;
            let bottom = frame.y + (frame.height - sized.height) as i32;
            match edge {
                WindowEdge::Left => Bounds::new(frame.x, sized.y, sized.width, sized.height),
                WindowEdge::Right => Bounds::new(right, sized.y, sized.width, sized.height),
                WindowEdge::Top => Bounds::new(sized.x, frame.y, sized.width, sized.height),
                WindowEdge::Bottom => Bounds::new(sized.x, bottom, sized.width, sized.height),
            }
        }
        WindowAdjustment::ReasonableSize => {
            let width = ((frame.width * 6) / 10).min(1024);
            let height = ((frame.height * 6) / 10).min(900);
            let x = frame.x + ((frame.width - width) / 2) as i32;
            let y = frame.y + ((frame.height - height) / 2) as i32;
            Bounds::new(x, y, width, height)
        }
        WindowAdjustment::MakeLarger => clamp_into(
            frame,
            Bounds::new(
                current.x - STEP,
                current.y - STEP,
                current.width.saturating_add(2 * STEP as u32),
                current.height.saturating_add(2 * STEP as u32),
            ),
        ),
        WindowAdjustment::MakeSmaller => {
            let min_width = (frame.width / 4).max(1);
            let min_height = (frame.height / 4).max(1);
            let width = current.width.saturating_sub(2 * STEP as u32).max(min_width);
            let height = current
                .height
                .saturating_sub(2 * STEP as u32)
                .max(min_height);
            // Shrink around the window's center.
            let x = current.x + ((current.width.saturating_sub(width)) / 2) as i32;
            let y = current.y + ((current.height.saturating_sub(height)) / 2) as i32;
            clamp_into(frame, Bounds::new(x, y, width, height))
        }
    }
}

/// Apply `adjustment` to the window with this id.
pub fn adjust_window(window_id: u32, adjustment: WindowAdjustment) -> Result<()> {
    let (observation, _window) = super::actions::resolve_action_target(window_id)?;
    let current = observation.bounds;
    let frame = super::display::get_visible_display_bounds(current.x, current.y);
    let bounds = adjusted_bounds(frame, current, adjustment);
    tracing::info!(window_id, ?adjustment, ?bounds, "Adjusting window");
    super::actions::set_window_bounds(window_id, bounds)
}

/// How a protocol tile position is carried out: a fixed tile, or a
/// placement computed from the window's current frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolPlacement {
    Tile(super::types::TilePosition),
    Adjust(WindowAdjustment),
}

pub fn protocol_placement(pos: &crate::protocol::TilePosition) -> ProtocolPlacement {
    use super::types::TilePosition as WC;
    use crate::protocol::TilePosition as P;
    use ProtocolPlacement::{Adjust, Tile};
    use {WindowAdjustment as A, WindowEdge as E};
    match pos {
        P::Left => Tile(WC::LeftHalf),
        P::Right => Tile(WC::RightHalf),
        P::Top => Tile(WC::TopHalf),
        P::Bottom => Tile(WC::BottomHalf),
        P::TopLeft => Tile(WC::TopLeft),
        P::TopRight => Tile(WC::TopRight),
        P::BottomLeft => Tile(WC::BottomLeft),
        P::BottomRight => Tile(WC::BottomRight),
        P::LeftThird => Tile(WC::LeftThird),
        P::CenterThird => Tile(WC::CenterThird),
        P::RightThird => Tile(WC::RightThird),
        P::TopThird => Tile(WC::TopThird),
        P::MiddleThird => Tile(WC::MiddleThird),
        P::BottomThird => Tile(WC::BottomThird),
        P::FirstTwoThirds => Tile(WC::FirstTwoThirds),
        P::LastTwoThirds => Tile(WC::LastTwoThirds),
        P::TopTwoThirds => Tile(WC::TopTwoThirds),
        P::BottomTwoThirds => Tile(WC::BottomTwoThirds),
        P::Center => Tile(WC::Center),
        P::AlmostMaximize => Tile(WC::AlmostMaximize),
        // Maximize fills the entire visible screen area (fullscreen)
        P::Maximize => Tile(WC::Fullscreen),
        P::TopLeftSixth => Tile(WC::TopLeftSixth),
        P::TopCenterSixth => Tile(WC::TopCenterSixth),
        P::TopRightSixth => Tile(WC::TopRightSixth),
        P::BottomLeftSixth => Tile(WC::BottomLeftSixth),
        P::BottomCenterSixth => Tile(WC::BottomCenterSixth),
        P::BottomRightSixth => Tile(WC::BottomRightSixth),
        P::LeftFourth => Adjust(A::Fourth(0)),
        P::CenterLeftFourth => Adjust(A::Fourth(1)),
        P::CenterRightFourth => Adjust(A::Fourth(2)),
        P::RightFourth => Adjust(A::Fourth(3)),
        P::FirstThreeFourths => Adjust(A::ThreeFourths { first: true }),
        P::LastThreeFourths => Adjust(A::ThreeFourths { first: false }),
        P::MaximizeHeight => Adjust(A::MaximizeHeight),
        P::MaximizeWidth => Adjust(A::MaximizeWidth),
        P::MoveLeft => Adjust(A::MoveToEdge(E::Left)),
        P::MoveRight => Adjust(A::MoveToEdge(E::Right)),
        P::MoveUp => Adjust(A::MoveToEdge(E::Top)),
        P::MoveDown => Adjust(A::MoveToEdge(E::Bottom)),
        P::ReasonableSize => Adjust(A::ReasonableSize),
        P::MakeLarger => Adjust(A::MakeLarger),
        P::MakeSmaller => Adjust(A::MakeSmaller),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Bounds = Bounds {
        x: 0,
        y: 25,
        width: 1441,
        height: 875,
    };

    #[test]
    fn fourths_and_three_fourths_tile_the_display_exactly() {
        let fourths: Vec<Bounds> = (0..4)
            .map(|i| adjusted_bounds(FRAME, FRAME, WindowAdjustment::Fourth(i)))
            .collect();
        assert_eq!(fourths[0].x, 0);
        for pair in fourths.windows(2) {
            assert_eq!(
                pair[0].x + pair[0].width as i32,
                pair[1].x,
                "no gaps or overlaps"
            );
        }
        let last = fourths[3];
        assert_eq!(last.x + last.width as i32, 1441);
        let first_three =
            adjusted_bounds(FRAME, FRAME, WindowAdjustment::ThreeFourths { first: true });
        assert_eq!(first_three.x + first_three.width as i32, fourths[3].x);
        let last_three = adjusted_bounds(
            FRAME,
            FRAME,
            WindowAdjustment::ThreeFourths { first: false },
        );
        assert_eq!(last_three.x, fourths[1].x);
        assert_eq!(last_three.x + last_three.width as i32, 1441);
        let bottom_right = adjusted_bounds(
            FRAME,
            FRAME,
            WindowAdjustment::Sixth {
                column: 2,
                top: false,
            },
        );
        assert_eq!(bottom_right.x + bottom_right.width as i32, 1441);
        assert_eq!(bottom_right.y + bottom_right.height as i32, 900);
    }

    #[test]
    fn edge_moves_keep_the_size_and_stay_on_screen() {
        let window = Bounds::new(300, 200, 500, 400);
        let left = adjusted_bounds(
            FRAME,
            window,
            WindowAdjustment::MoveToEdge(WindowEdge::Left),
        );
        assert_eq!(left, Bounds::new(0, 200, 500, 400));
        let right = adjusted_bounds(
            FRAME,
            window,
            WindowAdjustment::MoveToEdge(WindowEdge::Right),
        );
        assert_eq!(right, Bounds::new(941, 200, 500, 400));
        let bottom = adjusted_bounds(
            FRAME,
            window,
            WindowAdjustment::MoveToEdge(WindowEdge::Bottom),
        );
        assert_eq!(bottom, Bounds::new(300, 500, 500, 400));
        // A window larger than the display is shrunk to fit.
        let huge = Bounds::new(-50, 0, 3000, 2000);
        let top = adjusted_bounds(FRAME, huge, WindowAdjustment::MoveToEdge(WindowEdge::Top));
        assert_eq!(top, FRAME);
    }

    #[test]
    fn sizes_grow_shrink_and_maximize_within_limits() {
        let window = Bounds::new(300, 200, 500, 400);
        assert_eq!(
            adjusted_bounds(FRAME, window, WindowAdjustment::MakeLarger),
            Bounds::new(268, 168, 564, 464)
        );
        assert_eq!(
            adjusted_bounds(FRAME, window, WindowAdjustment::MakeSmaller),
            Bounds::new(332, 232, 436, 336)
        );
        // Make Larger never leaves the display; Make Smaller stops at a quarter.
        assert_eq!(
            adjusted_bounds(FRAME, FRAME, WindowAdjustment::MakeLarger),
            FRAME
        );
        let tiny = Bounds::new(100, 100, 300, 200);
        let smaller = adjusted_bounds(FRAME, tiny, WindowAdjustment::MakeSmaller);
        assert_eq!((smaller.width, smaller.height), (360, 218));
        assert_eq!(
            adjusted_bounds(FRAME, window, WindowAdjustment::MaximizeHeight),
            Bounds::new(300, 25, 500, 875)
        );
        assert_eq!(
            adjusted_bounds(FRAME, window, WindowAdjustment::MaximizeWidth),
            Bounds::new(0, 200, 1441, 400)
        );
        let reasonable = adjusted_bounds(FRAME, window, WindowAdjustment::ReasonableSize);
        assert_eq!((reasonable.width, reasonable.height), (864, 525));
        assert_eq!(reasonable.x, 288);
    }

    #[test]
    fn new_positions_parse_from_their_wire_names_and_route_correctly() {
        use ProtocolPlacement::{Adjust, Tile};
        use {WindowAdjustment as A, WindowEdge as E};
        for (wire, expected) in [
            ("left-fourth", Adjust(A::Fourth(0))),
            ("right-fourth", Adjust(A::Fourth(3))),
            (
                "last-three-fourths",
                Adjust(A::ThreeFourths { first: false }),
            ),
            (
                "top-right-sixth",
                Tile(super::super::types::TilePosition::TopRightSixth),
            ),
            ("maximize-height", Adjust(A::MaximizeHeight)),
            ("move-up", Adjust(A::MoveToEdge(E::Top))),
            ("reasonable-size", Adjust(A::ReasonableSize)),
            ("make-smaller", Adjust(A::MakeSmaller)),
        ] {
            let position: crate::protocol::TilePosition =
                serde_json::from_value(serde_json::json!(wire)).expect(wire);
            assert_eq!(protocol_placement(&position), expected, "{wire}");
        }
        let restore: crate::protocol::WindowActionType =
            serde_json::from_value(serde_json::json!("restore")).expect("restore");
        assert_eq!(restore, crate::protocol::WindowActionType::Restore);
    }
}
