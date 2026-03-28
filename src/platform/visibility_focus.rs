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
#[allow(dead_code)]
fn hide_main_window() {
    // On non-macOS, window hiding is done via GPUI in defer_hide_main_window.
    // This standalone function is kept as a no-op; the real hide happens through cx.
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
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        // On macOS, use the ObjC-based hide (orderOut: on NSWindow).
        #[cfg(target_os = "macos")]
        hide_main_window();

        // On non-macOS (Windows/Linux), use GPUI's app-level hide to
        // hide the main window since we don't have a raw HWND wired up yet.
        #[cfg(not(target_os = "macos"))]
        {
            let _ = cx.update(|cx| {
                cx.hide();
            });
        }
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

// ── Windows HWND-based show/hide ──────────────────────────────────────────
// We capture the HWND the first time the window is hidden (at that point the
// foreground window IS our app window). Subsequent show/hide use the stored
// handle so we never accidentally touch GPUI's internal helper windows.

#[cfg(target_os = "windows")]
static WIN32_MAIN_HWND: std::sync::OnceLock<std::sync::atomic::AtomicIsize> =
    std::sync::OnceLock::new();

/// Store the HWND of the main application window.
/// Extracts the real HWND from GPUI's window handle via raw_window_handle.
#[cfg(target_os = "windows")]
pub fn win32_capture_main_hwnd_from_gpui(cx: &mut gpui::App) {
    use raw_window_handle::HasWindowHandle;

    if let Some(handle) = crate::get_main_window_handle() {
        let result = cx.update_window(handle, |_view, window, _cx| {
            if let Ok(wh) = window.window_handle() {
                match wh.as_raw() {
                    raw_window_handle::RawWindowHandle::Win32(h) => {
                        let hwnd = h.hwnd.get() as isize;
                        let atom = WIN32_MAIN_HWND.get_or_init(|| {
                            std::sync::atomic::AtomicIsize::new(0)
                        });
                        atom.store(hwnd, std::sync::atomic::Ordering::Relaxed);
                        logging::log(
                            "VISIBILITY",
                            &format!("Windows: captured GPUI main HWND {:#x}", hwnd),
                        );
                    }
                    _ => {
                        logging::log("VISIBILITY", "Windows: unexpected raw window handle type");
                    }
                }
            }
        });
        if let Err(e) = result {
            logging::log(
                "VISIBILITY",
                &format!("Windows: update_window failed: {}", e),
            );
        }
    } else {
        logging::log("VISIBILITY", "Windows: no MAIN_WINDOW_HANDLE set, cannot capture HWND");
    }
}

/// Get the stored main HWND (returns 0 if not captured yet).
#[cfg(target_os = "windows")]
fn win32_get_main_hwnd() -> isize {
    WIN32_MAIN_HWND
        .get()
        .map(|a| a.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(0)
}

/// Show the main application window and bring it to the foreground.
#[cfg(target_os = "windows")]
pub fn win32_show_app_window() {
    extern "system" {
        fn ShowWindow(hwnd: isize, cmd_show: i32) -> i32;
        fn SetForegroundWindow(hwnd: isize) -> i32;
    }
    const SW_SHOW: i32 = 5;
    let hwnd = win32_get_main_hwnd();
    if hwnd != 0 {
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
        logging::log("VISIBILITY", "Windows: ShowWindow(SW_SHOW) + SetForegroundWindow");
    } else {
        logging::log("VISIBILITY", "Windows: no stored HWND, cannot show");
    }
}

/// Hide the main application window.
#[cfg(target_os = "windows")]
pub fn win32_hide_app_window() {
    extern "system" {
        fn ShowWindow(hwnd: isize, cmd_show: i32) -> i32;
    }
    const SW_HIDE: i32 = 0;
    let hwnd = win32_get_main_hwnd();
    if hwnd != 0 {
        unsafe { ShowWindow(hwnd, SW_HIDE); }
        logging::log("VISIBILITY", "Windows: ShowWindow(SW_HIDE)");
    } else {
        logging::log("VISIBILITY", "Windows: no stored HWND, cannot hide");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn show_main_window_without_activation() {
    #[cfg(target_os = "windows")]
    {
        logging::log("PANEL", "Windows: showing window via ShowWindow");
        win32_show_app_window();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        logging::log(
            "PANEL",
            "show_main_window_without_activation: Not implemented on this platform",
        );
    }
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
