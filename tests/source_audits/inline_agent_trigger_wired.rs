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
        assert!(source.contains("crate::inline_agent::launch_inline_agent_from_focused_text"));
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
fn launch_path_refuses_to_recapture_when_overlay_is_open() {
    assert!(INLINE_AGENT_WINDOW.contains("is_inline_agent_overlay_window_open"));
    assert!(INLINE_AGENT_WINDOW.contains("inline_agent_launch_ignored_existing_overlay"));

    let launch_body = INLINE_AGENT_WINDOW
        .split("pub fn launch_inline_agent_from_focused_text")
        .nth(1)
        .expect("launch function should exist");
    let guard_index = launch_body
        .find("if is_inline_agent_overlay_window_open()")
        .expect("launch path should guard against recapturing the inline overlay");
    let capture_index = launch_body
        .find("capture_focused_text_field")
        .expect("launch path should call focused text capture");
    assert!(
        guard_index < capture_index,
        "overlay-open guard must run before focused text capture"
    );
}

#[test]
fn inline_agent_launch_does_not_use_selected_text_fallback() {
    assert!(!INLINE_AGENT_WINDOW.contains("get_selected_text("));
    assert!(!FOCUSED_TEXT.contains("get_selected_text("));
}

#[test]
fn automation_collector_exposes_inline_agent_semantic_elements() {
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("AutomationWindowKind::MiniAi"));
    assert!(AUTOMATION_SURFACE_COLLECTOR.contains("collect_inline_agent_snapshot"));
    for semantic_id in [
        "INLINE_AGENT_COMPACT_ID",
        "INLINE_AGENT_INPUT_ID",
        "INLINE_AGENT_APP_BADGE_ID",
        "INLINE_AGENT_METRICS_ID",
    ] {
        assert!(
            AUTOMATION_SURFACE_COLLECTOR.contains(semantic_id),
            "inline agent collector should expose {semantic_id}"
        );
    }
}
