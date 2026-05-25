const HOTKEYS: &str = include_str!("../../src/hotkeys/mod.rs");
const CONFIG_TYPES: &str = include_str!("../../src/config/types.rs");
const CONFIG_LOADER: &str = include_str!("../../src/config/loader.rs");
const CONFIG_SCHEMA: &str = include_str!("../../scripts/config-schema.ts");
const APP_RUN_SETUP: &str = include_str!("../../src/main_entry/app_run_setup.rs");
const RUNTIME_TRAY_HOTKEYS: &str = include_str!("../../src/main_entry/runtime_tray_hotkeys.rs");
const AUTOMATION_SURFACE_COLLECTOR: &str =
    include_str!("../../src/windows/automation_surface_collector.rs");
const PROMPT_HANDLER: &str = include_str!("../../src/prompt_handler/mod.rs");
const STDIN_COMMANDS: &str = include_str!("../../src/stdin_commands/mod.rs");
const RUNTIME_STDIN: &str = include_str!("../../src/main_entry/runtime_stdin.rs");
const SIMULATE_KEY_DISPATCH: &str = include_str!("../../src/app_impl/simulate_key_dispatch.rs");
const FOCUSED_TEXT: &str = include_str!("../../src/platform/accessibility/focused_text.rs");

#[test]
fn hotkeys_define_inline_ai_action_and_channel() {
    assert!(HOTKEYS.contains("InlineAiTextEdit"));
    assert!(HOTKEYS.contains("INLINE_AI_HOTKEY_CHANNEL"));
    assert!(HOTKEYS.contains("inline_ai_hotkey_channel"));
    assert!(HOTKEYS.contains("hotkey:inline-ai"));
}

#[test]
fn config_defines_inline_ai_hotkey_surface() {
    assert!(CONFIG_TYPES.contains("inline_ai_hotkey: Option<HotkeyConfig>"));
    assert!(CONFIG_TYPES.contains("inline_ai_hotkey_enabled: Option<bool>"));
    assert!(CONFIG_TYPES.contains("default_inline_ai_hotkey"));
    assert!(CONFIG_TYPES.contains("get_inline_ai_hotkey"));
    assert!(CONFIG_LOADER.contains("inlineAiHotkey"));
    assert!(CONFIG_LOADER.contains("inlineAiHotkeyEnabled"));
    assert!(CONFIG_SCHEMA.contains("inlineAiHotkey?: HotkeyConfig"));
    assert!(CONFIG_SCHEMA.contains("inlineAiHotkeyEnabled?: boolean"));
}

#[test]
fn hotkeys_register_inline_ai_action_on_startup_and_reload() {
    assert!(HOTKEYS.contains("cfg.get_inline_ai_hotkey()"));
    assert!(HOTKEYS.contains("config.get_inline_ai_hotkey()"));
    assert!(HOTKEYS.contains("rebind_hotkey_transactional(\n                &manager_guard,\n                HotkeyAction::InlineAiTextEdit"));
    assert!(HOTKEYS.contains("register_builtin_hotkey(\n                &manager_guard,\n                HotkeyAction::InlineAiTextEdit"));
}

#[test]
fn app_hotkey_listeners_launch_inline_agent_from_channel() {
    for source in [APP_RUN_SETUP, RUNTIME_TRAY_HOTKEYS] {
        assert!(source.contains("inline_ai_hotkey_channel().1.recv().await"));
        assert!(source.contains("capture_focused_text_field"));
        assert!(source.contains("open_focused_text_agent_chat_from_snapshot"));
        assert!(!source.contains("crate::inline_agent::launch_inline_agent_from_focused_text"));
        assert!(!source.contains("sync_inline_agent_overlay_window"));
    }
}

#[test]
fn production_hotkey_path_captures_before_opening_focused_text_agent_chat() {
    for source in [APP_RUN_SETUP, RUNTIME_TRAY_HOTKEYS] {
        let channel = source
            .find("inline_ai_hotkey_channel().1.recv().await")
            .expect("inline AI listener must exist");
        let capture_index = source[channel..]
            .find("capture_focused_text_field")
            .expect("inline AI listener should capture focused text")
            + channel;
        let open_index = source[channel..]
            .find("open_focused_text_agent_chat_from_snapshot")
            .expect("inline AI listener should open focused-text Agent Chat")
            + channel;

        assert!(
            capture_index < open_index,
            "focused text capture must happen before Agent Chat opens"
        );
        assert!(source.contains("focused_text_capture_complete_before_agent_chat"));
        assert!(source.contains("\"inline_ai_hotkey\""));
    }
}

#[test]
fn inline_agent_launch_does_not_use_selected_text_fallback() {
    assert!(!FOCUSED_TEXT.contains("get_selected_text("));
    assert!(FOCUSED_TEXT.contains("copy_all_plain_text_preserving_clipboard"));
    for source in [APP_RUN_SETUP, RUNTIME_TRAY_HOTKEYS] {
        assert!(!source.contains("get_selected_text("));
    }
}

#[test]
fn targeted_escape_closes_inline_agent_overlay() {
    assert!(SIMULATE_KEY_DISPATCH.contains("target_val"));
    assert!(SIMULATE_KEY_DISPATCH.contains("INLINE_AGENT_WINDOW_AUTOMATION_ID"));
    assert!(SIMULATE_KEY_DISPATCH.contains("AutomationWindowKind::MiniAi"));
    assert!(SIMULATE_KEY_DISPATCH.contains("close_inline_agent_overlay_window(ctx)"));
    assert!(
        SIMULATE_KEY_DISPATCH.contains("SimulateKey: Escape - close Inline Agent target"),
        "targeted simulateKey Escape must close the inline-agent overlay instead of falling through to the main window"
    );

    let escape_body = SIMULATE_KEY_DISPATCH
        .split("if key_lower == \"escape\"")
        .nth(1)
        .expect("targeted Escape branch should exist");
    let close_index = escape_body
        .find("close_inline_agent_overlay_window(ctx)")
        .expect("targeted Escape should close inline-agent");
    let notes_index = escape_body
        .find("simulate_key_target_is_notes")
        .expect("notes routing should remain after inline-agent target handling");
    assert!(
        close_index < notes_index,
        "inline-agent target Escape must run before generic main/notes routing"
    );
}

#[test]
fn automation_collector_exposes_inline_agent_semantic_elements() {
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("AutomationWindowKind::MiniAi"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("collect_inline_agent_snapshot"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("inline_agent_current_window_snapshot"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("inline_agent_run_state_kind"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("inline_agent_action_disabled_reason"));
    for semantic_id in [
        "INLINE_AGENT_COMPACT_ID",
        "INLINE_AGENT_EXPANDED_ID",
        "INLINE_AGENT_HEADER_ID",
        "INLINE_AGENT_INPUT_ID",
        "INLINE_AGENT_EXPANDED_COMPOSER_ID",
        "INLINE_AGENT_APP_BADGE_ID",
        "INLINE_AGENT_METRICS_ID",
        "INLINE_AGENT_THINKING_BAR_ID",
        "INLINE_AGENT_THINKING_LABEL_ID",
        "INLINE_AGENT_OUTPUT_PREVIEW_ID",
        "INLINE_AGENT_ACTION_REPLACE_ID",
        "INLINE_AGENT_ACTION_APPEND_ID",
        "INLINE_AGENT_ACTION_COPY_ID",
        "INLINE_AGENT_ACTION_CHAT_ID",
        "INLINE_AGENT_ACTION_STOP_ID",
        "INLINE_AGENT_ACTION_RETRY_ID",
        "INLINE_AGENT_TURN_LIST_ID",
        "INLINE_AGENT_COLLAPSE_ID",
    ] {
        assert!(
            AUTOMATION_SURFACE_COLLECTOR.contains(semantic_id),
            "inline agent collector should expose {semantic_id}"
        );
    }
    for phase in [
        "\"idle\"",
        "\"thinking\"",
        "\"streaming\"",
        "\"completed\"",
        "\"error\"",
        "\"applying\"",
        "\"applied\"",
    ] {
        assert!(
            AUTOMATION_SURFACE_COLLECTOR.contains(phase),
            "inline agent collector should expose redacted phase {phase}"
        );
    }
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("Some(\"Output preview\".to_string())"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("Some(\"no-output\")"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("Some(\"active-turn\")"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("Some(\"not-retryable\")"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("Some(\"already-expanded\")"));
    assert!(!AUTOMATION_SURFACE_COLLECTOR.contains("partial_output.clone()"));
    assert!(!AUTOMATION_SURFACE_COLLECTOR.contains("latest_complete_output().map"));
}

#[test]
fn get_state_routes_inline_agent_target_to_redacted_state_envelope() {
    assert!(PROMPT_HANDLER.contains("GetStateTargetResolution::InlineAgent"));
    assert!(PROMPT_HANDLER.contains("AutomationWindowKind::MiniAi"));
    assert!(PROMPT_HANDLER.contains("INLINE_AGENT_WINDOW_AUTOMATION_ID"));
    assert!(PROMPT_HANDLER.contains("inline_agent_automation_state()"));
    assert!(PROMPT_HANDLER.contains("\"inlineAgent\".to_string()"));
    assert!(PROMPT_HANDLER.contains("Some(state)"));

    assert!(PROMPT_HANDLER.contains("\"inlineAgent\".to_string()"));
}

#[test]
fn stdin_exposes_inline_agent_mock_fixture_without_changing_pi_default() {
    assert!(STDIN_COMMANDS.contains("OpenFocusedTextAgentChatWithMockData"));
    assert!(STDIN_COMMANDS.contains("OpenFocusedTextAgentChatWithPiData"));
    assert!(STDIN_COMMANDS.contains("\"openFocusedTextAgentChatWithMockData\""));
    assert!(STDIN_COMMANDS.contains("\"openFocusedTextAgentChatWithPiData\""));
    assert!(STDIN_COMMANDS.contains("OpenInlineAgentWithMockData"));
    assert!(STDIN_COMMANDS.contains("OpenInlineAgentWithPiData"));
    assert!(STDIN_COMMANDS.contains("\"openInlineAgentWithMockData\""));
    assert!(STDIN_COMMANDS.contains("\"openInlineAgentWithPiData\""));
    assert!(STDIN_COMMANDS.contains("instruction: Option<String>"));
    assert!(STDIN_COMMANDS.contains("text: Option<String>"));

    for dispatch in [RUNTIME_STDIN, APP_RUN_SETUP] {
        assert!(dispatch.contains("ExternalCommand::OpenFocusedTextAgentChatWithMockData"));
        assert!(dispatch.contains("ExternalCommand::OpenFocusedTextAgentChatWithPiData"));
        assert!(dispatch.contains("ExternalCommand::OpenInlineAgentWithMockData"));
        assert!(dispatch.contains("ExternalCommand::OpenInlineAgentWithPiData"));
        assert!(dispatch.contains("open_focused_text_agent_chat_fixture"));
        assert!(!dispatch.contains("open_inline_agent_mock_fixture"));
        assert!(!dispatch.contains("open_inline_agent_pi_fixture"));
    }
}
