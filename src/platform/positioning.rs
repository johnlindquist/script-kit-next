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

const EYE_LINE_Y_RATIO: f64 = 0.14;
const FALLBACK_VISIBLE_WIDTH: f64 = 1512.0;
const FALLBACK_VISIBLE_HEIGHT: f64 = 982.0;

#[derive(Clone, Copy)]
enum MouseDisplayPlacement {
    EyeLine,
    Centered,
}

fn display_edges(bounds: &DisplayBounds) -> (f64, f64) {
    (
        bounds.origin_x + bounds.width,
        bounds.origin_y + bounds.height,
    )
}

fn log_positioning_banner(title_line: &str) {
    logging::log("POSITION", "");
    logging::log(
        "POSITION",
        "╔════════════════════════════════════════════════════════════╗",
    );
    logging::log("POSITION", title_line);
    logging::log(
        "POSITION",
        "╚════════════════════════════════════════════════════════════╝",
    );
}

fn log_available_displays(displays: &[VisibleDisplayBounds]) {
    for (idx, display) in displays.iter().enumerate() {
        let (frame_right, frame_bottom) = display_edges(&display.frame);
        let (visible_right, visible_bottom) = display_edges(&display.visible_area);

        logging::log(
            "POSITION",
            &format!(
                "  Display {}: frame=({:.0},{:.0}) {:.0}x{:.0} [x={:.0}..{:.0}, y={:.0}..{:.0}] visible=({:.0},{:.0}) {:.0}x{:.0} [x={:.0}..{:.0}, y={:.0}..{:.0}]",
                idx,
                display.frame.origin_x,
                display.frame.origin_y,
                display.frame.width,
                display.frame.height,
                display.frame.origin_x,
                frame_right,
                display.frame.origin_y,
                frame_bottom,
                display.visible_area.origin_x,
                display.visible_area.origin_y,
                display.visible_area.width,
                display.visible_area.height,
                display.visible_area.origin_x,
                visible_right,
                display.visible_area.origin_y,
                visible_bottom
            ),
        );
    }
}

fn select_display_for_mouse(displays: &[VisibleDisplayBounds]) -> Option<VisibleDisplayBounds> {
    if let Some((mouse_x, mouse_y)) = get_global_mouse_position() {
        logging::log(
            "POSITION",
            &format!("Mouse cursor at ({:.0}, {:.0})", mouse_x, mouse_y),
        );

        if let Some(display) = display_for_point((mouse_x, mouse_y), displays) {
            return Some(display);
        }

        logging::log(
            "POSITION",
            "Mouse is not inside any display frame, falling back to primary display",
        );
    } else {
        logging::log(
            "POSITION",
            "Could not get mouse position, using primary display",
        );
    }

    displays.first().cloned()
}

fn fallback_display_bounds() -> VisibleDisplayBounds {
    let fallback = DisplayBounds {
        origin_x: 0.0,
        origin_y: 0.0,
        width: FALLBACK_VISIBLE_WIDTH,
        height: FALLBACK_VISIBLE_HEIGHT,
    };

    VisibleDisplayBounds {
        frame: fallback.clone(),
        visible_area: fallback,
    }
}

fn calculate_bounds_on_mouse_display(
    window_size: gpui::Size<Pixels>,
    placement: MouseDisplayPlacement,
) -> Bounds<Pixels> {
    let displays = get_macos_visible_displays();
    let title_line = match placement {
        MouseDisplayPlacement::EyeLine => {
            "║  CALCULATING WINDOW POSITION FOR MOUSE DISPLAY             ║"
        }
        MouseDisplayPlacement::Centered => {
            "║  CALCULATING CENTERED POSITION ON MOUSE DISPLAY            ║"
        }
    };
    log_positioning_banner(title_line);
    logging::log(
        "POSITION",
        &format!("Available displays: {}", displays.len()),
    );
    log_available_displays(&displays);

    let mut used_fallback_display = false;
    let display = select_display_for_mouse(&displays).unwrap_or_else(|| {
        used_fallback_display = true;
        logging::log(
            "POSITION",
            "No displays found, using default centered bounds",
        );
        fallback_display_bounds()
    });

    let visible = &display.visible_area;
    logging::log(
        "POSITION",
        &format!(
            "Selected display visible area: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
            visible.origin_x, visible.origin_y, visible.width, visible.height
        ),
    );

    let window_width: f64 = window_size.width.into();
    let window_height: f64 = window_size.height.into();
    let origin_x = visible.origin_x + (visible.width - window_width) / 2.0;
    let origin_y = match placement {
        // Keep fallback behavior centered while normal eye-line placement uses the ratio.
        MouseDisplayPlacement::EyeLine if used_fallback_display => {
            visible.origin_y + (visible.height - window_height) / 2.0
        }
        MouseDisplayPlacement::EyeLine => visible.origin_y + visible.height * EYE_LINE_Y_RATIO,
        MouseDisplayPlacement::Centered => visible.origin_y + (visible.height - window_height) / 2.0,
    };

    let desired_bounds = Bounds {
        origin: point(px(origin_x as f32), px(origin_y as f32)),
        size: window_size,
    };
    let final_bounds = clamp_to_visible(desired_bounds, visible);

    let final_x: f64 = final_bounds.origin.x.into();
    let final_y: f64 = final_bounds.origin.y.into();
    let final_width: f64 = final_bounds.size.width.into();
    let final_height: f64 = final_bounds.size.height.into();
    let final_message = match placement {
        MouseDisplayPlacement::EyeLine => format!(
            "Final window bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
            final_x, final_y, final_width, final_height
        ),
        MouseDisplayPlacement::Centered => format!(
            "Final centered bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
            final_x, final_y, final_width, final_height
        ),
    };
    logging::log("POSITION", &final_message);

    final_bounds
}

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
    calculate_bounds_on_mouse_display(window_size, MouseDisplayPlacement::EyeLine)
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
    calculate_bounds_on_mouse_display(window_size, MouseDisplayPlacement::Centered)
}

#[cfg(test)]
mod positioning_bounds_tests {
    use super::*;
    use gpui::size;

    #[test]
    fn test_fallback_display_bounds_sets_frame_and_visible_area() {
        let fallback = fallback_display_bounds();

        assert_eq!(fallback.frame.origin_x, 0.0);
        assert_eq!(fallback.frame.origin_y, 0.0);
        assert_eq!(fallback.frame.width, FALLBACK_VISIBLE_WIDTH);
        assert_eq!(fallback.frame.height, FALLBACK_VISIBLE_HEIGHT);
        assert_eq!(fallback.visible_area.origin_x, 0.0);
        assert_eq!(fallback.visible_area.origin_y, 0.0);
        assert_eq!(fallback.visible_area.width, FALLBACK_VISIBLE_WIDTH);
        assert_eq!(fallback.visible_area.height, FALLBACK_VISIBLE_HEIGHT);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_calculate_eye_line_bounds_on_mouse_display_centers_fallback_for_window_size() {
        let window_size = size(px(1000.0), px(200.0));

        let eye_line = calculate_eye_line_bounds_on_mouse_display(window_size);
        let centered = calculate_centered_bounds_on_mouse_display(window_size);

        let eye_line_x: f64 = eye_line.origin.x.into();
        let eye_line_y: f64 = eye_line.origin.y.into();
        let centered_x: f64 = centered.origin.x.into();
        let centered_y: f64 = centered.origin.y.into();

        assert_eq!(eye_line_x, centered_x);
        assert_eq!(eye_line_y, centered_y);
    }
}
