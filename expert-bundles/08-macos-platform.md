🧩 Packing 4 file(s)...
📝 Files selected:
  • src/platform.rs
  • src/window_control.rs
  • src/panel.rs
  • src/frontmost_app_tracker.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 4
</notes>
</file_summary>

<directory_structure>
src/platform.rs
src/window_control.rs
src/panel.rs
src/frontmost_app_tracker.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/platform.rs">
//! Platform-specific window configuration abstraction.
//!
//! This module provides cross-platform abstractions for window behavior configuration,
//! with macOS-specific implementations for floating panel behavior and space management.
//!
//! # macOS Behavior
//!
//! On macOS, this module configures windows as floating panels that:
//! - Float above normal windows (NSFloatingWindowLevel = 3)
//! - Move to the active space when shown (NSWindowCollectionBehaviorMoveToActiveSpace = 2)
//! - Disable window state restoration to prevent position caching
//!
//! # Other Platforms
//!
//! On non-macOS platforms, these functions are no-ops, allowing cross-platform code
//! to call them without conditional compilation at the call site.

use crate::logging;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
use crate::window_manager;

// ============================================================================
// Application Activation Policy
// ============================================================================

/// Configure the app as an "accessory" application.
///
/// This is equivalent to setting `LSUIElement=true` in Info.plist, but done at runtime.
/// Accessory apps:
/// - Do NOT appear in the Dock
/// - Do NOT take menu bar ownership when activated
/// - Can still show windows that float above other apps
///
/// This is critical for window management actions (tile, maximize, etc.) because
/// it allows us to query `menuBarOwningApplication` to find the previously active app.
///
/// # macOS Behavior
///
/// Sets NSApplicationActivationPolicyAccessory (value = 1) on the app.
/// Must be called early in app startup, before any windows are shown.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn configure_as_accessory_app() {
    unsafe {
        let app: id = NSApp();
        // NSApplicationActivationPolicyAccessory = 1
        // This makes the app not appear in Dock and not take menu bar ownership
        let _: () = msg_send![app, setActivationPolicy: 1i64];
        logging::log(
            "PANEL",
            "Configured app as accessory (no Dock icon, no menu bar ownership)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_as_accessory_app() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Space Management
// ============================================================================

/// Ensure the main window moves to the currently active macOS space when shown.
///
/// This function sets NSWindowCollectionBehaviorMoveToActiveSpace on the main window,
/// which causes it to move to whichever space is currently active when the window
/// becomes visible, rather than forcing the user back to the space where the window
/// was last shown.
///
/// # macOS Behavior
///
/// Uses the WindowManager to get the main window (not keyWindow, which may not exist
/// yet during app startup) and sets the collection behavior.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
///
/// # Safety
///
/// Uses Objective-C message sending internally on macOS.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn ensure_move_to_active_space() {
    unsafe {
        // Use WindowManager to get the main window (not keyWindow, which may not exist yet)
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "WARNING: Main window not registered, cannot set MoveToActiveSpace",
                );
                return;
            }
        };

        // NSWindowCollectionBehaviorMoveToActiveSpace = (1 << 1) = 2
        // This makes the window MOVE to the current active space when shown
        let collection_behavior: u64 = 2;
        let _: () = msg_send![window, setCollectionBehavior:collection_behavior];

        logging::log(
            "PANEL",
            "Set MoveToActiveSpace collection behavior (before activation)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ensure_move_to_active_space() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Floating Panel Configuration
// ============================================================================

/// Configure the current key window as a floating macOS panel.
///
/// This function configures the key window (most recently activated window) with:
/// - Floating window level (NSFloatingWindowLevel = 3) - appears above normal windows
/// - MoveToActiveSpace collection behavior - moves to current space when shown
/// - Disabled window restoration - prevents macOS from remembering window position
/// - Empty frame autosave name - prevents position caching
///
/// # macOS Behavior
///
/// Uses NSApp to get the keyWindow and applies all configurations. If no key window
/// is found (e.g., during app startup), logs a warning and returns.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
///
/// # Safety
///
/// Uses Objective-C message sending internally on macOS.
///
#[cfg(target_os = "macos")]
pub fn configure_as_floating_panel() {
    unsafe {
        let app: id = NSApp();

        // Get the key window (the most recently activated window)
        let window: id = msg_send![app, keyWindow];

        if window != nil {
            // NSFloatingWindowLevel = 3
            // This makes the window float above normal windows
            let floating_level: i32 = 3;
            let _: () = msg_send![window, setLevel:floating_level];

            // NSWindowCollectionBehaviorMoveToActiveSpace = (1 << 1)
            // This makes the window MOVE to the current active space when shown
            // (instead of forcing user back to the space where window was last visible)
            let collection_behavior: u64 = 2;
            let _: () = msg_send![window, setCollectionBehavior:collection_behavior];

            // CRITICAL: Disable macOS window state restoration
            // This prevents macOS from remembering and restoring the window position
            // when the app is relaunched or the window is shown again
            let _: () = msg_send![window, setRestorable:false];

            // Also disable the window's autosave frame name which can cause position caching
            let empty_string: id = msg_send![class!(NSString), string];
            let _: () = msg_send![window, setFrameAutosaveName:empty_string];

            logging::log(
                "PANEL",
                "Configured window as floating panel (level=3, MoveToActiveSpace, restorable=false, no autosave)",
            );
        } else {
            logging::log(
                "PANEL",
                "Warning: No key window found to configure as panel",
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_as_floating_panel() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Main Window Visibility Control
// ============================================================================

/// Hide the main window without hiding the entire app.
///
/// This is used when opening secondary windows (Notes, AI) to ensure the main
/// window stays hidden while the secondary window is shown. Unlike cx.hide(),
/// this doesn't hide all windows - only the main window.
///
/// # macOS Behavior
///
/// Uses NSWindow orderOut: to remove the main window from the screen without
/// affecting other windows. The window is not minimized, just hidden.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn hide_main_window() {
    unsafe {
        // Use WindowManager to get the main window
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "hide_main_window: Main window not registered, nothing to hide",
                );
                return;
            }
        };

        // orderOut: removes the window from the screen without affecting other windows
        // nil sender means the action is programmatic, not from a menu item
        let _: () = msg_send![window, orderOut:nil];

        logging::log("PANEL", "Main window hidden via orderOut:");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn hide_main_window() {
    // No-op on non-macOS platforms
}

// ============================================================================
// App Active State Detection
// ============================================================================

/// Check if the application is currently active (has focus).
///
/// On macOS, this uses NSApplication's isActive property to determine
/// if our app is the frontmost app receiving keyboard events.
///
/// # Returns
/// - `true` if the app is active (user is interacting with our windows)
/// - `false` if another app is active (user clicked on another app)
///
/// # Platform Support
/// - macOS: Uses NSApplication isActive
/// - Other platforms: Always returns true (not yet implemented)
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn is_app_active() -> bool {
    unsafe {
        let app: id = NSApp();
        let is_active: bool = msg_send![app, isActive];
        is_active
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn is_app_active() -> bool {
    // TODO: Implement for other platforms
    // On non-macOS, assume always active
    true
}

/// Check if the main window is currently focused (key window).
///
/// This is used to detect focus loss even when the app remains active
/// (e.g., when switching focus to Notes/AI windows).
#[cfg(target_os = "macos")]
pub fn is_main_window_focused() -> bool {
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(window) => window,
            None => return false,
        };

        let is_key: bool = msg_send![window, isKeyWindow];
        is_key
    }
}

#[cfg(not(target_os = "macos"))]
pub fn is_main_window_focused() -> bool {
    // TODO: Implement for other platforms
    // On non-macOS, assume focused to avoid auto-dismiss behavior.
    true
}

// ============================================================================
// Constants
// ============================================================================

/// NSFloatingWindowLevel constant value (3)
/// Windows at this level float above normal windows but below modal dialogs.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_FLOATING_WINDOW_LEVEL: i32 = 3;

/// NSWindowCollectionBehaviorMoveToActiveSpace constant value (1 << 1 = 2)
/// When set, the window moves to the currently active space when shown.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE: u64 = 2;

// ============================================================================
// Mouse Position
// ============================================================================

#[cfg(target_os = "macos")]
use core_graphics::event::CGEvent;
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Get the current global mouse cursor position using macOS Core Graphics API.
/// Returns the position in screen coordinates.
#[cfg(target_os = "macos")]
pub fn get_global_mouse_position() -> Option<(f64, f64)> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let event = CGEvent::new(source).ok()?;
    let location = event.location();
    Some((location.x, location.y))
}

#[cfg(not(target_os = "macos"))]
pub fn get_global_mouse_position() -> Option<(f64, f64)> {
    // TODO: Implement for other platforms
    None
}

// ============================================================================
// Display Information
// ============================================================================

/// Represents a display's bounds in macOS global coordinate space
#[derive(Debug, Clone)]
pub struct DisplayBounds {
    pub origin_x: f64,
    pub origin_y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;

/// Get all displays with their actual bounds in macOS global coordinates.
/// This uses NSScreen directly because GPUI's display.bounds() doesn't return
/// correct origins for secondary displays.
#[cfg(target_os = "macos")]
pub fn get_macos_displays() -> Vec<DisplayBounds> {
    unsafe {
        let screens: id = msg_send![class!(NSScreen), screens];
        let count: usize = msg_send![screens, count];

        // Get primary screen height for coordinate flipping
        // macOS coordinates: Y=0 at bottom of primary screen
        let main_screen: id = msg_send![screens, firstObject];
        let main_frame: NSRect = msg_send![main_screen, frame];
        let primary_height = main_frame.size.height;

        let mut displays = Vec::with_capacity(count);

        for i in 0..count {
            let screen: id = msg_send![screens, objectAtIndex:i];
            let frame: NSRect = msg_send![screen, frame];

            // Convert from macOS bottom-left origin to top-left origin
            // macOS: y=0 at bottom, increasing upward
            // We want: y=0 at top, increasing downward
            let flipped_y = primary_height - frame.origin.y - frame.size.height;

            displays.push(DisplayBounds {
                origin_x: frame.origin.x,
                origin_y: flipped_y,
                width: frame.size.width,
                height: frame.size.height,
            });
        }

        displays
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_macos_displays() -> Vec<DisplayBounds> {
    // Fallback: return a single default display
    vec![DisplayBounds {
        origin_x: 0.0,
        origin_y: 0.0,
        width: 1920.0,
        height: 1080.0,
    }]
}

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
        let screens: id = msg_send![class!(NSScreen), screens];
        let main_screen: id = msg_send![screens, firstObject];
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
        let found = displays.iter().enumerate().find(|(idx, display)| {
            let contains = mouse_x >= display.origin_x
                && mouse_x < display.origin_x + display.width
                && mouse_y >= display.origin_y
                && mouse_y < display.origin_y + display.height;

            if contains {
                logging::log("POSITION", &format!("  -> Mouse is on display {}", idx));
            }
            contains
        });

        found.map(|(_, d)| d.clone())
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

// ============================================================================
// Screenshot Capture
// ============================================================================

/// Capture a screenshot of the app window using xcap for cross-platform support.
///
/// Returns a tuple of (png_data, width, height) on success.
/// The function:
/// 1. Uses xcap::Window::all() to enumerate windows
/// 2. Finds the Script Kit window by app name or title
/// 3. Captures the window directly to an image buffer
/// 4. Optionally scales down to 1x resolution if hi_dpi is false
/// 5. Encodes to PNG in memory (no temp files)
///
/// # Arguments
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
pub fn capture_app_screenshot(
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    struct Candidate {
        window: Window,
        title: String,
        width: u32,
        height: u32,
    }

    let mut candidates = Vec::new();
    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match our app window by name
        let is_our_window = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || title.contains("Script Kit");

        let is_minimized = window.is_minimized().unwrap_or(true);

        // Get window dimensions to filter out tiny windows (tooltips, list items, etc.)
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);

        // Only consider windows that are reasonably sized (at least 200x200)
        // This filters out tooltips, list items, icons, etc.
        let is_reasonable_size = width >= 200 && height >= 200;

        if is_our_window && !is_minimized && is_reasonable_size {
            candidates.push(Candidate {
                window,
                title,
                width,
                height,
            });
        }
    }

    // Sort by size (largest first) - the main window is typically the largest
    candidates.sort_by(|a, b| {
        let area_a = a.width as u64 * a.height as u64;
        let area_b = b.width as u64 * b.height as u64;
        area_b.cmp(&area_a)
    });

    let mut target = candidates
        .iter()
        .filter(|candidate| candidate.title.contains("Notes") || candidate.title.contains("AI"))
        .find(|candidate| candidate.window.is_focused().unwrap_or(false))
        .map(|candidate| candidate.window.clone());

    if target.is_none() {
        target = candidates
            .iter()
            .find(|candidate| candidate.title.contains("Notes") || candidate.title.contains("AI"))
            .map(|candidate| candidate.window.clone());
    }

    if target.is_none() {
        target = candidates
            .iter()
            .find(|candidate| candidate.window.is_focused().unwrap_or(false))
            .map(|candidate| candidate.window.clone());
    }

    let Some(window) =
        target.or_else(|| candidates.first().map(|candidate| candidate.window.clone()))
    else {
        return Err("Script Kit window not found".into());
    };

    let title = window.title().unwrap_or_else(|_| String::new());
    let app_name = window.app_name().unwrap_or_else(|_| String::new());

    tracing::debug!(
        app_name = %app_name,
        title = %title,
        hi_dpi = hi_dpi,
        "Found Script Kit window for screenshot"
    );

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    // Scale down to 1x if not hi_dpi mode (xcap captures at retina resolution on macOS)
    let (final_image, width, height) = if hi_dpi {
        (image, original_width, original_height)
    } else {
        // Scale down by 2x for 1x resolution
        let new_width = original_width / 2;
        let new_height = original_height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        tracing::debug!(
            original_width = original_width,
            original_height = original_height,
            new_width = new_width,
            new_height = new_height,
            "Scaled screenshot to 1x resolution"
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory (no temp files needed)
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    tracing::debug!(
        width = width,
        height = height,
        hi_dpi = hi_dpi,
        file_size = png_data.len(),
        "Screenshot captured with xcap"
    );

    Ok((png_data, width, height))
}

/// Capture a screenshot of a window by its title pattern.
///
/// Similar to `capture_app_screenshot` but allows specifying which window to capture
/// by matching the title. This is useful for secondary windows like the AI Chat window.
///
/// # Arguments
/// * `title_pattern` - A string that the window title must contain
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
///
/// # Returns
/// A tuple of (png_data, width, height) on success.
pub fn capture_window_by_title(
    title_pattern: &str,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match window by title pattern (must also be our app)
        let is_our_app = app_name.contains("script-kit-gpui") || app_name == "Script Kit";
        let title_matches = title.contains(title_pattern);
        let is_minimized = window.is_minimized().unwrap_or(true);

        if is_our_app && title_matches && !is_minimized {
            tracing::debug!(
                app_name = %app_name,
                title = %title,
                title_pattern = %title_pattern,
                hi_dpi = hi_dpi,
                "Found window matching title pattern for screenshot"
            );

            let image = window.capture_image()?;
            let original_width = image.width();
            let original_height = image.height();

            // Scale down to 1x if not hi_dpi mode
            let (final_image, width, height) = if hi_dpi {
                (image, original_width, original_height)
            } else {
                let new_width = original_width / 2;
                let new_height = original_height / 2;
                let resized = image::imageops::resize(
                    &image,
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,
                );
                tracing::debug!(
                    original_width = original_width,
                    original_height = original_height,
                    new_width = new_width,
                    new_height = new_height,
                    "Scaled screenshot to 1x resolution"
                );
                (resized, new_width, new_height)
            };

            // Encode to PNG in memory
            let mut png_data = Vec::new();
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

            tracing::debug!(
                width = width,
                height = height,
                hi_dpi = hi_dpi,
                file_size = png_data.len(),
                title_pattern = %title_pattern,
                "Screenshot captured for window by title"
            );

            return Ok((png_data, width, height));
        }
    }

    Err(format!("Window with title containing '{}' not found", title_pattern).into())
}

// ============================================================================
// Open Path with System Default
// ============================================================================

/// Open a path (file or folder) with the system default application.
/// On macOS: uses `open` command
/// On Linux: uses `xdg-open` command
/// On Windows: uses `cmd /C start` command
///
/// This can be used to open files, folders, URLs, or any path that the
/// system knows how to handle.
#[allow(dead_code)]
pub fn open_path_with_system_default(path: &str) {
    logging::log("UI", &format!("Opening path with system default: {}", path));
    let path_owned = path.to_string();

    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        {
            match std::process::Command::new("open").arg(&path_owned).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }

        #[cfg(target_os = "linux")]
        {
            match std::process::Command::new("xdg-open")
                .arg(&path_owned)
                .spawn()
            {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }

        #[cfg(target_os = "windows")]
        {
            match std::process::Command::new("cmd")
                .args(["/C", "start", "", &path_owned])
                .spawn()
            {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }
    });
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that ensure_move_to_active_space can be called without panicking.
    /// This is a characterization test - it verifies the function doesn't crash.
    /// On non-macOS, this is a no-op. On macOS without a window, it logs a warning.
    #[test]
    fn test_ensure_move_to_active_space_does_not_panic() {
        // Should not panic even without a window registered
        ensure_move_to_active_space();
    }

    /// Test that configure_as_floating_panel can be called without panicking.
    /// This is a characterization test - it verifies the function doesn't crash.
    /// On non-macOS, this is a no-op. On macOS without NSApp/keyWindow, it handles gracefully.
    #[test]
    fn test_configure_as_floating_panel_does_not_panic() {
        // Should not panic even without an app running
        configure_as_floating_panel();
    }

    /// Verify the macOS constants have the correct values.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_constants() {
        assert_eq!(NS_FLOATING_WINDOW_LEVEL, 3);
        assert_eq!(NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE, 2);
    }

    /// Test that both functions can be called in sequence.
    /// This mirrors the typical usage pattern in main.rs where both are called
    /// during window setup.
    #[test]
    fn test_functions_can_be_called_in_sequence() {
        // This is the typical call order in main.rs
        ensure_move_to_active_space();
        configure_as_floating_panel();
        // Should complete without panicking
    }

    /// Test that functions are idempotent - can be called multiple times safely.
    #[test]
    fn test_functions_are_idempotent() {
        for _ in 0..3 {
            ensure_move_to_active_space();
            configure_as_floating_panel();
        }
        // Should complete without panicking or causing issues
    }

    // =========================================================================
    // Mouse Position Tests
    // =========================================================================

    /// Test get_global_mouse_position returns valid coordinates or None.
    /// On macOS with display, returns Some((x, y)).
    /// On other platforms or without display, returns None.
    #[test]
    fn test_get_global_mouse_position_does_not_panic() {
        // Should not panic regardless of whether we can get the position
        let _ = get_global_mouse_position();
    }

    // =========================================================================
    // Display Information Tests
    // =========================================================================

    /// Test DisplayBounds struct creation and field access.
    #[test]
    fn test_display_bounds_struct() {
        let bounds = DisplayBounds {
            origin_x: 100.0,
            origin_y: 200.0,
            width: 1920.0,
            height: 1080.0,
        };

        assert_eq!(bounds.origin_x, 100.0);
        assert_eq!(bounds.origin_y, 200.0);
        assert_eq!(bounds.width, 1920.0);
        assert_eq!(bounds.height, 1080.0);
    }

    /// Test DisplayBounds Clone implementation.
    #[test]
    fn test_display_bounds_clone() {
        let original = DisplayBounds {
            origin_x: 0.0,
            origin_y: 0.0,
            width: 2560.0,
            height: 1440.0,
        };

        let cloned = original.clone();
        assert_eq!(cloned.width, 2560.0);
        assert_eq!(cloned.height, 1440.0);
    }

    /// Test get_macos_displays returns at least one display (or fallback).
    #[test]
    fn test_get_macos_displays_returns_at_least_one() {
        let displays = get_macos_displays();
        assert!(!displays.is_empty(), "Should return at least one display");
    }

    /// Test get_macos_displays returns displays with valid dimensions.
    #[test]
    fn test_get_macos_displays_valid_dimensions() {
        let displays = get_macos_displays();
        for display in displays {
            assert!(display.width > 0.0, "Display width must be positive");
            assert!(display.height > 0.0, "Display height must be positive");
        }
    }

    // =========================================================================
    // Window Movement Tests
    // =========================================================================

    /// Test move_first_window_to does not panic without a window.
    #[test]
    fn test_move_first_window_to_does_not_panic() {
        // Should not panic even without a registered window
        move_first_window_to(100.0, 100.0, 800.0, 600.0);
    }

    /// Test move_first_window_to_bounds wrapper function.
    #[test]
    fn test_move_first_window_to_bounds_does_not_panic() {
        use gpui::size;
        let bounds = Bounds {
            origin: point(px(100.0), px(100.0)),
            size: size(px(800.0), px(600.0)),
        };
        // Should not panic
        move_first_window_to_bounds(&bounds);
    }

    // =========================================================================
    // Eye-line Positioning Tests
    // =========================================================================

    /// Test calculate_eye_line_bounds returns valid bounds.
    #[test]
    fn test_calculate_eye_line_bounds_returns_valid() {
        use gpui::size;
        let window_size = size(px(750.0), px(500.0));
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Bounds should have the same size as input
        assert_eq!(bounds.size.width, window_size.width);
        assert_eq!(bounds.size.height, window_size.height);
    }

    /// Test eye-line calculation positions window in upper portion of screen.
    #[test]
    fn test_calculate_eye_line_bounds_upper_portion() {
        use gpui::size;
        let window_size = size(px(750.0), px(500.0));
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Get the display bounds for comparison
        let displays = get_macos_displays();
        if let Some(display) = displays.first() {
            let origin_y: f64 = bounds.origin.y.into();
            let display_height = display.height;

            // Eye-line should be in upper half of screen (top 50%)
            assert!(
                origin_y < display.origin_y + display_height * 0.5,
                "Window should be in upper half: origin_y={}, display_mid={}",
                origin_y,
                display.origin_y + display_height * 0.5
            );
        }
    }
}

</file>

<file path="src/window_control.rs">
//! Window Control module using macOS Accessibility APIs
//!
//! This module provides window management functionality including:
//! - Listing all visible windows with their properties
//! - Moving, resizing, minimizing, maximizing, and closing windows
//! - Tiling windows to predefined positions (halves, quadrants, fullscreen)
//!
//! ## Architecture
//!
//! Uses macOS Accessibility APIs (AXUIElement) to control windows across applications.
//! The accessibility framework allows querying and modifying window properties for any
//! application, provided the user has granted accessibility permissions.
//!
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility
//!

#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use core_graphics::display::{CGDisplay, CGRect};
use macos_accessibility_client::accessibility;
use std::ffi::c_void;
use tracing::{debug, info, instrument, warn};

// ============================================================================
// CoreFoundation FFI bindings
// ============================================================================

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

// ============================================================================
// ApplicationServices (Accessibility) FFI bindings
// ============================================================================

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> i32;
    fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> i32;
    fn AXValueCreate(value_type: i32, value: *const c_void) -> AXValueRef;
    fn AXValueGetValue(value: AXValueRef, value_type: i32, value_out: *mut c_void) -> bool;
    fn AXValueGetType(value: AXValueRef) -> i32;
}

// AXValue types
const kAXValueTypeCGPoint: i32 = 1;
const kAXValueTypeCGSize: i32 = 2;

// AXError codes
const kAXErrorSuccess: i32 = 0;
const kAXErrorAPIDisabled: i32 = -25211;
const kAXErrorNoValue: i32 = -25212;

type AXUIElementRef = *const c_void;
type AXValueRef = *const c_void;
type CFTypeRef = *const c_void;
type CFStringRef = *const c_void;
type CFArrayRef = *const c_void;

// ============================================================================
// CoreFoundation String FFI bindings
// ============================================================================

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetCString(
        string: CFStringRef,
        buffer: *mut i8,
        buffer_size: i64,
        encoding: u32,
    ) -> bool;
    fn CFStringGetLength(string: CFStringRef) -> i64;
    fn CFArrayGetCount(array: CFArrayRef) -> i64;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: i64) -> CFTypeRef;
    fn CFGetTypeID(cf: CFTypeRef) -> u64;
    fn CFStringGetTypeID() -> u64;
    fn CFNumberGetValue(number: CFTypeRef, number_type: i32, value_ptr: *mut c_void) -> bool;
}

const kCFStringEncodingUTF8: u32 = 0x08000100;
const kCFNumberSInt32Type: i32 = 3;

// ============================================================================
// AppKit (NSWorkspace/NSRunningApplication) FFI bindings
// ============================================================================

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // We'll use objc crate for AppKit access instead of direct FFI
}

// ============================================================================
// Public Types
// ============================================================================

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
    fn from_cg_rect(rect: CGRect) -> Self {
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
    /// Get the internal window reference for operations
    fn window_ref(&self) -> Option<AXUIElementRef> {
        self.ax_window.map(|ptr| ptr as AXUIElementRef)
    }
}

/// Tiling positions for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePosition {
    /// Left half of the screen
    LeftHalf,
    /// Right half of the screen
    RightHalf,
    /// Top half of the screen
    TopHalf,
    /// Bottom half of the screen
    BottomHalf,
    /// Top-left quadrant
    TopLeft,
    /// Top-right quadrant
    TopRight,
    /// Bottom-left quadrant
    BottomLeft,
    /// Bottom-right quadrant
    BottomRight,
    /// Fullscreen (covers entire display)
    Fullscreen,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a CFString from a Rust string
fn create_cf_string(s: &str) -> CFStringRef {
    unsafe {
        let c_str = std::ffi::CString::new(s).unwrap();
        CFStringCreateWithCString(std::ptr::null(), c_str.as_ptr(), kCFStringEncodingUTF8)
    }
}

/// Convert a CFString to a Rust String
fn cf_string_to_string(cf_string: CFStringRef) -> Option<String> {
    if cf_string.is_null() {
        return None;
    }

    unsafe {
        let length = CFStringGetLength(cf_string);
        if length <= 0 {
            return Some(String::new());
        }

        // Allocate buffer with extra space for UTF-8 expansion
        let buffer_size = (length * 4 + 1) as usize;
        let mut buffer: Vec<i8> = vec![0; buffer_size];

        if CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr(),
            buffer_size as i64,
            kCFStringEncodingUTF8,
        ) {
            let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr());
            c_str.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }
}

/// Release a CoreFoundation object
fn cf_release(cf: CFTypeRef) {
    if !cf.is_null() {
        unsafe {
            CFRelease(cf);
        }
    }
}

/// Get an attribute value from an AXUIElement
fn get_ax_attribute(element: AXUIElementRef, attribute: &str) -> Result<CFTypeRef> {
    let attr_str = create_cf_string(attribute);
    let mut value: CFTypeRef = std::ptr::null();

    let result =
        unsafe { AXUIElementCopyAttributeValue(element, attr_str, &mut value as *mut CFTypeRef) };

    cf_release(attr_str);

    match result {
        kAXErrorSuccess => Ok(value),
        kAXErrorAPIDisabled => bail!("Accessibility API is disabled"),
        kAXErrorNoValue => bail!("No value for attribute: {}", attribute),
        _ => bail!("Failed to get attribute {}: error {}", attribute, result),
    }
}

/// Set an attribute value on an AXUIElement
fn set_ax_attribute(element: AXUIElementRef, attribute: &str, value: CFTypeRef) -> Result<()> {
    let attr_str = create_cf_string(attribute);

    let result = unsafe { AXUIElementSetAttributeValue(element, attr_str, value) };

    cf_release(attr_str);

    match result {
        kAXErrorSuccess => Ok(()),
        kAXErrorAPIDisabled => bail!("Accessibility API is disabled"),
        _ => bail!("Failed to set attribute {}: error {}", attribute, result),
    }
}

/// Perform an action on an AXUIElement
fn perform_ax_action(element: AXUIElementRef, action: &str) -> Result<()> {
    let action_str = create_cf_string(action);

    let result = unsafe { AXUIElementPerformAction(element, action_str) };

    cf_release(action_str);

    match result {
        kAXErrorSuccess => Ok(()),
        kAXErrorAPIDisabled => bail!("Accessibility API is disabled"),
        _ => bail!("Failed to perform action {}: error {}", action, result),
    }
}

/// Get the position of a window
fn get_window_position(window: AXUIElementRef) -> Result<(i32, i32)> {
    let value = get_ax_attribute(window, "AXPosition")?;

    let mut point = core_graphics::geometry::CGPoint::new(0.0, 0.0);
    let success = unsafe {
        AXValueGetValue(
            value,
            kAXValueTypeCGPoint,
            &mut point as *mut _ as *mut c_void,
        )
    };

    cf_release(value);

    if success {
        Ok((point.x as i32, point.y as i32))
    } else {
        bail!("Failed to extract position value")
    }
}

/// Get the size of a window
fn get_window_size(window: AXUIElementRef) -> Result<(u32, u32)> {
    let value = get_ax_attribute(window, "AXSize")?;

    let mut size = core_graphics::geometry::CGSize::new(0.0, 0.0);
    let success = unsafe {
        AXValueGetValue(
            value,
            kAXValueTypeCGSize,
            &mut size as *mut _ as *mut c_void,
        )
    };

    cf_release(value);

    if success {
        Ok((size.width as u32, size.height as u32))
    } else {
        bail!("Failed to extract size value")
    }
}

/// Set the position of a window
fn set_window_position(window: AXUIElementRef, x: i32, y: i32) -> Result<()> {
    let point = core_graphics::geometry::CGPoint::new(x as f64, y as f64);
    let value = unsafe { AXValueCreate(kAXValueTypeCGPoint, &point as *const _ as *const c_void) };

    if value.is_null() {
        bail!("Failed to create AXValue for position");
    }

    let result = set_ax_attribute(window, "AXPosition", value);
    cf_release(value);
    result
}

/// Set the size of a window
fn set_window_size(window: AXUIElementRef, width: u32, height: u32) -> Result<()> {
    let size = core_graphics::geometry::CGSize::new(width as f64, height as f64);
    let value = unsafe { AXValueCreate(kAXValueTypeCGSize, &size as *const _ as *const c_void) };

    if value.is_null() {
        bail!("Failed to create AXValue for size");
    }

    let result = set_ax_attribute(window, "AXSize", value);
    cf_release(value);
    result
}

/// Get the string value of a window attribute
fn get_window_string_attribute(window: AXUIElementRef, attribute: &str) -> Option<String> {
    match get_ax_attribute(window, attribute) {
        Ok(value) => {
            // Check if it's a CFString
            let type_id = unsafe { CFGetTypeID(value) };
            let string_type_id = unsafe { CFStringGetTypeID() };

            let result = if type_id == string_type_id {
                cf_string_to_string(value as CFStringRef)
            } else {
                None
            };

            cf_release(value);
            result
        }
        Err(_) => None,
    }
}

/// Get the main display bounds
fn get_main_display_bounds() -> Bounds {
    let main_display = CGDisplay::main();
    let rect = main_display.bounds();
    Bounds::from_cg_rect(rect)
}

/// Get the display bounds for the display containing a point
fn get_display_bounds_at_point(_x: i32, _y: i32) -> Bounds {
    // For simplicity, we'll use the main display
    // A more complete implementation would find the display containing the point
    get_main_display_bounds()
}

// ============================================================================
// Window Cache for lookups
// ============================================================================

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Global window cache using OnceLock (std alternative to lazy_static)
static WINDOW_CACHE: OnceLock<Mutex<HashMap<u32, usize>>> = OnceLock::new();

/// Get or initialize the window cache
fn get_cache() -> &'static Mutex<HashMap<u32, usize>> {
    WINDOW_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_window(id: u32, window_ref: AXUIElementRef) {
    if let Ok(mut cache) = get_cache().lock() {
        cache.insert(id, window_ref as usize);
    }
}

fn get_cached_window(id: u32) -> Option<AXUIElementRef> {
    get_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(&id).map(|&ptr| ptr as AXUIElementRef))
}

fn clear_window_cache() {
    if let Ok(mut cache) = get_cache().lock() {
        cache.clear();
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Check if accessibility permissions are granted.
///
/// Window control operations require the application to have accessibility
/// permissions granted by the user.
///
/// # Returns
/// `true` if permission is granted, `false` otherwise.
#[instrument]
pub fn has_accessibility_permission() -> bool {
    let result = accessibility::application_is_trusted();
    debug!(granted = result, "Checked accessibility permission");
    result
}

/// Request accessibility permissions (opens System Preferences).
///
/// # Returns
/// `true` if permission is granted after the request, `false` otherwise.
#[instrument]
pub fn request_accessibility_permission() -> bool {
    info!("Requesting accessibility permission for window control");
    accessibility::application_is_trusted_with_prompt()
}

/// List all visible windows across all applications.
///
/// Returns a vector of `WindowInfo` structs containing window metadata.
/// Windows are filtered to only include visible, standard windows.
///
/// # Returns
/// A vector of window information structs.
///
/// # Errors
/// Returns error if accessibility permission is not granted.
///
#[instrument]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    if !has_accessibility_permission() {
        bail!("Accessibility permission required for window control");
    }

    // Clear the cache before listing
    clear_window_cache();

    let mut windows = Vec::new();

    // Get list of running applications using objc
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        let app_count: usize = msg_send![running_apps, count];

        for i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];

            // Check activation policy (skip background apps)
            let activation_policy: i64 = msg_send![app, activationPolicy];
            if activation_policy != 0 {
                // 0 = NSApplicationActivationPolicyRegular
                continue;
            }

            // Get app name
            let app_name: *mut Object = msg_send![app, localizedName];
            let app_name_str = if !app_name.is_null() {
                let utf8: *const i8 = msg_send![app_name, UTF8String];
                if !utf8.is_null() {
                    std::ffi::CStr::from_ptr(utf8)
                        .to_str()
                        .unwrap_or("Unknown")
                        .to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            // Get PID
            let pid: i32 = msg_send![app, processIdentifier];

            // Create AXUIElement for this application
            let ax_app = AXUIElementCreateApplication(pid);
            if ax_app.is_null() {
                continue;
            }

            // Get windows for this app
            if let Ok(windows_value) = get_ax_attribute(ax_app, "AXWindows") {
                let window_count = CFArrayGetCount(windows_value as CFArrayRef);

                for j in 0..window_count {
                    let ax_window = CFArrayGetValueAtIndex(windows_value as CFArrayRef, j);

                    // Get window title
                    let title = get_window_string_attribute(ax_window as AXUIElementRef, "AXTitle")
                        .unwrap_or_default();

                    // Skip windows without titles (often utility windows)
                    if title.is_empty() {
                        continue;
                    }

                    // Get window position and size
                    let (x, y) = get_window_position(ax_window as AXUIElementRef).unwrap_or((0, 0));
                    let (width, height) =
                        get_window_size(ax_window as AXUIElementRef).unwrap_or((0, 0));

                    // Skip very small windows (likely invisible or popups)
                    if width < 50 || height < 50 {
                        continue;
                    }

                    // Create a unique window ID: (pid << 16) | window_index
                    let window_id = ((pid as u32) << 16) | (j as u32);

                    // Cache the window reference
                    cache_window(window_id, ax_window as AXUIElementRef);

                    windows.push(WindowInfo {
                        id: window_id,
                        app: app_name_str.clone(),
                        title,
                        bounds: Bounds::new(x, y, width, height),
                        pid,
                        ax_window: Some(ax_window as usize),
                    });
                }

                // Don't release windows_value here - the AXUIElement owns it
            }

            // Don't release ax_app - we need it for the windows
        }
    }

    info!(window_count = windows.len(), "Listed windows");
    Ok(windows)
}

/// Get the PID of the application that owns the menu bar.
///
/// When Script Kit (an accessory/LSUIElement app) is active, it does NOT take
/// menu bar ownership. The previously active "regular" app still owns the menu bar.
/// This is exactly what we need for window actions - we want to act on the
/// window that was focused before Script Kit was shown.
///
/// # Returns
/// The process identifier (PID) of the menu bar owning application.
///
/// # Errors
/// Returns error if no menu bar owner is found or if the PID is invalid.
#[instrument]
pub fn get_menu_bar_owner_pid() -> Result<i32> {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let menu_owner: *mut Object = msg_send![workspace, menuBarOwningApplication];

        if menu_owner.is_null() {
            bail!("No menu bar owning application found");
        }

        let pid: i32 = msg_send![menu_owner, processIdentifier];

        if pid <= 0 {
            bail!("Invalid process identifier for menu bar owner");
        }

        // Log for debugging
        let name: *mut Object = msg_send![menu_owner, localizedName];
        let name_str = if !name.is_null() {
            let utf8: *const i8 = msg_send![name, UTF8String];
            if !utf8.is_null() {
                std::ffi::CStr::from_ptr(utf8).to_str().unwrap_or("unknown")
            } else {
                "unknown"
            }
        } else {
            "unknown"
        };

        info!(pid, app_name = name_str, "Got menu bar owner");
        Ok(pid)
    }
}

/// Get the frontmost window of the menu bar owning application.
///
/// This is the key function for window actions from Script Kit. When the user
/// executes "Tile Window Left" etc., we want to act on the window they were
/// using before invoking Script Kit, not Script Kit's own window.
///
/// Since Script Kit is an LSUIElement (accessory app), it doesn't take menu bar
/// ownership. The menu bar owner is the previously active app.
///
/// # Returns
/// The first (frontmost) window of the menu bar owning application, or None if
/// no windows are found.
#[instrument]
pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
    let target_pid = get_menu_bar_owner_pid()?;
    let windows = list_windows()?;

    let target_window = windows.into_iter().find(|w| w.pid == target_pid);

    if let Some(ref w) = target_window {
        info!(
            window_id = w.id,
            app = %w.app,
            title = %w.title,
            "Found frontmost window of previous app"
        );
    } else {
        warn!(target_pid, "No windows found for menu bar owner");
    }

    Ok(target_window)
}

/// Move a window to a new position.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `x` - The new X coordinate (screen pixels from left)
/// * `y` - The new Y coordinate (screen pixels from top)
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn move_window(window_id: u32, x: i32, y: i32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            // Try to refresh the cache
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    set_window_position(window, x, y)?;
    info!(window_id, x, y, "Moved window");
    Ok(())
}

/// Resize a window to new dimensions.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `width` - The new width in pixels
/// * `height` - The new height in pixels
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn resize_window(window_id: u32, width: u32, height: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    set_window_size(window, width, height)?;
    info!(window_id, width, height, "Resized window");
    Ok(())
}

/// Set the complete bounds (position and size) of a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `bounds` - The new bounds for the window
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn set_window_bounds(window_id: u32, bounds: Bounds) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Set position first, then size
    set_window_position(window, bounds.x, bounds.y)?;
    set_window_size(window, bounds.width, bounds.height)?;

    info!(
        window_id,
        x = bounds.x,
        y = bounds.y,
        width = bounds.width,
        height = bounds.height,
        "Set window bounds"
    );
    Ok(())
}

/// Minimize a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn minimize_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Use AXMinimized attribute to minimize
    let true_value = create_cf_string("1");
    let minimize_attr = create_cf_string("AXMinimized");

    // AXMinimized expects a CFBoolean, so we need to use the attribute differently
    // Actually, we should perform the press action on the minimize button
    // or set the AXMinimized attribute to true

    // Try setting AXMinimized directly with a boolean value
    unsafe {
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            static kCFBooleanTrue: CFTypeRef;
        }

        let result = AXUIElementSetAttributeValue(window, minimize_attr, kCFBooleanTrue);

        cf_release(minimize_attr);
        cf_release(true_value);

        if result != kAXErrorSuccess {
            bail!("Failed to minimize window: error {}", result);
        }
    }

    info!(window_id, "Minimized window");
    Ok(())
}

/// Maximize a window (fills the display without entering fullscreen mode).
///
/// This positions the window to fill the available display area (excluding
/// dock and menu bar) without entering macOS fullscreen mode.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn maximize_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get current position to determine which display the window is on
    let (current_x, current_y) = get_window_position(window).unwrap_or((0, 0));

    // Get the display bounds (accounting for menu bar and dock)
    let display_bounds = get_visible_display_bounds(current_x, current_y);

    // Set the window to fill the visible area
    set_window_position(window, display_bounds.x, display_bounds.y)?;
    set_window_size(window, display_bounds.width, display_bounds.height)?;

    info!(window_id, "Maximized window");
    Ok(())
}

/// Tile a window to a predefined position on the screen.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `position` - The tiling position (half, quadrant, or fullscreen)
///
/// # Errors
/// Returns error if window not found or operation fails.
///
#[instrument]
pub fn tile_window(window_id: u32, position: TilePosition) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get current position to determine which display the window is on
    let (current_x, current_y) = get_window_position(window).unwrap_or((0, 0));

    // Get the visible display bounds (accounting for menu bar and dock)
    let display = get_visible_display_bounds(current_x, current_y);

    let bounds = calculate_tile_bounds(&display, position);

    set_window_position(window, bounds.x, bounds.y)?;
    set_window_size(window, bounds.width, bounds.height)?;

    info!(window_id, ?position, "Tiled window");
    Ok(())
}

/// Close a window.
///
/// Note: This performs the close action on the window, which may prompt
/// the user to save unsaved changes depending on the application.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn close_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get the close button and press it
    if let Ok(close_button) = get_ax_attribute(window, "AXCloseButton") {
        perform_ax_action(close_button as AXUIElementRef, "AXPress")?;
        cf_release(close_button);
    } else {
        bail!("Window does not have a close button");
    }

    info!(window_id, "Closed window");
    Ok(())
}

/// Focus (bring to front) a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn focus_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Raise the window
    perform_ax_action(window, "AXRaise")?;

    // Also activate the owning application
    let pid = (window_id >> 16) as i32;

    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        let app_count: usize = msg_send![running_apps, count];

        for i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
            let app_pid: i32 = msg_send![app, processIdentifier];

            if app_pid == pid {
                let _: bool = msg_send![app, activateWithOptions: 1u64]; // NSApplicationActivateIgnoringOtherApps
                break;
            }
        }
    }

    info!(window_id, "Focused window");
    Ok(())
}

// ============================================================================
// Helper Functions for Display Bounds
// ============================================================================

/// Get the visible display bounds (excluding menu bar and dock) for the display
/// containing the given point.
///
/// Uses NSScreen.visibleFrame to get accurate bounds that account for:
/// - Menu bar (on main display)
/// - Dock (on any edge, any display)
/// - Notch area (on newer MacBooks)
fn get_visible_display_bounds(x: i32, y: i32) -> Bounds {
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

        // Find the screen containing the point
        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            // Get the full frame (in Cocoa coordinates - origin at bottom-left)
            let frame: CGRect = msg_send![screen, frame];

            // Convert point to Cocoa coordinates for comparison
            // Cocoa Y increases upward, CoreGraphics Y increases downward
            // For the main screen, Cocoa origin.y is 0 at bottom
            // We need to check if (x, y) in CG coords falls within this screen

            // Get the main screen height for coordinate conversion
            let main_screen: *mut Object = msg_send![nsscreen_class, mainScreen];
            let main_frame: CGRect = msg_send![main_screen, frame];
            let main_height = main_frame.size.height;

            // Convert CG y to Cocoa y
            let cocoa_y = main_height - y as f64;

            // Check if point is within this screen's frame
            if (x as f64) >= frame.origin.x
                && (x as f64) < frame.origin.x + frame.size.width
                && cocoa_y >= frame.origin.y
                && cocoa_y < frame.origin.y + frame.size.height
            {
                // Get the visible frame (excludes menu bar and dock)
                let visible_frame: CGRect = msg_send![screen, visibleFrame];

                // Convert Cocoa coordinates back to CoreGraphics coordinates
                // CG origin is at top-left of main screen
                // Cocoa origin.y is distance from bottom of main screen
                // CG y = main_height - (cocoa_y + height)
                let cg_y = main_height - (visible_frame.origin.y + visible_frame.size.height);

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
fn get_visible_display_bounds_fallback(x: i32, y: i32) -> Bounds {
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

/// Calculate the bounds for a tiling position within a display.
fn calculate_tile_bounds(display: &Bounds, position: TilePosition) -> Bounds {
    let half_width = display.width / 2;
    let half_height = display.height / 2;

    match position {
        TilePosition::LeftHalf => Bounds {
            x: display.x,
            y: display.y,
            width: half_width,
            height: display.height,
        },
        TilePosition::RightHalf => Bounds {
            x: display.x + half_width as i32,
            y: display.y,
            width: half_width,
            height: display.height,
        },
        TilePosition::TopHalf => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: half_height,
        },
        TilePosition::BottomHalf => Bounds {
            x: display.x,
            y: display.y + half_height as i32,
            width: display.width,
            height: half_height,
        },
        TilePosition::TopLeft => Bounds {
            x: display.x,
            y: display.y,
            width: half_width,
            height: half_height,
        },
        TilePosition::TopRight => Bounds {
            x: display.x + half_width as i32,
            y: display.y,
            width: half_width,
            height: half_height,
        },
        TilePosition::BottomLeft => Bounds {
            x: display.x,
            y: display.y + half_height as i32,
            width: half_width,
            height: half_height,
        },
        TilePosition::BottomRight => Bounds {
            x: display.x + half_width as i32,
            y: display.y + half_height as i32,
            width: half_width,
            height: half_height,
        },
        TilePosition::Fullscreen => *display,
    }
}

// ============================================================================
// Tests
// ============================================================================

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
    fn test_calculate_tile_bounds_left_half() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::LeftHalf);

        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 1055);
    }

    #[test]
    fn test_calculate_tile_bounds_right_half() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::RightHalf);

        assert_eq!(bounds.x, 960);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 1055);
    }

    #[test]
    fn test_calculate_tile_bounds_top_left() {
        let display = Bounds::new(0, 25, 1920, 1080);
        let bounds = calculate_tile_bounds(&display, TilePosition::TopLeft);

        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 540);
    }

    #[test]
    fn test_calculate_tile_bounds_fullscreen() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::Fullscreen);

        assert_eq!(bounds, display);
    }

    #[test]
    fn test_tile_position_equality() {
        assert_eq!(TilePosition::LeftHalf, TilePosition::LeftHalf);
        assert_ne!(TilePosition::LeftHalf, TilePosition::RightHalf);
    }

    #[test]
    fn test_permission_check_does_not_panic() {
        // This test verifies the permission check doesn't panic
        let _has_permission = has_accessibility_permission();
    }

    #[test]
    #[ignore] // Requires accessibility permission
    fn test_list_windows() {
        let windows = list_windows().expect("Should list windows");
        println!("Found {} windows:", windows.len());
        for window in &windows {
            println!(
                "  [{:08x}] {}: {} ({:?})",
                window.id, window.app, window.title, window.bounds
            );
        }
    }

    #[test]
    #[ignore] // Requires accessibility permission and a visible window
    fn test_tile_window_left_half() {
        let windows = list_windows().expect("Should list windows");
        if let Some(window) = windows.first() {
            tile_window(window.id, TilePosition::LeftHalf).expect("Should tile window");
            println!("Tiled '{}' to left half", window.title);
        } else {
            panic!("No windows found to test with");
        }
    }
}

</file>

<file path="src/panel.rs">
// macOS Panel Configuration Module
// This module configures GPUI windows as macOS floating panels
// that appear above other applications with native vibrancy effects
//
// Also provides placeholder configuration for input fields

#![allow(dead_code)]

/// Vibrancy configuration for GPUI window background appearance
///
/// GPUI supports three WindowBackgroundAppearance values:
/// - Opaque: Solid, no transparency
/// - Transparent: Fully transparent
/// - Blurred: macOS vibrancy effect (recommended for Spotlight/Raycast-like feel)
///
/// The actual vibrancy effect is achieved through:
/// 1. Setting `WindowBackgroundAppearance::Blurred` in WindowOptions (done in main.rs)
/// 2. Using semi-transparent background colors (controlled by theme opacity settings)
///
/// The blur shows through the transparent portions of the window background,
/// creating the native macOS vibrancy effect similar to Spotlight and Raycast.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WindowVibrancy {
    /// Solid, opaque background - no vibrancy effect
    Opaque,
    /// Transparent background without blur
    Transparent,
    /// macOS vibrancy/blur effect - the native feel
    /// This is the recommended setting for Spotlight/Raycast-like appearance
    #[default]
    Blurred,
}

impl WindowVibrancy {
    /// Check if this vibrancy setting enables the blur effect
    pub fn is_blurred(&self) -> bool {
        matches!(self, WindowVibrancy::Blurred)
    }

    /// Check if this vibrancy setting is fully opaque
    pub fn is_opaque(&self) -> bool {
        matches!(self, WindowVibrancy::Opaque)
    }
}

// ============================================================================
// Header Layout Constants (Reference: Main Menu)
// ============================================================================
// These constants define the canonical header layout used by the main script list.
// All prompts (ArgPrompt, EnvPrompt, etc.) should use these exact values to ensure
// visual consistency with the main menu search input.

/// Header horizontal padding (px) - matches main menu
pub const HEADER_PADDING_X: f32 = 16.0;

/// Header vertical padding (px) - matches main menu
/// NOTE: This is 8px, NOT 12px (design_spacing.padding_md). The main menu uses
/// a tighter vertical padding for a more compact header appearance.
pub const HEADER_PADDING_Y: f32 = 8.0;

/// Header gap between input and buttons (px) - matches main menu
pub const HEADER_GAP: f32 = 12.0;

// ============================================================================
// Input Placeholder Configuration
// ============================================================================

/// Default placeholder text for the main search input
pub const DEFAULT_PLACEHOLDER: &str = "Script Kit";

/// Configuration for input field placeholder behavior
///
/// When using this configuration:
/// - Cursor should be positioned at FAR LEFT (index 0) when input is empty
/// - Placeholder text appears dimmed/muted when no user input
/// - Placeholder disappears immediately when user starts typing
#[derive(Debug, Clone)]
pub struct PlaceholderConfig {
    /// The placeholder text to display when input is empty
    pub text: String,
    /// Whether cursor should appear at left (true) or right (false) of placeholder
    pub cursor_at_left: bool,
}

impl Default for PlaceholderConfig {
    fn default() -> Self {
        Self {
            text: DEFAULT_PLACEHOLDER.to_string(),
            cursor_at_left: true,
        }
    }
}

impl PlaceholderConfig {
    /// Create a new placeholder configuration with custom text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            cursor_at_left: true,
        }
    }

    /// Log when placeholder state changes (for observability)
    pub fn log_state_change(&self, is_showing_placeholder: bool) {
        crate::logging::log(
            "PLACEHOLDER",
            &format!(
                "Placeholder state changed: showing={}, text='{}', cursor_at_left={}",
                is_showing_placeholder, self.text, self.cursor_at_left
            ),
        );
    }

    /// Log cursor position on input focus (for observability)
    pub fn log_cursor_position(&self, position: usize, is_empty: bool) {
        crate::logging::log(
            "PLACEHOLDER",
            &format!(
                "Cursor position on focus: pos={}, input_empty={}, expected_left={}",
                position,
                is_empty,
                is_empty && self.cursor_at_left
            ),
        );
    }
}

// ============================================================================
// Cursor Styling Constants
// ============================================================================

/// Standard cursor width in pixels for text input fields
///
/// This matches the standard cursor width used in editor.rs and provides
/// visual consistency across all input fields.
pub const CURSOR_WIDTH: f32 = 2.0;

/// Horizontal gap between the cursor and adjacent text/placeholder, in pixels.
///
/// Keep this identical in empty and non-empty states to avoid horizontal shifting
/// when switching between placeholder and typed text.
pub const CURSOR_GAP_X: f32 = 2.0;

/// Cursor height for large text (.text_lg() / 18px font)
///
/// This value is calculated to align properly with GPUI's .text_lg() text rendering:
/// - GPUI's text_lg() uses ~18px font size
/// - With natural line height (~1.55), this gives ~28px line height
/// - Cursor should be 18px with 5px top/bottom spacing for vertical centering
///
/// NOTE: This value differs from `font_size_lg * line_height_normal` in design tokens
/// because GPUI's .text_lg() has different line-height than our token calculations.
/// Using this constant ensures proper cursor-text alignment.
pub const CURSOR_HEIGHT_LG: f32 = 18.0;

/// Cursor height for small text (.text_sm() / 12px font)
pub const CURSOR_HEIGHT_SM: f32 = 14.0;

/// Cursor height for medium text (.text_md() / 14px font)
pub const CURSOR_HEIGHT_MD: f32 = 16.0;

/// Vertical margin for cursor centering within text line
///
/// Apply this as `.my(px(CURSOR_MARGIN_Y))` to vertically center the cursor
/// within its text line. This follows the editor.rs pattern.
pub const CURSOR_MARGIN_Y: f32 = 2.0;

/// Configuration for input cursor styling
///
/// Use this struct to ensure consistent cursor appearance across all input fields.
/// The cursor should:
/// 1. Use a fixed height matching the text size (not calculated from design tokens)
/// 2. Use vertical margin for centering within the line
/// 3. Be rendered as an always-present div to prevent layout shift, with bg toggled
#[derive(Debug, Clone, Copy)]
pub struct CursorStyle {
    /// Cursor width in pixels
    pub width: f32,
    /// Cursor height in pixels (should match text size, not line height)
    pub height: f32,
    /// Vertical margin for centering
    pub margin_y: f32,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::large()
    }
}

impl CursorStyle {
    /// Cursor style for large text (.text_lg())
    pub const fn large() -> Self {
        Self {
            width: CURSOR_WIDTH,
            height: CURSOR_HEIGHT_LG,
            margin_y: CURSOR_MARGIN_Y,
        }
    }

    /// Cursor style for medium text (.text_md())
    pub const fn medium() -> Self {
        Self {
            width: CURSOR_WIDTH,
            height: CURSOR_HEIGHT_MD,
            margin_y: CURSOR_MARGIN_Y,
        }
    }

    /// Cursor style for small text (.text_sm())
    pub const fn small() -> Self {
        Self {
            width: CURSOR_WIDTH,
            height: CURSOR_HEIGHT_SM,
            margin_y: CURSOR_MARGIN_Y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_vibrancy() {
        assert_eq!(WindowVibrancy::default(), WindowVibrancy::Blurred);
    }

    #[test]
    fn test_vibrancy_is_blurred() {
        assert!(WindowVibrancy::Blurred.is_blurred());
        assert!(!WindowVibrancy::Opaque.is_blurred());
        assert!(!WindowVibrancy::Transparent.is_blurred());
    }

    #[test]
    fn test_vibrancy_is_opaque() {
        assert!(WindowVibrancy::Opaque.is_opaque());
        assert!(!WindowVibrancy::Blurred.is_opaque());
        assert!(!WindowVibrancy::Transparent.is_opaque());
    }

    // Placeholder configuration tests

    #[test]
    fn test_default_placeholder_text() {
        assert_eq!(DEFAULT_PLACEHOLDER, "Script Kit");
    }

    #[test]
    fn test_placeholder_config_default() {
        let config = PlaceholderConfig::default();
        assert_eq!(config.text, "Script Kit");
        assert!(config.cursor_at_left);
    }

    #[test]
    fn test_placeholder_config_new() {
        let config = PlaceholderConfig::new("Custom Placeholder");
        assert_eq!(config.text, "Custom Placeholder");
        assert!(config.cursor_at_left);
    }

    #[test]
    fn test_placeholder_cursor_at_left_by_default() {
        // Verify that cursor_at_left is true by default
        // This ensures cursor appears at FAR LEFT when input is empty
        let config = PlaceholderConfig::default();
        assert!(
            config.cursor_at_left,
            "Cursor should be at left by default for proper placeholder behavior"
        );
    }

    // Cursor styling tests

    #[test]
    fn test_cursor_width_constant() {
        assert_eq!(CURSOR_WIDTH, 2.0);
    }

    #[test]
    fn test_cursor_height_lg_matches_text_lg() {
        // CURSOR_HEIGHT_LG should be 18px to match GPUI's .text_lg() font size
        // This ensures proper vertical alignment of cursor with text
        assert_eq!(CURSOR_HEIGHT_LG, 18.0);
    }

    #[test]
    fn test_cursor_heights_proportional() {
        // Cursor heights should be proportional to text sizes
        // Use const blocks to satisfy clippy::assertions_on_constants
        const _: () = {
            assert!(CURSOR_HEIGHT_SM < CURSOR_HEIGHT_MD);
        };
        const _: () = {
            assert!(CURSOR_HEIGHT_MD < CURSOR_HEIGHT_LG);
        };
    }

    #[test]
    fn test_cursor_style_default_is_large() {
        let style = CursorStyle::default();
        assert_eq!(style.height, CURSOR_HEIGHT_LG);
        assert_eq!(style.width, CURSOR_WIDTH);
    }

    #[test]
    fn test_cursor_style_constructors() {
        let large = CursorStyle::large();
        assert_eq!(large.height, CURSOR_HEIGHT_LG);

        let medium = CursorStyle::medium();
        assert_eq!(medium.height, CURSOR_HEIGHT_MD);

        let small = CursorStyle::small();
        assert_eq!(small.height, CURSOR_HEIGHT_SM);
    }

    #[test]
    fn test_cursor_margin_constant() {
        // Margin should be 2px for proper vertical centering
        assert_eq!(CURSOR_MARGIN_Y, 2.0);
    }
}

</file>

<file path="src/frontmost_app_tracker.rs">
//! Frontmost Application Tracker
//!
//! Tracks the "last real application" that was active before Script Kit.
//! This module provides a global, always-updated view of what app the user
//! was working in, which is useful for:
//!
//! - **Menu Bar Actions**: Get menu items from the app the user was in
//! - **Window Tiling**: Tile/move windows of the previous app
//! - **Context Actions**: Any action that should target "the app I was just using"
//!
//! ## Architecture
//!
//! A background observer watches for `NSWorkspaceDidActivateApplicationNotification`.
//! When an app activates:
//! - If it's NOT Script Kit → update the tracked "last real app"
//! - If it IS Script Kit → ignore (keep tracking the previous app)
//!
//! This means when Script Kit opens, we already know which app was active,
//! with no race conditions or timing issues.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::frontmost_app_tracker::{start_tracking, get_last_real_app, get_cached_menu_items};
//!
//! // Start tracking (call once at app startup)
//! start_tracking();
//!
//! // Get the last real app info
//! if let Some(app) = get_last_real_app() {
//!     println!("Last app: {} ({})", app.name, app.bundle_id);
//! }
//!
//! // Get cached menu items (pre-fetched in background)
//! let menu_items = get_cached_menu_items();
//! ```

use crate::logging;
use crate::menu_bar::{get_menu_bar_for_pid, MenuBarItem};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

/// Information about a tracked application
#[derive(Debug, Clone)]
pub struct TrackedApp {
    /// Process ID
    pub pid: i32,
    /// Bundle identifier (e.g., "com.google.Chrome")
    pub bundle_id: String,
    /// Localized display name (e.g., "Google Chrome")
    pub name: String,
}

/// Global state for the frontmost app tracker
#[derive(Default)]
struct TrackerState {
    /// The last "real" application (not Script Kit)
    last_real_app: Option<TrackedApp>,
    /// Cached menu bar items for the last real app
    cached_menu_items: Vec<MenuBarItem>,
    /// Whether menu items are currently being fetched
    fetching_menu: bool,
}

/// Global tracker state, protected by RwLock for concurrent access
static TRACKER_STATE: LazyLock<RwLock<TrackerState>> =
    LazyLock::new(|| RwLock::new(TrackerState::default()));

/// Whether tracking has been started
static TRACKING_STARTED: AtomicBool = AtomicBool::new(false);

/// Our own bundle ID to filter out
const SCRIPT_KIT_BUNDLE_ID: &str = "dev.scriptkit.scriptkit";

/// Start the background frontmost app tracker.
///
/// This should be called once at application startup. It sets up an
/// NSWorkspace observer to watch for application activation events.
///
/// Safe to call multiple times - subsequent calls are no-ops.
#[cfg(target_os = "macos")]
pub fn start_tracking() {
    if TRACKING_STARTED.swap(true, Ordering::SeqCst) {
        // Already started
        return;
    }

    logging::log("APP", "Starting frontmost app tracker");

    // Capture initial state - get current menu bar owner
    capture_current_frontmost_app();

    // Set up NSWorkspace observer for app activation
    std::thread::spawn(|| {
        setup_workspace_observer();
    });
}

#[cfg(not(target_os = "macos"))]
pub fn start_tracking() {
    // No-op on non-macOS platforms
    logging::log(
        "APP",
        "Frontmost app tracking not available on this platform",
    );
}

/// Get the last "real" application that was active before Script Kit.
///
/// Returns `None` if no app has been tracked yet (e.g., Script Kit just launched).
pub fn get_last_real_app() -> Option<TrackedApp> {
    TRACKER_STATE.read().last_real_app.clone()
}

/// Get the cached menu bar items for the last real app.
///
/// These are pre-fetched in the background when the app becomes active,
/// so they're available immediately when Script Kit opens.
///
/// Returns an empty Vec if no menu items are cached.
pub fn get_cached_menu_items() -> Vec<MenuBarItem> {
    TRACKER_STATE.read().cached_menu_items.clone()
}

/// Check if menu items are currently being fetched in the background.
#[allow(dead_code)] // Public API for future use
pub fn is_fetching_menu() -> bool {
    TRACKER_STATE.read().fetching_menu
}

/// Capture the current frontmost app (used at startup and for manual refresh)
#[cfg(target_os = "macos")]
fn capture_current_frontmost_app() {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let workspace_class = match Class::get("NSWorkspace") {
            Some(c) => c,
            None => return,
        };

        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];

        // Use menuBarOwningApplication since Script Kit is LSUIElement
        let app: *mut Object = msg_send![workspace, menuBarOwningApplication];

        if app.is_null() {
            return;
        }

        let bundle_id = get_nsstring(msg_send![app, bundleIdentifier]);
        let name = get_nsstring(msg_send![app, localizedName]);
        let pid: i32 = msg_send![app, processIdentifier];

        if let Some(bundle_id) = bundle_id {
            // Don't track Script Kit itself
            if bundle_id == SCRIPT_KIT_BUNDLE_ID {
                return;
            }

            let tracked = TrackedApp {
                pid,
                bundle_id: bundle_id.clone(),
                name: name.unwrap_or_else(|| bundle_id.clone()),
            };

            logging::log(
                "APP",
                &format!(
                    "Initial frontmost app: {} ({}) PID {}",
                    tracked.name, tracked.bundle_id, tracked.pid
                ),
            );

            // Update state
            {
                let mut state = TRACKER_STATE.write();
                state.last_real_app = Some(tracked.clone());
            }

            // Fetch menu items in background
            fetch_menu_items_async(tracked.pid, tracked.bundle_id);
        }
    }
}

/// Set up the NSWorkspace notification observer
#[cfg(target_os = "macos")]
fn setup_workspace_observer() {
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{msg_send, sel, sel_impl};
    use std::os::raw::c_void;

    unsafe {
        // Create a custom class to receive notifications
        let superclass = Class::get("NSObject").unwrap();

        let mut decl = match ClassDecl::new("ScriptKitAppObserver", superclass) {
            Some(d) => d,
            None => {
                // Class might already exist from previous call
                logging::log("WARN", "AppObserver class already exists");
                return;
            }
        };

        // Add the notification handler method
        extern "C" fn handle_app_activation(_this: &Object, _sel: Sel, notification: *mut Object) {
            unsafe {
                if notification.is_null() {
                    return;
                }

                // Get userInfo dictionary
                let user_info: *mut Object = msg_send![notification, userInfo];
                if user_info.is_null() {
                    return;
                }

                // Get the NSRunningApplication from userInfo
                let key = objc_nsstring("NSWorkspaceApplicationKey");
                let app: *mut Object = msg_send![user_info, objectForKey: key];

                if app.is_null() {
                    return;
                }

                let bundle_id = get_nsstring(msg_send![app, bundleIdentifier]);
                let name = get_nsstring(msg_send![app, localizedName]);
                let pid: i32 = msg_send![app, processIdentifier];

                if let Some(bundle_id) = bundle_id {
                    // Skip Script Kit itself
                    if bundle_id == SCRIPT_KIT_BUNDLE_ID
                        || bundle_id.contains("scriptkit")
                        || bundle_id.contains("script-kit")
                    {
                        logging::log("APP", "App activated: Script Kit (ignoring)");
                        return;
                    }

                    let tracked = TrackedApp {
                        pid,
                        bundle_id: bundle_id.clone(),
                        name: name.unwrap_or_else(|| bundle_id.clone()),
                    };

                    logging::log(
                        "APP",
                        &format!(
                            "App activated: {} ({}) PID {}",
                            tracked.name, tracked.bundle_id, tracked.pid
                        ),
                    );

                    // Check if this is a different app than currently tracked
                    let should_update = {
                        let state = TRACKER_STATE.read();
                        state
                            .last_real_app
                            .as_ref()
                            .map(|a| a.bundle_id != tracked.bundle_id)
                            .unwrap_or(true)
                    };

                    if should_update {
                        // Update state
                        {
                            let mut state = TRACKER_STATE.write();
                            state.last_real_app = Some(tracked.clone());
                            state.cached_menu_items.clear(); // Clear old cache
                        }

                        // Fetch menu items in background
                        fetch_menu_items_async(tracked.pid, tracked.bundle_id);
                    }
                }
            }
        }

        decl.add_method(
            sel!(handleAppActivation:),
            handle_app_activation as extern "C" fn(&Object, Sel, *mut Object),
        );

        let observer_class = decl.register();

        // Create an instance of our observer
        let observer: *mut Object = msg_send![observer_class, alloc];
        let observer: *mut Object = msg_send![observer, init];

        // Get the notification center
        let workspace_class = Class::get("NSWorkspace").unwrap();
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let notification_center: *mut Object = msg_send![workspace, notificationCenter];

        // Register for NSWorkspaceDidActivateApplicationNotification
        let notification_name = objc_nsstring("NSWorkspaceDidActivateApplicationNotification");

        let _: () = msg_send![
            notification_center,
            addObserver: observer
            selector: sel!(handleAppActivation:)
            name: notification_name
            object: std::ptr::null::<c_void>()
        ];

        logging::log("APP", "NSWorkspace observer registered for app activation");

        // Run the run loop to receive notifications
        // This thread will run forever, receiving notifications
        let run_loop: *mut Object = msg_send![Class::get("NSRunLoop").unwrap(), currentRunLoop];
        let _: () = msg_send![run_loop, run];
    }
}

/// Fetch menu items asynchronously for the given app
#[cfg(target_os = "macos")]
fn fetch_menu_items_async(pid: i32, bundle_id: String) {
    // Mark as fetching
    {
        let mut state = TRACKER_STATE.write();
        state.fetching_menu = true;
    }

    std::thread::spawn(move || {
        let start = std::time::Instant::now();

        match get_menu_bar_for_pid(pid) {
            Ok(items) => {
                let elapsed = start.elapsed();
                let count = items.len();

                logging::log(
                    "APP",
                    &format!(
                        "Pre-fetched {} menu items for {} in {:.2}ms",
                        count,
                        bundle_id,
                        elapsed.as_secs_f64() * 1000.0
                    ),
                );

                // Update cache
                let mut state = TRACKER_STATE.write();

                // Only update if still tracking the same app
                if state.last_real_app.as_ref().map(|a| &a.bundle_id) == Some(&bundle_id) {
                    state.cached_menu_items = items;
                }
                state.fetching_menu = false;
            }
            Err(e) => {
                logging::log(
                    "WARN",
                    &format!("Failed to pre-fetch menu items for {}: {}", bundle_id, e),
                );

                let mut state = TRACKER_STATE.write();
                state.fetching_menu = false;
            }
        }
    });
}

/// Helper to convert NSString to Rust String
#[cfg(target_os = "macos")]
unsafe fn get_nsstring(nsstring: *mut objc::runtime::Object) -> Option<String> {
    use objc::{msg_send, sel, sel_impl};

    if nsstring.is_null() {
        return None;
    }

    let utf8: *const i8 = msg_send![nsstring, UTF8String];
    if utf8.is_null() {
        return None;
    }

    std::ffi::CStr::from_ptr(utf8)
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

/// Helper to create an NSString from a Rust string
#[cfg(target_os = "macos")]
unsafe fn objc_nsstring(s: &str) -> *mut objc::runtime::Object {
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    let nsstring_class = Class::get("NSString").unwrap();
    let cstr = std::ffi::CString::new(s).unwrap();
    msg_send![nsstring_class, stringWithUTF8String: cstr.as_ptr()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_state_default() {
        let state = TrackerState::default();
        assert!(state.last_real_app.is_none());
        assert!(state.cached_menu_items.is_empty());
        assert!(!state.fetching_menu);
    }

    #[test]
    fn test_tracked_app_clone() {
        let app = TrackedApp {
            pid: 123,
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        };
        let cloned = app.clone();
        assert_eq!(cloned.pid, 123);
        assert_eq!(cloned.bundle_id, "com.test.app");
        assert_eq!(cloned.name, "Test App");
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 4 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~22.9K (22,926 exact)
  Total Chars: 105,321 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
      9.0K - src/window_control.rs
      8.6K - src/platform.rs
      3.0K - src/frontmost_app_tracker.rs
      2.4K - src/panel.rs
