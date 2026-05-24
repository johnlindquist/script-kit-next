const ACCESSIBILITY_MOD: &str = include_str!("../../src/platform/accessibility/mod.rs");
const FOCUSED_TEXT: &str = include_str!("../../src/platform/accessibility/focused_text.rs");
const AX: &str = include_str!("../../src/platform/accessibility/ax.rs");

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
