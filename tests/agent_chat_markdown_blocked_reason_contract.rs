const ACTION_HANDLER: &str = include_str!("../src/app_actions/handle_action/mod.rs");

#[test]
fn agent_chat_markdown_blocked_reason_is_named_state() {
    assert!(
        ACTION_HANDLER.contains("enum AgentChatConversationMarkdownBlockedReason")
            && ACTION_HANDLER.contains("NoMessages")
            && ACTION_HANDLER.contains("EmptyRenderableMessages"),
        "Agent Chat markdown export/save blocking should use a named reason state"
    );
    assert!(
        ACTION_HANDLER.contains("fn from_message_count(message_count: usize) -> Self")
            && ACTION_HANDLER.contains("0 => Self::NoMessages")
            && ACTION_HANDLER.contains("_ => Self::EmptyRenderableMessages")
            && ACTION_HANDLER.contains("fn trace_value(self) -> &'static str")
            && ACTION_HANDLER.contains("\"no_messages\"")
            && ACTION_HANDLER.contains("\"empty_renderable_messages\""),
        "Agent Chat markdown blocked reason should own message-count classification and trace values"
    );
}

#[test]
fn agent_chat_markdown_handlers_route_blocked_reason_through_action_state() {
    assert!(
        ACTION_HANDLER.contains(
            "fn blocked_reason(self, message_count: usize) -> AgentChatConversationMarkdownBlockedReason"
        ),
        "Agent Chat markdown action should expose the blocked-reason transition"
    );
    assert!(
        ACTION_HANDLER
            .matches("markdown_action.blocked_reason(message_count)")
            .count()
            >= 2
            && ACTION_HANDLER.contains("reason = %reason.trace_value()"),
        "Agent Chat markdown export and save-as-note should log named blocked reasons"
    );
    assert!(
        !ACTION_HANDLER.contains("let reason = if message_count == 0"),
        "Agent Chat markdown blocked telemetry must not regress to inline message-count branching"
    );
}
