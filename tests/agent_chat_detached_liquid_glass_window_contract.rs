//! Source contracts for the detached Agent Chat Liquid Glass window proof slice.

const CHAT_WINDOW: &str = include_str!("../src/ai/agent_chat/ui/chat_window.rs");
const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");

fn source_after<'a>(source: &'a str, needle: &str) -> &'a str {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} should exist"));
    &source[start..]
}

#[test]
fn detached_agent_chat_placeholder_fixture_is_first_class_stdin_command() {
    assert!(
        STDIN_COMMANDS.contains("OpenAgentChatDetachedFixture")
            && STDIN_COMMANDS.contains("\"openAgentChatDetachedFixture\""),
        "stdin protocol must expose a deterministic detached Agent Chat fixture command"
    );
    assert!(
        RUNTIME_STDIN.contains("openAgentChatDetachedFixture")
            && RUNTIME_STDIN.contains("open_chat_window(ctx)")
            && RUNTIME_STDIN.contains("set_chat_window_fixture_bounds"),
        "runtime stdin must open the detached Agent Chat fixture without provider credentials"
    );
}

#[test]
fn detached_agent_chat_placeholder_registers_metadata_and_bounds() {
    assert!(
        CHAT_WINDOW.contains("upsert_agent_chat_detached_automation_window")
            && CHAT_WINDOW.contains("AutomationWindowKind::AgentChatDetached")
            && CHAT_WINDOW.contains("semantic_surface: Some(\"agentChatChat\".to_string())"),
        "detached Agent Chat windows must be discoverable by automation target kind"
    );
    assert!(
        CHAT_WINDOW.contains("automation_bounds_from_window_bounds")
            && CHAT_WINDOW.contains("set_automation_bounds"),
        "detached Agent Chat windows must publish target bounds for window-priority layout proof"
    );
    assert!(
        CHAT_WINDOW.contains("remove_automation_window(id)"),
        "detached Agent Chat cleanup must remove both runtime and metadata registry entries"
    );
}

#[test]
fn get_layout_info_routes_agent_chat_detached_targets_to_shell_metrics() {
    assert!(
        PROMPT_HANDLER.contains("AutomationWindowKind::AgentChatDetached")
            && PROMPT_HANDLER.contains("automation_layout_info(&resolved)")
            && PROMPT_HANDLER.contains("placeholder_automation_layout_info(&resolved)"),
        "getLayoutInfo(target agentChatDetached) must return detached window shell metrics, not an empty rejection"
    );
}

#[test]
fn detached_agent_chat_layout_info_exposes_liquid_glass_shell_components() {
    for component in [
        "AgentChatDetachedWindow",
        "AgentChatMessageViewport",
        "AgentChatComposerBar",
        "AgentChatFooterRail",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(component),
            "detached Agent Chat layout info must expose {component}"
        );
    }
    assert!(
        AGENT_CHAT_VIEW.contains("LIQUID_GLASS_WINDOW_RADIUS_PX")
            && AGENT_CHAT_VIEW.contains("LIQUID_GLASS_PANEL_RADIUS_PX")
            && AGENT_CHAT_VIEW.contains("LIQUID_GLASS_COMPACT_RADIUS_PX")
            && AGENT_CHAT_VIEW.contains("MATERIAL_NS_VISUAL_EFFECT"),
        "detached Agent Chat layout info must carry Liquid Glass radius/material tokens"
    );

    let viewport = source_after(
        AGENT_CHAT_VIEW,
        "LayoutComponentInfo::new(\"AgentChatMessageViewport\"",
    );
    let before_token = &viewport[..viewport
        .find(".with_visual_token(\"content.agent_chatMessages\")")
        .expect("AgentChatMessageViewport should declare its visual token")];
    assert!(
        before_token.contains("Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX)"),
        "AgentChatMessageViewport must expose a positive Liquid Glass radius in layout proof"
    );
}
