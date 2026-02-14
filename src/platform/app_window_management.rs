use crate::logging;
use crate::windows::DisplayBounds;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString as CocoaNSString;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
use crate::window_manager;

/// NSWindowCollectionBehaviorCanJoinAllSpaces constant value (1 << 0 = 1)
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
#[cfg(target_os = "macos")]
const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i64 = 0;
#[cfg(target_os = "macos")]
const NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY: i64 = 1;
#[cfg(target_os = "macos")]
const NS_WINDOW_ANIMATION_BEHAVIOR_NONE: i64 = 2;

// ============================================================================
// Thread Safety
// ============================================================================

/// Check whether the current thread is the main thread (works in release builds).
#[cfg(target_os = "macos")]
fn is_main_thread() -> bool {
    // SAFETY: NSThread.isMainThread is a class method that only reads thread
    // identity. It is safe to call from any thread and does not mutate state.
    unsafe {
        let is_main: bool = msg_send![class!(NSThread), isMainThread];
        is_main
    }
}

/// Runtime guard: logs an error and returns `true` (caller should bail)
/// when called from a non-main thread. Works in both debug and release builds.
#[cfg(target_os = "macos")]
fn require_main_thread(fn_name: &str) -> bool {
    if !is_main_thread() {
        logging::log(
            "ERROR",
            &format!(
                "{} called from non-main thread; AppKit requires main thread",
                fn_name
            ),
        );
        return true;
    }
    false
}

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
    if require_main_thread("configure_as_accessory_app") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid after app launch.
    unsafe {
        let app: id = NSApp();
        // NSApplicationActivationPolicyAccessory = 1
        // This makes the app not appear in Dock and not take menu bar ownership
        let _: () = msg_send![
            app,
            setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY
        ];
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

/// Temporarily switch to "regular" app mode so the app appears in Cmd+Tab.
///
/// This is used when the AI window is opened - it needs to be Cmd+Tab accessible
/// unlike the main menu which is a utility panel.
///
/// # macOS Behavior
///
/// Sets NSApplicationActivationPolicyRegular (value = 0) on the app.
/// The app will appear in the Dock and Cmd+Tab while in this mode.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn set_regular_app_mode() {
    if require_main_thread("set_regular_app_mode") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid after app launch.
    unsafe {
        let app: id = NSApp();
        // NSApplicationActivationPolicyRegular = 0
        // This makes the app appear in Dock and Cmd+Tab
        let _: () = msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
        logging::log(
            "PANEL",
            "Switched to regular app mode (appears in Dock and Cmd+Tab)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_regular_app_mode() {
    // No-op on non-macOS platforms
}

/// Switch back to "accessory" app mode after AI window is closed.
///
/// This restores the app to its normal state where it doesn't appear in
/// the Dock or Cmd+Tab.
///
/// # macOS Behavior
///
/// Sets NSApplicationActivationPolicyAccessory (value = 1) on the app.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn set_accessory_app_mode() {
    if require_main_thread("set_accessory_app_mode") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid after app launch.
    unsafe {
        let app: id = NSApp();
        // NSApplicationActivationPolicyAccessory = 1
        let _: () = msg_send![
            app,
            setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY
        ];
        logging::log(
            "PANEL",
            "Switched to accessory app mode (no Dock icon, no Cmd+Tab)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_accessory_app_mode() {
    // No-op on non-macOS platforms
}

/// Send the AI window to the back (behind other apps' windows).
///
/// This is called when the main menu is shown to prevent the AI window
/// from being brought forward along with the main menu. The AI window
/// should only come forward via Cmd+Tab or explicit user action.
///
/// # macOS Behavior
///
/// Finds the AI window by title and uses orderBack: to send it behind
/// other windows without hiding it.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn send_ai_window_to_back() {
    if require_main_thread("send_ai_window_to_back") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid. We check title
    // and UTF8String for nil/null before dereferencing via CStr::from_ptr.
    unsafe {
        use std::ffi::CStr;

        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        if windows.is_null() {
            return;
        }
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Found the AI window - send it to the back
                        let _: () = msg_send![window, orderBack: nil];
                        logging::log(
                            "PANEL",
                            "AI window sent to back (won't come forward with main menu)",
                        );
                        return;
                    }
                }
            }
        }
        // AI window not found - that's fine, it may not be open
    }
}

#[cfg(not(target_os = "macos"))]
pub fn send_ai_window_to_back() {
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
    if require_main_thread("ensure_move_to_active_space") {
        return;
    }
    // SAFETY: Main thread verified. Window pointer from WindowManager is valid.
    // collectionBehavior / setCollectionBehavior are standard NSWindow methods.
    unsafe {
        // Use WindowManager to get the main window (not keyWindow, which may not exist yet)
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "WARNING: Main window not registered, cannot set collection behavior",
                );
                return;
            }
        };

        // Get current collection behavior to preserve existing flags
        let current: u64 = msg_send![window, collectionBehavior];

        // Check if window has CanJoinAllSpaces (set by GPUI for PopUp windows)
        // If so, we can't add MoveToActiveSpace (they're mutually exclusive)
        let has_can_join_all_spaces =
            (current & NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES) != 0;

        let desired = if has_can_join_all_spaces {
            // PopUp window - only add FullScreenAuxiliary (no MoveToActiveSpace)
            current | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
        } else {
            // Normal window - add both MoveToActiveSpace and FullScreenAuxiliary
            current
                | NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE
                | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
        };

        let _: () = msg_send![window, setCollectionBehavior:desired];

        logging::log(
            "PANEL",
            &format!(
                "Set collection behavior: {} -> {} (CanJoinAllSpaces={}, FullScreenAuxiliary)",
                current, desired, has_can_join_all_spaces
            ),
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

/// Configure the main window as a floating macOS panel.
///
/// This function configures the main window (via WindowManager) with:
/// - Floating window level (NSFloatingWindowLevel = 3) - appears above normal windows
/// - MoveToActiveSpace collection behavior - moves to current space when shown
/// - Disabled window restoration - prevents macOS from remembering window position
/// - Empty frame autosave name - prevents position caching
///
/// # macOS Behavior
///
/// Uses WindowManager to get the main window (more reliable than NSApp.keyWindow,
/// which is timing-sensitive and can return nil during startup or the wrong window
/// in multi-window scenarios). If no main window is registered, logs a warning.
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
    if require_main_thread("configure_as_floating_panel") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // All msg_send! calls target standard NSWindow property setters.
    unsafe {
        // Use WindowManager to get the main window (more reliable than keyWindow)
        // keyWindow is timing-sensitive and can return nil during startup,
        // or the wrong window (Notes/AI) in multi-window scenarios.
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "WARNING: Main window not registered, cannot configure as floating panel",
                );
                return;
            }
        };

        // NSFloatingWindowLevel = 3
        // This makes the window float above normal windows
        // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
        let _: () = msg_send![window, setLevel:NS_FLOATING_WINDOW_LEVEL];

        // Get current collection behavior to preserve existing flags set by GPUI/AppKit
        let current: u64 = msg_send![window, collectionBehavior];

        // Check if window has CanJoinAllSpaces (set by GPUI for PopUp windows)
        // If so, we can't add MoveToActiveSpace (they're mutually exclusive)
        let has_can_join_all_spaces =
            (current & NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES) != 0;

        // OR in our desired flags instead of replacing:
        // - FullScreenAuxiliary: window can show over fullscreen apps without disrupting
        // - IgnoresCycle: exclude from Cmd+Tab app switcher (main window is a utility)
        // - MoveToActiveSpace: ONLY if CanJoinAllSpaces is not set (they're mutually exclusive)
        let desired = if has_can_join_all_spaces {
            // PopUp window - don't add MoveToActiveSpace
            current
                | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
                | NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE
        } else {
            // Normal window - add MoveToActiveSpace
            current
                | NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE
                | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
                | NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE
        };

        let _: () = msg_send![window, setCollectionBehavior:desired];

        // CRITICAL: Disable macOS window state restoration
        // This prevents macOS from remembering and restoring the window position
        // when the app is relaunched or the window is shown again
        let _: () = msg_send![window, setRestorable:false];

        // Also disable the window's autosave frame name which can cause position caching
        let empty_string: id = msg_send![class!(NSString), string];
        let _: () = msg_send![window, setFrameAutosaveName:empty_string];

        // Disable close/hide animation for instant dismiss (NSWindowAnimationBehaviorNone = 2)
        let _: () = msg_send![window, setAnimationBehavior: NS_WINDOW_ANIMATION_BEHAVIOR_NONE];

        // Log detailed breakdown of collection behavior bits
        let has_can_join = (desired & NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES) != 0;
        let has_ignores = (desired & NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE) != 0;
        let has_move_to_active =
            (desired & NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE) != 0;

        logging::log(
            "PANEL",
            &format!(
                "Main window: behavior={}->{} [CanJoinAllSpaces={}, IgnoresCycle={}, MoveToActiveSpace={}]",
                current, desired, has_can_join, has_ignores, has_move_to_active
            ),
        );
        logging::log(
            "PANEL",
            "Main window: Will NOT appear in Cmd+Tab app switcher (floating utility panel)",
        );

        // Install cursor rect management so the underlying app's cursor
        // (e.g. Terminal's I-beam) doesn't bleed through our panel.
        install_cursor_tracking();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_as_floating_panel() {
    // No-op on non-macOS platforms
}
