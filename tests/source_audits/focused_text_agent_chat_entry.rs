const APP_RUN_SETUP: &str = include_str!("../../src/main_entry/app_run_setup.rs");
const RUNTIME_TRAY_HOTKEYS: &str = include_str!("../../src/main_entry/runtime_tray_hotkeys.rs");
const FOCUSED_TEXT_ENTRY: &str =
    include_str!("../../src/app_impl/agent_handoff/focused_text_entry.rs");
const AGENT_CHAT_VIEW: &str = include_str!("../../src/ai/agent_chat/ui/view.rs");
const AGENT_CHAT_TRANSCRIPT: &str =
    include_str!("../../src/ai/agent_chat/ui/components/transcript.rs");
const AGENT_CHAT_UI_VARIANT: &str = include_str!("../../src/ai/agent_chat/ui/ui_variant.rs");
const AGENT_CHAT_PI_LAUNCH: &str = include_str!("../../src/ai/agent_chat/launch.rs");
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
const AGENT_CHAT_STATE_TYPES: &str = include_str!("../../src/protocol/types/agent_chat_state.rs");
const APP_LAUNCHER_DB_CACHE: &str = include_str!("../../src/app_launcher/db_cache.rs");
const WINDOW_RESIZE: &str = include_str!("../../src/window_resize/mod.rs");
const PROTOCOL_SYSTEM_CONTROL: &str =
    include_str!("../../src/protocol/message/variants/system_control.rs");
const PROTOCOL_GENERAL_CONSTRUCTORS: &str =
    include_str!("../../src/protocol/message/constructors/general.rs");
const PROTOCOL_PROMPT_CONSTRUCTORS: &str =
    include_str!("../../src/protocol/message/constructors/prompts.rs");
const PROMPT_HANDLER: &str = include_str!("../../src/prompt_handler/mod.rs");

fn source_between<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = source.find(start_marker).unwrap_or_else(|| {
        panic!("missing start marker `{start_marker}`");
    });
    let after_start = &source[start..];
    let end = after_start.find(end_marker).unwrap_or_else(|| {
        panic!("missing end marker `{end_marker}` after `{start_marker}`");
    });
    &after_start[..end]
}

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
        "AgentChatUiVariant::FocusedTextMini",
        "begin_tab_ai_harness_entry_from_source_view",
        "force_agent_chat_surface",
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
fn agent_chat_view_can_stage_focused_text_as_host_owned_context() {
    for required in [
        "stage_focused_text_from_host",
        "focused-text://",
        "Focused Text ·",
        "replace_pending_context_parts(vec![part], source, cx)",
        "focused_text_context_staged",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(required),
            "missing focused-text Agent Chat staging contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_uses_pi_text_profile_not_agent_chat_backend_fallback() {
    for required in [
        "resolve_focused_text_pi_launch",
        "selected_profile_id: Some(BUILTIN_TEXT_PROFILE_ID.to_string())",
        "AgentChatBackend::Pi",
        "PiAgentChatLaunch::from_profile(resolve_effective_profile(&text_ai, ctx))",
    ] {
        assert!(
            AGENT_CHAT_PI_LAUNCH.contains(required),
            "missing focused-text Pi launch resolver contract: {required}"
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
        "set_ui_variant(AgentChatUiVariant::Standard",
        "set_ui_variant(AgentChatUiVariant::FocusedTextMini",
        "set_on_focused_text_expand_requested",
        "set_on_focused_text_collapse_requested",
        "MainWindowMode::Full",
        "MainWindowMode::Mini",
        "focused_text_expand_agent_chat",
        "focused_text_collapse_agent_chat",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(required)
                || include_str!("../../src/app_impl/agent_handoff/mod.rs").contains(required),
            "missing focused-text footer dispatch contract: {required}"
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

#[test]
fn focused_text_mini_initial_state_is_input_only_without_native_footer() {
    for required in [
        "enum FocusedTextMiniPhase",
        "FocusedTextMiniPhase::InputOnly",
        "focused_text_mini_phase_for_thread",
        "focused_text_mini_footer_visible_for_thread",
        "main_window_footer_visible",
        "main_window_footer_slot",
        "agent_chat_footer_hidden",
        "visible: bool",
        "FOCUSED_TEXT_MINI_SIZE_INPUT_ONLY",
        "crate::window_resize::ViewType::FocusedTextMini",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(required)
                || FOCUSED_TEXT_ENTRY.contains(required)
                || include_str!("../../src/app_impl/ui_window.rs").contains(required)
                || PROMPT_HANDLER.contains(required),
            "missing focused-text input-only footer contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_has_four_sizing_phases() {
    for required in [
        "FOCUSED_TEXT_MINI_SIZE_INPUT_ONLY",
        "FOCUSED_TEXT_MINI_SIZE_RESULT",
        "FocusedTextMiniPhase::InputOnly",
        "FocusedTextMiniPhase::Loading",
        "FocusedTextMiniPhase::Streaming",
        "FocusedTextMiniPhase::Result",
        "FocusedTextMiniPhase::Error",
        "FOCUSED_TEXT_MINI_SIZE_VARIATIONS",
        "let has_variations = !self.focused_text_variations.is_empty();",
        "FocusedTextMiniPhase::Loading if has_variations => Some(result_size + scope_extra)",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(required),
            "missing focused-text mini sizing phase: {required}"
        );
    }
    assert!(
        AGENT_CHAT_VIEW.contains(
            "FocusedTextMiniPhase::Loading => Some(FOCUSED_TEXT_MINI_SIZE_INPUT_ONLY + scope_extra)"
        ),
        "loading focused-text mini without variations should keep the compact input-only size before assistant output"
    );
    assert!(
        AGENT_CHAT_VIEW
            .contains("FocusedTextMiniPhase::Streaming => Some(result_size + scope_extra)"),
        "streaming focused-text mini should grow once assistant output or variations are visible"
    );
    for required in [
        "focused_text_mini_input_height",
        "focused_text_mini_result_height",
        "focused_text_mini_preview_height",
        "focused_text_mini_inner_height",
        "FOCUSED_TEXT_MINI_INPUT_ONLY_HEIGHT",
        "FOCUSED_TEXT_MINI_RESULT_HEIGHT",
        "layout::WINDOW_BORDER_Y",
        "ViewType::FocusedTextMini",
    ] {
        assert!(
            WINDOW_RESIZE.contains(required),
            "missing focused-text mini resize constant: {required}"
        );
    }
    assert!(
        WINDOW_RESIZE.contains(
            "const FOCUSED_TEXT_MINI_INPUT_ONLY_HEIGHT: f32 = crate::panel::PROMPT_INPUT_FIELD_HEIGHT;"
        ),
        "input-only focused-text mini must match the shared prompt input height"
    );
}

#[test]
fn focused_text_mini_result_uses_shared_agent_chat_transcript_component() {
    let render_fn = source_between(
        AGENT_CHAT_VIEW,
        "fn render_focused_text_mini",
        "fn render_pending_context_chips",
    );
    let mini_branch = source_between(
        AGENT_CHAT_VIEW,
        "if self.ui_variant == AgentChatUiVariant::FocusedTextMini {\n            let focused_phase",
        ".child(self.render_focused_text_mini(",
    );

    assert!(
        mini_branch.contains("Some(self.ensure_transcript(cx).into_any_element())"),
        "focused-text mini result must render the shared Agent Chat transcript entity"
    );
    assert!(
        render_fn.contains("transcript: Option<gpui::AnyElement>"),
        "focused-text mini render should receive the transcript element instead of building custom markdown"
    );
    assert!(
        !render_fn.contains("render_markdown_with_scope"),
        "focused-text mini result must not use a bespoke markdown preview"
    );
    assert!(
        AGENT_CHAT_TRANSCRIPT.contains("AgentChatTranscriptPresentation::FocusedTextPreview"),
        "AgentChatTranscript must own the focused-text preview presentation"
    );
    assert!(
        AGENT_CHAT_TRANSCRIPT.contains("let messages_snapshot = self.messages.clone();"),
        "focused-text preview must keep the shared transcript message model intact"
    );
    assert!(
        !AGENT_CHAT_TRANSCRIPT.contains(".filter(|message|"),
        "focused-text preview must hide rows at render time, not build a filtered message model"
    );
    assert!(
        AGENT_CHAT_TRANSCRIPT.contains("if focused_text_preview")
            && AGENT_CHAT_TRANSCRIPT.contains("return div().into_any();"),
        "focused-text preview should suppress non-assistant/empty rows in the render path"
    );
    assert!(
        render_fn.contains("\"focused-text-preview\""),
        "focused-text transcript wrapper should preserve the stable DevTools preview id"
    );
}

#[test]
fn focused_text_mini_window_animation_reveals_fixed_height_content() {
    let render_fn = source_between(
        AGENT_CHAT_VIEW,
        "fn render_focused_text_mini",
        "fn render_pending_context_chips",
    );
    let input_row = source_between(
        render_fn,
        ".id(\"focused-text-mini-input-row\")",
        ".when_some(self.focused_text.as_ref()",
    );
    let preview_block = source_between(
        render_fn,
        ".id(\"focused-text-preview\")",
        ".child(transcript)",
    );

    for required in [
        "crate::window_resize::focused_text_mini_input_height()",
        "crate::window_resize::focused_text_mini_result_height()",
        "crate::window_resize::focused_text_mini_preview_height()",
        "focused_text_mini_layout_budget(total_height, self.scope_visible, footer_height)",
        "let content_height = budget.content_height;",
        ".when(reserve_native_footer",
        "\"focused-text-mini-content\"",
        ".h(px(content_height))",
        ".max_h(px(content_height))",
        ".flex_none()",
        ".overflow_hidden()",
    ] {
        assert!(
            render_fn.contains(required),
            "focused-text mini animation must reveal fixed content instead of stretching: {required}"
        );
    }
    assert!(input_row.contains(".h(px(input_height))"));
    assert!(input_row.contains(".max_h(px(input_height))"));
    assert!(input_row.contains(".flex_none()"));
    assert!(preview_block.contains(".h(px(preview_height))"));
    assert!(preview_block.contains(".max_h(px(preview_height))"));
    assert!(preview_block.contains(".flex_none()"));
    assert!(
        !preview_block.contains(".flex_1()"),
        "focused-text preview viewport must not flex during window height animation"
    );
    assert!(
        !preview_block.contains(".min_h("),
        "focused-text preview viewport should use fixed height, not min-height fallback"
    );
    assert!(render_fn.contains("content = content.child("));
    assert!(render_fn.contains(".child(transcript)"));
}

#[test]
fn focused_text_mini_height_helpers_match_resize_contract() {
    for required in [
        "pub(crate) fn focused_text_mini_input_height() -> f32",
        "pub(crate) fn focused_text_mini_result_height() -> f32",
        "pub(crate) fn focused_text_mini_preview_height() -> f32",
        "pub(crate) fn focused_text_mini_inner_height(window_height: f32) -> f32",
        "FOCUSED_TEXT_MINI_RESULT_HEIGHT - FOCUSED_TEXT_MINI_INPUT_ONLY_HEIGHT",
        "layout::WINDOW_BORDER_Y",
        "ViewType::FocusedTextMini => match item_count",
        "FOCUSED_TEXT_MINI_VARIATIONS_HEIGHT",
        "FOCUSED_TEXT_MINI_RESIZE_ANIMATE",
    ] {
        assert!(
            WINDOW_RESIZE.contains(required),
            "focused-text mini sizing helpers must stay aligned with resize contract: {required}"
        );
    }
    for required in [
        "let scope_extra = if self.scope_visible { 1 } else { 0 };",
        "focused_text_mini_sizing_count",
        ".debug_selector(|| \"focused-text-mini-scope-row\".to_string())",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(required),
            "focused-text mini scope-aware sizing path must contain: {required}"
        );
    }
}

#[test]
fn focused_text_mini_reuses_app_icon_cache_and_focuses_composer_on_open() {
    assert!(
        APP_LAUNCHER_DB_CACHE.contains("pub fn cached_app_icon_for_bundle"),
        "main-menu app icon cache helper must exist"
    );
    for required in [
        "app_bundle_id: Option<String>",
        "snapshot.app.bundle_id.clone()",
        "bundle_id.trim()",
        "crate::app_launcher::cached_app_icon_for_bundle",
        "crate::icons::render_image",
        "gpui_component::IconName::AppWindow",
        "\"focused-text-context-badge\"",
    ] {
        assert!(
            AGENT_CHAT_VIEW.contains(required),
            "focused-text badge must reuse the main-menu app icon cache: {required}"
        );
    }
    let badge_fn = source_between(
        AGENT_CHAT_VIEW,
        "fn render_focused_text_app_icon_badge",
        "#[allow(clippy::too_many_arguments)]",
    );
    assert!(
        !badge_fn.contains(".chars().next()"),
        "icon cache misses must render a generic icon, not an app-name initial"
    );
    assert!(
        FOCUSED_TEXT_ENTRY.contains("self.request_focus(FocusTarget::ChatPrompt, cx);"),
        "focused-text mini open must request composer focus after entering the Agent Chat surface"
    );
}

#[test]
fn focused_text_mini_uses_targeted_animated_resize_without_global_resize_animation() {
    for required in [
        "const WINDOW_RESIZE_ANIMATE: bool = false",
        "const FOCUSED_TEXT_MINI_RESIZE_ANIMATE: bool = true",
        "resize_first_window_to_size_with_animation",
        "matches!(view_type, ViewType::FocusedTextMini)",
    ] {
        assert!(
            WINDOW_RESIZE.contains(required),
            "focused-text mini resize animation contract missing: {required}"
        );
    }
}

#[test]
fn focused_text_agent_chat_stdin_verbs_use_explicit_focused_text_names() {
    for required in [
        "OpenFocusedTextAgentChatWithMockData",
        "OpenFocusedTextAgentChatWithPiData",
        "\"openFocusedTextAgentChatWithMockData\"",
        "\"openFocusedTextAgentChatWithPiData\"",
    ] {
        assert!(
            STDIN_COMMANDS.contains(required),
            "missing focused-text stdin command contract: {required}"
        );
    }

    for required in [
        "ExternalCommand::OpenFocusedTextAgentChatWithMockData",
        "ExternalCommand::OpenFocusedTextAgentChatWithPiData",
        "open_focused_text_agent_chat_fixture",
        "focused_text_mock_fixture",
        "focused_text_pi_fixture",
    ] {
        assert!(
            RUNTIME_STDIN.contains(required),
            "missing focused-text stdin dispatch contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_has_redacted_agent_chat_state_and_semantic_elements() {
    for required in [
        "pub focused_text: Option<AgentChatFocusedTextState>",
        "pub struct AgentChatFocusedTextState",
        "char_count",
        "has_output",
        "last_apply_action",
    ] {
        assert!(
            AGENT_CHAT_STATE_TYPES.contains(required),
            "missing focused-text Agent Chat state contract: {required}"
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
            AGENT_CHAT_VIEW.contains(required),
            "missing focused-text Agent Chat automation contract: {required}"
        );
    }

    assert!(APP_LAYOUT_COLLECT_ELEMENTS.contains("AppView::AgentChatView { entity }"));
    assert!(APP_LAYOUT_COLLECT_ELEMENTS.contains("collect_focused_text_mini_elements"));
}

#[test]
fn focused_text_mini_escape_hides_instead_of_returning_to_main_menu() {
    assert!(
        AGENT_CHAT_VIEW.contains("pub(crate) fn is_focused_text_mini")
            && AGENT_CHAT_VIEW.contains("focused_text_originated_from_quick_prompt")
            && AGENT_CHAT_VIEW.contains("event = \"focused_text_escape_progressive\"")
            && AGENT_CHAT_VIEW.contains("self.trigger_close_window_requested(window, cx);"),
        "focused-text quick-prompt Escape must progressively unwind before requesting close/hide"
    );
    assert!(
        STARTUP.contains("agent_chat_escape_focused_text_origin")
            && STARTUP.contains("focused_text_originated_from_quick_prompt")
            && STARTUP.contains("!agent_chat_escape_focused_text_origin"),
        "physical Escape from focused-text quick prompt origin must avoid generic streaming cancel before Agent Chat handles the key"
    );
    assert!(
        STARTUP_NEW_ACTIONS.contains("agent_chat_escape_focused_text_origin")
            && STARTUP_NEW_ACTIONS.contains("focused_text_originated_from_quick_prompt")
            && STARTUP_NEW_ACTIONS.contains("!agent_chat_escape_focused_text_origin"),
        "startup_new_actions Escape interceptor must preserve focused-text quick prompt Agent Chat key handling"
    );
    assert!(
        SIMULATE_KEY_DISPATCH.contains("chat.is_focused_text_mini()")
            && SIMULATE_KEY_DISPATCH.contains("chat.focused_text_originated_from_quick_prompt()")
            && SIMULATE_KEY_DISPATCH
                .contains("SimulateKey: Escape - hide focused-text quick prompt Agent Chat")
            && SIMULATE_KEY_DISPATCH
                .contains("view.close_agent_chat_main_window_state_first(ctx);"),
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
            AGENT_CHAT_VIEW.contains(required) || FOCUSED_TEXT_ENTRY.contains(required),
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
        for command in ["setAgentChatInput", "setAgentChatTestFixture"] {
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
