//! Source-level guard for popup lists that are allowed to remain outside the
//! shared ActionsDialog and main-list surfaces.

const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");
const AGENT_CHAT_WINDOW: &str = include_str!("../src/ai/agent_chat/ui/chat_window.rs");
const NOTES_AGENT_CHAT_HOST: &str = include_str!("../src/notes/window/agent_chat_host.rs");
const AGENT_CHAT_TESTS: &str = include_str!("../src/ai/agent_chat/ui/tests.rs");
const AI_PRESETS_OVERLAYS: &str = include_str!("../src/ai/window/render_overlays_dropdowns.rs");
const AI_PRESETS_DROPDOWNS: &str = include_str!("../src/ai/window/dropdowns.rs");
const DICTATION_MIC_POPUP: &str = include_str!("../src/dictation/microphone_popup_window.rs");

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn agent_chat_history_prompt_popup_is_only_used_for_composer_portals() {
    assert!(
        AGENT_CHAT_VIEW.contains("pub(crate) fn open_history_portal_with_entries(")
            && AGENT_CHAT_VIEW.contains("pub(crate) fn open_history_popup_from_host("),
        "Agent Chat may keep a PromptPopup for the inline composer @history portal flow"
    );

    let detached_portal_body = function_body(
        AGENT_CHAT_WINDOW,
        "fn open_history_portal_in_detached_chat_window(",
        "fn close_history_portal_in_detached_chat_window(",
    );
    assert!(
        detached_portal_body.contains("view.open_history_portal_with_entries(query, hits, cx);")
            && detached_portal_body.contains("view.open_history_popup_from_host("),
        "detached Agent Chat should use the history popup only after a composer portal request has staged rows"
    );

    let notes_portal_body = function_body(
        NOTES_AGENT_CHAT_HOST,
        "fn handle_agent_chat_portal_static(",
        "/// Wire Agent Chat host callbacks",
    );
    assert!(
        notes_portal_body.contains("view.open_history_portal_with_entries(query, hits, cx)")
            && !NOTES_AGENT_CHAT_HOST.contains("open_embedded_agent_chat_history_popup"),
        "Notes-hosted Agent Chat history shortcuts must use actions; only composer portal rows may use the popup"
    );

    assert!(
        AGENT_CHAT_TESTS.contains("agent_chat_show_history_action_opens_main_history_list"),
        "global Agent Chat history command routing must stay pinned to the main AgentChatHistoryView list"
    );
}

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

#[test]
fn ai_presets_dropdown_rows_use_shared_list_item_chrome() {
    let row_body = function_body(
        AI_PRESETS_OVERLAYS,
        "pub(super) fn render_presets_dropdown",
        "let dropdown = InlineDropdown::new",
    );

    for required in [
        "crate::list_item::ListItem::new",
        "crate::list_item::ListItemColors::from_theme",
        ".selected(is_selected)",
        ".main_menu_theme(",
        ".semantic_id(format!(\"preset-{idx}\"",
        "crate::list_item::effective_list_item_height_for_theme",
    ] {
        assert!(
            row_body.contains(required),
            "missing shared ListItem row contract: {required}"
        );
    }

    for forbidden in [
        "render_dense_monoline_picker_row_with_leading_visual",
        "render_dense_monoline_picker_row",
        "render_soft_compact_picker_row",
        ".border_l(gpui::px(2.0))",
        "selected_row_bg",
        "hover_row_bg",
    ] {
        assert!(
            !row_body.contains(forbidden),
            "must not reintroduce bespoke row chrome: {forbidden}"
        );
    }
}

#[test]
fn ai_presets_dropdown_preserves_shell_navigation_and_activation_contract() {
    for required in [
        "InlineDropdown::new(SharedString::from(\"presets-dropdown\")",
        ".empty_state_opt(",
        ".synopsis(synopsis)",
        "\"presets-dropdown-overlay\"",
        "\"presets-dropdown-container\"",
        "this.presets_selected_index = idx;",
        "this.confirm_presets_selection(window, cx);",
    ] {
        assert!(
            AI_PRESETS_OVERLAYS.contains(required),
            "missing shell/click contract: {required}"
        );
    }

    for required in [
        "pub(super) fn presets_select_prev",
        "pub(super) fn presets_select_next",
        "inline_dropdown_select_prev",
        "inline_dropdown_select_next",
        "inline_dropdown_visible_range",
        "pub(super) fn confirm_presets_selection",
        "create_chat_with_preset(window, cx)",
    ] {
        assert!(
            AI_PRESETS_DROPDOWNS.contains(required),
            "missing navigation/activation contract: {required}"
        );
    }
}
