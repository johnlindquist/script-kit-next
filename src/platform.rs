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
/// # Example
///
/// ```ignore
/// // Call after window is created and visible
/// configure_as_floating_panel();
/// ```
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
}
