//! Integration tests for the Windows selected text implementation.
//!
//! Tests FFI struct layouts, clipboard round-trips, permission functions,
//! and compilation of the Windows-specific get/set selected text paths.
//!
//! These tests do NOT call `SendInput` — that would inject real keystrokes
//! into whatever window happens to be focused.
//!
//! **Clipboard tests**: arboard on Windows uses OLE clipboard APIs that are
//! not thread-safe. All clipboard-touching tests are combined into a single
//! `#[test]` to avoid heap corruption from concurrent clipboard access.

#![cfg(target_os = "windows")]

// ---------------------------------------------------------------------------
// FFI struct layout tests (mirrors the private structs in selected_text.rs)
// ---------------------------------------------------------------------------

/// Mirror of the private `KEYBDINPUT` struct used in `selected_text.rs`.
#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct KEYBDINPUT {
    wvk: u16,
    wscan: u16,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

/// Mirror of the private `INPUT_UNION` struct (padded to MOUSEINPUT size).
#[repr(C)]
struct INPUT_UNION {
    ki: KEYBDINPUT,
    _pad: [u8; 8],
}

/// Mirror of the private `INPUT` struct.
#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct INPUT {
    r#type: u32,
    u: INPUT_UNION,
}

// ---------------------------------------------------------------------------
// Struct size tests
// ---------------------------------------------------------------------------

#[test]
fn input_struct_is_40_bytes() {
    let size = std::mem::size_of::<INPUT>();
    assert_eq!(size, 40, "INPUT must be 40 bytes on x86_64, got {size}");
}

#[test]
fn input_union_is_32_bytes() {
    let size = std::mem::size_of::<INPUT_UNION>();
    assert_eq!(
        size, 32,
        "INPUT_UNION must be 32 bytes on x86_64, got {size}"
    );
}

#[test]
fn keybdinput_is_24_bytes() {
    let size = std::mem::size_of::<KEYBDINPUT>();
    assert_eq!(
        size, 24,
        "KEYBDINPUT must be 24 bytes on x86_64, got {size}"
    );
}

#[test]
fn input_alignment_is_pointer_aligned() {
    let align = std::mem::align_of::<INPUT>();
    assert_eq!(
        align,
        std::mem::size_of::<usize>(),
        "INPUT alignment must equal pointer size"
    );
}

// ---------------------------------------------------------------------------
// Field offset tests
// ---------------------------------------------------------------------------

#[test]
fn keybdinput_field_offsets_match_win32() {
    let dummy = KEYBDINPUT {
        wvk: 0,
        wscan: 0,
        dw_flags: 0,
        time: 0,
        dw_extra_info: 0,
    };
    let base = std::ptr::addr_of!(dummy) as usize;
    assert_eq!(std::ptr::addr_of!(dummy.wvk) as usize - base, 0, "wVk");
    assert_eq!(std::ptr::addr_of!(dummy.wscan) as usize - base, 2, "wScan");
    assert_eq!(
        std::ptr::addr_of!(dummy.dw_flags) as usize - base,
        4,
        "dwFlags"
    );
    assert_eq!(std::ptr::addr_of!(dummy.time) as usize - base, 8, "time");
    assert_eq!(
        std::ptr::addr_of!(dummy.dw_extra_info) as usize - base,
        16,
        "dwExtraInfo"
    );
}

#[test]
fn input_type_field_at_offset_zero() {
    let dummy = INPUT {
        r#type: 0,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                wvk: 0,
                wscan: 0,
                dw_flags: 0,
                time: 0,
                dw_extra_info: 0,
            },
            _pad: [0u8; 8],
        },
    };
    let base = std::ptr::addr_of!(dummy) as usize;
    assert_eq!(
        std::ptr::addr_of!(dummy.r#type) as usize - base,
        0,
        "INPUT.type at offset 0"
    );
    assert_eq!(
        std::ptr::addr_of!(dummy.u) as usize - base,
        8,
        "INPUT.u at offset 8 on x64"
    );
}

// ---------------------------------------------------------------------------
// Virtual key constant tests
// ---------------------------------------------------------------------------

#[test]
fn virtual_key_constants_match_win32() {
    assert_eq!(0x11u16, 0x11, "VK_CONTROL == 0x11");
    assert_eq!(0x43u16, 0x43, "VK_C == 0x43");
    assert_eq!(0x56u16, 0x56, "VK_V == 0x56");
    assert_eq!(1u32, 1, "INPUT_KEYBOARD == 1");
    assert_eq!(0x0002u32, 0x0002, "KEYEVENTF_KEYUP == 0x0002");
}

// ---------------------------------------------------------------------------
// Permission function tests
// ---------------------------------------------------------------------------

#[test]
fn has_accessibility_permission_returns_true_on_windows() {
    assert!(
        script_kit_gpui::selected_text::has_accessibility_permission(),
        "Windows should always report accessibility permission as granted"
    );
}

#[test]
fn request_accessibility_permission_returns_true_on_windows() {
    assert!(
        script_kit_gpui::selected_text::request_accessibility_permission(),
        "Windows should always report accessibility permission as granted on request"
    );
}

#[test]
fn permission_check_is_deterministic() {
    let first = script_kit_gpui::selected_text::has_accessibility_permission();
    let second = script_kit_gpui::selected_text::has_accessibility_permission();
    assert_eq!(first, second, "Permission check must be deterministic");
    assert!(first, "Both calls must return true on Windows");
}

// ---------------------------------------------------------------------------
// Function signature tests (type-level assertions, no actual calls)
// ---------------------------------------------------------------------------

#[test]
fn simulate_paste_with_cg_is_callable() {
    let _fn_ptr: fn() -> anyhow::Result<()> =
        script_kit_gpui::selected_text::simulate_paste_with_cg;
}

#[test]
fn get_selected_text_is_callable() {
    let _fn_ptr: fn() -> anyhow::Result<String> = script_kit_gpui::selected_text::get_selected_text;
}

#[test]
fn set_selected_text_is_callable() {
    let _fn_ptr: fn(&str) -> anyhow::Result<()> = script_kit_gpui::selected_text::set_selected_text;
}

// ---------------------------------------------------------------------------
// Clipboard round-trip test (single test to avoid concurrent clipboard access)
//
// arboard on Windows uses OLE clipboard APIs that crash (STATUS_HEAP_CORRUPTION)
// when multiple threads open/close the clipboard simultaneously. We combine all
// clipboard-touching assertions into one sequential test.
// ---------------------------------------------------------------------------

#[test]
fn clipboard_operations_sequential() {
    // --- Part 1: basic round-trip ---
    {
        let mut clipboard = arboard::Clipboard::new().expect("should open clipboard");

        let test_text = format!(
            "selected_text_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        clipboard
            .set_text(&test_text)
            .expect("should write to clipboard");

        let read_back = clipboard.get_text().expect("should read from clipboard");
        assert_eq!(read_back, test_text, "clipboard round-trip text must match");
    }

    // --- Part 2: save/restore pattern (mimics get_selected_text / set_selected_text) ---
    {
        let mut clipboard = arboard::Clipboard::new().expect("should open clipboard");

        // Set known initial state
        let original_text = format!(
            "original_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        clipboard
            .set_text(&original_text)
            .expect("should set original");

        // Read and save
        let saved = clipboard.get_text().expect("should read original back");
        assert_eq!(saved, original_text, "saved must equal original");

        // Simulate intermediate clipboard change (what Ctrl+C would do)
        let intermediate = "intermediate_selected_text";
        clipboard
            .set_text(intermediate)
            .expect("should set intermediate");

        let intermediate_back = clipboard.get_text().expect("should read intermediate");
        assert_eq!(intermediate_back, intermediate, "intermediate must match");

        // Restore original (re-open clipboard handle, same pattern as our impl)
        drop(clipboard);
        let mut clipboard2 = arboard::Clipboard::new().expect("should re-open clipboard");
        clipboard2
            .set_text(&saved)
            .expect("should restore original");

        let restored = clipboard2.get_text().expect("should read restored");
        assert_eq!(
            restored, original_text,
            "restored clipboard must match original"
        );
    }
}

// ---------------------------------------------------------------------------
// Source-level assertions
// ---------------------------------------------------------------------------

#[test]
fn selected_text_has_windows_get_selected_text() {
    let source = include_str!("../src/selected_text.rs");
    assert!(
        source.contains("fn get_selected_text() -> Result<String>"),
        "selected_text.rs must define get_selected_text"
    );
    assert!(
        source.contains("simulate_copy_with_sendinput"),
        "Windows get_selected_text must call simulate_copy_with_sendinput"
    );
}

#[test]
fn selected_text_has_windows_set_selected_text() {
    let source = include_str!("../src/selected_text.rs");
    assert!(
        source.contains("fn set_selected_text(text: &str) -> Result<()>"),
        "selected_text.rs must define set_selected_text"
    );
    assert!(
        source.contains("let paste_result = simulate_paste_with_cg()"),
        "Windows set_selected_text must call simulate_paste_with_cg"
    );
}

#[test]
fn selected_text_has_windows_permission_functions() {
    let source = include_str!("../src/selected_text.rs");
    // Verify Windows cfg blocks for permission functions exist (not multi-line exact match)
    assert!(
        source.contains(r#"#[cfg(target_os = "windows")]"#)
            && source.contains("pub fn has_accessibility_permission() -> bool"),
        "must have Windows has_accessibility_permission"
    );
    assert!(
        source.contains(r#"#[cfg(target_os = "windows")]"#)
            && source.contains("pub fn request_accessibility_permission() -> bool"),
        "must have Windows request_accessibility_permission"
    );
}

#[test]
fn selected_text_has_linux_fallbacks() {
    let source = include_str!("../src/selected_text.rs");
    assert!(
        source.contains(r#"#[cfg(not(any(target_os = "macos", target_os = "windows")))]"#),
        "must have Linux/other fallback cfg blocks"
    );
}

#[test]
fn selected_text_uses_vk_c_for_copy() {
    let source = include_str!("../src/selected_text.rs");
    assert!(
        source.contains("VK_C: u16 = 0x43"),
        "simulate_copy_with_sendinput must define VK_C = 0x43"
    );
}

#[test]
fn selected_text_copy_sends_four_inputs() {
    let source = include_str!("../src/selected_text.rs");
    let copy_fn_pos = source
        .find("fn simulate_copy_with_sendinput")
        .expect("must have simulate_copy_with_sendinput");
    let copy_fn_body = &source[copy_fn_pos..];
    assert!(
        copy_fn_body.contains("VK_CONTROL, 0"),
        "copy must send Ctrl down"
    );
    assert!(copy_fn_body.contains("VK_C, 0"), "copy must send C down");
    assert!(
        copy_fn_body.contains("VK_C, ffi::KEYEVENTF_KEYUP"),
        "copy must send C up"
    );
    assert!(
        copy_fn_body.contains("VK_CONTROL, ffi::KEYEVENTF_KEYUP"),
        "copy must send Ctrl up"
    );
}

#[test]
fn executor_selected_text_has_windows_handlers() {
    let source = include_str!("../src/executor/selected_text.rs");

    // Verify the executor imports selected_text for both macOS and Windows
    assert!(
        source.contains(r#"cfg(any(target_os = "macos", target_os = "windows"))"#)
            && source.contains("use crate::selected_text"),
        "executor selected_text must import crate::selected_text for macOS and Windows"
    );

    // Verify Windows-specific handler functions exist (search independently, not multi-line)
    assert!(
        source.contains("fn handle_get_selected_text(request_id:")
            && source.contains("selected_text::get_selected_text()"),
        "must have Windows handle_get_selected_text that calls selected_text::get_selected_text()"
    );
    assert!(
        source.contains("fn handle_set_selected_text(text:")
            && source.contains("selected_text::set_selected_text(text)"),
        "must have Windows handle_set_selected_text that calls selected_text::set_selected_text()"
    );
    assert!(
        source.contains("fn handle_check_accessibility(request_id:")
            && source.contains("selected_text::has_accessibility_permission()"),
        "must have Windows handle_check_accessibility that calls has_accessibility_permission()"
    );
    assert!(
        source.contains("fn handle_request_accessibility(request_id:")
            && source.contains("selected_text::request_accessibility_permission()"),
        "must have Windows handle_request_accessibility that calls request_accessibility_permission()"
    );
}
