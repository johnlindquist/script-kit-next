const INLINE_AGENT_MOD: &str = include_str!("../../src/inline_agent/mod.rs");
const INLINE_AGENT_AUTOMATION: &str = include_str!("../../src/inline_agent/automation.rs");
const INLINE_AGENT_BRIDGE: &str = include_str!("../../src/inline_agent/platform_bridge.rs");
const INLINE_AGENT_WINDOW: &str = include_str!("../../src/inline_agent/window.rs");

#[test]
fn overlay_declares_required_ownership_modules() {
    for module in [
        "pub mod types;",
        "pub mod state;",
        "pub mod layout;",
        "pub mod window;",
        "pub mod render_compact;",
        "pub mod render_expanded;",
        "pub mod render_actions;",
        "pub mod telemetry;",
        "pub mod automation;",
        "pub mod platform_bridge;",
        "pub mod theme;",
    ] {
        assert!(INLINE_AGENT_MOD.contains(module), "missing {module}");
    }
}

#[test]
fn overlay_bridge_depends_on_snapshots_not_ax_handles() {
    assert!(INLINE_AGENT_BRIDGE.contains("trait InlineAgentPlatformBridge"));
    assert!(INLINE_AGENT_BRIDGE.contains("capture_focused_text_snapshot"));
    assert!(INLINE_AGENT_BRIDGE.contains("apply_text_mutation"));
    assert!(!INLINE_AGENT_BRIDGE.contains("AXUIElement"));
}

#[test]
fn overlay_pins_stable_automation_ids() {
    for id in [
        "inline-agent-compact",
        "inline-agent-header",
        "inline-agent-app-badge",
        "inline-agent-metrics",
        "inline-agent-input",
        "inline-agent-thinking-bar",
        "inline-agent-thinking-label",
        "inline-agent-output-preview",
        "inline-agent-action-replace",
        "inline-agent-action-append",
        "inline-agent-action-copy",
        "inline-agent-action-chat",
        "inline-agent-expanded",
        "inline-agent-turn-list",
        "inline-agent-expanded-composer",
        "inline-agent-collapse",
    ] {
        assert!(
            INLINE_AGENT_AUTOMATION.contains(id),
            "missing stable id {id}"
        );
    }
}

#[test]
fn standalone_overlay_attachment_is_explicit() {
    assert!(INLINE_AGENT_WINDOW.contains("InlineOverlayAttachment"));
    assert!(INLINE_AGENT_WINDOW.contains("Standalone"));
    assert!(INLINE_AGENT_WINDOW.contains("AttachedToParent"));
}

#[test]
fn window_plan_requires_capture_before_opening_overlay() {
    assert!(INLINE_AGENT_WINDOW
        .contains("pub fn plan_open_inline_agent_overlay(\n    snapshot: &InlineAgentSnapshot,"));
    assert!(INLINE_AGENT_WINDOW.contains("place_compact_overlay"));
    assert!(INLINE_AGENT_WINDOW.contains("focus_prompt: true"));
}

#[test]
fn overlay_window_snapshot_keeps_ui_safe_metadata_only() {
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentWindowSnapshot"));
    assert!(INLINE_AGENT_WINDOW.contains("session_id: String"));
    assert!(INLINE_AGENT_WINDOW.contains("app_name: String"));
    assert!(!INLINE_AGENT_WINDOW.contains("AXUIElement"));
}

#[test]
fn overlay_declares_theme_contrast_resolver() {
    let theme = include_str!("../../src/inline_agent/theme.rs");
    assert!(theme.contains("InlineAgentColors"));
    assert!(theme.contains("from_theme"));
    assert!(theme.contains("contrast_ratio"));
    assert!(theme.contains("best_readable_text_hex"));
}

#[test]
fn compact_and_expanded_renderers_use_view_models() {
    let compact = include_str!("../../src/inline_agent/render_compact.rs");
    let expanded = include_str!("../../src/inline_agent/render_expanded.rs");

    assert!(compact.contains("InlineAgentCompactViewModel"));
    assert!(compact.contains("INLINE_AGENT_INPUT_PLACEHOLDER"));
    assert!(compact.contains("THINKING_LABEL"));
    assert!(compact.contains("is_action_enabled_for_snapshot"));
    assert!(expanded.contains("InlineAgentExpandedViewModel"));
    assert!(expanded.contains("expanded_header_label"));
}

#[test]
fn window_module_declares_gpui_options_and_automation_registration() {
    assert!(INLINE_AGENT_WINDOW.contains("inline_agent_window_options"));
    assert!(INLINE_AGENT_WINDOW.contains("WindowKind::PopUp"));
    assert!(INLINE_AGENT_WINDOW.contains("WindowBounds::Windowed"));
    assert!(INLINE_AGENT_WINDOW.contains("is_resizable: false"));
    assert!(INLINE_AGENT_WINDOW.contains("INLINE_AGENT_WINDOW_AUTOMATION_ID"));
    assert!(INLINE_AGENT_WINDOW.contains("AutomationWindowKind::MiniAi"));
    assert!(INLINE_AGENT_WINDOW.contains("upsert_automation_window"));
    assert!(INLINE_AGENT_WINDOW.contains("set_automation_bounds"));
    assert!(INLINE_AGENT_WINDOW.contains("remove_automation_window"));
}

#[test]
fn window_module_declares_real_gpui_overlay_lifecycle() {
    assert!(INLINE_AGENT_WINDOW.contains("struct InlineAgentOverlayWindow"));
    assert!(INLINE_AGENT_WINDOW.contains("impl Render for InlineAgentOverlayWindow"));
    assert!(INLINE_AGENT_WINDOW.contains("sync_inline_agent_overlay_window"));
    assert!(INLINE_AGENT_WINDOW.contains("cx.open_window"));
    assert!(INLINE_AGENT_WINDOW.contains("close_inline_agent_overlay_window"));
    assert!(INLINE_AGENT_WINDOW.contains("configure_actions_popup_window"));
    assert!(INLINE_AGENT_WINDOW.contains("set_inline_popup_window_bounds"));
}

#[test]
fn compact_action_buttons_route_to_platform_bridge_and_chat_expands_same_overlay() {
    assert!(INLINE_AGENT_WINDOW.contains("handle_output_action"));
    assert!(INLINE_AGENT_WINDOW.contains("apply_latest_output_action"));
    assert!(INLINE_AGENT_WINDOW.contains("SystemInlineAgentPlatformBridge"));
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentRunState::Applying"));
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentRunState::Applied"));
    assert!(INLINE_AGENT_WINDOW.contains("plan_expanded_inline_agent_overlay"));
    assert!(INLINE_AGENT_WINDOW.contains(".on_click(cx.listener"));
}

#[test]
fn expanded_collapse_returns_to_compact_same_window() {
    assert!(INLINE_AGENT_WINDOW.contains("collapse_expanded"));
    assert!(INLINE_AGENT_WINDOW.contains("plan_compact_inline_agent_overlay"));
    assert!(INLINE_AGENT_WINDOW.contains("INLINE_AGENT_COLLAPSE_ID"));
    assert!(INLINE_AGENT_WINDOW.contains("update_inline_agent_automation_bounds"));
}

#[test]
fn compact_prompt_input_accepts_keyboard_and_submit_updates_output_state() {
    assert!(INLINE_AGENT_WINDOW.contains("instruction_text: String"));
    assert!(INLINE_AGENT_WINDOW.contains("ai_session: InlineAgentSession"));
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentSession::new(focused_snapshot)"));
    assert!(INLINE_AGENT_WINDOW.contains("handle_key_down"));
    assert!(INLINE_AGENT_WINDOW.contains("submit_instruction"));
    assert!(INLINE_AGENT_WINDOW.contains("is_key_enter(key)"));
    assert!(INLINE_AGENT_WINDOW.contains("event.keystroke.key_char"));
    assert!(
        INLINE_AGENT_WINDOW.contains("begin_turn(instruction, InlineAgentEditSemantics::Replace")
    );
    assert!(INLINE_AGENT_WINDOW.contains("spawn_default_acp_inline_agent_executor"));
    assert!(INLINE_AGENT_WINDOW.contains("active_executor: Option<Box<dyn InlineAgentExecutor>>"));
    assert!(!INLINE_AGENT_WINDOW.contains("MockInlineAgentExecutor"));
    assert!(INLINE_AGENT_WINDOW.contains("bind_provider_stream(events, request_id"));
    assert!(INLINE_AGENT_WINDOW.contains("sync_run_state_from_ai_session"));
    assert!(INLINE_AGENT_WINDOW.contains("stream_generation"));
    assert!(INLINE_AGENT_WINDOW.contains("inline_agent_stream_event_discarded_stale_generation"));
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentRunState::Thinking"));
    assert!(INLINE_AGENT_WINDOW.contains("InlineAgentRunState::Completed"));
    assert!(INLINE_AGENT_WINDOW.contains("self.instruction_text.clear()"));
}
