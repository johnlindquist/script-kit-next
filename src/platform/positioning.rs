// ============================================================================
// Window Movement
// ============================================================================

#[cfg(target_os = "macos")]
use cocoa::foundation::{NSPoint, NSSize};

/// Move the application's main window to new bounds using WindowManager.
/// This uses the registered main window instead of objectAtIndex:0, which
/// avoids issues with tray icons and other system windows in the array.
///
/// IMPORTANT: macOS uses a global coordinate space where Y=0 is at the BOTTOM of the
/// PRIMARY screen, and Y increases upward. The primary screen's origin is always (0,0)
/// at its bottom-left corner. Secondary displays have their own position in this space.
#[cfg(target_os = "macos")]
pub fn move_first_window_to(x: f64, y: f64, width: f64, height: f64) {
    if require_main_thread("move_first_window_to") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // NSScreen.mainScreen nil-checked with fallback. setFrame:display:animate:
    // is a standard NSWindow method.
    unsafe {
        // Use WindowManager to get the main window reliably
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "POSITION",
                    "WARNING: Main window not registered in WindowManager, cannot move",
                );
                return;
            }
        };

        // Get the PRIMARY screen's height for coordinate conversion
        // CRITICAL: Use mainScreen, not firstObject - they can differ when display arrangement changes
        let main_screen: id = msg_send![class!(NSScreen), mainScreen];
        let main_screen = if main_screen == nil {
            // Fallback to firstObject if mainScreen is nil (shouldn't happen but be safe)
            let screens: id = msg_send![class!(NSScreen), screens];
            if screens.is_null() {
                logging::log("POSITION", "WARNING: NSScreen.screens returned nil");
                return;
            }
            let fallback: id = msg_send![screens, firstObject];
            if fallback.is_null() {
                logging::log("POSITION", "WARNING: No screens available");
                return;
            }
            fallback
        } else {
            main_screen
        };
        let main_screen_frame: NSRect = msg_send![main_screen, frame];
        let primary_screen_height = main_screen_frame.size.height;

        // Log current window position before move
        let current_frame: NSRect = msg_send![window, frame];
        logging::log(
            "POSITION",
            &format!(
                "Current window frame: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                current_frame.origin.x,
                current_frame.origin.y,
                current_frame.size.width,
                current_frame.size.height
            ),
        );

        // Convert from top-left origin (y down) to bottom-left origin (y up)
        let flipped_y = primary_screen_height - y - height;

        logging::log(
            "POSITION",
            &format!(
                "Moving window: target=({:.0}, {:.0}) flipped_y={:.0}",
                x, y, flipped_y
            ),
        );

        let new_frame = NSRect::new(NSPoint::new(x, flipped_y), NSSize::new(width, height));

        // Move the window
        let _: () = msg_send![window, setFrame:new_frame display:true animate:false];

        // NOTE: We no longer call makeKeyAndOrderFront here.
        // Window ordering/activation is handled by GPUI's cx.activate() and win.activate_window()
        // which is called AFTER ensure_move_to_active_space() sets the collection behavior.

        // Verify the move worked
        let after_frame: NSRect = msg_send![window, frame];
        logging::log(
            "POSITION",
            &format!(
                "Window moved: actual=({:.0}, {:.0}) size={:.0}x{:.0}",
                after_frame.origin.x,
                after_frame.origin.y,
                after_frame.size.width,
                after_frame.size.height
            ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn move_first_window_to(_x: f64, _y: f64, _width: f64, _height: f64) {
    // TODO: Implement for other platforms
    logging::log(
        "POSITION",
        "move_first_window_to is not implemented for this platform",
    );
}

use gpui::{point, px, Bounds, Pixels};

/// Move the first window to new bounds (wrapper for Bounds<Pixels>)
pub fn move_first_window_to_bounds(bounds: &Bounds<Pixels>) {
    let x: f64 = bounds.origin.x.into();
    let y: f64 = bounds.origin.y.into();
    let width: f64 = bounds.size.width.into();
    let height: f64 = bounds.size.height.into();
    move_first_window_to(x, y, width, height);
}

// ============================================================================
// Window Positioning (Eye-line)
// ============================================================================

/// Calculate window bounds positioned at eye-line height on the display containing the mouse cursor.
///
/// - Finds the display where the mouse cursor is located
/// - Centers the window horizontally on that display
/// - Positions the window at "eye-line" height (upper 14% of the screen)
///
/// This matches the behavior of Raycast/Alfred where the prompt appears on the active display.
pub fn calculate_eye_line_bounds_on_mouse_display(
    window_size: gpui::Size<Pixels>,
) -> Bounds<Pixels> {
    // Use native macOS API to get actual display bounds with correct origins
    // GPUI's cx.displays() returns incorrect origins for secondary displays
    let displays = get_macos_displays();

    logging::log("POSITION", "");
    logging::log(
        "POSITION",
        "╔════════════════════════════════════════════════════════════╗",
    );
    logging::log(
        "POSITION",
        "║  CALCULATING WINDOW POSITION FOR MOUSE DISPLAY             ║",
    );
    logging::log(
        "POSITION",
        "╚════════════════════════════════════════════════════════════╝",
    );
    logging::log(
        "POSITION",
        &format!("Available displays: {}", displays.len()),
    );

    // Log all available displays for debugging
    for (idx, display) in displays.iter().enumerate() {
        let right = display.origin_x + display.width;
        let bottom = display.origin_y + display.height;
        logging::log("POSITION", &format!(
            "  Display {}: origin=({:.0}, {:.0}) size={:.0}x{:.0} [bounds: x={:.0}..{:.0}, y={:.0}..{:.0}]",
            idx, display.origin_x, display.origin_y, display.width, display.height,
            display.origin_x, right, display.origin_y, bottom
        ));
    }

    // Try to get mouse position and find which display contains it
    let target_display = if let Some((mouse_x, mouse_y)) = get_global_mouse_position() {
        logging::log(
            "POSITION",
            &format!("Mouse cursor at ({:.0}, {:.0})", mouse_x, mouse_y),
        );

        // Find the display that contains the mouse cursor
        let mut found_display = None;
        for (idx, display) in displays.iter().enumerate() {
            let in_x = mouse_x >= display.origin_x && mouse_x < display.origin_x + display.width;
            let in_y = mouse_y >= display.origin_y && mouse_y < display.origin_y + display.height;
            let contains = in_x && in_y;

            if contains {
                logging::log("POSITION", &format!("  -> Mouse is on display {}", idx));
                found_display = Some(display.clone());
                break;
            } else {
                // Log why this display didn't match (helpful for debugging edge cases)
                logging::log(
                    "POSITION",
                    &format!(
                        "  Display {} rejected: in_x={} in_y={} (mouse: {:.0},{:.0} vs bounds: x={:.0}..{:.0}, y={:.0}..{:.0})",
                        idx, in_x, in_y, mouse_x, mouse_y,
                        display.origin_x, display.origin_x + display.width,
                        display.origin_y, display.origin_y + display.height
                    ),
                );
            }
        }

        found_display
    } else {
        logging::log(
            "POSITION",
            "Could not get mouse position, using primary display",
        );
        None
    };

    // Use the found display, or fall back to first display (primary)
    let display = target_display.or_else(|| {
        logging::log(
            "POSITION",
            "No display contains mouse, falling back to primary",
        );
        displays.first().cloned()
    });

    if let Some(display) = display {
        logging::log(
            "POSITION",
            &format!(
                "Selected display: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                display.origin_x, display.origin_y, display.width, display.height
            ),
        );

        // Eye-line: position window top at ~14% from screen top (input bar at eye level)
        let eye_line_y = display.origin_y + display.height * 0.14;

        // Center horizontally on the display
        let window_width: f64 = window_size.width.into();
        let center_x = display.origin_x + (display.width - window_width) / 2.0;

        let final_bounds = Bounds {
            origin: point(px(center_x as f32), px(eye_line_y as f32)),
            size: window_size,
        };

        logging::log(
            "POSITION",
            &format!(
                "Final window bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                center_x,
                eye_line_y,
                f64::from(window_size.width),
                f64::from(window_size.height)
            ),
        );

        final_bounds
    } else {
        logging::log(
            "POSITION",
            "No displays found, using default centered bounds",
        );
        // Fallback: just center on screen using 1512x982 as default (common MacBook)
        Bounds {
            origin: point(px(381.0), px(246.0)),
            size: window_size,
        }
    }
}

/// Calculate window bounds centered on the display containing the mouse cursor.
///
/// Similar to `calculate_eye_line_bounds_on_mouse_display` but centers the window
/// both horizontally and vertically on the mouse's display. Used for secondary windows
/// like AI chat that should appear on the same display as the main window.
///
/// # Arguments
/// * `window_size` - The desired size of the window
///
/// # Returns
/// Bounds positioned at center of the display containing the mouse cursor,
/// or centered on primary display if mouse position cannot be determined.
pub fn calculate_centered_bounds_on_mouse_display(
    window_size: gpui::Size<Pixels>,
) -> Bounds<Pixels> {
    let displays = get_macos_displays();

    logging::log("POSITION", "");
    logging::log(
        "POSITION",
        "╔════════════════════════════════════════════════════════════╗",
    );
    logging::log(
        "POSITION",
        "║  CALCULATING CENTERED POSITION ON MOUSE DISPLAY            ║",
    );
    logging::log(
        "POSITION",
        "╚════════════════════════════════════════════════════════════╝",
    );
    logging::log(
        "POSITION",
        &format!("Available displays: {}", displays.len()),
    );

    // Log all available displays for debugging
    for (idx, display) in displays.iter().enumerate() {
        let right = display.origin_x + display.width;
        let bottom = display.origin_y + display.height;
        logging::log(
            "POSITION",
            &format!(
                "  Display {}: origin=({:.0}, {:.0}) size={:.0}x{:.0} [bounds: x={:.0}..{:.0}, y={:.0}..{:.0}]",
                idx, display.origin_x, display.origin_y, display.width, display.height,
                display.origin_x, right, display.origin_y, bottom
            ),
        );
    }

    // Try to get mouse position and find which display contains it
    let target_display = if let Some((mouse_x, mouse_y)) = get_global_mouse_position() {
        logging::log(
            "POSITION",
            &format!("Mouse cursor at ({:.0}, {:.0})", mouse_x, mouse_y),
        );

        // Find the display that contains the mouse cursor
        let mut found_display = None;
        for (idx, display) in displays.iter().enumerate() {
            let in_x = mouse_x >= display.origin_x && mouse_x < display.origin_x + display.width;
            let in_y = mouse_y >= display.origin_y && mouse_y < display.origin_y + display.height;
            let contains = in_x && in_y;

            if contains {
                logging::log("POSITION", &format!("  -> Mouse is on display {}", idx));
                found_display = Some(display.clone());
                break;
            }
        }

        found_display
    } else {
        logging::log(
            "POSITION",
            "Could not get mouse position, using primary display",
        );
        None
    };

    // Use the found display, or fall back to first display (primary)
    let display = target_display.or_else(|| {
        logging::log(
            "POSITION",
            "No display contains mouse, falling back to primary",
        );
        displays.first().cloned()
    });

    if let Some(display) = display {
        logging::log(
            "POSITION",
            &format!(
                "Selected display: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                display.origin_x, display.origin_y, display.width, display.height
            ),
        );

        // Center both horizontally and vertically on the display
        let window_width: f64 = window_size.width.into();
        let window_height: f64 = window_size.height.into();
        let center_x = display.origin_x + (display.width - window_width) / 2.0;
        let center_y = display.origin_y + (display.height - window_height) / 2.0;

        let final_bounds = Bounds {
            origin: point(px(center_x as f32), px(center_y as f32)),
            size: window_size,
        };

        logging::log(
            "POSITION",
            &format!(
                "Final centered bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                center_x,
                center_y,
                f64::from(window_size.width),
                f64::from(window_size.height)
            ),
        );

        final_bounds
    } else {
        logging::log(
            "POSITION",
            "No displays found, using default centered bounds",
        );
        // Fallback: just center on screen using 1512x982 as default (common MacBook)
        let window_width: f64 = window_size.width.into();
        let window_height: f64 = window_size.height.into();
        Bounds {
            origin: point(
                px(((1512.0 - window_width) / 2.0) as f32),
                px(((982.0 - window_height) / 2.0) as f32),
            ),
            size: window_size,
        }
    }
}

