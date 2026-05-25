use std::path::Path;

const INLINE_AGENT_MOD: &str = include_str!("../src/ai/inline_agent/mod.rs");
const INLINE_AGENT_WINDOW: &str = include_str!("../src/inline_agent/window.rs");
const AGENT_CHAT_ADAPTER: &str = include_str!("../src/ai/inline_agent/agent_chat_adapter.rs");

#[test]
fn inline_agent_no_longer_declares_or_ships_acp_adapter() {
    assert!(!INLINE_AGENT_MOD.contains("mod acp_adapter"));
    assert!(!Path::new("src/ai/inline_agent/acp_adapter.rs").exists());
}

#[test]
fn inline_agent_window_uses_agent_chat_pi_executor_without_acp_fallback() {
    assert!(INLINE_AGENT_MOD.contains("Legacy AI execution boundary"));
    assert!(INLINE_AGENT_WINDOW.contains("spawn_default_agent_chat_inline_agent_executor"));
    for forbidden in [
        "spawn_default_acp_inline_agent_executor",
        "AcpInlineAgentExecutor",
    ] {
        assert!(
            !INLINE_AGENT_WINDOW.contains(forbidden),
            "inline agent window must not fallback to ACP symbol {forbidden}"
        );
    }
}

#[test]
fn legacy_agent_chat_adapter_remains_pi_only_without_acp_fallback() {
    assert!(AGENT_CHAT_ADAPTER.contains("Legacy compatibility adapter"));
    assert!(AGENT_CHAT_ADAPTER.contains("AgentChatInlineAgentExecutor"));
    assert!(AGENT_CHAT_ADAPTER.contains("resolve_focused_text_pi_launch"));
    assert!(AGENT_CHAT_ADAPTER.contains("warm_session_manager"));
    for forbidden in [
        "load_acp_agent_catalog_entries",
        "resolve_acp_launch_with_requirements",
    ] {
        assert!(
            !AGENT_CHAT_ADAPTER.contains(forbidden),
            "Agent Chat inline adapter must not route through ACP symbol {forbidden}"
        );
    }
}
