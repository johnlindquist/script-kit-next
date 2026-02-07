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
    if require_main_thread("hide_main_window") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // orderOut: is a standard NSWindow method; nil sender is valid.
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

/// Show the main window WITHOUT activating the application.
///
/// This is critical for floating panel behavior - the window should appear
/// and be able to receive keyboard input, but the previously focused app
/// should remain the "active" app at the OS level. This allows features like
/// "copy selected text from previous app" to still work.
///
/// # macOS Behavior
///
/// For PopUp windows (NSPanel with NonactivatingPanel style), uses
/// `orderFrontRegardless` + `makeKeyWindow` to show the window and give it
/// keyboard focus without activating the application.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn show_main_window_without_activation() {
    if require_main_thread("show_main_window_without_activation") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // orderFrontRegardless and makeKeyWindow are standard NSWindow methods.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "show_main_window_without_activation: Main window not registered",
                );
                return;
            }
        };

        // orderFrontRegardless brings window to front without activating the app
        let _: () = msg_send![window, orderFrontRegardless];

        // Make the window key so it can receive keyboard input
        // For NSPanel with NonactivatingPanel style (PopUp windows), this works
        // without activating the application
        let _: () = msg_send![window, makeKeyWindow];

        logging::log(
            "PANEL",
            "Main window shown without activation (orderFrontRegardless + makeKeyWindow)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn show_main_window_without_activation() {
    logging::log(
        "PANEL",
        "show_main_window_without_activation: Not implemented on this platform",
    );
}

/// Activate the main window and bring it to front.
///
/// This makes the main window the key window and activates the application.
/// Used when returning focus to the main window after closing overlays like the actions popup.
#[cfg(target_os = "macos")]
pub fn activate_main_window() {
    if require_main_thread("activate_main_window") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid.
    // activateIgnoringOtherApps: and makeKeyAndOrderFront: are standard methods.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log("PANEL", "activate_main_window: Main window not registered");
                return;
            }
        };

        // Get the NSApplication
        let app: id = NSApp();

        // Activate the application, ignoring other apps
        let _: () = msg_send![app, activateIgnoringOtherApps: true];

        // Make our window key and bring it to front
        let _: () = msg_send![window, makeKeyAndOrderFront: nil];

        logging::log(
            "PANEL",
            "Main window activated (activateIgnoringOtherApps + makeKeyAndOrderFront)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn activate_main_window() {
    logging::log(
        "PANEL",
        "activate_main_window: Not implemented on this platform",
    );
}

// ============================================================================
// Share Sheet (macOS)
// ============================================================================

/// Content for the macOS share sheet.
#[derive(Debug)]
pub enum ShareSheetItem {
    Text(String),
    ImagePng(Vec<u8>),
}

/// Show the macOS share sheet anchored to the main window contentView.
#[cfg(target_os = "macos")]
pub fn show_share_sheet(item: ShareSheetItem) {
    if require_main_thread("show_share_sheet") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // All Objective-C objects are checked for nil before use.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log("PANEL", "show_share_sheet: Main window not registered");
                return;
            }
        };

        let content_view: id = msg_send![window, contentView];
        if content_view == nil {
            logging::log("PANEL", "show_share_sheet: contentView is nil");
            return;
        }

        let share_item: id = match item {
            ShareSheetItem::Text(text) => {
                let ns_string = CocoaNSString::alloc(nil).init_str(&text);
                if ns_string == nil {
                    logging::log("PANEL", "show_share_sheet: Failed to create NSString");
                    return;
                }
                ns_string
            }
            ShareSheetItem::ImagePng(png_bytes) => {
                if png_bytes.is_empty() {
                    logging::log("PANEL", "show_share_sheet: Empty PNG data");
                    return;
                }

                let data: id = msg_send![class!(NSData), dataWithBytes: png_bytes.as_ptr() length: png_bytes.len()];
                if data == nil {
                    logging::log("PANEL", "show_share_sheet: Failed to create NSData");
                    return;
                }

                let image: id = msg_send![class!(NSImage), alloc];
                let image: id = msg_send![image, initWithData: data];
                if image == nil {
                    logging::log("PANEL", "show_share_sheet: Failed to create NSImage");
                    return;
                }
                image
            }
        };

        let items: id = msg_send![class!(NSArray), arrayWithObject: share_item];
        if items == nil {
            logging::log("PANEL", "show_share_sheet: Failed to create NSArray");
            return;
        }

        let picker: id = msg_send![class!(NSSharingServicePicker), alloc];
        let picker: id = msg_send![picker, initWithItems: items];
        if picker == nil {
            logging::log(
                "PANEL",
                "show_share_sheet: Failed to create NSSharingServicePicker",
            );
            return;
        }

        let bounds: NSRect = msg_send![content_view, bounds];
        let preferred_edge: i64 = 1; // NSMinYEdge
        let _: () = msg_send![
            picker,
            showRelativeToRect: bounds
            ofView: content_view
            preferredEdge: preferred_edge
        ];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn show_share_sheet(_item: ShareSheetItem) {
    logging::log(
        "PANEL",
        "show_share_sheet: Not implemented on this platform",
    );
}

/// Get the current main window bounds in canonical top-left coordinates.
/// Returns (x, y, width, height) or None if window not available.
#[cfg(target_os = "macos")]
pub fn get_main_window_bounds() -> Option<(f64, f64, f64, f64)> {
    if require_main_thread("get_main_window_bounds") {
        return None;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // frame returns a value type (NSRect), no pointer dereference needed.
    unsafe {
        let window = window_manager::get_main_window()?;
        let frame: NSRect = msg_send![window, frame];

        // Get primary screen height for coordinate conversion
        let primary_height = primary_screen_height()?;

        // Convert from AppKit bottom-left origin to our top-left canonical space
        let top_left_y = flip_y(primary_height, frame.origin.y, frame.size.height);

        Some((
            frame.origin.x,
            top_left_y,
            frame.size.width,
            frame.size.height,
        ))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_main_window_bounds() -> Option<(f64, f64, f64, f64)> {
    None
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
    if require_main_thread("is_app_active") {
        return false;
    }
    // SAFETY: Main thread verified. NSApp() is always valid after app launch.
    // isActive returns a BOOL value type.
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

// ============================================================================
// Focus State Cache (avoids FFI calls on every render frame)
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

/// Baseline instant for relative time calculations
static FOCUS_CACHE_BASELINE: OnceLock<Instant> = OnceLock::new();
/// Cached focus state to avoid repeated FFI calls
static FOCUS_CACHE_VALUE: AtomicBool = AtomicBool::new(false);
/// Timestamp (millis since baseline) when focus was last checked
static FOCUS_CACHE_TIME: AtomicU64 = AtomicU64::new(0);
/// Cache TTL in milliseconds (16ms = ~1 frame at 60fps)
const FOCUS_CACHE_TTL_MS: u64 = 16;

/// Check if the main window is currently focused (key window).
///
/// This is used to detect focus loss even when the app remains active
/// (e.g., when switching focus to Notes/AI windows).
///
/// **Performance**: Uses a 16ms cache to avoid repeated FFI calls during
/// render. Multiple calls within the same frame return the cached value.
#[cfg(target_os = "macos")]
pub fn is_main_window_focused() -> bool {
    // Get or create baseline instant
    let baseline = FOCUS_CACHE_BASELINE.get_or_init(Instant::now);
    let now_ms = baseline.elapsed().as_millis() as u64;

    let last_check = FOCUS_CACHE_TIME.load(Ordering::Relaxed);
    if now_ms.saturating_sub(last_check) < FOCUS_CACHE_TTL_MS {
        return FOCUS_CACHE_VALUE.load(Ordering::Relaxed);
    }

    // Cache expired, do actual FFI call
    let is_focused = is_main_window_focused_uncached();

    // Update cache
    FOCUS_CACHE_VALUE.store(is_focused, Ordering::Relaxed);
    FOCUS_CACHE_TIME.store(now_ms, Ordering::Relaxed);

    is_focused
}

/// Uncached version that always makes the FFI call
#[cfg(target_os = "macos")]
fn is_main_window_focused_uncached() -> bool {
    if require_main_thread("is_main_window_focused_uncached") {
        return false;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // isKeyWindow returns a BOOL value type.
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

/// Invalidate the focus cache (call when focus changes are expected)
#[allow(dead_code)]
pub fn invalidate_focus_cache() {
    FOCUS_CACHE_TIME.store(0, Ordering::Relaxed);
}

