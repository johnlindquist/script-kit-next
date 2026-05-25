use script_kit_gpui::inline_agent::{
    compact_root_automation_id, inline_agent_automation_info, inline_agent_window_options,
    plan_compact_inline_agent_overlay, plan_expanded_inline_agent_overlay,
    plan_open_inline_agent_overlay, InlineAgentMode, InlineAgentRunState, InlineOverlayAttachment,
    INLINE_AGENT_SEMANTIC_SURFACE, INLINE_AGENT_WINDOW_AUTOMATION_ID, INLINE_AGENT_WINDOW_TITLE,
};
use script_kit_gpui::platform::accessibility::focused_text::focused_text_snapshot_for_tests;
use script_kit_gpui::platform::accessibility::geometry::{
    DisplayBounds, FocusedFieldGeometry, RectPx,
};
use script_kit_gpui::protocol::AutomationWindowKind;

fn snapshot_for_tests() -> script_kit_gpui::inline_agent::InlineAgentSnapshot {
    let mut focused = focused_text_snapshot_for_tests("hello world");
    focused.app.name = "Notes".to_string();
    focused.geometry = FocusedFieldGeometry {
        caret_bounds: Some(RectPx {
            x: 100.0,
            y: 120.0,
            width: 2.0,
            height: 18.0,
        }),
        selection_bounds: None,
        field_bounds: None,
        window_bounds: None,
        display_bounds: DisplayBounds::default(),
    };

    script_kit_gpui::inline_agent::InlineAgentSnapshot {
        session_id: focused.session_id,
        app: focused.app,
        text: focused.text,
        metrics: focused.metrics,
        capabilities: focused.capabilities,
        anchor: script_kit_gpui::inline_agent::types::InlineAgentAnchor {
            geometry: focused.geometry,
        },
    }
}

#[test]
fn open_plan_requires_captured_snapshot_and_starts_standalone_compact() {
    let snapshot = snapshot_for_tests();
    let plan = plan_open_inline_agent_overlay(&snapshot, InlineOverlayAttachment::Standalone);

    assert_eq!(plan.attachment, InlineOverlayAttachment::Standalone);
    assert_eq!(plan.mode, InlineAgentMode::Compact);
    assert_eq!(plan.run_state, InlineAgentRunState::Idle);
    assert_eq!(plan.session_id, snapshot.session_id.to_string());
    assert_eq!(plan.app_name, "Notes");
    assert!(plan.focus_prompt);
    assert!(plan.bounds.y > 120.0);
    assert_eq!(plan.bounds.height, 118.0);
}

#[test]
fn expanded_and_compact_plans_preserve_session_and_latest_state() {
    let snapshot = snapshot_for_tests();
    let mut compact =
        plan_open_inline_agent_overlay(&snapshot, InlineOverlayAttachment::Standalone);
    compact.run_state = InlineAgentRunState::Completed {
        output: "tightened copy".to_string(),
    };

    let expanded = plan_expanded_inline_agent_overlay(&snapshot, &compact);
    assert_eq!(expanded.mode, InlineAgentMode::Expanded);
    assert_eq!(expanded.session_id, compact.session_id);
    assert_eq!(expanded.run_state, compact.run_state);
    assert_eq!(expanded.attachment, InlineOverlayAttachment::Standalone);
    assert!(expanded.bounds.width > compact.bounds.width);

    let collapsed = plan_compact_inline_agent_overlay(&snapshot, &expanded);
    assert_eq!(collapsed.mode, InlineAgentMode::Compact);
    assert_eq!(collapsed.run_state, expanded.run_state);
    assert_eq!(collapsed.bounds.height, 252.0);
}

#[test]
fn window_options_use_popup_bounds_without_manual_resize() {
    let snapshot = snapshot_for_tests();
    let plan = plan_open_inline_agent_overlay(&snapshot, InlineOverlayAttachment::Standalone);
    let options = inline_agent_window_options(&plan, None);

    assert_eq!(options.focus, plan.focus_prompt);
    assert!(options.show);
    assert!(!options.is_movable);
    assert!(!options.is_resizable);
    assert!(options.titlebar.is_none());
    assert!(options.window_bounds.is_some());
}

#[test]
fn automation_info_registers_standalone_mini_ai_surface() {
    let snapshot = snapshot_for_tests();
    let plan = plan_open_inline_agent_overlay(&snapshot, InlineOverlayAttachment::Standalone);
    let info = inline_agent_automation_info(&plan);

    assert_eq!(info.id, INLINE_AGENT_WINDOW_AUTOMATION_ID);
    assert_eq!(info.kind, AutomationWindowKind::MiniAi);
    assert_eq!(info.title.as_deref(), Some(INLINE_AGENT_WINDOW_TITLE));
    assert_eq!(
        info.semantic_surface.as_deref(),
        Some(INLINE_AGENT_SEMANTIC_SURFACE)
    );
    assert_eq!(info.parent_window_id, None);
    assert_eq!(
        info.bounds.as_ref().map(|bounds| bounds.width),
        Some(plan.bounds.width)
    );
    assert_eq!(compact_root_automation_id(), "inline-agent-compact");
}
