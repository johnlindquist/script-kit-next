const HOTKEYS: &str = include_str!("../../src/hotkeys/mod.rs");
const CONFIG_TYPES: &str = include_str!("../../src/config/types.rs");
const CONFIG_LOADER: &str = include_str!("../../src/config/loader.rs");
const CONFIG_SCHEMA: &str = include_str!("../../scripts/config-schema.ts");
const APP_RUN_SETUP: &str = include_str!("../../src/main_entry/app_run_setup.rs");
const RUNTIME_TRAY_HOTKEYS: &str = include_str!("../../src/main_entry/runtime_tray_hotkeys.rs");
const INLINE_AGENT_WINDOW: &str = include_str!("../../src/inline_agent/window.rs");
const INLINE_AGENT_TYPES: &str = include_str!("../../src/inline_agent/types.rs");
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
    }
}

#[test]
fn launch_path_captures_focused_text_before_opening_overlay() {
    let launch_body = INLINE_AGENT_WINDOW
        .split("pub fn launch_inline_agent_from_focused_text")
        .nth(1)
        .expect("launch function should exist");
    let capture_index = launch_body
        .find("capture_focused_text_field")
        .expect("launch path should call focused text capture");
    let sync_index = launch_body
        .find("sync_inline_agent_overlay_window(cx, focused_snapshot, plan, None)")
        .expect("launch path should open overlay after planning");

    assert!(
        capture_index < sync_index,
        "inline agent launch must capture focused text before opening the overlay"
    );
    assert!(INLINE_AGENT_TYPES.contains("impl From<FocusedTextSnapshot> for InlineAgentSnapshot"));
}

#[test]
fn launch_path_has_capture_before_overlay_runtime_trace_points() {
    for event in [
        "inline_agent_capture_start",
        "inline_agent_capture_complete_before_overlay",
        "inline_agent_overlay_sync_start",
    ] {
        assert!(
            INLINE_AGENT_WINDOW.contains(event),
            "missing launch trace event {event}"
        );
    }

    let capture_complete = INLINE_AGENT_WINDOW
        .find("inline_agent_capture_complete_before_overlay")
        .expect("capture completion trace should exist");
    let overlay_sync = INLINE_AGENT_WINDOW
        .find("inline_agent_overlay_sync_start")
        .expect("overlay sync trace should exist");
    assert!(
        capture_complete < overlay_sync,
        "capture completion trace must precede overlay sync trace"
    );
}

#[test]
fn launch_path_resets_existing_overlay_before_recapturing() {
    assert!(INLINE_AGENT_WINDOW.contains("is_inline_agent_overlay_window_open"));
    assert!(INLINE_AGENT_WINDOW.contains("inline_agent_launch_reset_existing_overlay"));

    let launch_body = INLINE_AGENT_WINDOW
        .split("pub fn launch_inline_agent_from_focused_text")
        .nth(1)
        .expect("launch function should exist");
    let guard_index = launch_body
        .find("if is_inline_agent_overlay_window_open()")
        .expect("launch path should check for an existing inline overlay");
    let close_index = launch_body
        .find("close_inline_agent_overlay_window(cx)")
        .expect("launch path should close the existing overlay before recapture");
    let capture_index = launch_body
        .find("capture_focused_text_field")
        .expect("launch path should call focused text capture");
    assert!(
        guard_index < close_index && close_index < capture_index,
        "existing overlay reset must run before focused text capture"
    );
}

#[test]
fn inline_agent_launch_does_not_use_selected_text_fallback() {
    assert!(!INLINE_AGENT_WINDOW.contains("get_selected_text("));
    assert!(!FOCUSED_TEXT.contains("get_selected_text("));
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

    for redacted_field in [
        "\"phase\"",
        "\"mode\"",
        "\"output\"",
        "\"latestCompleteChars\"",
        "\"streamingPartialChars\"",
        "\"actions\"",
        "\"replaceEnabled\"",
        "\"stopEnabled\"",
        "\"retryEnabled\"",
    ] {
        assert!(
            INLINE_AGENT_WINDOW.contains(redacted_field),
            "inline-agent getState envelope should expose redacted field {redacted_field}"
        );
    }

    for forbidden in [
        "\"capturedText\"",
        "\"instruction\"",
        "\"prompt\"",
        "\"assistantOutput\"",
        "\"clipboard\"",
    ] {
        assert!(
            !INLINE_AGENT_WINDOW.contains(forbidden),
            "inline-agent getState envelope must not expose sensitive field/pattern {forbidden}"
        );
    }
}

#[test]
fn stdin_exposes_inline_agent_mock_fixture_without_changing_pi_default() {
    assert!(STDIN_COMMANDS.contains("OpenInlineAgentWithMockData"));
    assert!(STDIN_COMMANDS.contains("OpenInlineAgentWithPiData"));
    assert!(STDIN_COMMANDS.contains("\"openInlineAgentWithMockData\""));
    assert!(STDIN_COMMANDS.contains("\"openInlineAgentWithPiData\""));
    assert!(STDIN_COMMANDS.contains("instruction: Option<String>"));
    assert!(STDIN_COMMANDS.contains("text: Option<String>"));

    for dispatch in [RUNTIME_STDIN, APP_RUN_SETUP] {
        assert!(dispatch.contains("ExternalCommand::OpenInlineAgentWithMockData"));
        assert!(dispatch.contains("ExternalCommand::OpenInlineAgentWithPiData"));
        assert!(dispatch.contains("open_inline_agent_mock_fixture"));
        assert!(dispatch.contains("open_inline_agent_pi_fixture"));
    }

    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentExecutorMode::AgentChatPi"));
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentExecutorMode::MockFixture"));
    assert!(INLINE_AGENT_WINDOW.contains("spawn_default_agent_chat_inline_agent_executor()"));
    assert!(INLINE_AGENT_WINDOW.contains("MockInlineAgentExecutor"));
    assert!(INLINE_AGENT_WINDOW.contains("focused_text_snapshot_for_tests"));
    assert!(INLINE_AGENT_WINDOW.contains("SCRIPT_KIT_INLINE_AGENT_REAL_PI_FIXTURE"));
    assert!(INLINE_AGENT_WINDOW.contains("open_inline_agent_fixture_with_executor_mode"));

    let default_submit = INLINE_AGENT_WINDOW
        .split("fn spawn_executor_for_turn")
        .nth(1)
        .expect("inline agent should centralize executor selection");
    assert!(
        default_submit.contains("InlineAgentExecutorMode::AgentChatPi")
            && default_submit.contains("spawn_default_agent_chat_inline_agent_executor()"),
        "default inline-agent submit path must remain warm Pi-backed"
    );

    let pi_fixture = INLINE_AGENT_WINDOW
        .split("pub fn open_inline_agent_pi_fixture")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn open_inline_agent_fixture_with_executor_mode")
                .next()
        })
        .expect("real Pi fixture should be explicit and gated");
    assert!(pi_fixture.contains("INLINE_AGENT_REAL_PI_FIXTURE_ENV"));
    assert!(pi_fixture.contains("InlineAgentExecutorMode::AgentChatPi"));
    assert!(!pi_fixture.contains("MockInlineAgentExecutor"));
}
