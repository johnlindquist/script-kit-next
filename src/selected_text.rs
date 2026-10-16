//! Selected text operations using macOS Accessibility APIs
//!
//! This module provides getSelectedText() and setSelectedText() operations
//! using a hybrid approach: Accessibility API primary, clipboard fallback.
//!
//! ## Architecture
//!
//! - `get_selected_text()`: per-pid AX read of the source app's focused
//!   element first, then a selection-only simulated ⌘C with pasteboard
//!   change-count polling and snapshot/restore (works in AX-opaque editors
//!   like Google Docs)
//! - `set_selected_text()`: Uses clipboard + keyboard simulation (Cmd+V)
//!
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility

#[cfg(target_os = "macos")]
use anyhow::Context;
use anyhow::{bail, Result};
#[cfg(target_os = "macos")]
use macos_accessibility_client::accessibility;
#[cfg(target_os = "macos")]
use std::ffi::{c_void, CStr};
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "macos")]
use std::time::Duration;
#[cfg(target_os = "macos")]
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
    let result = crate::permissions::accessibility_granted();
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
    if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeVisibility)
        .is_err()
    {
        return false;
    }
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
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::OpenExternal)?;
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
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeVisibility)?;
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

#[cfg(target_os = "macos")]
type AXUIElementRef = *const c_void;
#[cfg(target_os = "macos")]
type CFTypeRef = *const c_void;
#[cfg(target_os = "macos")]
type CFStringRef = *const c_void;

#[cfg(target_os = "macos")]
const K_AX_ERROR_SUCCESS: i32 = 0;
#[cfg(target_os = "macos")]
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
#[cfg(target_os = "macos")]
const K_AX_VALUE_CF_RANGE_TYPE: i32 = 4;
#[cfg(target_os = "macos")]
const AX_FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
#[cfg(target_os = "macos")]
const AX_SELECTED_TEXT: &str = "AXSelectedText";
#[cfg(target_os = "macos")]
const AX_SELECTED_TEXT_RANGE: &str = "AXSelectedTextRange";
#[cfg(target_os = "macos")]
const AX_STRING_FOR_RANGE: &str = "AXStringForRange";

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct CFRange {
    location: isize,
    length: isize,
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementCopyParameterizedAttributeValue(
        element: AXUIElementRef,
        parameterized_attribute: CFStringRef,
        parameter: CFTypeRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXValueGetValue(value: CFTypeRef, the_type: i32, value_ptr: *mut c_void) -> bool;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFGetTypeID(cf: CFTypeRef) -> u64;
    fn CFStringGetTypeID() -> u64;
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetCString(
        string: CFStringRef,
        buffer: *mut i8,
        buffer_size: i64,
        encoding: u32,
    ) -> bool;
    fn CFStringGetLength(string: CFStringRef) -> i64;
}

/// Read selected text using Accessibility attributes only.
///
/// Unlike `get_selected_text`, this helper must never fall back to clipboard
/// simulation. It is for passive preview surfaces where posting Cmd+C would be
/// surprising system input.
#[instrument(skip_all)]
#[cfg(target_os = "macos")]
pub fn get_selected_text_ax_only() -> Result<Option<String>> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)?;
    if !has_accessibility_permission() {
        bail!(
            "Accessibility permission required. Enable in System Preferences > Privacy & Security > Accessibility"
        );
    }

    let system = unsafe { AXUIElementCreateSystemWide() };
    if system.is_null() {
        bail!("AXUIElementCreateSystemWide returned null");
    }

    let focused = match ax_copy_attribute(system, AX_FOCUSED_UI_ELEMENT) {
        Ok(value) => value as AXUIElementRef,
        Err(error) => {
            unsafe { CFRelease(system as CFTypeRef) };
            return Err(error).context("AXFocusedUIElement unavailable");
        }
    };
    unsafe { CFRelease(system as CFTypeRef) };

    let selected_text = ax_selected_text_for_element(focused);
    unsafe { CFRelease(focused as CFTypeRef) };
    selected_text
}

#[cfg(target_os = "macos")]
fn ax_selected_text_for_element(element: AXUIElementRef) -> Result<Option<String>> {
    if element.is_null() {
        return Ok(None);
    }

    if let Ok(value) = ax_copy_attribute(element, AX_SELECTED_TEXT) {
        let text = cf_string_to_string_if_string(value);
        unsafe { CFRelease(value) };
        if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
            return Ok(Some(text));
        }
    }

    let Some(range) = ax_selected_text_range(element)? else {
        return Ok(None);
    };
    if range.length <= 0 {
        return Ok(None);
    }

    ax_string_for_range(element, range).map(|text| text.filter(|text| !text.trim().is_empty()))
}

#[cfg(target_os = "macos")]
fn ax_selected_text_range(element: AXUIElementRef) -> Result<Option<CFRange>> {
    let value = match ax_copy_attribute(element, AX_SELECTED_TEXT_RANGE) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    let mut range = CFRange::default();
    let ok = unsafe {
        AXValueGetValue(
            value,
            K_AX_VALUE_CF_RANGE_TYPE,
            &mut range as *mut CFRange as *mut c_void,
        )
    };
    unsafe { CFRelease(value) };

    if ok {
        Ok(Some(range))
    } else {
        Ok(None)
    }
}

#[cfg(target_os = "macos")]
fn ax_string_for_range(element: AXUIElementRef, range: CFRange) -> Result<Option<String>> {
    let range_value = ax_value_create_cf_range(range)?;
    let value = match ax_copy_parameterized_attribute(element, AX_STRING_FOR_RANGE, range_value) {
        Ok(value) => value,
        Err(_) => {
            unsafe { CFRelease(range_value) };
            return Ok(None);
        }
    };
    unsafe { CFRelease(range_value) };

    let text = cf_string_to_string_if_string(value);
    unsafe { CFRelease(value) };
    Ok(text)
}

#[cfg(target_os = "macos")]
fn ax_value_create_cf_range(range: CFRange) -> Result<CFTypeRef> {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXValueCreate(the_type: i32, value_ptr: *const c_void) -> CFTypeRef;
    }

    let value = unsafe {
        AXValueCreate(
            K_AX_VALUE_CF_RANGE_TYPE,
            &range as *const CFRange as *const c_void,
        )
    };
    if value.is_null() {
        bail!("AXValueCreate failed for selected text range");
    }
    Ok(value)
}

#[cfg(target_os = "macos")]
fn ax_copy_attribute(element: AXUIElementRef, attribute: &str) -> Result<CFTypeRef> {
    let attr = create_cf_string(attribute)?;
    let mut value: CFTypeRef = std::ptr::null();
    let result = unsafe { AXUIElementCopyAttributeValue(element, attr, &mut value) };
    unsafe { CFRelease(attr as CFTypeRef) };
    if result != K_AX_ERROR_SUCCESS || value.is_null() {
        bail!("AX attribute {attribute} unavailable: error {result}");
    }
    Ok(value)
}

#[cfg(target_os = "macos")]
fn ax_copy_parameterized_attribute(
    element: AXUIElementRef,
    attribute: &str,
    parameter: CFTypeRef,
) -> Result<CFTypeRef> {
    let attr = create_cf_string(attribute)?;
    let mut value: CFTypeRef = std::ptr::null();
    let result =
        unsafe { AXUIElementCopyParameterizedAttributeValue(element, attr, parameter, &mut value) };
    unsafe { CFRelease(attr as CFTypeRef) };
    if result != K_AX_ERROR_SUCCESS || value.is_null() {
        bail!("AX parameterized attribute {attribute} unavailable: error {result}");
    }
    Ok(value)
}

#[cfg(target_os = "macos")]
fn create_cf_string(s: &str) -> Result<CFStringRef> {
    let c_string = std::ffi::CString::new(s)
        .with_context(|| format!("CFString input contains interior NUL: {s:?}"))?;
    let cf_string = unsafe {
        CFStringCreateWithCString(
            std::ptr::null(),
            c_string.as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        )
    };
    if cf_string.is_null() {
        bail!("CFStringCreateWithCString returned null for {s:?}");
    }
    Ok(cf_string)
}

#[cfg(target_os = "macos")]
fn cf_string_to_string_if_string(value: CFTypeRef) -> Option<String> {
    if unsafe { CFGetTypeID(value) } != unsafe { CFStringGetTypeID() } {
        return None;
    }
    unsafe {
        let length = CFStringGetLength(value as CFStringRef);
        let buffer_size = length.saturating_mul(4).saturating_add(1).max(1);
        let mut buffer = vec![0_i8; buffer_size as usize];
        let ok = CFStringGetCString(
            value as CFStringRef,
            buffer.as_mut_ptr(),
            buffer_size,
            K_CF_STRING_ENCODING_UTF8,
        );
        if !ok {
            return None;
        }
        CStr::from_ptr(buffer.as_ptr())
            .to_str()
            .ok()
            .map(str::to_string)
    }
}

/// Get the currently selected text from the focused application.
///
/// Chain (Raycast parity):
/// 1. Per-pid AX read of the source app's focused element (AXSelectedText,
///    then AXSelectedTextRange + AXStringForRange). Targeting the last real
///    app by pid matters: the system-wide focused element is unreliable
///    cross-process and can resolve to our own panel once it is key.
/// 2. Simulated ⌘C with pasteboard change-count polling and snapshot/restore.
///    This is the only channel that works in AX-opaque editors like Google
///    Docs, whose canvas renderer exposes no AX text at all. It never posts
///    ⌘A, so the user's selection survives.
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
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)?;
    // Check permissions first
    if !has_accessibility_permission() {
        bail!(
            "Accessibility permission required. Enable in System Preferences > Privacy & Security > Accessibility"
        );
    }

    debug!("Attempting to get selected text");

    let source_pid = crate::frontmost_app_tracker::get_last_real_app().map(|app| app.pid);
    match crate::platform::accessibility::focused_text::selected_text_for_app_ax_only(source_pid) {
        Ok(Some(text)) => {
            info!(text_len = text.len(), "Got selected text via AX");
            return Ok(text);
        }
        Ok(None) => {
            debug!("AX reported no selection; trying selection copy fallback");
        }
        Err(error) => {
            debug!(%error, "AX selection read failed; trying selection copy fallback");
        }
    }

    match crate::platform::accessibility::clipboard::copy_selection_plain_text_preserving_clipboard(
    ) {
        Ok(Some(text)) => {
            info!(text_len = text.len(), "Got selected text via copy fallback");
            Ok(text)
        }
        Ok(None) => {
            debug!("No text selected (copy fallback produced no clipboard change)");
            Ok(String::new())
        }
        Err(e) => {
            warn!(error = %e, "Failed to get selected text");
            bail!("Failed to get selected text: {}", e)
        }
    }
}

/// What kind of text a rewrite capture actually grabbed. Surfaces use this
/// to tell the user where the text came from ("Selection" vs "Draft").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapturedTextKind {
    /// The user's explicit selection.
    Selection,
    /// The focused field's whole text — no selection existed at capture time.
    FocusedField,
}

/// Text captured for a rewrite, plus where it was grabbed from.
#[derive(Debug, Clone)]
pub struct CapturedText {
    pub text: String,
    pub kind: CapturedTextKind,
    /// Name of the app the text was captured from, when known.
    pub source_app: Option<String>,
}

/// Selection if any; otherwise the focused field's whole text — "what the
/// user is working on".
///
/// Used by the rewrite flow at explicit submit: a user who types a tweet and
/// invokes `.concise` without selecting first still means "rewrite my draft".
/// Chain after `get_selected_text` comes back empty:
/// 1. Passive per-pid AX whole-field read (no keystrokes, no pasteboard).
/// 2. ⌘A+⌘C with clipboard snapshot/restore for AX-opaque editors (Chrome,
///    Google Docs). Side effect: leaves the field's text selected.
///
/// All results are capped at the focused-text capture ceiling so a huge
/// document can never flood the prompt.
#[instrument(skip_all)]
#[cfg(target_os = "macos")]
pub fn capture_selection_or_focused_text() -> Result<Option<CapturedText>> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)?;
    let source = crate::frontmost_app_tracker::get_last_real_app();
    let source_app = source.as_ref().map(|app| app.name.clone());
    let source_pid = source.map(|app| app.pid);

    let selection = get_selected_text()?;
    if !selection.trim().is_empty() {
        return Ok(Some(CapturedText {
            text: selection,
            kind: CapturedTextKind::Selection,
            source_app,
        }));
    }

    match crate::platform::accessibility::focused_text::focused_text_for_app_ax_only(source_pid) {
        Ok(Some(text)) => {
            info!(
                text_len = text.len(),
                "No selection; using focused-field text via AX"
            );
            return Ok(Some(CapturedText {
                text,
                kind: CapturedTextKind::FocusedField,
                source_app,
            }));
        }
        Ok(None) => {
            debug!("AX reported empty focused field; trying copy-all fallback");
        }
        Err(error) => {
            debug!(%error, "AX focused-field read failed; trying copy-all fallback");
        }
    }

    match crate::platform::accessibility::clipboard::copy_all_plain_text_preserving_clipboard() {
        Ok(text) => {
            let (text, truncated) =
                crate::platform::accessibility::focused_text::truncate_focused_text_capture(text);
            info!(
                text_len = text.len(),
                truncated, "No selection; using focused-field text via copy-all fallback"
            );
            Ok(Some(CapturedText {
                text,
                kind: CapturedTextKind::FocusedField,
                source_app,
            })
            .filter(|captured| !captured.text.trim().is_empty()))
        }
        Err(error) => {
            debug!(%error, "Copy-all fallback captured nothing; treating as no text");
            Ok(None)
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[instrument(skip_all)]
pub fn capture_selection_or_focused_text() -> Result<Option<CapturedText>> {
    let text = get_selected_text()?;
    Ok((!text.trim().is_empty()).then(|| CapturedText {
        text,
        kind: CapturedTextKind::Selection,
        source_app: None,
    }))
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
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
    if !has_accessibility_permission() {
        bail!("Accessibility permission required");
    }

    debug!("Attempting to set selected text");

    // Use clipboard fallback (AX write is complex and not widely supported)
    set_via_clipboard_fallback(text)
}

/// Clipboard-based fallback for setting selected text.
///
/// Pastes through the shared promised-paste path: the user's full clipboard
/// (every item and type) is snapshotted, the text is pasted into the focused
/// app, and the snapshot is restored once the app has read the text. A
/// clipboard written by someone else during the paste is kept, and reported.
#[cfg(target_os = "macos")]
fn set_via_clipboard_fallback(text: &str) -> Result<()> {
    use crate::platform::promised_paste::{paste_text_preserving_clipboard, RestoreOutcome};

    let report = paste_text_preserving_clipboard(text, None)
        .context("Failed to paste replacement text into the focused app")?;
    debug!(
        item_count = report.original.item_count,
        type_count = report.original.type_count,
        total_bytes = report.original.total_bytes,
        has_text = report.original.has_text,
        has_rich_text = report.original.has_rich_text,
        has_image = report.original.has_image,
        has_file_url = report.original.has_file_url,
        has_other = report.original.has_other,
        delivery = report.delivery.as_str(),
        consumption = report.consumption.as_str(),
        restore = report.restore.as_str(),
        "Selected-text replacement clipboard round trip"
    );
    if matches!(
        report.consumption,
        crate::platform::promised_paste::PasteConsumption::NotRequested { .. }
    ) {
        bail!("The focused app didn't take the replacement text");
    }
    match report.restore {
        RestoreOutcome::Restored | RestoreOutcome::KeptPastedText => {}
        RestoreOutcome::SkippedExternalChange => bail!(
            "Clipboard changed during selected-text replacement; skipped restore to avoid overwriting external clipboard update"
        ),
        RestoreOutcome::Failed(error) => bail!(
            "Failed to restore original clipboard after selected-text replacement: {error}"
        ),
    }

    info!("Set selected text via clipboard fallback");
    Ok(())
}

/// Post ⌘V to the app that regains focus after Script Kit hides.
///
/// Callers put content on the clipboard and hide the window first. This waits
/// (briefly) until focus has actually left Script Kit, then posts ⌘V straight
/// to that app, using the current layout's V key and no stray modifiers.
#[cfg(target_os = "macos")]
pub fn simulate_paste_with_cg() -> Result<()> {
    use crate::platform::synthetic_keys;
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;

    const FOCUS_HANDOFF_TIMEOUT: Duration = Duration::from_millis(400);
    let target = synthetic_keys::key_target_after_hiding_self(FOCUS_HANDOFF_TIMEOUT);
    synthetic_keys::post_paste_shortcut(target)?;
    debug!(route = target.route_name(), "Simulated Cmd+V");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[instrument]
pub fn has_accessibility_permission() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
#[instrument]
pub fn request_accessibility_permission() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
#[instrument]
pub fn open_accessibility_settings() -> Result<()> {
    bail!("Accessibility settings are only available on macOS")
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
#[instrument]
pub fn show_permission_dialog() -> Result<bool> {
    Ok(false)
}

#[cfg(not(target_os = "macos"))]
#[instrument(skip_all)]
pub fn get_selected_text() -> Result<String> {
    warn!("get_selected_text requested on unsupported platform");
    bail!("Selected text APIs are only supported on macOS")
}

#[cfg(not(target_os = "macos"))]
#[instrument(skip_all)]
pub fn get_selected_text_ax_only() -> Result<Option<String>> {
    Ok(None)
}

#[cfg(not(target_os = "macos"))]
#[instrument(skip(text), fields(text_len = text.len()))]
pub fn set_selected_text(text: &str) -> Result<()> {
    let _ = text;
    warn!("set_selected_text requested on unsupported platform");
    bail!("Selected text APIs are only supported on macOS")
}

#[cfg(not(target_os = "macos"))]
pub fn simulate_paste_with_cg() -> Result<()> {
    bail!("Paste simulation is only supported on macOS")
}

// ============================================================================
// Tests
// ============================================================================

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
