const APP_RUN_SETUP: &str = include_str!("../../src/main_entry/app_run_setup.rs");
const RUNTIME_TRAY_HOTKEYS: &str = include_str!("../../src/main_entry/runtime_tray_hotkeys.rs");
const FOCUSED_TEXT_ENTRY: &str =
    include_str!("../../src/app_impl/tab_ai_mode/focused_text_entry.rs");
const ACP_VIEW: &str = include_str!("../../src/ai/acp/view.rs");
const ACP_UI_VARIANT: &str = include_str!("../../src/ai/acp/ui_variant.rs");
const ACP_LAUNCH: &str = include_str!("../../src/app_impl/tab_ai_mode/acp_launch.rs");
const FOOTER_POPUP: &str = include_str!("../../src/footer_popup.rs");
const STDIN_COMMANDS: &str = include_str!("../../src/stdin_commands/mod.rs");
const RUNTIME_STDIN: &str = include_str!("../../src/main_entry/runtime_stdin.rs");
const APP_LAYOUT_COLLECT_ELEMENTS: &str = include_str!("../../src/app_layout/collect_elements.rs");
const ACP_STATE_TYPES: &str = include_str!("../../src/protocol/types/acp_state.rs");

#[test]
fn inline_ai_hotkeys_capture_before_opening_focused_text_agent_chat() {
    for source in [APP_RUN_SETUP, RUNTIME_TRAY_HOTKEYS] {
        let channel = source
            .find("inline_ai_hotkey_channel().1.recv().await")
            .expect("inline AI listener must exist");
        let capture = source[channel..]
            .find("capture_focused_text_field")
            .expect("inline AI listener must capture focused text")
            + channel;
        let open = source[channel..]
            .find("open_focused_text_agent_chat_from_snapshot")
            .expect("inline AI listener must open focused-text Agent Chat")
            + channel;

        assert!(
            capture < open,
            "focused text must be captured before opening Agent Chat"
        );
        assert!(!source[channel..].contains("launch_inline_agent_from_focused_text"));
    }
}

#[test]
fn focused_text_entry_forces_embedded_mini_agent_chat_surface() {
    for required in [
        "open_focused_text_agent_chat_from_snapshot",
        "AcpChatUiVariant::FocusedTextMini",
        "begin_tab_ai_harness_entry_from_source_view",
        "force_acp_surface",
        "stage_focused_text_from_host",
        "focused_text_agent_chat_open",
    ] {
        assert!(
            FOCUSED_TEXT_ENTRY.contains(required),
            "missing focused-text Agent Chat entry contract: {required}"
        );
    }
}

#[test]
fn acp_view_can_stage_focused_text_as_host_owned_context() {
    for required in [
        "stage_focused_text_from_host",
        "focused-text://",
        "Focused Text ·",
        "replace_pending_context_parts(vec![part], source, cx)",
        "focused_text_context_staged",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text ACP staging contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_uses_pi_text_profile_not_acp_backend_fallback() {
    for required in [
        "request.ui_variant == crate::ai::acp::ui_variant::AcpChatUiVariant::FocusedTextMini",
        "resolve_focused_text_pi_launch",
        "focused_text_mini",
        "\"Pi Text profile is unavailable\"",
    ] {
        assert!(
            ACP_LAUNCH.contains(required),
            "missing focused-text Pi launch contract: {required}"
        );
    }
}

#[test]
fn focused_text_footer_actions_are_explicit_and_dispatch_apply_back() {
    for required in [
        "FooterAction::Replace",
        "FooterAction::Append",
        "FooterAction::Copy",
        "FooterAction::Expand",
        "FooterAction::Retry",
    ] {
        assert!(
            FOOTER_POPUP.contains(required),
            "missing focused-text footer action: {required}"
        );
    }

    for required in [
        "focused_text_footer_buttons",
        "apply_focused_text_output",
        "FocusedTextMutation::Replace",
        "FocusedTextMutation::Append",
        "FocusedTextMutation::Copy",
        "SystemFocusedTextPlatformBridge",
        "set_ui_variant(AcpChatUiVariant::Standard",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text footer dispatch contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_variant_is_protocol_visible() {
    for required in [
        "FocusedTextMini",
        "\"focused-text-mini\"",
        "AcpTranscriptPresentation::FocusedTextPreview",
        "AcpComposerPlacement::FocusedTextSingleLine",
        "AcpChromeDensity::Mini",
    ] {
        assert!(
            ACP_UI_VARIANT.contains(required),
            "missing focused-text mini variant contract: {required}"
        );
    }
}

#[test]
fn focused_text_agent_chat_stdin_verbs_alias_old_inline_fixture_verbs() {
    for required in [
        "OpenFocusedTextAgentChatWithMockData",
        "OpenFocusedTextAgentChatWithPiData",
        "\"openFocusedTextAgentChatWithMockData\"",
        "\"openFocusedTextAgentChatWithPiData\"",
        "OpenInlineAgentWithMockData",
        "OpenInlineAgentWithPiData",
    ] {
        assert!(
            STDIN_COMMANDS.contains(required),
            "missing focused-text stdin command contract: {required}"
        );
    }

    for required in [
        "ExternalCommand::OpenFocusedTextAgentChatWithMockData",
        "ExternalCommand::OpenInlineAgentWithMockData",
        "ExternalCommand::OpenFocusedTextAgentChatWithPiData",
        "ExternalCommand::OpenInlineAgentWithPiData",
        "open_focused_text_agent_chat_fixture",
        "focused_text_mock_fixture",
        "focused_text_pi_fixture",
    ] {
        assert!(
            RUNTIME_STDIN.contains(required),
            "missing focused-text stdin dispatch contract: {required}"
        );
    }
    assert!(!RUNTIME_STDIN.contains("open_inline_agent_mock_fixture"));
    assert!(!RUNTIME_STDIN.contains("open_inline_agent_pi_fixture"));
}

#[test]
fn focused_text_mini_has_redacted_acp_state_and_semantic_elements() {
    for required in [
        "pub focused_text: Option<AcpFocusedTextState>",
        "pub struct AcpFocusedTextState",
        "char_count",
        "has_output",
        "last_apply_action",
    ] {
        assert!(
            ACP_STATE_TYPES.contains(required),
            "missing focused-text ACP state contract: {required}"
        );
    }

    for required in [
        "focused_text_state_snapshot",
        "collect_focused_text_mini_elements",
        "\"focused-text-mini-root\"",
        "\"focused-text-input\"",
        "\"focused-text-preview\"",
        "\"focused-text-action-replace\"",
        "\"focused-text-action-append\"",
        "\"focused-text-action-copy\"",
        "\"focused-text-action-expand\"",
        "\"focused-text-action-stop\"",
        "\"focused-text-action-retry\"",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text ACP automation contract: {required}"
        );
    }

    assert!(APP_LAYOUT_COLLECT_ELEMENTS.contains("AppView::AcpChatView { entity }"));
    assert!(APP_LAYOUT_COLLECT_ELEMENTS.contains("collect_focused_text_mini_elements"));
}
