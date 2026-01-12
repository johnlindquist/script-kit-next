//! Display/monitor detection and management

use super::bounds::WindowBounds;
use super::coords::nsrect_to_bounds;
use anyhow::{bail, Context, Result};
use tracing::{info, instrument};

/// Information about a display/monitor
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    /// Display identifier
    pub id: u32,
    /// Display name (if available)
    pub name: Option<String>,
    /// Full display bounds in AX coordinates
    pub frame: WindowBounds,
    /// Visible bounds (excluding dock, menu bar) in AX coordinates
    pub visible_bounds: WindowBounds,
    /// Whether this is the main (primary) display
    pub is_main: bool,
}

impl DisplayInfo {
    /// Calculate relative position within this display
    ///
    /// Returns (relative_x, relative_y) as fractions [0.0, 1.0]
    pub fn relative_position(&self, bounds: &WindowBounds) -> (f64, f64) {
        let rel_x = (bounds.x - self.visible_bounds.x) / self.visible_bounds.width;
        let rel_y = (bounds.y - self.visible_bounds.y) / self.visible_bounds.height;
        (rel_x, rel_y)
    }

    /// Apply relative position to get absolute bounds on this display
    pub fn apply_relative_position(
        &self,
        rel_x: f64,
        rel_y: f64,
        width: f64,
        height: f64,
    ) -> WindowBounds {
        WindowBounds {
            x: self.visible_bounds.x + rel_x * self.visible_bounds.width,
            y: self.visible_bounds.y + rel_y * self.visible_bounds.height,
            width,
            height,
        }
    }
}

/// Get information about all connected displays
#[cfg(target_os = "macos")]
#[instrument]
pub fn get_all_displays() -> Result<Vec<DisplayInfo>> {
    use core_graphics::display::CGDisplay;

    let display_ids = CGDisplay::active_displays()
        .map_err(|e| anyhow::anyhow!("Failed to get active displays: error code {}", e))?;
    let primary_screen_height = get_primary_screen_height()?;

    let mut displays = Vec::with_capacity(display_ids.len());

    for display_id in display_ids {
        let display = CGDisplay::new(display_id);
        let frame = display.bounds();
        let is_main = display_id == CGDisplay::main().id;

        // Get visible frame from NSScreen
        let visible_bounds = get_nsscreen_visible_frame(display_id, primary_screen_height)
            .unwrap_or_else(|| {
                // Fallback: estimate visible area
                let menu_bar = if is_main { 25.0 } else { 0.0 };
                let dock = if is_main { 70.0 } else { 0.0 };
                WindowBounds::new(
                    frame.origin.x,
                    frame.origin.y + menu_bar,
                    frame.size.width,
                    frame.size.height - menu_bar - dock,
                )
            });

        displays.push(DisplayInfo {
            id: display_id,
            name: None,
            frame: WindowBounds::new(
                frame.origin.x,
                frame.origin.y,
                frame.size.width,
                frame.size.height,
            ),
            visible_bounds,
            is_main,
        });
    }

    info!(display_count = displays.len(), "Got all displays");
    Ok(displays)
}

#[cfg(not(target_os = "macos"))]
pub fn get_all_displays() -> Result<Vec<DisplayInfo>> {
    Ok(vec![DisplayInfo {
        id: 0,
        name: Some("Primary Display".to_string()),
        frame: WindowBounds::new(0.0, 0.0, 1920.0, 1080.0),
        visible_bounds: WindowBounds::new(0.0, 25.0, 1920.0, 1055.0),
        is_main: true,
    }])
}

/// Get the display that contains the largest portion of a window
#[instrument(skip(displays))]
pub fn get_display_for_window<'a>(
    window_bounds: &WindowBounds,
    displays: &'a [DisplayInfo],
) -> Option<&'a DisplayInfo> {
    displays
        .iter()
        .max_by(|a, b| {
            let area_a = window_bounds.intersection_area(&a.visible_bounds);
            let area_b = window_bounds.intersection_area(&b.visible_bounds);
            area_a
                .partial_cmp(&area_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .filter(|d| window_bounds.intersection_area(&d.visible_bounds) > 0.0)
}

/// Get the PRIMARY screen height for coordinate conversion.
/// IMPORTANT: We use screens[0] (the primary screen with the menu bar),
/// NOT mainScreen (which is the screen with the key window).
/// Cocoa coordinates have their origin at bottom-left of the PRIMARY screen,
/// and CoreGraphics coordinates have origin at top-left of PRIMARY screen.
/// Using mainScreen causes incorrect coordinate conversion on multi-monitor setups.
#[cfg(target_os = "macos")]
fn get_primary_screen_height() -> Result<f64> {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen = Class::get("NSScreen").context("Failed to get NSScreen class")?;
        let screens: *mut Object = msg_send![nsscreen, screens];

        if screens.is_null() {
            bail!("No screens found");
        }

        let count: usize = msg_send![screens, count];
        if count == 0 {
            bail!("No screens available");
        }

        // Use screens[0] which is always the primary screen (with menu bar)
        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        if primary_screen.is_null() {
            bail!("Primary screen is null");
        }

        #[repr(C)]
        struct NSRect {
            origin: NSPoint,
            size: NSSize,
        }
        #[repr(C)]
        struct NSPoint {
            x: f64,
            y: f64,
        }
        #[repr(C)]
        struct NSSize {
            width: f64,
            height: f64,
        }

        let frame: NSRect = msg_send![primary_screen, frame];
        Ok(frame.size.height)
    }
}

#[cfg(not(target_os = "macos"))]
fn get_primary_screen_height() -> Result<f64> {
    Ok(1080.0)
}

/// Get NSScreen.visibleFrame for a display, converted to AX coordinates
#[cfg(target_os = "macos")]
fn get_nsscreen_visible_frame(display_id: u32, primary_screen_height: f64) -> Option<WindowBounds> {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen = Class::get("NSScreen")?;
        let screens: *mut Object = msg_send![nsscreen, screens];

        if screens.is_null() {
            return None;
        }

        let count: usize = msg_send![screens, count];

        #[repr(C)]
        struct NSRect {
            origin: NSPoint,
            size: NSSize,
        }
        #[repr(C)]
        struct NSPoint {
            x: f64,
            y: f64,
        }
        #[repr(C)]
        struct NSSize {
            width: f64,
            height: f64,
        }

        let target_display = core_graphics::display::CGDisplay::new(display_id);
        let target_frame = target_display.bounds();

        for i in 0..count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            let frame: NSRect = msg_send![screen, frame];

            // Match by origin and size
            let matches = (frame.origin.x - target_frame.origin.x).abs() < 1.0
                && (frame.size.width - target_frame.size.width).abs() < 1.0
                && (frame.size.height - target_frame.size.height).abs() < 1.0;

            if matches {
                let visible: NSRect = msg_send![screen, visibleFrame];

                return Some(nsrect_to_bounds(
                    visible.origin.x,
                    visible.origin.y,
                    visible.size.width,
                    visible.size.height,
                    primary_screen_height,
                ));
            }
        }

        None
    }
}

#[cfg(not(target_os = "macos"))]
fn get_nsscreen_visible_frame(
    _display_id: u32,
    _primary_screen_height: f64,
) -> Option<WindowBounds> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_displays() -> Vec<DisplayInfo> {
        vec![
            DisplayInfo {
                id: 1,
                name: Some("Main".to_string()),
                frame: WindowBounds::new(0.0, 0.0, 1920.0, 1080.0),
                visible_bounds: WindowBounds::new(0.0, 25.0, 1920.0, 1055.0),
                is_main: true,
            },
            DisplayInfo {
                id: 2,
                name: Some("Secondary".to_string()),
                frame: WindowBounds::new(1920.0, 0.0, 1920.0, 1080.0),
                visible_bounds: WindowBounds::new(1920.0, 0.0, 1920.0, 1080.0),
                is_main: false,
            },
        ]
    }

    #[test]
    fn test_relative_position() {
        let display = DisplayInfo {
            id: 1,
            name: None,
            frame: WindowBounds::new(0.0, 0.0, 1920.0, 1080.0),
            visible_bounds: WindowBounds::new(0.0, 25.0, 1920.0, 1055.0),
            is_main: true,
        };

        let window = WindowBounds::new(0.0, 25.0, 800.0, 600.0);
        let (rel_x, rel_y) = display.relative_position(&window);
        assert!((rel_x - 0.0).abs() < 0.001);
        assert!((rel_y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_relative_position() {
        let display = DisplayInfo {
            id: 1,
            name: None,
            frame: WindowBounds::new(0.0, 0.0, 1920.0, 1080.0),
            visible_bounds: WindowBounds::new(0.0, 25.0, 1920.0, 1055.0),
            is_main: true,
        };

        let bounds = display.apply_relative_position(0.5, 0.5, 800.0, 600.0);
        assert_eq!(bounds.x, 960.0);
        assert_eq!(bounds.y, 552.5);
        assert_eq!(bounds.width, 800.0);
        assert_eq!(bounds.height, 600.0);
    }

    #[test]
    fn test_window_fully_on_main() {
        let displays = mock_displays();
        let window = WindowBounds::new(100.0, 100.0, 800.0, 600.0);

        let display = get_display_for_window(&window, &displays);
        assert!(display.is_some());
        assert_eq!(display.unwrap().id, 1);
    }

    #[test]
    fn test_window_fully_on_secondary() {
        let displays = mock_displays();
        let window = WindowBounds::new(2000.0, 100.0, 800.0, 600.0);

        let display = get_display_for_window(&window, &displays);
        assert!(display.is_some());
        assert_eq!(display.unwrap().id, 2);
    }

    #[test]
    fn test_window_spanning_displays() {
        let displays = mock_displays();
        // Window mostly on secondary (larger overlap)
        let window = WindowBounds::new(1800.0, 100.0, 400.0, 600.0);

        let display = get_display_for_window(&window, &displays);
        assert!(display.is_some());
        // 120px on main, 280px on secondary -> secondary wins
        assert_eq!(display.unwrap().id, 2);
    }

    #[test]
    fn test_window_not_on_any_display() {
        let displays = mock_displays();
        let window = WindowBounds::new(5000.0, 5000.0, 800.0, 600.0);

        let display = get_display_for_window(&window, &displays);
        assert!(display.is_none());
    }
}
