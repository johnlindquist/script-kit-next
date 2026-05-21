use core_graphics::display::CGRect;
use std::path::PathBuf;

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
    /// Bundle identifier of the owning application, when available.
    pub bundle_id: Option<String>,
    /// Path to the owning application bundle, when available.
    pub app_path: Option<PathBuf>,
    /// Index of the owning app in the current NSWorkspace enumeration.
    pub app_order: usize,
    /// Index of this window in the owning app's AXWindows list.
    pub window_index: usize,
    /// Monotonic order assigned during the current list_windows() enumeration.
    pub global_order: usize,
    /// True when the owning app is the current frontmost app.
    pub is_frontmost_app: bool,
    /// True when this window matches the app's AXFocusedWindow.
    pub is_focused: bool,
    /// True when this window matches the app's AXMainWindow.
    pub is_main: bool,
    /// True when AXMinimized reports the window is minimized.
    pub is_minimized: bool,
    /// True when CoreGraphics reports the window is visible in the current Space.
    pub is_on_current_space: bool,
    /// Precomputed native descriptor for list rows and receipts.
    pub descriptor: String,
    /// The AXUIElement reference (internal, for operations)
    #[doc(hidden)]
    ax_window: Option<usize>, // Store as usize to avoid lifetime issues
}

pub(super) struct WindowInfoInit {
    pub id: u32,
    pub app: String,
    pub title: String,
    pub bounds: Bounds,
    pub pid: i32,
    pub bundle_id: Option<String>,
    pub app_path: Option<PathBuf>,
    pub app_order: usize,
    pub window_index: usize,
    pub global_order: usize,
    pub is_frontmost_app: bool,
    pub is_focused: bool,
    pub is_main: bool,
    pub is_minimized: bool,
    pub is_on_current_space: bool,
    pub ax_window: Option<usize>,
}

/// Provider state for root unified window search.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RootWindowsProviderStatus {
    #[default]
    Unknown,
    Refreshing {
        count: usize,
    },
    Ready {
        count: usize,
    },
    PermissionRequired,
    ProviderError {
        message: String,
    },
}

impl WindowInfo {
    pub(super) fn new(init: WindowInfoInit) -> Self {
        let descriptor = build_window_descriptor(
            &init.app,
            init.pid,
            init.bounds,
            init.is_frontmost_app,
            init.is_focused,
            init.is_main,
            init.is_minimized,
            init.is_on_current_space,
            None,
        );
        Self {
            id: init.id,
            app: init.app,
            title: init.title,
            bounds: init.bounds,
            pid: init.pid,
            bundle_id: init.bundle_id,
            app_path: init.app_path,
            app_order: init.app_order,
            window_index: init.window_index,
            global_order: init.global_order,
            is_frontmost_app: init.is_frontmost_app,
            is_focused: init.is_focused,
            is_main: init.is_main,
            is_minimized: init.is_minimized,
            is_on_current_space: init.is_on_current_space,
            descriptor,
            ax_window: init.ax_window,
        }
    }

    /// Create a WindowInfo without an AX reference (e.g. for testing).
    #[doc(hidden)]
    pub fn for_test(id: u32, app: String, title: String, bounds: Bounds, pid: i32) -> Self {
        Self::new(WindowInfoInit {
            id,
            app,
            title,
            bounds,
            pid,
            bundle_id: None,
            app_path: None,
            app_order: 0,
            window_index: id as usize,
            global_order: id as usize,
            is_frontmost_app: false,
            is_focused: false,
            is_main: false,
            is_minimized: false,
            is_on_current_space: true,
            ax_window: None,
        })
    }

    pub fn selection_key(&self) -> String {
        let app_key = self.bundle_id.as_deref().unwrap_or(self.app.as_str());
        format!("window:{app_key}:{}:{}", self.pid, self.id)
    }

    /// Get the internal window reference for operations
    fn window_ref(&self) -> Option<AXUIElementRef> {
        self.ax_window.map(|ptr| ptr as AXUIElementRef)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_window_descriptor(
    app: &str,
    pid: i32,
    bounds: Bounds,
    is_frontmost_app: bool,
    is_focused: bool,
    is_main: bool,
    is_minimized: bool,
    is_on_current_space: bool,
    duplicate_label: Option<&str>,
) -> String {
    let mut parts = vec![app.to_string()];
    if is_frontmost_app {
        parts.push("Frontmost".to_string());
    }
    if is_focused {
        parts.push("Focused".to_string());
    } else if is_main {
        parts.push("Main".to_string());
    }
    if is_minimized {
        parts.push("Minimized".to_string());
    }
    if !is_on_current_space {
        parts.push("Other Space".to_string());
    }
    if let Some(label) = duplicate_label {
        parts.push(label.to_string());
    }
    parts.push(format!("{}x{}", bounds.width, bounds.height));
    parts.push(format!("pid {pid}"));
    parts.join(" - ")
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
