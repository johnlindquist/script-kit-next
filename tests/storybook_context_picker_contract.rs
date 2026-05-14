#[test]
fn context_picker_storybook_uses_shared_inline_dropdown_renderer() {
    let playground = include_str!("../src/storybook/context_picker_popup_playground/mod.rs");
    let acp_picker = include_str!("../src/ai/acp/picker_popup.rs");

    for token in [
        "acp_context_picker_item_to_inline_picker_row",
        "render_soft_compact_picker_row",
        "InlinePickerRow",
        "inline_picker_normalize_selected_index",
        "SOFT_COMPACT_PICKER_ROW_HEIGHT",
    ] {
        assert!(
            playground.contains(token) || acp_picker.contains(token),
            "context-picker Storybook/ACP path must contain {token}"
        );
    }

    assert!(
        !playground.contains("context_picker_row::render"),
        "ACP-local context_picker_row must not own Storybook popup row mechanics"
    );
}
