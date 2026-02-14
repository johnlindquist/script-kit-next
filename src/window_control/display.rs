use anyhow::{bail, Context, Result};
use core_graphics::display::{CGDisplay, CGRect};
use tracing::debug;

use super::types::*;

/// Get the main display bounds
pub(super) fn get_main_display_bounds() -> Bounds {
    let main_display = CGDisplay::main();
    let rect = main_display.bounds();
    Bounds::from_cg_rect(rect)
}

/// Get the display bounds for the display containing a point
pub(super) fn get_display_bounds_at_point(_x: i32, _y: i32) -> Bounds {
    // For simplicity, we'll use the main display
    // A more complete implementation would find the display containing the point
    get_main_display_bounds()
}

/// Get bounds for all available displays
pub(super) fn get_all_display_bounds() -> Result<Vec<Bounds>> {
    let mut displays = Vec::new();

    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen_class = Class::get("NSScreen").context("Failed to get NSScreen class")?;
        let screens: *mut Object = msg_send![nsscreen_class, screens];
        if screens.is_null() {
            bail!("Failed to get screens");
        }

        let screen_count: usize = msg_send![screens, count];
        if screen_count == 0 {
            bail!("No screens found");
        }

        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        let primary_frame: CGRect = msg_send![primary_screen, frame];
        let primary_height = primary_frame.size.height;

        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            let visible_frame: CGRect = msg_send![screen, visibleFrame];
            let cg_y = primary_height - (visible_frame.origin.y + visible_frame.size.height);

            displays.push(Bounds {
                x: visible_frame.origin.x as i32,
                y: cg_y as i32,
                width: visible_frame.size.width as u32,
                height: visible_frame.size.height as u32,
            });
        }
    }

    Ok(displays)
}

/// Get the visible display bounds (excluding menu bar and dock) for the display
/// containing the given point.
///
/// Uses NSScreen.visibleFrame to get accurate bounds that account for:
/// - Menu bar (on main display)
/// - Dock (on any edge, any display)
/// - Notch area (on newer MacBooks)
pub(super) fn get_visible_display_bounds(x: i32, y: i32) -> Bounds {
    // Use NSScreen to get accurate visible frame
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen_class = match Class::get("NSScreen") {
            Some(c) => c,
            None => return get_visible_display_bounds_fallback(x, y),
        };

        // Get all screens
        let screens: *mut Object = msg_send![nsscreen_class, screens];
        if screens.is_null() {
            return get_visible_display_bounds_fallback(x, y);
        }

        let screen_count: usize = msg_send![screens, count];
        if screen_count == 0 {
            return get_visible_display_bounds_fallback(x, y);
        }

        // Get the PRIMARY screen height for coordinate conversion (do this once outside the loop)
        // IMPORTANT: We MUST use screens[0] (the primary screen with the menu bar),
        // NOT mainScreen (which is the screen with the key window).
        // Cocoa coordinates have their origin at bottom-left of the PRIMARY screen,
        // and CoreGraphics coordinates have origin at top-left of PRIMARY screen.
        // Using mainScreen causes incorrect coordinate conversion on multi-monitor setups
        // when the focused window is on a secondary display.
        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        let primary_frame: CGRect = msg_send![primary_screen, frame];
        let primary_height = primary_frame.size.height;

        // Convert CG y to Cocoa y (once, outside the loop)
        let cocoa_y = primary_height - y as f64;

        // Find the screen containing the point
        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            // Get the full frame (in Cocoa coordinates - origin at bottom-left)
            let frame: CGRect = msg_send![screen, frame];

            // Check if point is within this screen's frame
            if (x as f64) >= frame.origin.x
                && (x as f64) < frame.origin.x + frame.size.width
                && cocoa_y >= frame.origin.y
                && cocoa_y < frame.origin.y + frame.size.height
            {
                // Get the visible frame (excludes menu bar and dock)
                let visible_frame: CGRect = msg_send![screen, visibleFrame];

                // Convert Cocoa coordinates back to CoreGraphics coordinates
                // CG origin is at top-left of primary screen
                // Cocoa origin.y is distance from bottom of primary screen
                // CG y = primary_height - (cocoa_y + height)
                let cg_y = primary_height - (visible_frame.origin.y + visible_frame.size.height);

                debug!(
                    screen_index = i,
                    frame_x = frame.origin.x,
                    frame_y = frame.origin.y,
                    frame_w = frame.size.width,
                    frame_h = frame.size.height,
                    visible_x = visible_frame.origin.x,
                    visible_y = visible_frame.origin.y,
                    visible_w = visible_frame.size.width,
                    visible_h = visible_frame.size.height,
                    cg_y = cg_y,
                    "Found screen for point ({}, {})",
                    x,
                    y
                );

                return Bounds {
                    x: visible_frame.origin.x as i32,
                    y: cg_y as i32,
                    width: visible_frame.size.width as u32,
                    height: visible_frame.size.height as u32,
                };
            }
        }
    }

    // Fallback if no screen found
    get_visible_display_bounds_fallback(x, y)
}

/// Fallback method using CGDisplay when NSScreen is unavailable
pub(super) fn get_visible_display_bounds_fallback(x: i32, y: i32) -> Bounds {
    // Get all displays
    if let Ok(display_ids) = CGDisplay::active_displays() {
        for display_id in display_ids {
            let display = CGDisplay::new(display_id);
            let frame = display.bounds();

            // Check if point is within this display
            if x >= frame.origin.x as i32
                && x < (frame.origin.x + frame.size.width) as i32
                && y >= frame.origin.y as i32
                && y < (frame.origin.y + frame.size.height) as i32
            {
                let is_main = display_id == CGDisplay::main().id;

                // Conservative estimates for menu bar and dock
                let menu_bar_height = if is_main { 25 } else { 0 };
                let dock_height = if is_main { 70 } else { 0 };

                return Bounds {
                    x: frame.origin.x as i32,
                    y: frame.origin.y as i32 + menu_bar_height,
                    width: frame.size.width as u32,
                    height: (frame.size.height as i32 - menu_bar_height - dock_height) as u32,
                };
            }
        }
    }

    // Final fallback to main display
    let main = CGDisplay::main();
    let frame = main.bounds();
    Bounds {
        x: frame.origin.x as i32,
        y: frame.origin.y as i32 + 25,
        width: frame.size.width as u32,
        height: (frame.size.height - 95.0) as u32,
    }
}
