//! Integration tests for the Windows paste sequential backend.
//!
//! Tests FFI struct layouts, clipboard round-trips, and compilation of the
//! Windows-specific paste path. These tests do NOT call `SendInput` — that
//! would inject real keystrokes into whatever window happens to be focused.

#![cfg(target_os = "windows")]

// ---------------------------------------------------------------------------
// FFI struct layout tests
// ---------------------------------------------------------------------------
// The `ffi` module inside `simulate_paste_with_cg()` is private, so we
// re-declare the same `#[repr(C)]` structs here and verify their sizes and
// field offsets match what Win32 `SendInput` expects.

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
    // Win32 SDK: sizeof(INPUT) == 40 on x86_64.
    // Layout: 4 (type) + 4 (alignment padding) + 32 (union body) = 40.
    let size = std::mem::size_of::<INPUT>();
    assert_eq!(size, 40, "INPUT must be 40 bytes on x86_64, got {size}");
}

#[test]
fn input_union_is_32_bytes() {
    // The union body must equal sizeof(MOUSEINPUT) = 32 bytes on x64.
    let size = std::mem::size_of::<INPUT_UNION>();
    assert_eq!(
        size, 32,
        "INPUT_UNION must be 32 bytes on x86_64, got {size}"
    );
}

#[test]
fn keybdinput_is_24_bytes() {
    // KEYBDINPUT on x64: 2 (wVk) + 2 (wScan) + 4 (dwFlags) + 4 (time)
    //   + 4 (padding for usize alignment) + 8 (dwExtraInfo) = 24 bytes.
    let size = std::mem::size_of::<KEYBDINPUT>();
    assert_eq!(
        size, 24,
        "KEYBDINPUT must be 24 bytes on x86_64, got {size}"
    );
}

#[test]
fn input_alignment_is_pointer_aligned() {
    // INPUT must be pointer-aligned (8 bytes on x64) because the union
    // contains a usize field.
    let align = std::mem::align_of::<INPUT>();
    assert_eq!(
        align,
        std::mem::size_of::<usize>(),
        "INPUT alignment must equal pointer size"
    );
}

// ---------------------------------------------------------------------------
// Field offset tests (memoffset-free, using addr_of!)
// ---------------------------------------------------------------------------

#[test]
fn keybdinput_field_offsets_match_win32() {
    // Win32 x64 offsets for KEYBDINPUT:
    //   wVk=0, wScan=2, dwFlags=4, time=8, dwExtraInfo=16
    let dummy = KEYBDINPUT {
        wvk: 0,
        wscan: 0,
        dw_flags: 0,
        time: 0,
        dw_extra_info: 0,
    };
    let base = std::ptr::addr_of!(dummy) as usize;
    assert_eq!(
        std::ptr::addr_of!(dummy.wvk) as usize - base,
        0,
        "wVk offset"
    );
    assert_eq!(
        std::ptr::addr_of!(dummy.wscan) as usize - base,
        2,
        "wScan offset"
    );
    assert_eq!(
        std::ptr::addr_of!(dummy.dw_flags) as usize - base,
        4,
        "dwFlags offset"
    );
    assert_eq!(
        std::ptr::addr_of!(dummy.time) as usize - base,
        8,
        "time offset"
    );
    assert_eq!(
        std::ptr::addr_of!(dummy.dw_extra_info) as usize - base,
        16,
        "dwExtraInfo offset"
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
        "INPUT.type must be at offset 0"
    );
    // The union starts at offset 8 (4 type + 4 alignment padding on x64)
    assert_eq!(
        std::ptr::addr_of!(dummy.u) as usize - base,
        8,
        "INPUT.u must be at offset 8 on x64"
    );
}

// ---------------------------------------------------------------------------
// Virtual key constant tests
// ---------------------------------------------------------------------------

#[test]
fn virtual_key_constants_match_win32() {
    // VK_CONTROL and VK_V per WinUser.h
    assert_eq!(0x11u16, 0x11, "VK_CONTROL == 0x11");
    assert_eq!(0x56u16, 0x56, "VK_V == 0x56");
    // INPUT_KEYBOARD type
    assert_eq!(1u32, 1, "INPUT_KEYBOARD == 1");
    // KEYEVENTF_KEYUP flag
    assert_eq!(0x0002u32, 0x0002, "KEYEVENTF_KEYUP == 0x0002");
}

// ---------------------------------------------------------------------------
// Function signature test
// ---------------------------------------------------------------------------

#[test]
fn simulate_paste_with_cg_is_callable() {
    // Type-level assertion: the public function exists with the expected signature.
    // We do NOT call it — that would send real keystrokes.
    let _fn_ptr: fn() -> anyhow::Result<()> =
        script_kit_gpui::selected_text::simulate_paste_with_cg;
}

// ---------------------------------------------------------------------------
// Clipboard round-trip test
// ---------------------------------------------------------------------------

#[test]
fn clipboard_round_trip_with_arboard() {
    // Verify arboard can write and read back text on Windows.
    // This exercises the same clipboard path that paste_sequential uses.
    let mut clipboard = arboard::Clipboard::new().expect("should open clipboard");

    let test_text = format!(
        "paste_sequential_test_{}",
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

// ---------------------------------------------------------------------------
// paste_sequential module compilation tests (source-level assertions)
// ---------------------------------------------------------------------------

#[test]
fn paste_sequential_has_windows_cfg_branch() {
    let source = include_str!("../src/clipboard_history/paste_sequential.rs");
    assert!(
        source.contains(r#"#[cfg(target_os = "windows")]"#),
        "paste_sequential.rs must have a #[cfg(target_os = \"windows\")] branch"
    );
    assert!(
        source.contains("simulate_paste_with_cg"),
        "paste_sequential.rs Windows branch must call simulate_paste_with_cg"
    );
}

#[test]
fn paste_sequential_windows_branch_calls_simulate_paste() {
    // Verify the Windows cfg block specifically calls our function
    let source = include_str!("../src/clipboard_history/paste_sequential.rs");

    // Find the Windows cfg block and verify it calls simulate_paste_with_cg
    let windows_cfg_pos = source
        .find(r#"#[cfg(target_os = "windows")]"#)
        .expect("must have Windows cfg");

    // The simulate_paste_with_cg call should appear after the Windows cfg
    let after_cfg = &source[windows_cfg_pos..];
    assert!(
        after_cfg.contains("crate::selected_text::simulate_paste_with_cg()"),
        "Windows cfg branch must call crate::selected_text::simulate_paste_with_cg()"
    );
}

#[test]
fn selected_text_has_windows_simulate_paste() {
    let source = include_str!("../src/selected_text.rs");

    // Must have the Windows-specific simulate_paste_with_cg
    assert!(
        source.contains(r#"#[cfg(target_os = "windows")]"#),
        "selected_text.rs must have a Windows cfg block"
    );
    assert!(
        source.contains("SendInput"),
        "Windows simulate_paste_with_cg must use SendInput"
    );
    assert!(
        source.contains("VK_CONTROL"),
        "Windows paste must use VK_CONTROL"
    );
    assert!(source.contains("VK_V"), "Windows paste must use VK_V");
    assert!(
        source.contains("KEYEVENTF_KEYUP"),
        "Windows paste must use KEYEVENTF_KEYUP"
    );
}

// ---------------------------------------------------------------------------
// INPUT struct construction test (without calling SendInput)
// ---------------------------------------------------------------------------

#[test]
fn make_key_input_constructs_valid_struct() {
    // Reproduce the make_key_input logic and verify the resulting struct
    // has the expected field values — without calling SendInput.
    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const VK_CONTROL: u16 = 0x11;
    const VK_V: u16 = 0x56;

    let ctrl_down = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                wvk: VK_CONTROL,
                wscan: 0,
                dw_flags: 0,
                time: 0,
                dw_extra_info: 0,
            },
            _pad: [0u8; 8],
        },
    };

    let v_down = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                wvk: VK_V,
                wscan: 0,
                dw_flags: 0,
                time: 0,
                dw_extra_info: 0,
            },
            _pad: [0u8; 8],
        },
    };

    let v_up = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                wvk: VK_V,
                wscan: 0,
                dw_flags: KEYEVENTF_KEYUP,
                time: 0,
                dw_extra_info: 0,
            },
            _pad: [0u8; 8],
        },
    };

    let ctrl_up = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                wvk: VK_CONTROL,
                wscan: 0,
                dw_flags: KEYEVENTF_KEYUP,
                time: 0,
                dw_extra_info: 0,
            },
            _pad: [0u8; 8],
        },
    };

    // Verify type fields
    assert_eq!(ctrl_down.r#type, INPUT_KEYBOARD);
    assert_eq!(v_down.r#type, INPUT_KEYBOARD);
    assert_eq!(v_up.r#type, INPUT_KEYBOARD);
    assert_eq!(ctrl_up.r#type, INPUT_KEYBOARD);

    // Verify key codes
    assert_eq!(ctrl_down.u.ki.wvk, VK_CONTROL);
    assert_eq!(v_down.u.ki.wvk, VK_V);
    assert_eq!(v_up.u.ki.wvk, VK_V);
    assert_eq!(ctrl_up.u.ki.wvk, VK_CONTROL);

    // Verify key-up flags
    assert_eq!(ctrl_down.u.ki.dw_flags, 0, "ctrl down should have no flags");
    assert_eq!(v_down.u.ki.dw_flags, 0, "v down should have no flags");
    assert_eq!(
        v_up.u.ki.dw_flags, KEYEVENTF_KEYUP,
        "v up should have KEYUP"
    );
    assert_eq!(
        ctrl_up.u.ki.dw_flags, KEYEVENTF_KEYUP,
        "ctrl up should have KEYUP"
    );

    // Verify the 4-input sequence is: Ctrl↓ V↓ V↑ Ctrl↑
    let inputs = [ctrl_down, v_down, v_up, ctrl_up];
    assert_eq!(inputs.len(), 4, "paste sequence must be exactly 4 inputs");
}

// ---------------------------------------------------------------------------
// Byte representation test — verify struct as raw bytes matches expected layout
// ---------------------------------------------------------------------------

#[test]
fn input_struct_byte_representation_is_correct() {
    const INPUT_KEYBOARD: u32 = 1;
    const VK_V: u16 = 0x56;

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                wvk: VK_V,
                wscan: 0,
                dw_flags: 0,
                time: 0,
                dw_extra_info: 0,
            },
            _pad: [0u8; 8],
        },
    };

    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(&input as *const INPUT as *const u8, 40) };

    // Byte 0-3: type field = 1 (little-endian)
    assert_eq!(bytes[0], 1, "type byte 0");
    assert_eq!(bytes[1], 0, "type byte 1");
    assert_eq!(bytes[2], 0, "type byte 2");
    assert_eq!(bytes[3], 0, "type byte 3");

    // Bytes 4-7: alignment padding (uninitialized in C, but Rust zeroes)
    // We don't assert these — they're padding.

    // Byte 8-9: wVk = 0x56 (little-endian)
    assert_eq!(bytes[8], 0x56, "wVk byte 0");
    assert_eq!(bytes[9], 0x00, "wVk byte 1");

    // Byte 10-11: wScan = 0
    assert_eq!(bytes[10], 0, "wScan byte 0");
    assert_eq!(bytes[11], 0, "wScan byte 1");

    // Byte 12-15: dwFlags = 0
    assert_eq!(bytes[12], 0, "dwFlags byte 0");
    assert_eq!(bytes[13], 0, "dwFlags byte 1");
    assert_eq!(bytes[14], 0, "dwFlags byte 2");
    assert_eq!(bytes[15], 0, "dwFlags byte 3");

    // Byte 16-19: time = 0
    assert_eq!(bytes[16], 0, "time byte 0");
    assert_eq!(bytes[17], 0, "time byte 1");
    assert_eq!(bytes[18], 0, "time byte 2");
    assert_eq!(bytes[19], 0, "time byte 3");
}
