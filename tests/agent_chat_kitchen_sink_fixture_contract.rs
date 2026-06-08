const AGENT_CHAT_MOD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/mod.rs");
const FIXTURE_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/kitchen_sink_fixture.rs");

#[test]
fn kitchen_sink_fixture_module_is_registered() {
    assert!(
        AGENT_CHAT_MOD_SOURCE.contains("pub(crate) mod kitchen_sink_fixture;"),
        "Agent Chat must register the kitchen sink fixture module for runtime fixture launchers"
    );
}

#[test]
fn kitchen_sink_fixture_has_stable_identity_and_manifest() {
    assert!(FIXTURE_SOURCE.contains(
        "pub(crate) const AGENT_CHAT_KITCHEN_SINK_FIXTURE_ID: &str = \"agent-chat-kitchen-sink\";"
    ));
    assert!(FIXTURE_SOURCE.contains("pub(crate) fn agent_chat_kitchen_sink_fixture()"));
    assert!(FIXTURE_SOURCE.contains("pub(crate) fn kitchen_sink_feature_manifest()"));

    for feature in [
        "role:user",
        "role:assistant",
        "role:thought",
        "role:tool",
        "role:system",
        "role:error",
        "markdown:heading",
        "markdown:table",
        "markdown:fenced-code",
        "markdown:inline-code",
        "markdown:blockquote",
        "markdown:link",
        "markdown:task-list",
        "conversation:long-transcript",
        "conversation:result-artifacts",
        "conversation:next-actions",
        "conversation:tool-call-id",
        "conversation:collapsible-thought",
        "conversation:collapsible-tool",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(feature),
            "fixture manifest must include {feature}"
        );
    }
}

#[test]
fn kitchen_sink_fixture_covers_every_agent_chat_role() {
    for role in [
        "AgentChatKitchenSinkFixtureRole::User",
        "AgentChatKitchenSinkFixtureRole::Assistant",
        "AgentChatKitchenSinkFixtureRole::Thought",
        "AgentChatKitchenSinkFixtureRole::Tool",
        "AgentChatKitchenSinkFixtureRole::System",
        "AgentChatKitchenSinkFixtureRole::Error",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(role),
            "fixture must include role {role}"
        );
    }
}

#[test]
fn kitchen_sink_fixture_has_enough_messages_for_scroll_and_virtual_list_proof() {
    let message_count = FIXTURE_SOURCE
        .matches("AgentChatKitchenSinkFixtureMessage {")
        .count();
    assert!(
        message_count >= 18,
        "fixture should have enough messages to stress transcript scrolling; got {message_count}"
    );
}

#[test]
fn kitchen_sink_fixture_contains_markdown_and_result_card_sentinels() {
    for sentinel in [
        "# Agent Chat Kitchen Sink",
        "| Feature | Sentinel |",
        "```rust",
        "```json",
        "> A blockquote",
        "- [x] Cover headings",
        "`agentChat.transcript.rowGapY`",
        "[Script Kit](https://scriptkit.com)",
        "NEXT_ACTIONS:",
        "[Kitchen Sink Report](https://example.com/kitchen-sink-report)",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(sentinel),
            "fixture must contain markdown/result-card sentinel: {sentinel}"
        );
    }
}

#[test]
fn kitchen_sink_fixture_has_tool_call_ids_for_tool_messages() {
    assert!(FIXTURE_SOURCE.contains("tool_call_id: Some(\"tool-read-transcript-owner\")"));
    assert!(FIXTURE_SOURCE.contains("tool_call_id: Some(\"tool-search-docs\")"));
}
