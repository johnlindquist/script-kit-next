use std::cmp::Ordering;

use super::tiling::calculate_tile_bounds;
use super::types::{Bounds, TilePosition};

/// A candidate snap position derived from existing tiling geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapTarget {
    pub tile: TilePosition,
    pub bounds: Bounds,
}

/// The result of matching a dragged window against snap targets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapMatch {
    pub target: SnapTarget,
    pub overlap_ratio: f64,
}

/// Tile positions suitable for snap previews.
/// Excludes Fullscreen/NextDisplay/PreviousDisplay (routing-only positions).
const DEFAULT_SNAP_TILES: &[TilePosition] = &[
    TilePosition::LeftHalf,
    TilePosition::RightHalf,
    TilePosition::TopHalf,
    TilePosition::BottomHalf,
    TilePosition::TopLeft,
    TilePosition::TopRight,
    TilePosition::BottomLeft,
    TilePosition::BottomRight,
    TilePosition::LeftThird,
    TilePosition::CenterThird,
    TilePosition::RightThird,
    TilePosition::FirstTwoThirds,
    TilePosition::LastTwoThirds,
    TilePosition::Center,
    TilePosition::AlmostMaximize,
];

/// Build snap targets for a display by reusing existing tiling math.
pub fn build_snap_targets(screen: &Bounds) -> Vec<SnapTarget> {
    let targets: Vec<SnapTarget> = DEFAULT_SNAP_TILES
        .iter()
        .copied()
        .map(|tile| SnapTarget {
            tile,
            bounds: calculate_tile_bounds(screen, tile),
        })
        .collect();

    tracing::info!(
        target: "script_kit::snap",
        screen_x = screen.x,
        screen_y = screen.y,
        screen_w = screen.width,
        screen_h = screen.height,
        target_count = targets.len(),
        "built snap targets"
    );

    targets
}

/// Compute the intersection area of two bounds rectangles.
fn intersection_area(a: &Bounds, b: &Bounds) -> u64 {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width as i32).min(b.x + b.width as i32);
    let bottom = (a.y + a.height as i32).min(b.y + b.height as i32);

    if right <= left || bottom <= top {
        return 0;
    }

    ((right - left) as u64) * ((bottom - top) as u64)
}

/// Intersection-over-union (Jaccard index) of two bounds rectangles.
/// Returns 0.0 when there is no overlap, 1.0 for identical bounds.
fn overlap_ratio(a: &Bounds, b: &Bounds) -> f64 {
    let intersection = intersection_area(a, b) as f64;
    if intersection == 0.0 {
        return 0.0;
    }
    let a_area = a.width as f64 * a.height as f64;
    let b_area = b.width as f64 * b.height as f64;
    let union = a_area + b_area - intersection;
    if union == 0.0 {
        return 0.0;
    }
    intersection / union
}

/// Find the best snap target for a dragged window, or `None` if no target
/// exceeds `min_overlap_ratio`.
pub fn best_snap_match(
    dragged_window: &Bounds,
    targets: &[SnapTarget],
    min_overlap_ratio: f64,
) -> Option<SnapMatch> {
    let best = targets
        .iter()
        .copied()
        .map(|target| SnapMatch {
            overlap_ratio: overlap_ratio(dragged_window, &target.bounds),
            target,
        })
        .filter(|candidate| candidate.overlap_ratio >= min_overlap_ratio)
        .max_by(|a, b| {
            a.overlap_ratio
                .partial_cmp(&b.overlap_ratio)
                .unwrap_or(Ordering::Equal)
        });

    tracing::info!(
        target: "script_kit::snap",
        window_x = dragged_window.x,
        window_y = dragged_window.y,
        window_width = dragged_window.width,
        window_height = dragged_window.height,
        min_overlap_ratio,
        matched = best.is_some(),
        best_tile = best.map(|m| format!("{:?}", m.target.tile)),
        best_overlap_ratio = best.map(|m| m.overlap_ratio),
        "evaluated snap match"
    );

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_snap_targets_returns_all_default_tiles() {
        let screen = Bounds::new(0, 0, 1440, 900);
        let targets = build_snap_targets(&screen);
        assert_eq!(targets.len(), DEFAULT_SNAP_TILES.len());
    }

    #[test]
    fn best_match_prefers_left_half_when_window_mostly_overlaps_left() {
        let screen = Bounds::new(0, 0, 1440, 900);
        let targets = build_snap_targets(&screen);
        let dragged = Bounds::new(10, 20, 760, 860);
        let best = best_snap_match(&dragged, &targets, 0.20).expect("expected a snap match");
        assert_eq!(best.target.tile, TilePosition::LeftHalf);
    }

    #[test]
    fn no_match_when_overlap_is_too_small() {
        let screen = Bounds::new(0, 0, 1440, 900);
        let targets = build_snap_targets(&screen);
        let dragged = Bounds::new(600, 400, 80, 80);
        let best = best_snap_match(&dragged, &targets, 0.50);
        assert!(best.is_none());
    }

    #[test]
    fn best_match_with_display_offset() {
        let screen = Bounds::new(1440, 24, 1440, 876);
        let targets = build_snap_targets(&screen);
        let dragged = Bounds::new(1440, 30, 720, 870);
        let best = best_snap_match(&dragged, &targets, 0.20).expect("expected a snap match");
        assert_eq!(best.target.tile, TilePosition::LeftHalf);
    }

    #[test]
    fn intersection_area_non_overlapping_is_zero() {
        let a = Bounds::new(0, 0, 100, 100);
        let b = Bounds::new(200, 200, 100, 100);
        assert_eq!(intersection_area(&a, &b), 0);
    }

    #[test]
    fn intersection_area_partial_overlap() {
        let a = Bounds::new(0, 0, 100, 100);
        let b = Bounds::new(50, 50, 100, 100);
        assert_eq!(intersection_area(&a, &b), 2500);
    }

    #[test]
    fn overlap_ratio_identical_bounds_is_one() {
        let a = Bounds::new(0, 0, 100, 100);
        let ratio = overlap_ratio(&a, &a);
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }
}
