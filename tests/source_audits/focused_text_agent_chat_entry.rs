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
const RUNTIME_STDIN_MATCH_TAIL: &str =
    include_str!("../../src/main_entry/runtime_stdin_match_tail.rs");
const RUNTIME_STDIN_MATCH_SIMULATE_KEY: &str =
    include_str!("../../src/main_entry/runtime_stdin_match_simulate_key.rs");
const STARTUP: &str = include_str!("../../src/app_impl/startup.rs");
const STARTUP_NEW_ACTIONS: &str = include_str!("../../src/app_impl/startup_new_actions.rs");
const SIMULATE_KEY_DISPATCH: &str = include_str!("../../src/app_impl/simulate_key_dispatch.rs");
const APP_LAYOUT_COLLECT_ELEMENTS: &str = include_str!("../../src/app_layout/collect_elements.rs");
const ACP_STATE_TYPES: &str = include_str!("../../src/protocol/types/acp_state.rs");
const PROTOCOL_SYSTEM_CONTROL: &str =
    include_str!("../../src/protocol/message/variants/system_control.rs");
const PROTOCOL_GENERAL_CONSTRUCTORS: &str =
    include_str!("../../src/protocol/message/constructors/general.rs");
const PROTOCOL_PROMPT_CONSTRUCTORS: &str =
    include_str!("../../src/protocol/message/constructors/prompts.rs");
const INLINE_AGENT_MOD: &str = include_str!("../../src/inline_agent/mod.rs");
const PROMPT_HANDLER: &str = include_str!("../../src/prompt_handler/mod.rs");

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
        let dismiss = source[channel..]
            .find("dismiss_focused_text_agent_chat_before_recapture")
            .expect("inline AI listener must dismiss prior focused-text Agent Chat before capture")
            + channel;
        let open = source[channel..]
            .find("open_focused_text_agent_chat_from_snapshot")
            .expect("inline AI listener must open focused-text Agent Chat")
            + channel;

        assert!(
            dismiss < capture,
            "prior focused-text Agent Chat must be dismissed before recapturing external text"
        );
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
        "dismiss_focused_text_agent_chat_before_recapture",
        "has_focused_text_context",
        "focused_text_recapture_dismiss_previous_session",
        "CloseMainWindowStateFirst",
        "AcpChatUiVariant::FocusedTextMini",
        "begin_tab_ai_harness_entry_from_source_view",
        "force_acp_surface",
        "stage_focused_text_from_host",
        "focused_text_agent_chat_open",
        "set_main_window_mode_state_only",
        "MainWindowMode::Mini",
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
        "focused_text_visible_footer_buttons",
        "focused_text_semantic_actions",
        "apply_focused_text_output",
        "FocusedTextMutation::Replace",
        "FocusedTextMutation::Append",
        "FocusedTextMutation::Copy",
        "SystemFocusedTextPlatformBridge",
        "set_ui_variant(AcpChatUiVariant::Standard",
        "set_ui_variant(AcpChatUiVariant::FocusedTextMini",
        "set_on_focused_text_expand_requested",
        "set_on_focused_text_collapse_requested",
        "MainWindowMode::Full",
        "MainWindowMode::Mini",
        "focused_text_expand_agent_chat",
        "focused_text_collapse_agent_chat",
    ] {
        assert!(
            ACP_VIEW.contains(required)
                || include_str!("../../src/app_impl/tab_ai_mode/mod.rs").contains(required),
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
fn focused_text_mini_initial_state_is_input_only_without_native_footer() {
    for required in [
        "enum FocusedTextMiniPhase",
        "FocusedTextMiniPhase::InputOnly",
        "focused_text_mini_phase_for_thread",
        "focused_text_mini_footer_visible_for_thread",
        "main_window_footer_visible",
        "main_window_footer_slot",
        "acp_footer_hidden",
        "visible: bool",
        "FOCUSED_TEXT_MINI_SIZE_INPUT_ONLY",
        "ViewType::FocusedTextMini, 0",
    ] {
        assert!(
            ACP_VIEW.contains(required)
                || FOCUSED_TEXT_ENTRY.contains(required)
                || include_str!("../../src/app_impl/ui_window.rs").contains(required)
                || PROMPT_HANDLER.contains(required),
            "missing focused-text input-only footer contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_has_three_sizing_phases() {
    let resize = include_str!("../../src/window_resize/mod.rs");
    for required in [
        "FOCUSED_TEXT_MINI_SIZE_INPUT_ONLY",
        "FOCUSED_TEXT_MINI_SIZE_STREAMING",
        "FOCUSED_TEXT_MINI_SIZE_RESULT",
        "FocusedTextMiniPhase::InputOnly",
        "FocusedTextMiniPhase::Streaming",
        "FocusedTextMiniPhase::Result",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text mini sizing phase: {required}"
        );
    }
    for required in [
        "FOCUSED_TEXT_MINI_INPUT_ONLY_HEIGHT",
        "FOCUSED_TEXT_MINI_STREAMING_HEIGHT",
        "FOCUSED_TEXT_MINI_RESULT_HEIGHT",
        "ViewType::FocusedTextMini",
    ] {
        assert!(
            resize.contains(required),
            "missing focused-text mini resize constant: {required}"
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
fn legacy_inline_agent_window_launch_is_not_public_product_api() {
    for forbidden in [
        "launch_inline_agent_from_focused_text",
        "sync_inline_agent_overlay_window",
        "open_inline_agent_mock_fixture",
        "open_inline_agent_pi_fixture",
    ] {
        assert!(
            !INLINE_AGENT_MOD.contains(forbidden),
            "legacy inline-agent window function must not be re-exported as product API: {forbidden}"
        );
    }
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
        "\"focused-text-action-collapse\"",
        "\"focused-text-action-stop\"",
        "\"focused-text-action-retry\"",
        "source_name: Some(\"Cmd+K\"",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text ACP automation contract: {required}"
        );
    }

    assert!(APP_LAYOUT_COLLECT_ELEMENTS.contains("AppView::AcpChatView { entity }"));
    assert!(APP_LAYOUT_COLLECT_ELEMENTS.contains("collect_focused_text_mini_elements"));
}

#[test]
fn focused_text_mini_escape_hides_instead_of_returning_to_main_menu() {
    assert!(
        ACP_VIEW.contains("pub(crate) fn is_focused_text_mini")
            && ACP_VIEW.contains("focused_text_originated_from_quick_prompt")
            && ACP_VIEW.contains("focused_text_quick_prompt_escape_hide_requested")
            && ACP_VIEW.contains("self.trigger_close_window_requested(window, cx);"),
        "focused-text quick-prompt Escape must request a state-first window close/hide"
    );
    assert!(
        STARTUP.contains("acp_escape_focused_text_origin")
            && STARTUP.contains("focused_text_originated_from_quick_prompt")
            && STARTUP.contains("this.close_acp_chat_main_window_state_first(cx);")
            && STARTUP.contains("Interceptor: Escape -> hide focused-text quick prompt Agent Chat"),
        "physical Escape from focused-text quick prompt origin must hide the main panel instead of restoring the launcher"
    );
    assert!(
        STARTUP_NEW_ACTIONS.contains("acp_escape_focused_text_origin")
            && STARTUP_NEW_ACTIONS.contains("focused_text_originated_from_quick_prompt")
            && STARTUP_NEW_ACTIONS.contains("this.close_acp_chat_main_window_state_first(cx);")
            && STARTUP_NEW_ACTIONS
                .contains("Interceptor: Escape -> hide focused-text quick prompt Agent Chat"),
        "startup_new_actions Escape interceptor must preserve the same focused-text quick prompt hide path"
    );
    assert!(
        SIMULATE_KEY_DISPATCH.contains("chat.is_focused_text_mini()")
            && SIMULATE_KEY_DISPATCH.contains("chat.focused_text_originated_from_quick_prompt()")
            && SIMULATE_KEY_DISPATCH
                .contains("SimulateKey: Escape - hide focused-text quick prompt Agent Chat")
            && SIMULATE_KEY_DISPATCH.contains("view.close_acp_chat_main_window_state_first(ctx);"),
        "simulateKey Escape must match physical Escape for focused-text quick prompt origin"
    );
}

#[test]
fn focused_text_quick_prompt_origin_survives_expansion_for_escape_hide() {
    for required in [
        "originated_from_quick_prompt",
        "mark_focused_text_originated_from_quick_prompt",
        "focused_text_originated_from_quick_prompt",
    ] {
        assert!(
            ACP_VIEW.contains(required) || FOCUSED_TEXT_ENTRY.contains(required),
            "missing focused-text quick-prompt origin marker: {required}"
        );
    }

    assert!(
        FOCUSED_TEXT_ENTRY.contains("mark_focused_text_originated_from_quick_prompt"),
        "focused-text entry must mark quick-prompt origin after staging"
    );
}

#[test]
fn focused_text_devtools_mutators_echo_redacted_response_envelopes() {
    for required in [
        "#[serde(rename = \"externalCommandResult\")]",
        "ExternalCommandResult",
        "#[serde(rename = \"requestId\")]",
        "command: String",
        "ok: bool",
        "error_code: Option<String>",
        "error_message: Option<String>",
    ] {
        assert!(
            PROTOCOL_SYSTEM_CONTROL.contains(required),
            "external command result protocol message must include `{required}`"
        );
    }

    assert!(
        PROTOCOL_GENERAL_CONSTRUCTORS.contains("pub fn external_command_result")
            && PROTOCOL_GENERAL_CONSTRUCTORS.contains("Message::ExternalCommandResult"),
        "external command result must use a normal protocol Message constructor"
    );
    assert!(
        PROTOCOL_PROMPT_CONSTRUCTORS.contains("Message::ExternalCommandResult { request_id, .. }"),
        "external command result must echo requestId through Message::request_id"
    );

    for (source_name, source) in [
        ("runtime_stdin.rs", RUNTIME_STDIN),
        ("app_run_setup.rs", APP_RUN_SETUP),
        ("runtime_stdin_match_tail.rs", RUNTIME_STDIN_MATCH_TAIL),
    ] {
        for command in ["setAcpInput", "setAcpTestFixture"] {
            assert!(
                source.contains(&format!("\"{command}\".to_string()"))
                    && source.contains("crate::protocol::Message::external_command_result")
                    && source.contains("let request_id_value = request_id.clone();"),
                "{source_name} must send a redacted response envelope for {command}"
            );
        }
    }

    for (source_name, source) in [
        ("runtime_stdin.rs", RUNTIME_STDIN),
        ("app_run_setup.rs", APP_RUN_SETUP),
        (
            "runtime_stdin_match_simulate_key.rs",
            RUNTIME_STDIN_MATCH_SIMULATE_KEY,
        ),
    ] {
        assert!(
            source.contains(
                "ExternalCommand::SimulateKey { ref key, ref modifiers, ref target, ref request_id }"
            ) && source.contains("\"simulateKey\".to_string()")
                && source.contains("crate::protocol::Message::external_command_result"),
            "{source_name} must send a response envelope for simulateKey"
        );
    }
}
