//! Source-level guard for popup lists that are allowed to remain outside the
//! shared ActionsDialog and main-list surfaces.

const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");
const AGENT_CHAT_WINDOW: &str = include_str!("../src/ai/agent_chat/ui/chat_window.rs");
const AGENT_CHAT_TESTS: &str = include_str!("../src/ai/agent_chat/ui/tests.rs");

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

    assert!(
        AGENT_CHAT_TESTS.contains("agent_chat_show_history_action_opens_main_history_list"),
        "global Agent Chat history command routing must stay pinned to the main AgentChatHistoryView list"
    );
}
