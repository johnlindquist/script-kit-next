const LIVE_PREVIEW: &str = include_str!("../../src/spine/live_preview.rs");
const SELECTED_TEXT: &str = include_str!("../../src/selected_text.rs");

fn function_body<'a>(source: &'a str, signature: &str, next_signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_start = &source[start..];
    let end = after_start
        .find(next_signature)
        .unwrap_or_else(|| panic!("missing next function signature: {next_signature}"));
    &after_start[..end]
}

/// Passive Spine previews run while the launcher is open and must never post
/// system input. The fallback-capable selected-text API can synthesize Cmd+C,
/// so previews are restricted to the AX-only helper.
#[test]
fn passive_spine_preview_uses_ax_only_selection_capture() {
    let preview_body = function_body(
        LIVE_PREVIEW,
        "pub(crate) fn refresh_preview_nonblocking",
        "pub(crate) fn set_script_count",
    );

    assert!(preview_body.contains("get_selected_text_ax_only()"));
    assert!(
        !preview_body.contains("get_selected_text()"),
        "passive Spine preview must not call the fallback-capable selected-text API"
    );
}

/// `get_selected_text_ax_only` is intentionally a separate passive API from
/// `get_selected_text`: it may read AX attributes, but it must not use the
/// third-party fallback helper or CoreGraphics keyboard event posting.
#[test]
fn ax_only_selected_text_helper_does_not_synthesize_copy() {
    let ax_only_body = function_body(
        SELECTED_TEXT,
        "pub fn get_selected_text_ax_only",
        "fn ax_selected_text_for_element",
    );

    for forbidden in [
        "get_selected_text_impl",
        "simulate_paste_with_cg",
        "CGEvent",
        "CGEventFlagCommand",
        "new_keyboard_event",
        "KEY_C",
        "Cmd+C",
    ] {
        assert!(
            !ax_only_body.contains(forbidden),
            "AX-only selected-text helper must not synthesize keyboard input: {forbidden}"
        );
    }

    for required in [
        "AXUIElementCreateSystemWide",
        "AX_FOCUSED_UI_ELEMENT",
        "ax_selected_text_for_element",
    ] {
        assert!(
            ax_only_body.contains(required),
            "AX-only selected-text helper should stay on the AX read path: {required}"
        );
    }
}
