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

n// This entire module is macOS-only
#![cfg(target_os = "macos")]

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
    /// Bundle ID currently being fetched, if any.
    /// Using Option<String> instead of a boolean flag avoids race conditions:
    /// - Thread A starts fetching for "com.app.A", sets fetching_bundle_id = Some("com.app.A")
    /// - Thread B starts fetching for "com.app.B", sets fetching_bundle_id = Some("com.app.B")
    /// - Thread A finishes first: only clears if still "com.app.A" (which it isn't)
    /// - Thread B finishes: clears correctly since it's still "com.app.B"
    fetching_bundle_id: Option<String>,
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
    TRACKER_STATE.read().fetching_bundle_id.is_some()
}

/// Get the bundle ID currently being fetched, if any.
/// This is more informative than `is_fetching_menu()` for debugging.
#[allow(dead_code)] // Public API for future use
pub fn get_fetching_bundle_id() -> Option<String> {
    TRACKER_STATE.read().fetching_bundle_id.clone()
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

        // Try to get or create the observer class
        // Fix: If the class already exists (e.g., from a previous call), fetch it
        // instead of returning early. This ensures the observer is always registered.
        let observer_class = if let Some(existing) = Class::get("ScriptKitAppObserver") {
            logging::log("APP", "Using existing ScriptKitAppObserver class");
            existing
        } else {
            // Class doesn't exist, create it
            let mut decl = match ClassDecl::new("ScriptKitAppObserver", superclass) {
                Some(d) => d,
                None => {
                    // This shouldn't happen if Class::get returned None, but handle it
                    logging::log("ERROR", "Failed to create ScriptKitAppObserver class");
                    return;
                }
            };

            // Add the notification handler method
            //
            // SAFETY: This callback is invoked from Objective-C on a background thread
            // (via NSRunLoop). We must:
            // 1. Use autoreleasepool to drain autoreleased objects created by msg_send!
            //    (e.g., NSStrings from stringWithUTF8String:). Without this, objects
            //    accumulate and leak since there's no enclosing pool on this thread.
            // 2. Use catch_unwind to prevent panics from unwinding across the FFI boundary,
            //    which is undefined behavior.
            extern "C" fn handle_app_activation(
                _this: &Object,
                _sel: Sel,
                notification: *mut Object,
            ) {
                // Catch any panic to prevent UB from unwinding across FFI boundary
                let _ = std::panic::catch_unwind(|| {
                    // Create an autorelease pool to drain autoreleased objects
                    // from msg_send! calls (e.g., NSStrings created in this callback)
                    objc::rc::autoreleasepool(|| unsafe {
                        handle_app_activation_inner(notification)
                    });
                });
            }

            /// Inner implementation of handle_app_activation.
            /// Separated to keep the extern "C" wrapper minimal and focused on FFI safety.
            ///
            /// # Safety
            /// - Must be called within an autoreleasepool
            /// - notification must be a valid Objective-C object pointer or null
            unsafe fn handle_app_activation_inner(notification: *mut Object) {
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
                    // CRITICAL: Also check PID! Same bundle_id with different PID means
                    // app was relaunched (quit and reopened), so we need to refresh the
                    // menu cache to avoid stale menus.
                    let should_update = {
                        let state = TRACKER_STATE.read();
                        state
                            .last_real_app
                            .as_ref()
                            .map(|a| a.bundle_id != tracked.bundle_id || a.pid != tracked.pid)
                            .unwrap_or(true)
                    };

                    if should_update {
                        // Log if this is a PID change (app relaunch) case
                        {
                            let state = TRACKER_STATE.read();
                            if let Some(old_app) = &state.last_real_app {
                                if old_app.bundle_id == tracked.bundle_id
                                    && old_app.pid != tracked.pid
                                {
                                    logging::log(
                                        "APP",
                                        &format!(
                                            "App relaunched (same bundle_id, new PID): {} PID {} -> {}",
                                            tracked.bundle_id, old_app.pid, tracked.pid
                                        ),
                                    );
                                }
                            }
                        }

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

            decl.add_method(
                sel!(handleAppActivation:),
                handle_app_activation as extern "C" fn(&Object, Sel, *mut Object),
            );

            decl.register()
        };

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
    // Mark which bundle we're fetching for (replaces boolean flag to avoid race)
    {
        let mut state = TRACKER_STATE.write();
        state.fetching_bundle_id = Some(bundle_id.clone());
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
                // Only clear fetching state if we're still the one fetching
                // (avoids race where a newer fetch started for a different app)
                if state.fetching_bundle_id.as_ref() == Some(&bundle_id) {
                    state.fetching_bundle_id = None;
                }
            }
            Err(e) => {
                logging::log(
                    "WARN",
                    &format!("Failed to pre-fetch menu items for {}: {}", bundle_id, e),
                );

                let mut state = TRACKER_STATE.write();
                // Only clear fetching state if we're still the one fetching
                if state.fetching_bundle_id.as_ref() == Some(&bundle_id) {
                    state.fetching_bundle_id = None;
                }
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
        assert!(state.fetching_bundle_id.is_none());
    }

    #[test]
    fn test_fetching_bundle_id_race_condition_fix() {
        // Test that the bundle_id tracking correctly handles concurrent fetches
        // Scenario: Thread A starts fetching for "com.app.A", then Thread B
        // starts for "com.app.B". Thread A finishes first - should NOT clear
        // the fetching state since B is now the active fetch.

        // Simulate Thread A starting fetch
        let mut state = TrackerState {
            fetching_bundle_id: Some("com.app.A".to_string()),
            ..Default::default()
        };
        assert_eq!(state.fetching_bundle_id.as_deref(), Some("com.app.A"));

        // Thread B starts fetching (overwrites A)
        state.fetching_bundle_id = Some("com.app.B".to_string());
        assert_eq!(state.fetching_bundle_id.as_deref(), Some("com.app.B"));

        // Thread A finishes - should NOT clear state (it's for B now)
        if state.fetching_bundle_id.as_deref() == Some("com.app.A") {
            state.fetching_bundle_id = None;
        }
        // State should still show B as fetching
        assert_eq!(
            state.fetching_bundle_id.as_deref(),
            Some("com.app.B"),
            "Thread A should not clear B's fetch state"
        );

        // Thread B finishes - should clear state
        if state.fetching_bundle_id.as_deref() == Some("com.app.B") {
            state.fetching_bundle_id = None;
        }
        assert!(
            state.fetching_bundle_id.is_none(),
            "Thread B should clear its own fetch state"
        );
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

    #[test]
    fn test_same_bundle_id_different_pid_triggers_update() {
        // Regression test: when an app is relaunched (quit and reopened),
        // the bundle_id stays the same but the PID changes.
        // We MUST trigger an update to refresh the menu cache.

        // Helper function that mirrors the should_update logic in handle_app_activation_inner
        fn should_update(tracked: &TrackedApp, current_app: &Option<TrackedApp>) -> bool {
            current_app
                .as_ref()
                .map(|a| a.bundle_id != tracked.bundle_id || a.pid != tracked.pid)
                .unwrap_or(true)
        }

        // Case 1: No current app tracked -> should update
        let new_app = TrackedApp {
            pid: 100,
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        };
        assert!(
            should_update(&new_app, &None),
            "Should update when no app is tracked"
        );

        // Case 2: Different bundle_id -> should update
        let current = Some(TrackedApp {
            pid: 100,
            bundle_id: "com.other.app".to_string(),
            name: "Other App".to_string(),
        });
        assert!(
            should_update(&new_app, &current),
            "Should update when bundle_id differs"
        );

        // Case 3: Same bundle_id AND same PID -> should NOT update
        let current_same = Some(TrackedApp {
            pid: 100,
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        });
        assert!(
            !should_update(&new_app, &current_same),
            "Should NOT update when both bundle_id and PID are the same"
        );

        // Case 4: CRITICAL - Same bundle_id but DIFFERENT PID -> MUST update
        // This is the bug fix: app was relaunched with new PID
        let current_relaunched = Some(TrackedApp {
            pid: 200, // Different PID - app was relaunched!
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        });
        assert!(
            should_update(&new_app, &current_relaunched),
            "MUST update when same bundle_id but different PID (app relaunched)"
        );
    }
}
