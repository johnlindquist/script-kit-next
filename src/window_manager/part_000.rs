#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "macos")]
use crate::logging;
// Re-export the canonical WindowRole from window_state
// This ensures a single source of truth for window roles across the codebase
pub use crate::window_state::WindowRole;
/// Stable window registration data captured at register time.
///
/// We store non-owning metadata (pointer address + window number) and resolve
/// to a live `NSWindow*` on each read. This prevents returning stale pointers
/// from a long-lived cache when windows are destroyed/recreated.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegisteredWindowHandle {
    window_ptr_addr: usize,
    window_number: i64,
}
#[cfg(target_os = "macos")]
impl RegisteredWindowHandle {
    /// Capture identifying metadata from a live NSWindow pointer.
    #[cfg(not(test))]
    fn from_window(window: id) -> Option<Self> {
        if window == nil {
            return None;
        }

        let window_number: i64 = unsafe { msg_send![window, windowNumber] };
        if window_number <= 0 {
            return None;
        }

        Some(Self {
            window_ptr_addr: window as usize,
            window_number,
        })
    }

    /// Test-only constructor that avoids Objective-C calls on mock pointers.
    #[cfg(test)]
    fn from_window(window: id) -> Option<Self> {
        if window == nil {
            return None;
        }

        Some(Self {
            window_ptr_addr: window as usize,
            window_number: 0,
        })
    }
}
/// Thread-safe window registry.
///
/// Maintains a mapping from window roles to their native macOS window IDs.
/// Access this through the module-level functions, not directly.
#[cfg(target_os = "macos")]
struct WindowManager {
    /// Map of window roles to captured registration metadata.
    windows: HashMap<WindowRole, RegisteredWindowHandle>,
}
#[cfg(target_os = "macos")]
impl WindowManager {
    /// Create a new empty WindowManager
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    /// Register a window with a specific role
    fn register(&mut self, role: WindowRole, window_id: id) {
        logging::log(
            "WINDOW_MGR",
            &format!("Registering window: {:?} -> {:?}", role, window_id),
        );
        if let Some(handle) = RegisteredWindowHandle::from_window(window_id) {
            self.windows.insert(role, handle);
        } else {
            logging::log(
                "WINDOW_MGR",
                &format!(
                    "WARNING: Skipping registration for {:?}: invalid NSWindow handle",
                    role
                ),
            );
        }
    }

    /// Get captured registration metadata by role.
    fn get_handle(&self, role: WindowRole) -> Option<RegisteredWindowHandle> {
        self.windows.get(&role).copied()
    }

    /// Remove a window registration
    #[allow(dead_code)]
    fn unregister(&mut self, role: WindowRole) -> Option<RegisteredWindowHandle> {
        logging::log("WINDOW_MGR", &format!("Unregistering window: {:?}", role));
        self.windows.remove(&role)
    }

    /// Check if a role is registered
    #[allow(dead_code)]
    fn is_registered(&self, role: WindowRole) -> bool {
        self.windows.contains_key(&role)
    }
}
/// Global singleton for the window manager
#[cfg(target_os = "macos")]
static WINDOW_MANAGER: OnceLock<Mutex<WindowManager>> = OnceLock::new();
/// Get or initialize the global WindowManager
#[cfg(target_os = "macos")]
fn get_manager() -> &'static Mutex<WindowManager> {
    WINDOW_MANAGER.get_or_init(|| Mutex::new(WindowManager::new()))
}
#[cfg(target_os = "macos")]
#[cfg(not(test))]
fn is_main_thread() -> bool {
    unsafe {
        if let Some(thread_class) = objc::runtime::Class::get("NSThread") {
            let result: bool = msg_send![thread_class, isMainThread];
            result
        } else {
            false
        }
    }
}
#[cfg(target_os = "macos")]
#[cfg(test)]
fn is_main_thread() -> bool {
    true
}
#[cfg(target_os = "macos")]
#[cfg(not(test))]
fn resolve_live_window(handle: RegisteredWindowHandle) -> Option<id> {
    unsafe {
        let app: id = NSApp();
        if app == nil {
            return None;
        }

        let windows: id = msg_send![app, windows];
        if windows == nil {
            return None;
        }

        let count: usize = msg_send![windows, count];
        for index in 0..count {
            let window: id = msg_send![windows, objectAtIndex: index];
            if window == nil {
                continue;
            }

            let window_number: i64 = msg_send![window, windowNumber];
            if window as usize == handle.window_ptr_addr && window_number == handle.window_number {
                return Some(window);
            }
        }
    }

    None
}
#[cfg(target_os = "macos")]
#[cfg(test)]
fn resolve_live_window(handle: RegisteredWindowHandle) -> Option<id> {
    Some(handle.window_ptr_addr as id)
}
// ============================================================================
// Public API - macOS Implementation
// ============================================================================

/// Register a window with a specific role.
///
/// Call this after GPUI creates a window to track it by role.
/// Subsequent calls with the same role will overwrite the previous registration.
///
/// # Arguments
/// * `role` - The purpose/role of this window
/// * `window_id` - The native macOS window ID (NSWindow pointer)
///
#[cfg(target_os = "macos")]
#[tracing::instrument(skip(window_id), fields(role = ?role))]
pub fn register_window(role: WindowRole, window_id: id) {
    if !is_main_thread() {
        logging::log(
            "WINDOW_MGR",
            &format!(
                "WARNING: register_window for {:?} ignored off main thread",
                role
            ),
        );
        return;
    }

    if let Ok(mut manager) = get_manager().lock() {
        manager.register(role, window_id);
    } else {
        logging::log("WINDOW_MGR", "ERROR: Failed to acquire lock for register");
    }
}
/// Get a window by its role.
///
/// # Arguments
/// * `role` - The role to look up
///
/// # Returns
/// The native window ID if registered, None otherwise
#[cfg(target_os = "macos")]
pub fn get_window(role: WindowRole) -> Option<id> {
    if !is_main_thread() {
        logging::log(
            "WINDOW_MGR",
            &format!("WARNING: get_window({:?}) called off main thread", role),
        );
        return None;
    }

    let handle = if let Ok(manager) = get_manager().lock() {
        manager.get_handle(role)
    } else {
        logging::log("WINDOW_MGR", "ERROR: Failed to acquire lock for get");
        None
    };

    let handle = handle?;

    if let Some(window) = resolve_live_window(handle) {
        return Some(window);
    }

    logging::log(
        "WINDOW_MGR",
        &format!("INFO: Pruning stale window handle for {:?}", role),
    );

    if let Ok(mut manager) = get_manager().lock() {
        if manager.get_handle(role) == Some(handle) {
            manager.unregister(role);
        }
    } else {
        logging::log(
            "WINDOW_MGR",
            "ERROR: Failed to acquire lock for stale prune",
        );
    }

    None
}
/// Convenience function to get the main window.
///
/// # Returns
/// The main window's native ID if registered, None otherwise
#[cfg(target_os = "macos")]
pub fn get_main_window() -> Option<id> {
    get_window(WindowRole::Main)
}
/// Find and register the main window by its expected size.
///
/// This function searches through NSApp's windows array and identifies
/// our main window by its characteristic size (750x~500 pixels).
/// This is necessary because tray icons and other system elements
/// create windows that appear before our main window in the array.
///
/// # Expected Window Size
/// - Width: ~750 pixels
/// - Height: ~400-600 pixels (varies based on content)
///
/// # Returns
/// `true` if the main window was found and registered, `false` otherwise
#[cfg(target_os = "macos")]
#[tracing::instrument(skip_all)]
pub fn find_and_register_main_window() -> bool {
    if !is_main_thread() {
        logging::log(
            "WINDOW_MGR",
            "WARNING: find_and_register_main_window called off main thread",
        );
        return false;
    }

    // Expected main window dimensions (with tolerance)
    const EXPECTED_WIDTH: f64 = 750.0;
    const WIDTH_TOLERANCE: f64 = 50.0;
    // MIN_HEIGHT lowered to 50 to accommodate compact arg prompts (76px)
    const MIN_HEIGHT: f64 = 50.0;
    const MAX_HEIGHT: f64 = 800.0;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        logging::log(
            "WINDOW_MGR",
            &format!(
                "Searching for main window among {} windows (expecting ~{:.0}x400-600)",
                count, EXPECTED_WIDTH
            ),
        );

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex:i];
            if window == nil {
                continue;
            }

            let frame: NSRect = msg_send![window, frame];
            let width = frame.size.width;
            let height = frame.size.height;

            logging::log(
                "WINDOW_MGR",
                &format!("  Window[{}]: {:.0}x{:.0}", i, width, height),
            );

            // Check if this looks like our main window
            let width_matches = (width - EXPECTED_WIDTH).abs() < WIDTH_TOLERANCE;
            let height_matches = (MIN_HEIGHT..=MAX_HEIGHT).contains(&height);

            if width_matches && height_matches {
                logging::log(
                    "WINDOW_MGR",
                    &format!(
                        "Found main window at index {}: {:.0}x{:.0}",
                        i, width, height
                    ),
                );
                register_window(WindowRole::Main, window);
                return true;
            }
        }

        logging::log(
            "WINDOW_MGR",
            "WARNING: Could not find main window by size. No window matched expected dimensions.",
        );
        false
    }
}
/// Unregister a window by role.
///
/// # Arguments
/// * `role` - The role to unregister
///
/// # Returns
/// The previously registered window ID, if any
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn unregister_window(role: WindowRole) -> Option<id> {
    if !is_main_thread() {
        logging::log(
            "WINDOW_MGR",
            &format!(
                "WARNING: unregister_window for {:?} ignored off main thread",
                role
            ),
        );
        return None;
    }

    let handle = if let Ok(mut manager) = get_manager().lock() {
        manager.unregister(role)
    } else {
        logging::log("WINDOW_MGR", "ERROR: Failed to acquire lock for unregister");
        None
    };

    handle.and_then(resolve_live_window)
}
/// Check if a window role is currently registered.
///
/// # Arguments
/// * `role` - The role to check
///
/// # Returns
/// `true` if a window is registered for this role
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn is_window_registered(role: WindowRole) -> bool {
    get_window(role).is_some()
}
// ============================================================================
// Public API - Non-macOS Stubs
// ============================================================================

/// Non-macOS stub: register_window is a no-op
#[cfg(not(target_os = "macos"))]
pub fn register_window(_role: WindowRole, _window_id: *mut std::ffi::c_void) {
    // No-op on non-macOS platforms
}
/// Non-macOS stub: get_window always returns None
#[cfg(not(target_os = "macos"))]
pub fn get_window(_role: WindowRole) -> Option<*mut std::ffi::c_void> {
    None
}
/// Non-macOS stub: get_main_window always returns None
#[cfg(not(target_os = "macos"))]
pub fn get_main_window() -> Option<*mut std::ffi::c_void> {
    None
}
/// Non-macOS stub: find_and_register_main_window always returns false
#[cfg(not(target_os = "macos"))]
pub fn find_and_register_main_window() -> bool {
    false
}
/// Non-macOS stub: unregister_window always returns None
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn unregister_window(_role: WindowRole) -> Option<*mut std::ffi::c_void> {
    None
}
/// Non-macOS stub: is_window_registered always returns false
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn is_window_registered(_role: WindowRole) -> bool {
    false
}
