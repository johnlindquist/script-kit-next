const ACCESSIBILITY_MOD: &str = include_str!("../../src/platform/accessibility/mod.rs");
const FOCUSED_TEXT: &str = include_str!("../../src/platform/accessibility/focused_text.rs");
const AX: &str = include_str!("../../src/platform/accessibility/ax.rs");
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
        "inline agent capture must not fake whole-field capture with selected-text APIs"
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
    assert!(
        !AX.contains("get_selected_text("),
        "focused-field AX capture must not call selected-text fallback"
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
        "capture should not pull target title text into inline-agent snapshots"
    );
}
