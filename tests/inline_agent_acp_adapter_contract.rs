const ACP_ADAPTER: &str = include_str!("../src/ai/inline_agent/acp_adapter.rs");

#[test]
fn acp_adapter_uses_event_driven_start_turn_not_legacy_stream_prompt() {
    assert!(ACP_ADAPTER.contains("AcpInlineAgentExecutor"));
    assert!(ACP_ADAPTER.contains("impl InlineAgentExecutor for AcpInlineAgentExecutor"));
    assert!(ACP_ADAPTER.contains("spawn_default_acp_inline_agent_executor"));
    assert!(ACP_ADAPTER.contains("resolve_acp_launch_with_requirements"));
    assert!(ACP_ADAPTER.contains("AcpConnection::spawn_with_approval"));
    assert!(ACP_ADAPTER.contains(".start_turn(crate::ai::acp::AcpPromptTurnRequest"));
    assert!(ACP_ADAPTER.contains("ContentBlock::Text(TextContent::new(request.prompt))"));
    assert!(!ACP_ADAPTER.contains("stream_prompt"));
}

#[test]
fn acp_adapter_maps_streaming_events_to_inline_agent_events() {
    for event_mapping in [
        "AcpEvent::AgentMessageDelta(text)",
        "InlineAgentProviderEvent::AgentMessageDelta { text }",
        "AcpEvent::AgentThoughtDelta(text)",
        "InlineAgentProviderEvent::AgentThoughtDelta { text }",
        "AcpEvent::UsageUpdated",
        "InlineAgentProviderEvent::UsageUpdated",
        "AcpEvent::TurnFinished",
        "InlineAgentProviderEvent::TurnFinished",
        "AcpEvent::Failed { error }",
        "InlineAgentProviderEvent::Failed { message: error }",
    ] {
        assert!(
            ACP_ADAPTER.contains(event_mapping),
            "ACP adapter missing mapping: {event_mapping}"
        );
    }
}

#[test]
fn acp_adapter_cancels_by_inline_session_thread_id() {
    assert!(ACP_ADAPTER.contains("fn cancel_turn("));
    assert!(ACP_ADAPTER.contains("self.connection.cancel_turn(session_id.0)"));
}
