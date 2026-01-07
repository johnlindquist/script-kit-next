//! Enhanced Window Control Module
//!
//! This module builds on the base `window_control` module to provide:
//! - **Coordinate foundation**: Standardized WindowBounds in AX coordinates (top-left origin)
//! - **Capability detection**: Settability checks via AXUIElementIsAttributeSettable
//! - **Enhanced window info**: EnhancedWindowInfo with capabilities (can_move/can_resize)
//! - **Multi-monitor support**: DisplayInfo with proper overlap-based display detection
//! - **Spaces backend**: Optional SpaceManager trait (defaults to Unsupported)
//!
//! ## Coordinate System
//!
//! Internally, all bounds use **AX coordinates** (top-left origin, Y grows downward).
//! This matches what `kAXPositionAttribute` get/set operations expect.
//!
//! AppKit's `NSScreen.visibleFrame` uses bottom-left origin, so we convert it
//! to AX coordinates when building `DisplayInfo`.

mod bounds;
mod capabilities;
mod coords;
mod display;
mod spaces;

pub use bounds::{SizeConstraints, WindowBounds};
pub use capabilities::{
    can_close_window, can_fullscreen_window, can_minimize_window, can_move_window,
    can_resize_window, detect_window_capabilities, EnhancedWindowInfo, WindowCapabilities,
};
pub use coords::{appkit_to_ax, ax_to_appkit, bounds_to_nsrect, nsrect_to_bounds};
pub use display::{get_all_displays, get_display_for_window, DisplayInfo};
pub use spaces::{
    get_space_manager, set_space_manager, SpaceError, SpaceInfo, SpaceManager, SpaceType,
    UnsupportedSpaceBackend,
};
