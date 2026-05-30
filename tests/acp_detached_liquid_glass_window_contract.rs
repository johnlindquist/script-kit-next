//! Source contracts for the detached ACP Liquid Glass window proof slice.

const CHAT_WINDOW: &str = include_str!("../src/ai/acp/chat_window.rs");
const ACP_VIEW: &str = include_str!("../src/ai/acp/view.rs");
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
fn detached_acp_placeholder_fixture_is_first_class_stdin_command() {
    assert!(
        STDIN_COMMANDS.contains("OpenAcpDetachedFixture")
            && STDIN_COMMANDS.contains("\"openAcpDetachedFixture\""),
        "stdin protocol must expose a deterministic detached ACP fixture command"
    );
    assert!(
        RUNTIME_STDIN.contains("openAcpDetachedFixture")
            && RUNTIME_STDIN.contains("open_chat_window(ctx)")
            && RUNTIME_STDIN.contains("set_chat_window_fixture_bounds"),
        "runtime stdin must open the detached ACP fixture without provider credentials"
    );
}

#[test]
fn detached_acp_placeholder_registers_metadata_and_bounds() {
    assert!(
        CHAT_WINDOW.contains("upsert_acp_detached_automation_window")
            && CHAT_WINDOW.contains("AutomationWindowKind::AcpDetached")
            && CHAT_WINDOW.contains("semantic_surface: Some(\"acpChat\".to_string())"),
        "detached ACP windows must be discoverable by automation target kind"
    );
    assert!(
        CHAT_WINDOW.contains("automation_bounds_from_window_bounds")
            && CHAT_WINDOW.contains("set_automation_bounds"),
        "detached ACP windows must publish target bounds for window-priority layout proof"
    );
    assert!(
        CHAT_WINDOW.contains("remove_automation_window(id)"),
        "detached ACP cleanup must remove both runtime and metadata registry entries"
    );
}

#[test]
fn get_layout_info_routes_acp_detached_targets_to_shell_metrics() {
    assert!(
        PROMPT_HANDLER.contains("AutomationWindowKind::AcpDetached")
            && PROMPT_HANDLER.contains("automation_layout_info(&resolved)")
            && PROMPT_HANDLER.contains("placeholder_automation_layout_info(&resolved)"),
        "getLayoutInfo(target acpDetached) must return detached window shell metrics, not an empty rejection"
    );
}

#[test]
fn detached_acp_layout_info_exposes_liquid_glass_shell_components() {
    for component in [
        "AcpDetachedWindow",
        "AcpMessageViewport",
        "AcpComposerBar",
        "AcpFooterRail",
    ] {
        assert!(
            ACP_VIEW.contains(component),
            "detached ACP layout info must expose {component}"
        );
    }
    assert!(
        ACP_VIEW.contains("LIQUID_GLASS_WINDOW_RADIUS_PX")
            && ACP_VIEW.contains("LIQUID_GLASS_PANEL_RADIUS_PX")
            && ACP_VIEW.contains("LIQUID_GLASS_COMPACT_RADIUS_PX")
            && ACP_VIEW.contains("MATERIAL_NS_VISUAL_EFFECT"),
        "detached ACP layout info must carry Liquid Glass radius/material tokens"
    );

    let viewport = source_after(ACP_VIEW, "LayoutComponentInfo::new(\"AcpMessageViewport\"");
    let before_token = &viewport[..viewport
        .find(".with_visual_token(\"content.acpMessages\")")
        .expect("AcpMessageViewport should declare its visual token")];
    assert!(
        before_token.contains("Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX)"),
        "AcpMessageViewport must expose a positive Liquid Glass radius in layout proof"
    );
}
