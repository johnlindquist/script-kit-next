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
