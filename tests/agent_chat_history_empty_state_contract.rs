const AGENT_CHAT_HISTORY: &str = include_str!("../src/render_builtins/agent_chat_history.rs");

#[test]
fn agent_chat_history_empty_state_copy_is_modeled() {
    assert!(
        AGENT_CHAT_HISTORY.contains("enum AgentChatHistoryEmptyState")
            && AGENT_CHAT_HISTORY.contains("NoConversationHistory")
            && AGENT_CHAT_HISTORY.contains("NoFilteredMatches"),
        "Agent Chat History empty-state copy should use named states"
    );
    assert!(
        AGENT_CHAT_HISTORY.contains("fn from_filter(filter: &str) -> Self")
            && AGENT_CHAT_HISTORY.contains("fn message(self) -> &'static str"),
        "Agent Chat History empty states should own filter classification and visible copy"
    );
    assert!(
        AGENT_CHAT_HISTORY.contains("AgentChatHistoryEmptyState::from_filter(&filter)")
            && AGENT_CHAT_HISTORY.contains(".message()"),
        "Agent Chat History renderer should derive empty-state copy from the model"
    );
    assert!(
        !AGENT_CHAT_HISTORY.contains("child(if filter.is_empty()"),
        "Agent Chat History empty-state copy must not regress to inline filter-empty branching"
    );
}
