use core_graphics::display::CGRect;

use super::AXUIElementRef;

/// Represents the bounds (position and size) of a window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    /// Create a new Bounds
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create bounds from CoreGraphics CGRect
    pub(super) fn from_cg_rect(rect: CGRect) -> Self {
        Self {
            x: rect.origin.x as i32,
            y: rect.origin.y as i32,
            width: rect.size.width as u32,
            height: rect.size.height as u32,
        }
    }
}

/// Information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Unique window identifier (process ID << 16 | window index)
    pub id: u32,
    /// Application name
    pub app: String,
    /// Window title
    pub title: String,
    /// Window position and size
    pub bounds: Bounds,
    /// Process ID of the owning application
    pub pid: i32,
    /// The AXUIElement reference (internal, for operations)
    #[doc(hidden)]
    ax_window: Option<usize>, // Store as usize to avoid lifetime issues
}

impl WindowInfo {
    pub(super) fn new(
        id: u32,
        app: String,
        title: String,
        bounds: Bounds,
        pid: i32,
        ax_window: Option<usize>,
    ) -> Self {
        Self {
            id,
            app,
            title,
            bounds,
            pid,
            ax_window,
        }
    }

    /// Get the internal window reference for operations
    fn window_ref(&self) -> Option<AXUIElementRef> {
        self.ax_window.map(|ptr| ptr as AXUIElementRef)
    }
}

/// Tiling positions for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePosition {
    // Half positions
    /// Left half of the screen
    LeftHalf,
    /// Right half of the screen
    RightHalf,
    /// Top half of the screen
    TopHalf,
    /// Bottom half of the screen
    BottomHalf,

    // Quadrant positions
    /// Top-left quadrant
    TopLeft,
    /// Top-right quadrant
    TopRight,
    /// Bottom-left quadrant
    BottomLeft,
    /// Bottom-right quadrant
    BottomRight,

    // Sixth positions (top/bottom row split into thirds)
    /// Top-left sixth (left third of top half)
    TopLeftSixth,
    /// Top-center sixth (center third of top half)
    TopCenterSixth,
    /// Top-right sixth (right third of top half)
    TopRightSixth,
    /// Bottom-left sixth (left third of bottom half)
    BottomLeftSixth,
    /// Bottom-center sixth (center third of bottom half)
    BottomCenterSixth,
    /// Bottom-right sixth (right third of bottom half)
    BottomRightSixth,

    // Horizontal thirds positions
    /// Left third of the screen
    LeftThird,
    /// Center third of the screen (horizontal)
    CenterThird,
    /// Right third of the screen
    RightThird,

    // Vertical thirds positions
    /// Top third of the screen
    TopThird,
    /// Middle third of the screen (vertical)
    MiddleThird,
    /// Bottom third of the screen
    BottomThird,

    // Horizontal two-thirds positions
    /// First two-thirds of the screen (left side)
    FirstTwoThirds,
    /// Last two-thirds of the screen (right side)
    LastTwoThirds,

    // Vertical two-thirds positions
    /// Top two-thirds of the screen
    TopTwoThirds,
    /// Bottom two-thirds of the screen
    BottomTwoThirds,

    // Centered positions
    /// Centered on screen (60% of screen dimensions)
    Center,
    /// Almost maximize (90% with margins)
    AlmostMaximize,

    /// Fullscreen (covers entire display)
    Fullscreen,
    /// Move to the next display (multi-display routing handled elsewhere)
    NextDisplay,
    /// Move to the previous display (multi-display routing handled elsewhere)
    PreviousDisplay,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_new() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 20);
        assert_eq!(bounds.width, 100);
        assert_eq!(bounds.height, 200);
    }

    #[test]
    fn test_tile_position_equality() {
        assert_eq!(TilePosition::LeftHalf, TilePosition::LeftHalf);
        assert_ne!(TilePosition::LeftHalf, TilePosition::RightHalf);
    }
}
