const ACCESSIBILITY_MOD: &str = include_str!("../../src/platform/accessibility/mod.rs");
const FOCUSED_TEXT: &str = include_str!("../../src/platform/accessibility/focused_text.rs");
const AX: &str = include_str!("../../src/platform/accessibility/ax.rs");
const CLIPBOARD: &str = include_str!("../../src/platform/accessibility/clipboard.rs");
const APP_IDENTITY: &str = include_str!("../../src/platform/accessibility/app_identity.rs");

#[test]
fn focused_text_platform_modules_exist() {
    for module in [
        "pub mod permissions;",
        "pub mod ax;",
        "pub mod app_identity;",
        "pub mod focused_text;",
        "pub mod geometry;",
        "pub mod mutation;",
        "pub mod clipboard;",
        "pub mod double_modifier_trigger;",
        "pub mod metrics;",
    ] {
        assert!(ACCESSIBILITY_MOD.contains(module), "missing {module}");
    }
}

#[test]
fn capture_api_returns_whole_field_snapshot_not_selected_text() {
    assert!(FOCUSED_TEXT.contains("capture_focused_text_field"));
    assert!(FOCUSED_TEXT.contains("FocusedTextSnapshot"));
    assert!(FOCUSED_TEXT.contains("text: String"));
    assert!(FOCUSED_TEXT.contains("selected_range_utf16"));
    assert!(
        !FOCUSED_TEXT.contains("get_selected_text("),
        "focused-text Agent Chat capture must not fake whole-field capture with selected-text APIs"
    );
}

#[test]
fn raw_ax_handles_stay_below_platform_boundary() {
    assert!(AX.contains("AxSessionHandle"));
    assert!(!FOCUSED_TEXT.contains("AXUIElementRef"));
}

#[test]
fn native_capture_uses_focused_ax_element_and_whole_value_fallback() {
    assert!(AX.contains("AXUIElementCreateSystemWide"));
    assert!(AX.contains("AXFocusedUIElement"));
    assert!(AX.contains("AXValue"));
    assert!(AX.contains("AXNumberOfCharacters"));
    assert!(AX.contains("AXStringForRange"));
    assert!(FOCUSED_TEXT.contains("copy_all_plain_text_preserving_clipboard"));
    assert!(FOCUSED_TEXT.contains("used_clipboard_fallback"));
    assert!(FOCUSED_TEXT.contains("|| used_clipboard_fallback"));
    assert!(
        !AX.contains("get_selected_text("),
        "focused-field AX capture must not call selected-text fallback"
    );
}

#[test]
fn google_docs_style_targets_can_fallback_to_clipboard_for_capture_and_replace() {
    for required in [
        "copy_all_plain_text_preserving_clipboard",
        "select_all_text_for_focused_text_fallback",
        "simulate_command_key(KEY_A)",
    ] {
        assert!(
            CLIPBOARD.contains(required),
            "clipboard fallback must support focused-text whole-field capture/apply: {required}"
        );
    }

    for required in [
        "paste_replace_fallback",
        "focused_text_replace_whole_text_failed_using_select_all_fallback",
        "select_all_text_for_focused_text_fallback",
        "verify_whole_text_or_clipboard_fallback",
        "verify_whole_text_or_clipboard_fallback(&target, text)",
        "focused_text_verify_whole_text_failed_trying_clipboard_fallback",
    ] {
        assert!(
            AX.contains(required),
            "AX mutation must support clipboard fallback verification for contenteditable targets: {required}"
        );
    }
}

#[test]
fn google_docs_style_append_uses_captured_text_before_clipboard_apply_fallback() {
    for required in [
        "captured_text: String",
        "captured_text: session.captured_text.clone()",
        "focused_text_append_whole_text_failed_using_captured_text_fallback",
        "target.captured_text.clone()",
        "verify_whole_text_or_clipboard_fallback(&target, &appended)",
    ] {
        assert!(
            AX.contains(required),
            "AX append fallback must preserve append semantics for contenteditable targets: {required}"
        );
    }
    assert!(
        !AX.contains("focused_text_append_whole_text_failed_pasting_at_caret"),
        "append fallback must not silently degrade to paste-at-caret when whole-text read fails"
    );
}

#[test]
fn native_capture_reads_app_identity_before_overlay_can_steal_focus() {
    assert!(APP_IDENTITY.contains("menuBarOwningApplication"));
    assert!(APP_IDENTITY.contains("processIdentifier"));
    assert!(APP_IDENTITY.contains("bundleIdentifier"));
    assert!(FOCUSED_TEXT.contains("current_frontmost_app_identity()"));
    assert!(FOCUSED_TEXT.contains("focused_ui_element_for_app(app.process_id)"));
}

#[test]
fn native_capture_rejects_secure_fields_and_omits_title_content() {
    assert!(FOCUSED_TEXT.contains("FocusedTextContentKind::Secure"));
    assert!(FOCUSED_TEXT.contains("return Err(FocusedTextError::SecureField)"));
    assert!(FOCUSED_TEXT.contains("title: None"));
    assert!(
        !AX.contains("AXTitle"),
        "capture should not pull target title text into focused-text Agent Chat snapshots"
    );
}
