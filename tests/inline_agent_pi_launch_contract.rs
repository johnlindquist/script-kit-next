const AGENT_CHAT_LAUNCH: &str = include_str!("../src/ai/agent_chat/launch.rs");
const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const INLINE_AGENT_WINDOW: &str = include_str!("../src/inline_agent/window.rs");

#[test]
fn inline_agent_pi_launch_is_owned_by_agent_chat_launch_module() {
    assert!(AGENT_CHAT_LAUNCH.contains("resolve_focused_text_pi_launch"));
    assert!(AGENT_CHAT_LAUNCH.contains("BUILTIN_TEXT_PROFILE_ID"));
    assert!(AGENT_CHAT_LAUNCH
        .contains("selected_profile_id: Some(BUILTIN_TEXT_PROFILE_ID.to_string())"));
    assert!(AGENT_CHAT_LAUNCH.contains("selected_backend: Some(AgentChatBackend::Pi)"));
    assert!(AGENT_CHAT_LAUNCH
        .contains("PiAgentChatLaunch::from_profile(resolve_effective_profile(&text_ai, ctx))"));
    assert!(!INLINE_AGENT_WINDOW.contains("PiAgentChatLaunch"));
}

#[test]
fn focused_text_pi_launch_uses_text_profile_policy_not_selected_agent_chat_backend() {
    for required in [
        "BUILTIN_TEXT_PROFILE_ID",
        "selected_backend: Some(AgentChatBackend::Pi)",
        "selected_profile_id: Some(BUILTIN_TEXT_PROFILE_ID.to_string())",
    ] {
        assert!(
            AGENT_CHAT_LAUNCH.contains(required),
            "missing focused-text Pi launch policy: {required}"
        );
    }
}

#[test]
fn inline_agent_pi_launch_does_not_route_through_tab_ai_surface() {
    assert!(!TAB_AI_MODE.contains("resolve_inline_agent_pi_launch"));
    assert!(!TAB_AI_MODE.contains("INLINE_AGENT_PI_APPEND_SYSTEM_PROMPT"));
}
