// ============================================================================
// Main Window Visibility Control
// ============================================================================

#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;

/// Hide the main window without hiding the entire app (synchronous, low-level).
///
/// # Reentrancy Danger — Do NOT call from GPUI callbacks
///
/// `orderOut:` on a key window synchronously triggers macOS's
/// `window_did_change_key_status` callback, which re-enters GPUI's `App`
/// `RefCell`. If the `RefCell` is already borrowed (inside any listener,
/// update, render, or entity callback), this causes a `RefCell already
/// borrowed` panic that aborts the process.
///
/// **Always use [`defer_hide_main_window`] instead** when inside any GPUI
/// borrow context. The only valid direct callers are the deferred wrapper and
/// code that is provably outside any GPUI borrow (e.g. a raw `dispatch_async`
/// block).
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
fn hide_main_window() {
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
fn hide_main_window() {
    // No-op on non-macOS platforms
}

/// Hide the main window, deferring the ObjC call to the next event-loop tick.
///
/// This is the **only safe way** to hide the main window from inside any GPUI
/// callback (listener, update, render, entity method, etc.).
///
/// Internally, `cx.spawn()` queues the work on the foreground executor.  When
/// the closure runs, all current `RefCell` borrows have been released, so the
/// macOS `window_did_change_key_status` callback can safely re-enter GPUI.
pub fn defer_hide_main_window(cx: &mut gpui::App) {
    cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
        hide_main_window();
    })
    .detach();
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

/// Show the main window WITHOUT activating the app and WITHOUT making it key.
///
/// This makes the window visible but does not steal keyboard focus from whatever
/// surface currently has it.  Use this when another surface (e.g., Notes or
/// Detached AI Chat) should remain the key window.
///
/// # macOS Behavior
///
/// Uses `orderFrontRegardless` only — no `makeKeyWindow` call.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn show_main_window_background() {
    if require_main_thread("show_main_window_background") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // orderFrontRegardless is a standard NSWindow method.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "show_main_window_background: Main window not registered",
                );
                return;
            }
        };

        // orderFrontRegardless brings the window to front without activating the app.
        // Crucially, we do NOT call makeKeyWindow so keyboard focus stays with
        // whatever surface currently owns it.
        let _: () = msg_send![window, orderFrontRegardless];

        logging::log(
            "PANEL",
            "Main window shown in background (orderFrontRegardless only, no makeKeyWindow)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn show_main_window_background() {
    logging::log(
        "PANEL",
        "show_main_window_background: Not implemented on this platform",
    );
}

/// Conceal the main window — hide it visually WITHOUT resetting AppView or canceling prompts.
///
/// Unlike `hide_main_window` (used by `defer_hide_main_window`), this only issues `orderOut:`
/// to remove the window from the screen. It preserves all internal state so the window can
/// be revealed later with its content intact. Use this for temporary flows like dictation
/// where the main window needs to get out of the way briefly.
///
/// # Reentrancy Danger
///
/// Same as `hide_main_window` — `orderOut:` triggers `window_did_change_key_status`.
/// **Always use [`defer_conceal_main_window`] from inside GPUI callbacks.**
///
/// # macOS Behavior
///
/// Uses NSWindow `orderOut:` to remove the window from the screen.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn conceal_main_window() {
    if require_main_thread("conceal_main_window") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // orderOut: is a standard NSWindow method; nil sender is valid.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "conceal_main_window: Main window not registered, nothing to conceal",
                );
                return;
            }
        };

        let _: () = msg_send![window, orderOut:nil];

        logging::log("PANEL", "Main window concealed via orderOut: (state preserved)");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn conceal_main_window() {
    // No-op on non-macOS platforms
}

/// Conceal the main window, deferring the ObjC call to the next event-loop tick.
///
/// This is the safe way to conceal the main window from inside any GPUI callback.
/// Unlike `defer_hide_main_window`, this preserves all internal state (AppView, prompt, etc.).
pub fn defer_conceal_main_window(cx: &mut gpui::App) {
    cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
        conceal_main_window();
    })
    .detach();
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
// Main Window Focus Detection
// ============================================================================

/// Check if the main window is currently focused (key window).
///
/// This is used to detect focus loss even when the app remains active
/// (e.g., when switching focus to Notes/AI windows).
///
#[cfg(target_os = "macos")]
pub fn is_main_window_focused() -> bool {
    if require_main_thread("is_main_window_focused") {
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
