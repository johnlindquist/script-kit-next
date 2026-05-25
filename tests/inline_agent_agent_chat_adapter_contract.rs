const INLINE_AGENT_MOD: &str = include_str!("../src/ai/inline_agent/mod.rs");
const AGENT_CHAT_ADAPTER: &str = include_str!("../src/ai/inline_agent/agent_chat_adapter.rs");
const INLINE_AGENT_WINDOW: &str = include_str!("../src/inline_agent/window.rs");

#[test]
fn inline_agent_declares_agent_chat_adapter_and_uses_it_as_default() {
    assert!(INLINE_AGENT_MOD.contains("pub(crate) mod agent_chat_adapter;"));
    assert!(AGENT_CHAT_ADAPTER.contains("AgentChatInlineAgentExecutor"));
    assert!(
        AGENT_CHAT_ADAPTER.contains("impl InlineAgentExecutor for AgentChatInlineAgentExecutor")
    );
    assert!(AGENT_CHAT_ADAPTER.contains("spawn_default_agent_chat_inline_agent_executor"));
    assert!(AGENT_CHAT_ADAPTER.contains("prepare_default_agent_chat_inline_agent_warm_session"));
    assert!(INLINE_AGENT_WINDOW.contains("spawn_default_agent_chat_inline_agent_executor"));
    assert!(INLINE_AGENT_WINDOW.contains("prewarm_inline_agent_executor_mode"));
    assert!(INLINE_AGENT_WINDOW.contains("prepare_default_agent_chat_inline_agent_warm_session"));
    assert!(!INLINE_AGENT_WINDOW.contains("spawn_default_acp_inline_agent_executor"));
}

#[test]
fn inline_agent_adapter_depends_on_agent_chat_connection_not_pi_runtime() {
    assert!(AGENT_CHAT_ADAPTER.contains("Arc<dyn AgentChatConnection>"));
    assert!(AGENT_CHAT_ADAPTER.contains("AgentChatTurnRequest"));
    assert!(AGENT_CHAT_ADAPTER.contains("resolve_focused_text_pi_launch"));
    assert!(AGENT_CHAT_ADAPTER.contains("warm_session_manager"));
    assert!(AGENT_CHAT_ADAPTER.contains("wait_for_prepared_warm_session"));
    assert!(AGENT_CHAT_ADAPTER.contains("AgentChatWarmSessionState::Preparing"));

    for forbidden in ["PiRpcRuntime", "PiLaunchSpec::", "Command::new"] {
        assert!(
            !AGENT_CHAT_ADAPTER.contains(forbidden),
            "inline Agent Chat adapter must not directly own {forbidden}"
        );
    }
}

#[test]
fn inline_agent_adapter_starts_turn_with_text_prompt_and_warm_thread() {
    for required in [
        "ui_thread_id: self.ui_thread_id.clone()",
        "cwd: self.cwd.clone()",
        "ContentBlock::Text(TextContent::new(request.prompt))",
        "model_id: self.model_id.clone()",
        "self.connection.start_turn(AgentChatTurnRequest",
    ] {
        assert!(
            AGENT_CHAT_ADAPTER.contains(required),
            "missing Agent Chat start_turn contract: {required}"
        );
    }
}

#[test]
fn inline_agent_adapter_maps_agent_chat_events_to_inline_events() {
    for event_mapping in [
        "AgentChatEvent::AgentMessageDelta(text)",
        "InlineAgentProviderEvent::AgentMessageDelta { text }",
        "AgentChatEvent::AgentThoughtDelta(text)",
        "InlineAgentProviderEvent::AgentThoughtDelta { text }",
        "AgentChatEvent::UsageUpdated",
        "InlineAgentProviderEvent::UsageUpdated",
        "AgentChatEvent::TurnFinished",
        "InlineAgentProviderEvent::TurnFinished",
        "AgentChatEvent::Failed { error }",
        "InlineAgentProviderEvent::Failed { message: error }",
        "AgentChatEvent::SetupRequired { reason, .. }",
        "InlineAgentProviderEvent::Failed { message: reason }",
    ] {
        assert!(
            AGENT_CHAT_ADAPTER.contains(event_mapping),
            "missing Agent Chat event mapping: {event_mapping}"
        );
    }
}

#[test]
fn inline_agent_adapter_cancels_and_releases_warm_lease() {
    assert!(AGENT_CHAT_ADAPTER.contains("self.connection.cancel_turn(self.ui_thread_id.clone())"));
    assert!(AGENT_CHAT_ADAPTER.contains("AgentChatWarmSessionLease"));
    assert!(AGENT_CHAT_ADAPTER.contains("release_warm_lease"));
    assert!(AGENT_CHAT_ADAPTER.contains("warm_session_manager().dismiss_reset(lease)"));
    assert!(AGENT_CHAT_ADAPTER.contains("inline_agent_pi_warm_dismiss_reset"));
    assert!(AGENT_CHAT_ADAPTER.contains("impl Drop for AgentChatInlineAgentExecutor"));
}

#[test]
fn inline_agent_adapter_logs_privacy_safe_warm_response_timing() {
    for required in [
        "std::time::{Duration, Instant}",
        "inline_agent_pi_start_turn_dispatch",
        "inline_agent_pi_start_turn_dispatch_failed",
        "inline_agent_pi_first_agent_delta",
        "inline_agent_pi_turn_terminal",
        "prompt_chars",
        "delta_chars = text.chars().count()",
        "elapsed_ms = submit_started.elapsed().as_millis() as u64",
        "first_agent_delta_logged",
        "terminal_kind",
        "warm_generation",
        "ui_thread_id = %self.ui_thread_id",
        "PREPARE_IN_PROGRESS_WAIT_TIMEOUT",
        "FIRST_AGENT_DELTA_TIMEOUT",
        "TOTAL_TURN_TIMEOUT",
        "EVENT_POLL_INTERVAL",
        "inline_agent_pi_turn_timeout",
        "first_agent_delta_timeout",
        "total_turn_timeout",
        "connection.cancel_turn(ui_thread_id.clone())",
    ] {
        assert!(
            AGENT_CHAT_ADAPTER.contains(required),
            "missing privacy-safe warm timing contract: {required}"
        );
    }
}
