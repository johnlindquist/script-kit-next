//! Source-level guard for popup lists that are allowed to remain outside the
//! shared ActionsDialog and main-list surfaces.

const ACP_VIEW: &str = include_str!("../src/ai/acp/view.rs");
const ACP_CHAT_WINDOW: &str = include_str!("../src/ai/acp/chat_window.rs");
const NOTES_ACP_HOST: &str = include_str!("../src/notes/window/acp_host.rs");
const ACP_TESTS: &str = include_str!("../src/ai/acp/tests.rs");
const DICTATION_MIC_POPUP: &str = include_str!("../src/dictation/microphone_popup_window.rs");

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn acp_history_prompt_popup_is_only_used_for_composer_portals() {
    assert!(
        ACP_VIEW.contains("pub(crate) fn open_history_portal_with_entries(")
            && ACP_VIEW.contains("pub(crate) fn open_history_popup_from_host("),
        "ACP may keep a PromptPopup for the inline composer @history portal flow"
    );

    let detached_portal_body = function_body(
        ACP_CHAT_WINDOW,
        "fn open_history_portal_in_detached_chat_window(",
        "fn close_history_portal_in_detached_chat_window(",
    );
    assert!(
        detached_portal_body.contains("view.open_history_portal_with_entries(query, hits, cx);")
            && detached_portal_body.contains("view.open_history_popup_from_host("),
        "detached ACP should use the history popup only after a composer portal request has staged rows"
    );

    let notes_portal_body = function_body(
        NOTES_ACP_HOST,
        "fn handle_acp_portal_static(",
        "/// Wire ACP host callbacks",
    );
    assert!(
        notes_portal_body.contains("view.open_history_portal_with_entries(query, hits, cx)")
            && !NOTES_ACP_HOST.contains("open_embedded_acp_history_popup"),
        "Notes-hosted ACP history shortcuts must use actions; only composer portal rows may use the popup"
    );

    assert!(
        ACP_TESTS.contains("acp_show_history_action_opens_main_history_list"),
        "global ACP history command routing must stay pinned to the main AcpHistoryView list"
    );
}

#[test]
fn dictation_microphone_popup_is_the_explicit_non_action_popup_list_exception() {
    assert!(
        DICTATION_MIC_POPUP.contains("DictationMicrophonePopupWindow")
            && DICTATION_MIC_POPUP.contains("AutomationWindowKind::PromptPopup")
            && DICTATION_MIC_POPUP.contains("InlineDropdown::new("),
        "dictation microphone selection is a unique window/search UX and may remain a PromptPopup list"
    );
}
