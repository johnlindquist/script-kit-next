use super::*;

#[cfg(target_os = "macos")]
pub(super) fn configure_ai_window_vibrancy() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Found the AI window - configure vibrancy
                        // Disable dragging by window background to prevent titlebar interference
                        // with mouse clicks on content (e.g., setup card buttons)
                        let _: () = msg_send![window, setMovableByWindowBackground: false];
                        let theme = crate::theme::load_theme();
                        let is_dark = theme.should_use_dark_vibrancy();
                        crate::platform::configure_secondary_window_vibrancy(window, "AI", is_dark);

                        // Configure as a regular window that participates in Cmd+Tab:
                        // - Keep default window level (0) so it doesn't float
                        // - Add ParticipatesInCycle (128) so it appears in Cmd+Tab
                        // - Remove IgnoresCycle (64) if somehow set
                        // - Add MoveToActiveSpace (2) so it follows the user
                        let current: u64 = msg_send![window, collectionBehavior];
                        // Clear IgnoresCycle bit, set ParticipatesInCycle and MoveToActiveSpace
                        let desired: u64 = (current & !64) | 128 | 2;
                        let _: () = msg_send![window, setCollectionBehavior:desired];

                        // Log detailed breakdown of collection behavior bits
                        let has_participates = (desired & 128) != 0;
                        let has_ignores = (desired & 64) != 0;
                        let has_move_to_active = (desired & 2) != 0;

                        logging::log(
                            "PANEL",
                            &format!(
                                "AI window: Cmd+Tab config - behavior={}->{} [ParticipatesInCycle={}, IgnoresCycle={}, MoveToActiveSpace={}]",
                                current, desired, has_participates, has_ignores, has_move_to_active
                            ),
                        );
                        logging::log(
                            "PANEL",
                            "AI window: WILL appear in Cmd+Tab app switcher (unique among Script Kit windows)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: AI window not found by title for vibrancy config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn configure_ai_window_vibrancy() {
    // No-op on non-macOS platforms
}

/// Configure the AI window as a floating panel (always on top).
///
/// This sets:
/// - NSFloatingWindowLevel (3) - floats above normal windows
/// - NSWindowCollectionBehaviorMoveToActiveSpace - moves to current space when shown
/// - Disabled window restoration - prevents macOS position caching
#[cfg(target_os = "macos")]
fn configure_ai_as_floating_panel() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Found the AI window - configure it

                        // NSFloatingWindowLevel = 3
                        // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
                        let floating_level: i64 = 3;
                        let _: () = msg_send![window, setLevel:floating_level];

                        // Get current collection behavior to preserve existing flags
                        let current: u64 = msg_send![window, collectionBehavior];
                        // OR in MoveToActiveSpace (2) + FullScreenAuxiliary (256)
                        let desired: u64 = current | 2 | 256;
                        let _: () = msg_send![window, setCollectionBehavior:desired];

                        // Disable window restoration
                        let _: () = msg_send![window, setRestorable:false];

                        // Disable close/hide animation for instant dismiss (NSWindowAnimationBehaviorNone = 2)
                        let _: () = msg_send![window, setAnimationBehavior: 2i64];

                        // ═══════════════════════════════════════════════════════════════════════════
                        // VIBRANCY CONFIGURATION - Match main window for consistent blur
                        // ═══════════════════════════════════════════════════════════════════════════
                        let theme = crate::theme::load_theme();
                        let is_dark = theme.should_use_dark_vibrancy();
                        crate::platform::configure_secondary_window_vibrancy(window, "AI", is_dark);

                        logging::log(
                            "PANEL",
                            "AI window configured as floating panel (level=3, MoveToActiveSpace, vibrancy)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: AI window not found by title for floating panel config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_ai_as_floating_panel() {
    // No-op on non-macOS platforms
}
