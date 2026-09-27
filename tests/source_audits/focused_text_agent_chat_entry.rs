const AGENT_CHAT_UI_VARIANT: &str = include_str!("../../src/ai/agent_chat/ui/ui_variant.rs");
const AGENT_CHAT_PI_LAUNCH: &str = include_str!("../../src/ai/agent_chat/launch.rs");

#[test]
fn focused_text_mini_uses_pi_text_profile_not_agent_chat_backend_fallback() {
    for required in [
        "resolve_focused_text_launch",
        "selected_profile_id: Some(BUILTIN_TEXT_PROFILE_ID.to_string())",
        "AgentChatBackend::Goose",
        "AgentChatLaunch::from_profile(resolve_effective_profile(&text_ai, ctx))",
    ] {
        assert!(
            AGENT_CHAT_PI_LAUNCH.contains(required),
            "missing focused-text Pi launch resolver contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_variant_is_protocol_visible() {
    for required in [
        "FocusedTextMini",
        "\"focused-text-mini\"",
        "AgentChatTranscriptPresentation::FocusedTextPreview",
        "AgentChatComposerPlacement::FocusedTextSingleLine",
        "AgentChatChromeDensity::Mini",
    ] {
        assert!(
            AGENT_CHAT_UI_VARIANT.contains(required),
            "missing focused-text mini variant contract: {required}"
        );
    }
}
