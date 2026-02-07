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
                    // CFArrayGetValueAtIndex returns a borrowed reference - we must retain
                    // if we want to keep it beyond the array's lifetime
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

                    // Retain the window ref before caching - CFArrayGetValueAtIndex returns
                    // a borrowed reference, so we need to retain it to extend its lifetime
                    // beyond when we release windows_value
                    let retained_window = cf_retain(ax_window);
                    cache_window(window_id, retained_window as AXUIElementRef);

                    windows.push(WindowInfo {
                        id: window_id,
                        app: app_name_str.clone(),
                        title,
                        bounds: Bounds::new(x, y, width, height),
                        pid,
                        ax_window: Some(retained_window as usize),
                    });
                }

                // Release windows_value - AXUIElementCopyAttributeValue returns an owned
                // CF object that we must release (the "Copy" in the name means we own it)
                cf_release(windows_value);
            }

            // Release ax_app - AXUIElementCreateApplication returns an owned CF object
            cf_release(ax_app);
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
/// # Window Selection Strategy
///
/// 1. First try AXFocusedWindow - the actual focused window of the app
/// 2. If that fails, try AXMainWindow - the app's designated "main" window
/// 3. Fall back to first window in AXWindows array if neither works
///
/// This is more accurate than just picking the first window with matching pid,
/// which can return the wrong window if the app has multiple windows open.
///
/// # Returns
/// The focused/main window of the menu bar owning application, or None if
/// no windows are found.
#[instrument]
pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
    let target_pid = get_menu_bar_owner_pid()?;

    unsafe {
        // Create AX element for the target application
        let ax_app = AXUIElementCreateApplication(target_pid);
        if ax_app.is_null() {
            warn!(target_pid, "Failed to create AXUIElement for app");
            return Ok(None);
        }

        // Strategy 1: Try to get the focused window (most accurate)
        let focused_window = get_ax_attribute(ax_app as AXUIElementRef, "AXFocusedWindow")
            .ok()
            .filter(|&w| !w.is_null());

        // Strategy 2: Fall back to main window
        let target_window = focused_window.or_else(|| {
            get_ax_attribute(ax_app as AXUIElementRef, "AXMainWindow")
                .ok()
                .filter(|&w| !w.is_null())
        });

        // Strategy 3: Fall back to first window in AXWindows list
        let target_window = target_window.or_else(|| {
            if let Ok(windows_value) = get_ax_attribute(ax_app as AXUIElementRef, "AXWindows") {
                let count = CFArrayGetCount(windows_value as CFArrayRef);
                if count > 0 {
                    let window = CFArrayGetValueAtIndex(windows_value as CFArrayRef, 0);
                    // Retain the window ref since CFArrayGetValueAtIndex returns borrowed
                    let retained = cf_retain(window);
                    cf_release(windows_value);
                    Some(retained)
                } else {
                    cf_release(windows_value);
                    None
                }
            } else {
                None
            }
        });

        // Release ax_app - we're done with it
        cf_release(ax_app);

        // If we found a target window, build WindowInfo for it
        if let Some(window_ref) = target_window {
            let ax_window = window_ref as AXUIElementRef;

            // Get window attributes
            let title = get_window_string_attribute(ax_window, "AXTitle").unwrap_or_default();
            let (x, y) = get_window_position(ax_window).unwrap_or((0, 0));
            let (width, height) = get_window_size(ax_window).unwrap_or((0, 0));

            // Get app name for logging
            let app_name = get_app_name_for_pid(target_pid);

            // Create a window ID (focused window uses index 0)
            let window_id = (target_pid as u32) << 16;

            // Cache the window reference for subsequent operations
            cache_window(window_id, ax_window);

            let window_info = WindowInfo {
                id: window_id,
                app: app_name.clone(),
                title: title.clone(),
                bounds: Bounds::new(x, y, width, height),
                pid: target_pid,
                ax_window: Some(window_ref as usize),
            };

            info!(
                window_id = window_info.id,
                app = %app_name,
                title = %title,
                "Found focused/main window of previous app via AX"
            );

            Ok(Some(window_info))
        } else {
            warn!(
                target_pid,
                "No focused or main window found for menu bar owner"
            );
            Ok(None)
        }
    }
}
/// Get the localized app name for a given PID.
fn get_app_name_for_pid(pid: i32) -> String {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        if let Some(workspace_class) = Class::get("NSWorkspace") {
            let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
            let running_apps: *mut Object = msg_send![workspace, runningApplications];
            let app_count: usize = msg_send![running_apps, count];

            for i in 0..app_count {
                let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
                let app_pid: i32 = msg_send![app, processIdentifier];

                if app_pid == pid {
                    let app_name: *mut Object = msg_send![app, localizedName];
                    if !app_name.is_null() {
                        let utf8: *const i8 = msg_send![app_name, UTF8String];
                        if !utf8.is_null() {
                            return std::ffi::CStr::from_ptr(utf8)
                                .to_str()
                                .unwrap_or("Unknown")
                                .to_string();
                        }
                    }
                    break;
                }
            }
        }

        "Unknown".to_string()
    }
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

    set_window_position(window.as_ptr(), x, y)?;
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

    set_window_size(window.as_ptr(), width, height)?;
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
    set_window_position(window.as_ptr(), bounds.x, bounds.y)?;
    set_window_size(window.as_ptr(), bounds.width, bounds.height)?;

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
    let minimize_attr = try_create_cf_string("AXMinimized")?;

    // AXMinimized expects a CFBoolean, so we need to use the attribute differently
    // Actually, we should perform the press action on the minimize button
    // or set the AXMinimized attribute to true

    // Try setting AXMinimized directly with a boolean value
    unsafe {
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            static kCFBooleanTrue: CFTypeRef;
        }

        let result = AXUIElementSetAttributeValue(window.as_ptr(), minimize_attr, kCFBooleanTrue);

        cf_release(minimize_attr);

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
    let (current_x, current_y) = get_window_position(window.as_ptr()).unwrap_or((0, 0));

    // Get the display bounds (accounting for menu bar and dock)
    let display_bounds = get_visible_display_bounds(current_x, current_y);

    // Set the window to fill the visible area
    set_window_position(window.as_ptr(), display_bounds.x, display_bounds.y)?;
    set_window_size(window.as_ptr(), display_bounds.width, display_bounds.height)?;

    info!(window_id, "Maximized window");
    Ok(())
}
