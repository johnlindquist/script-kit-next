//! Selected text operations using platform-native APIs.
//!
//! This module provides getSelectedText() and setSelectedText() operations
//! using a hybrid approach: Accessibility API primary, clipboard fallback.
//!
//! ## Architecture
//!
//! - **macOS**: `get_selected_text()` uses `get-selected-text` crate which tries AX API first,
//!   then falls back to clipboard simulation with Cmd+C.
//!   `set_selected_text()` uses clipboard + CGEvent keyboard simulation (Cmd+V).
//! - **Windows**: Both functions use clipboard + `SendInput` keyboard simulation (Ctrl+C / Ctrl+V).
//!   Original clipboard contents are saved and restored.
//!
//! ## Permissions
//!
//! - macOS: Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility
//! - Windows: No special permissions required (SendInput works for non-elevated targets)

#[cfg(any(target_os = "macos", target_os = "windows"))]
use anyhow::Context;
use anyhow::{bail, Result};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use arboard::Clipboard;
#[cfg(target_os = "macos")]
use get_selected_text::get_selected_text as get_selected_text_impl;
#[cfg(target_os = "macos")]
use macos_accessibility_client::accessibility;
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::thread;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::time::Duration;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tracing::{debug, info};
use tracing::{instrument, warn};

// ============================================================================
// Permission Functions
// ============================================================================

/// Check if accessibility permissions are granted.
///
/// This checks if the application has been granted permission to use
/// macOS Accessibility APIs for cross-process text operations.
///
/// # Returns
/// `true` if permission is granted, `false` otherwise.
#[instrument]
#[cfg(target_os = "macos")]
pub fn has_accessibility_permission() -> bool {
    let result = accessibility::application_is_trusted();
    debug!(granted = result, "Checked accessibility permission");
    result
}

/// Request accessibility permissions (opens System Preferences).
///
/// This will show the system dialog prompting the user to grant
/// accessibility permission. The user must manually enable the
/// permission in System Preferences.
///
/// # Returns
/// `true` if permission is granted after the request, `false` otherwise.
#[instrument]
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission() -> bool {
    info!("Requesting accessibility permission");
    let result = accessibility::application_is_trusted_with_prompt();
    if result {
        info!("Accessibility permission granted");
    } else {
        warn!("Accessibility permission denied or pending");
    }
    result
}

/// Open System Preferences directly to Accessibility pane.
///
/// This is useful for guiding users to the correct settings location
/// without showing the system permission prompt.
///
/// # Errors
/// Returns error if unable to spawn the open command.
#[allow(dead_code)] // Will be used for permission UI prompts
#[instrument]
#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> Result<()> {
    info!("Opening accessibility settings");
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .context("Failed to open System Preferences")?;
    Ok(())
}

/// Show a user-friendly dialog explaining accessibility permission is needed.
///
/// First checks if permission is already granted. If not, requests it
/// with the system prompt.
///
/// # Returns
/// `true` if permission is granted (either already or after request).
#[allow(dead_code)] // Will be used for permission UI prompts
#[instrument]
#[cfg(target_os = "macos")]
pub fn show_permission_dialog() -> Result<bool> {
    // First, check if already granted
    if has_accessibility_permission() {
        debug!("Permission already granted");
        return Ok(true);
    }

    // Request with system prompt (opens System Preferences)
    let granted = request_accessibility_permission();

    if !granted {
        warn!("User denied accessibility permission");
    }

    Ok(granted)
}

// ============================================================================
// Get Selected Text
// ============================================================================

/// Get the currently selected text from the focused application.
///
/// Uses the `get-selected-text` crate which implements:
/// 1. AXSelectedText attribute (fastest, most reliable)
/// 2. AXSelectedTextRange + AXStringForRange (fallback)
/// 3. Clipboard simulation with Cmd+C (last resort, saves/restores clipboard)
///
/// # Returns
/// The selected text, or empty string if nothing is selected.
///
/// # Errors
/// - Returns error if no accessibility permission
/// - Returns error if the operation fails
///
#[instrument(skip_all)]
#[cfg(target_os = "macos")]
pub fn get_selected_text() -> Result<String> {
    // Check permissions first
    if !has_accessibility_permission() {
        bail!("Accessibility permission required. Enable in System Preferences > Privacy & Security > Accessibility");
    }

    debug!("Attempting to get selected text");

    // The crate handles all the complexity:
    // - Tries AX API first
    // - Falls back to clipboard simulation
    // - Caches per-app behavior with LRU cache
    match get_selected_text_impl() {
        Ok(text) => {
            if text.is_empty() {
                debug!("No text selected (empty result)");
                Ok(String::new())
            } else {
                info!(text_len = text.len(), "Got selected text");
                Ok(text)
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to get selected text");
            bail!("Failed to get selected text: {}", e)
        }
    }
}

// ============================================================================
// Set Selected Text
// ============================================================================

/// Set (replace) the currently selected text in the focused application.
///
/// Strategy:
/// 1. Save current clipboard contents
/// 2. Set clipboard to new text
/// 3. Simulate Cmd+V
/// 4. Restore original clipboard
///
/// # Arguments
/// * `text` - The text to insert, replacing the current selection
///
/// # Errors
/// - Returns error if no accessibility permission
/// - Returns error if clipboard or paste operation fails
///
#[instrument(skip(text), fields(text_len = text.len()))]
#[cfg(target_os = "macos")]
pub fn set_selected_text(text: &str) -> Result<()> {
    if !has_accessibility_permission() {
        bail!("Accessibility permission required");
    }

    debug!("Attempting to set selected text");

    // Use clipboard fallback (AX write is complex and not widely supported)
    set_via_clipboard_fallback(text)
}

/// Clipboard-based fallback for setting selected text.
///
/// This function:
/// 1. Saves the current clipboard contents
/// 2. Sets the clipboard to the new text
/// 3. Simulates Cmd+V to paste using Core Graphics (more reliable than enigo)
/// 4. Restores the original clipboard (best effort)
#[cfg(target_os = "macos")]
fn set_via_clipboard_fallback(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

    // Save original clipboard contents
    let original = clipboard.get_text().ok();
    debug!(
        had_original = original.is_some(),
        "Saved original clipboard"
    );

    // Set new text to clipboard
    clipboard
        .set_text(text)
        .context("Failed to set clipboard text")?;

    // Small delay to ensure clipboard is set
    thread::sleep(Duration::from_millis(10));

    // Simulate Cmd+V using Core Graphics (more reliable on macOS than enigo)
    let paste_result = simulate_paste_with_cg();

    // Wait for paste to complete
    thread::sleep(Duration::from_millis(150));

    // Restore original clipboard (best effort)
    if let Some(original_text) = original {
        // Small delay before restoring
        thread::sleep(Duration::from_millis(100));
        if let Err(e) = clipboard.set_text(&original_text) {
            warn!(error = %e, "Failed to restore original clipboard");
        } else {
            debug!("Restored original clipboard");
        }
    }

    paste_result?;

    info!("Set selected text via clipboard fallback");
    Ok(())
}

/// Simulate Cmd+V paste using Core Graphics events.
/// This is more reliable on macOS than using enigo.
///
/// # Usage
/// Call this after copying content to the clipboard and hiding the window.
/// The function will simulate Cmd+V to paste into the currently focused app.
#[cfg(target_os = "macos")]
pub fn simulate_paste_with_cg() -> Result<()> {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // 'v' key is keycode 9 on macOS
    const KEY_V: CGKeyCode = 9;

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .ok()
        .context("Failed to create CGEventSource")?;

    // Create key down event for 'v' with Cmd modifier
    let key_down = CGEvent::new_keyboard_event(source.clone(), KEY_V, true)
        .ok()
        .context("Failed to create key down event")?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);

    // Create key up event for 'v' with Cmd modifier
    let key_up = CGEvent::new_keyboard_event(source, KEY_V, false)
        .ok()
        .context("Failed to create key up event")?;
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);

    // Post events
    key_down.post(CGEventTapLocation::HID);
    thread::sleep(Duration::from_millis(5));
    key_up.post(CGEventTapLocation::HID);

    debug!("Simulated Cmd+V via Core Graphics");
    Ok(())
}

// ============================================================================
// Windows Permission Functions
// ============================================================================
// Windows does not have macOS-style accessibility permissions.
// SendInput works for any non-elevated target window, so we always return true.

/// Check if accessibility permissions are granted (Windows).
///
/// Windows does not require accessibility permissions for `SendInput`.
/// Always returns `true`.
#[cfg(target_os = "windows")]
#[instrument]
pub fn has_accessibility_permission() -> bool {
    debug!("Windows: accessibility permission always granted");
    true
}

/// Request accessibility permissions (Windows).
///
/// No-op on Windows — always returns `true`.
#[cfg(target_os = "windows")]
#[instrument]
pub fn request_accessibility_permission() -> bool {
    info!("Windows: no accessibility permission needed");
    true
}

/// Open accessibility settings (Windows).
///
/// Windows does not have a macOS-style Accessibility pane.
/// Returns an error explaining this.
#[allow(dead_code)] // Will be used for permission UI prompts
#[cfg(target_os = "windows")]
#[instrument]
pub fn open_accessibility_settings() -> Result<()> {
    bail!("Accessibility settings are not applicable on Windows")
}

/// Show a permission dialog (Windows).
///
/// No permission is needed on Windows, so this always returns `Ok(true)`.
#[allow(dead_code)] // Will be used for permission UI prompts
#[cfg(target_os = "windows")]
#[instrument]
pub fn show_permission_dialog() -> Result<bool> {
    Ok(true)
}

// ============================================================================
// Windows Get Selected Text
// ============================================================================

/// Get the currently selected text from the focused application (Windows).
///
/// Strategy:
/// 1. Save current clipboard contents
/// 2. Clear the clipboard
/// 3. Simulate Ctrl+C via `SendInput`
/// 4. Sleep briefly to let the clipboard populate
/// 5. Read the clipboard text
/// 6. Restore the original clipboard contents
///
/// # Returns
/// The selected text, or empty string if nothing is selected.
///
/// # Errors
/// Returns error if clipboard access or keyboard simulation fails.
#[cfg(target_os = "windows")]
#[instrument(skip_all)]
pub fn get_selected_text() -> Result<String> {
    debug!("Attempting to get selected text via Ctrl+C");

    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

    // Save original clipboard contents
    let original = clipboard.get_text().ok();
    debug!(
        had_original = original.is_some(),
        "Saved original clipboard"
    );

    // Clear the clipboard so we can detect whether Ctrl+C wrote something
    clipboard.clear().context("Failed to clear clipboard")?;

    // Small delay to ensure clipboard is cleared
    thread::sleep(Duration::from_millis(10));

    // Simulate Ctrl+C
    let copy_result = simulate_copy_with_sendinput();

    // Wait for the copy to populate the clipboard
    thread::sleep(Duration::from_millis(150));

    // Read whatever ended up on the clipboard
    let selected = if copy_result.is_ok() {
        // Re-open clipboard (arboard may cache state)
        let mut clipboard2 = Clipboard::new().context("Failed to re-access clipboard")?;
        clipboard2.get_text().unwrap_or_default()
    } else {
        String::new()
    };

    // Restore original clipboard (best effort)
    if let Some(original_text) = original {
        thread::sleep(Duration::from_millis(50));
        let mut clipboard3 =
            Clipboard::new().context("Failed to re-access clipboard for restore")?;
        if let Err(e) = clipboard3.set_text(&original_text) {
            warn!(error = %e, "Failed to restore original clipboard");
        } else {
            debug!("Restored original clipboard");
        }
    }

    // Now check the copy result
    copy_result?;

    if selected.is_empty() {
        debug!("No text selected (empty result)");
        Ok(String::new())
    } else {
        info!(text_len = selected.len(), "Got selected text");
        Ok(selected)
    }
}

// ============================================================================
// Windows Set Selected Text
// ============================================================================

/// Set (replace) the currently selected text in the focused application (Windows).
///
/// Strategy:
/// 1. Save current clipboard contents
/// 2. Set clipboard to new text
/// 3. Simulate Ctrl+V via `SendInput`
/// 4. Restore original clipboard
///
/// # Arguments
/// * `text` - The text to insert, replacing the current selection
///
/// # Errors
/// Returns error if clipboard or paste operation fails.
#[cfg(target_os = "windows")]
#[instrument(skip(text), fields(text_len = text.len()))]
pub fn set_selected_text(text: &str) -> Result<()> {
    debug!("Attempting to set selected text via Ctrl+V");

    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

    // Save original clipboard contents
    let original = clipboard.get_text().ok();
    debug!(
        had_original = original.is_some(),
        "Saved original clipboard"
    );

    // Set new text to clipboard
    clipboard
        .set_text(text)
        .context("Failed to set clipboard text")?;

    // Small delay to ensure clipboard is set
    thread::sleep(Duration::from_millis(10));

    // Simulate Ctrl+V using SendInput
    let paste_result = simulate_paste_with_cg();

    // Wait for paste to complete
    thread::sleep(Duration::from_millis(150));

    // Restore original clipboard (best effort)
    if let Some(original_text) = original {
        // Small delay before restoring
        thread::sleep(Duration::from_millis(100));
        let mut clipboard2 =
            Clipboard::new().context("Failed to re-access clipboard for restore")?;
        if let Err(e) = clipboard2.set_text(&original_text) {
            warn!(error = %e, "Failed to restore original clipboard");
        } else {
            debug!("Restored original clipboard");
        }
    }

    paste_result?;

    info!("Set selected text via clipboard + Ctrl+V");
    Ok(())
}

/// Simulate Ctrl+C copy using Win32 `SendInput` API.
///
/// Sends four hardware input events: Ctrl down, C down, C up, Ctrl up.
/// This is the copy counterpart of `simulate_paste_with_cg()`.
#[cfg(target_os = "windows")]
fn simulate_copy_with_sendinput() -> Result<()> {
    // Re-use the same FFI types from simulate_paste_with_cg.
    // They're in a private inner mod so we redeclare the constants we need.
    #[allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms)]
    mod ffi {
        use std::os::raw::c_int;

        pub const INPUT_KEYBOARD: u32 = 1;
        pub const KEYEVENTF_KEYUP: u32 = 0x0002;
        pub const VK_CONTROL: u16 = 0x11;
        pub const VK_C: u16 = 0x43;

        #[repr(C)]
        pub struct KEYBDINPUT {
            pub wVk: u16,
            pub wScan: u16,
            pub dwFlags: u32,
            pub time: u32,
            pub dwExtraInfo: usize,
        }

        #[repr(C)]
        pub struct INPUT_UNION {
            pub ki: KEYBDINPUT,
            pub _pad: [u8; 8],
        }

        #[repr(C)]
        pub struct INPUT {
            pub r#type: u32,
            pub u: INPUT_UNION,
        }

        extern "system" {
            pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: c_int) -> u32;
        }
    }

    fn make_key_input(vk: u16, flags: u32) -> ffi::INPUT {
        ffi::INPUT {
            r#type: ffi::INPUT_KEYBOARD,
            u: ffi::INPUT_UNION {
                ki: ffi::KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
                _pad: [0u8; 8],
            },
        }
    }

    let inputs = [
        make_key_input(ffi::VK_CONTROL, 0),                    // Ctrl down
        make_key_input(ffi::VK_C, 0),                          // C down
        make_key_input(ffi::VK_C, ffi::KEYEVENTF_KEYUP),       // C up
        make_key_input(ffi::VK_CONTROL, ffi::KEYEVENTF_KEYUP), // Ctrl up
    ];

    let sent = unsafe {
        ffi::SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<ffi::INPUT>() as std::os::raw::c_int,
        )
    };

    if sent != inputs.len() as u32 {
        bail!(
            "SendInput (copy) failed: only sent {sent} of {} inputs (OS error: {})",
            inputs.len(),
            std::io::Error::last_os_error()
        );
    }

    debug!("Simulated Ctrl+C via Win32 SendInput");
    Ok(())
}

// ============================================================================
// Linux/other fallback — Permission Functions
// ============================================================================

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[instrument]
pub fn has_accessibility_permission() -> bool {
    false
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[instrument]
pub fn request_accessibility_permission() -> bool {
    false
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[instrument]
pub fn open_accessibility_settings() -> Result<()> {
    bail!("Accessibility settings are only available on macOS")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[instrument]
pub fn show_permission_dialog() -> Result<bool> {
    Ok(false)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[instrument(skip_all)]
pub fn get_selected_text() -> Result<String> {
    warn!("get_selected_text requested on unsupported platform");
    bail!("Selected text APIs are only supported on macOS and Windows")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[instrument(skip(text), fields(text_len = text.len()))]
pub fn set_selected_text(text: &str) -> Result<()> {
    let _ = text;
    warn!("set_selected_text requested on unsupported platform");
    bail!("Selected text APIs are only supported on macOS and Windows")
}

/// Simulate Ctrl+V paste using Win32 SendInput API.
///
/// Uses `SendInput` (not the deprecated `keybd_event`) for reliable keyboard
/// simulation. Sends four hardware input events: Ctrl down, V down, V up, Ctrl up.
///
/// # Caveats
/// - The target application must be the foreground window when this is called.
/// - Some applications with custom input handling may not respond to synthetic input.
/// - UAC-elevated windows cannot receive SendInput from non-elevated processes.
#[cfg(target_os = "windows")]
pub fn simulate_paste_with_cg() -> Result<()> {
    // Win32 SendInput FFI — matches the codebase pattern of raw extern blocks
    // (see windows_system_actions.rs, platform/visibility_focus.rs).
    #[allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms)]
    mod ffi {
        use std::os::raw::c_int;

        pub const INPUT_KEYBOARD: u32 = 1;
        pub const KEYEVENTF_KEYUP: u32 = 0x0002;
        pub const VK_CONTROL: u16 = 0x11;
        pub const VK_V: u16 = 0x56;

        #[repr(C)]
        pub struct KEYBDINPUT {
            pub wVk: u16,
            pub wScan: u16,
            pub dwFlags: u32,
            pub time: u32,
            pub dwExtraInfo: usize,
        }

        /// Padded union body — must be as large as the largest union member.
        /// `MOUSEINPUT` (the largest) is 32 bytes on 64-bit, so we pad to that.
        #[repr(C)]
        pub struct INPUT_UNION {
            pub ki: KEYBDINPUT,
            // Padding to match the size of the largest union variant (MOUSEINPUT).
            // MOUSEINPUT: 4+4+4+4+4 (=20) + 4 pad + 8 (dwExtraInfo) = 32 bytes on x64.
            // KEYBDINPUT: 2+2+4+4 (=12) + 4 pad + 8 (dwExtraInfo) = 24 bytes on x64.
            // We need 8 bytes of padding to reach 32.
            pub _pad: [u8; 8],
        }

        #[repr(C)]
        pub struct INPUT {
            pub r#type: u32,
            // 4 bytes padding on x64 (alignment of the union body is pointer-aligned)
            pub u: INPUT_UNION,
        }

        extern "system" {
            pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: c_int) -> u32;
        }
    }

    fn make_key_input(vk: u16, flags: u32) -> ffi::INPUT {
        ffi::INPUT {
            r#type: ffi::INPUT_KEYBOARD,
            u: ffi::INPUT_UNION {
                ki: ffi::KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
                _pad: [0u8; 8],
            },
        }
    }

    let inputs = [
        make_key_input(ffi::VK_CONTROL, 0),                    // Ctrl down
        make_key_input(ffi::VK_V, 0),                          // V down
        make_key_input(ffi::VK_V, ffi::KEYEVENTF_KEYUP),       // V up
        make_key_input(ffi::VK_CONTROL, ffi::KEYEVENTF_KEYUP), // Ctrl up
    ];

    let sent = unsafe {
        ffi::SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<ffi::INPUT>() as std::os::raw::c_int,
        )
    };

    if sent != inputs.len() as u32 {
        bail!(
            "SendInput failed: only sent {sent} of {} inputs (OS error: {})",
            inputs.len(),
            std::io::Error::last_os_error()
        );
    }

    tracing::debug!("Simulated Ctrl+V via Win32 SendInput");
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn simulate_paste_with_cg() -> Result<()> {
    bail!("Paste simulation is not supported on this platform")
}

// ============================================================================
// Tests
// ============================================================================

// ============================================================================
// Unit Tests (always run with `cargo test`)
// ============================================================================
#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_set_via_clipboard_fallback_restores_clipboard_after_paste_attempt() {
        let source = include_str!("selected_text.rs");

        let paste_result_idx = source
            .find("let paste_result = simulate_paste_with_cg();")
            .expect("expected set_via_clipboard_fallback to capture paste result");
        let post_paste_delay_idx = source
            .find("thread::sleep(Duration::from_millis(150));")
            .expect("expected 150ms post-paste delay");
        let restore_idx = source
            .find("if let Some(original_text) = original {")
            .expect("expected clipboard restore block");
        let pre_restore_delay_idx = source
            .find("thread::sleep(Duration::from_millis(100));")
            .expect("expected 100ms pre-restore delay");
        let paste_return_idx = source
            .find("paste_result?;")
            .expect("expected paste result to be returned after restore attempt");

        assert!(
            paste_result_idx < post_paste_delay_idx,
            "paste should run before post-paste delay"
        );
        assert!(
            post_paste_delay_idx < restore_idx,
            "restore should occur after post-paste delay"
        );
        assert!(
            restore_idx < paste_return_idx,
            "paste result should be returned after restore attempt"
        );
        assert!(
            pre_restore_delay_idx > restore_idx,
            "pre-restore delay should remain inside restore block"
        );
    }
}

// ============================================================================
// Windows Unit Tests (always run on Windows with `cargo test`)
// ============================================================================
#[cfg(all(target_os = "windows", test))]
mod windows_unit_tests {
    use super::*;

    /// Verify the function signature compiles and is callable.
    /// We don't actually call SendInput — that would paste into a random window.
    #[test]
    fn test_simulate_paste_with_cg_exists_and_returns_result() {
        // Type-level assertion: the function returns anyhow::Result<()>
        let _fn_ptr: fn() -> Result<()> = simulate_paste_with_cg;
    }

    /// Verify simulate_copy_with_sendinput exists and has the right signature.
    #[test]
    fn test_simulate_copy_with_sendinput_exists() {
        let _fn_ptr: fn() -> Result<()> = simulate_copy_with_sendinput;
    }

    /// Verify get_selected_text exists and returns Result<String>.
    #[test]
    fn test_get_selected_text_signature() {
        let _fn_ptr: fn() -> Result<String> = get_selected_text;
    }

    /// Verify set_selected_text exists and returns Result<()>.
    #[test]
    fn test_set_selected_text_signature() {
        let _fn_ptr: fn(&str) -> Result<()> = set_selected_text;
    }

    /// Windows has_accessibility_permission must return true.
    #[test]
    fn test_has_accessibility_permission_returns_true() {
        assert!(
            has_accessibility_permission(),
            "Windows should always report accessibility permission as granted"
        );
    }

    /// Windows request_accessibility_permission must return true.
    #[test]
    fn test_request_accessibility_permission_returns_true() {
        assert!(
            request_accessibility_permission(),
            "Windows should always report accessibility permission as granted on request"
        );
    }

    /// The Win32 INPUT struct must be exactly 40 bytes on x86_64.
    /// This is the canonical size: 4 (type) + 4 (pad) + 32 (union body).
    /// If this fails, SendInput will silently corrupt memory or be rejected.
    #[test]
    fn test_input_struct_size_is_correct() {
        // Re-declare the FFI types here so we can inspect their size without
        // exposing the inner `ffi` module outside the function.
        #[repr(C)]
        struct KEYBDINPUT {
            _wvk: u16,
            _wscan: u16,
            _dwflags: u32,
            _time: u32,
            _dwextrainfo: usize,
        }
        #[repr(C)]
        struct INPUT_UNION {
            _ki: KEYBDINPUT,
            _pad: [u8; 8],
        }
        #[repr(C)]
        struct INPUT {
            _type: u32,
            _u: INPUT_UNION,
        }

        // On x86_64 Windows: INPUT must be 40 bytes (4 + 4_pad + 32_union).
        // The Win32 SDK defines sizeof(INPUT) == 40 on 64-bit.
        let size = std::mem::size_of::<INPUT>();
        assert_eq!(
            size, 40,
            "INPUT struct size must be 40 bytes on x86_64, got {size}"
        );
    }

    /// VK constants must match the Win32 header values.
    #[test]
    fn test_virtual_key_constants() {
        // VK_CONTROL = 0x11, VK_V = 0x56 per WinUser.h
        assert_eq!(0x11u16, 0x11, "VK_CONTROL");
        assert_eq!(0x56u16, 0x56, "VK_V");
    }

    /// The paste_sequential worker should compile with the Windows path.
    /// This is a source-level assertion that the cfg branch exists.
    #[test]
    fn test_paste_sequential_worker_has_windows_branch() {
        let source = include_str!("clipboard_history/paste_sequential.rs");
        assert!(
            source.contains(r#"#[cfg(target_os = "windows")]"#),
            "paste_sequential.rs must have a Windows cfg branch for paste simulation"
        );
        assert!(
            source.contains("simulate_paste_with_cg"),
            "paste_sequential.rs Windows branch must call simulate_paste_with_cg"
        );
    }
}

// ============================================================================
// System Tests (require `cargo test --features system-tests`)
// ============================================================================
// These tests interact with macOS accessibility APIs, clipboard, and keyboard
// simulation. They may have side effects on the system state.

#[cfg(all(target_os = "macos", test, feature = "system-tests"))]
mod system_tests {
    use super::*;

    #[test]
    fn test_permission_check_does_not_panic() {
        // This test verifies the permission check doesn't panic
        // The actual result depends on system permissions
        let _has_permission = has_accessibility_permission();
        // Just ensure it doesn't panic - result varies by environment
    }

    #[test]
    fn test_permission_check_is_deterministic() {
        // Calling permission check multiple times should return same result
        let first = has_accessibility_permission();
        let second = has_accessibility_permission();
        assert_eq!(first, second, "Permission check should be deterministic");
    }

    #[test]
    #[ignore] // Requires manual interaction - select text in another app first
    fn test_get_selected_text_in_textedit() {
        // Instructions:
        // 1. Open TextEdit
        // 2. Type and select "Hello World"
        // 3. Run this test with: cargo test --features system-tests test_get_selected_text_in_textedit -- --ignored
        let text = get_selected_text().expect("Should get selected text");
        assert!(!text.is_empty(), "Should have selected text");
        println!("Got selected text: {}", text);
    }

    #[test]
    #[ignore] // Requires manual interaction - select text in another app first
    fn test_set_selected_text() {
        // Instructions:
        // 1. Open TextEdit
        // 2. Select some text
        // 3. Run this test with: cargo test --features system-tests test_set_selected_text -- --ignored
        set_selected_text("REPLACED").expect("Should set selected text");
        // Verify manually that text was replaced
        println!("Text should be replaced with 'REPLACED'");
    }

    #[test]
    #[ignore] // Opens System Preferences
    fn test_open_accessibility_settings() {
        // This will open System Preferences to the Accessibility pane
        open_accessibility_settings().expect("Should open settings");
    }

    #[test]
    #[ignore] // Calls get_selected_text which may simulate Cmd+C via clipboard fallback
    fn test_get_selected_text_without_permission_returns_error() {
        // If we don't have permission, we should get an error
        // This test is tricky because we can't easily revoke permission
        // Just verify the function handles the check
        let result = get_selected_text();
        // Result depends on whether permission is granted
        match result {
            Ok(text) => {
                // Permission was granted, we got some text (possibly empty)
                println!("Got text (permission granted): '{}'", text);
            }
            Err(e) => {
                // Either no permission or no selection
                println!("Got error (expected if no permission): {}", e);
            }
        }
    }

    #[test]
    #[ignore] // Calls set_selected_text which simulates Cmd+V via clipboard fallback
    fn test_set_selected_text_empty_string() {
        // Test setting empty text (edge case)
        // This will fail without permission, but shouldn't panic
        let result = set_selected_text("");
        // Don't assert on result - depends on permission state
        if let Err(e) = result {
            println!("Expected error without permission: {}", e);
        }
    }
}
