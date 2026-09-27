//! Source-level guard for popup lists that are allowed to remain outside the
//! shared ActionsDialog and main-list surfaces.

const DICTATION_MIC_POPUP: &str = include_str!("../src/dictation/microphone_popup_window.rs");

#[test]
fn dictation_microphone_popup_keeps_legacy_compact_rows_and_attached_popup_shell() {
    for required in [
        "DictationMicrophonePopupWindow",
        "AutomationWindowKind::PromptPopup",
        "InlineDropdown::new(",
        "render_soft_compact_picker_row",
        "SOFT_COMPACT_PICKER_ROW_HEIGHT",
        "dictation_microphone_popup_bounds_above",
        "parent_bounds.origin.x.as_f32()",
        "parent_bounds.origin.y.as_f32() - height",
        "this.handle_row_click(idx, event, window, cx);",
        "self.accept_row(current, window, cx)",
    ] {
        assert!(
            DICTATION_MIC_POPUP.contains(required),
            "missing dictation microphone popup shared-row/shell contract: {required}"
        );
    }

    for forbidden in [
        "crate::list_item::ListItem::new",
        "crate::list_item::ListItemColors::from_theme",
        "crate::list_item::effective_list_item_height_for_theme",
        "crate::designs::current_main_menu_theme",
        "render_dense_monoline_picker_row",
        "render_dense_monoline_picker_row_with_leading_visual",
        ".border_l(gpui::px(2.0))",
        "selected_row_bg",
        "hover_row_bg",
    ] {
        assert!(
            !DICTATION_MIC_POPUP.contains(forbidden),
            "dictation microphone popup must not reintroduce bespoke row chrome: {forbidden}"
        );
    }
}
