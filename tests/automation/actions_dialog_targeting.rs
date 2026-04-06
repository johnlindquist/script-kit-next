//! Actions dialog targeting regression tests.
//!
//! The actions dialog is an attached popup (no independent window handle).
//! These tests verify that targeting it resolves through the registry and
//! that the protocol correctly handles the attached-popup pattern.

use script_kit_gpui::protocol::{
    AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget, Message, SimulatedGpuiEvent,
};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(40_000);
fn prefix() -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("ad{n}")
}

fn cleanup(prefix: &str, ids: &[&str]) {
    for id in ids {
        script_kit_gpui::windows::remove_automation_window(&format!("{prefix}:{id}"));
    }
}

#[test]
fn actions_dialog_registered_as_popup() {
    let p = prefix();

    // Register main window
    let main = AutomationWindowInfo {
        id: format!("{p}:main"),
        kind: AutomationWindowKind::Main,
        title: Some("Script Kit".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("scriptList".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(main);

    // Register actions dialog as ActionsDialog kind
    let actions = AutomationWindowInfo {
        id: format!("{p}:actions"),
        kind: AutomationWindowKind::ActionsDialog,
        title: Some("Actions".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("actionsDialog".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(actions);

    // Resolve actions dialog by kind
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::ActionsDialog,
            index: None,
        }))
        .expect("resolve actions dialog");
    assert_eq!(resolved.kind, AutomationWindowKind::ActionsDialog);
    assert_eq!(resolved.semantic_surface.as_deref(), Some("actionsDialog"));

    // Actions dialog is distinct from main
    let main_resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Main))
            .expect("resolve main");
    assert_ne!(resolved.id, main_resolved.id);

    cleanup(&p, &["main", "actions"]);
}

#[test]
fn actions_dialog_simulate_event_targets_correctly() {
    // Verify a simulateGpuiEvent targeting ActionsDialog parses correctly
    let json = r#"{
        "type": "simulateGpuiEvent",
        "requestId": "actions-sim-1",
        "target": {"type": "kind", "kind": "actionsDialog"},
        "event": {"type": "keyDown", "key": "escape", "modifiers": []}
    }"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::SimulateGpuiEvent {
            request_id,
            target,
            event,
        } => {
            assert_eq!(request_id, "actions-sim-1");
            let target = target.expect("target should be present");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::ActionsDialog);
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
            match event {
                SimulatedGpuiEvent::KeyDown { key, .. } => {
                    assert_eq!(key, "escape");
                }
                other => panic!("Expected KeyDown, got: {:?}", other),
            }
        }
        other => panic!("Expected SimulateGpuiEvent, got: {:?}", other),
    }
}

#[test]
fn actions_dialog_close_removes_from_listing() {
    let p = prefix();

    let actions = AutomationWindowInfo {
        id: format!("{p}:actions-close"),
        kind: AutomationWindowKind::ActionsDialog,
        title: Some("Actions".into()),
        focused: false,
        visible: true,
        semantic_surface: None,
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(actions);

    // Should be in the list
    let all = script_kit_gpui::windows::list_automation_windows();
    assert!(all.iter().any(|w| w.id == format!("{p}:actions-close")));

    // Remove
    script_kit_gpui::windows::remove_automation_window(&format!("{p}:actions-close"));

    // No longer in the list
    let all = script_kit_gpui::windows::list_automation_windows();
    assert!(!all.iter().any(|w| w.id == format!("{p}:actions-close")));
}

#[test]
fn prompt_popup_kind_resolves_independently() {
    let p = prefix();

    let popup = AutomationWindowInfo {
        id: format!("{p}:popup-confirm"),
        kind: AutomationWindowKind::PromptPopup,
        title: Some("Confirm".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("confirmDialog".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(popup);

    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::PromptPopup,
            index: None,
        }))
        .expect("resolve prompt popup");
    assert_eq!(resolved.kind, AutomationWindowKind::PromptPopup);
    assert_eq!(resolved.semantic_surface.as_deref(), Some("confirmDialog"));

    cleanup(&p, &["popup-confirm"]);
}

#[test]
fn backward_compat_get_elements_without_target() {
    // Legacy getElements requests (no target field) should still parse
    let json = r#"{"type":"getElements","requestId":"legacy-1"}"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::GetElements { target, .. } => {
            assert!(target.is_none(), "target should default to None for legacy");
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

#[test]
fn backward_compat_get_acp_state_without_target() {
    let json = r#"{"type":"getAcpState","requestId":"legacy-acp-1"}"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::GetAcpState { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected GetAcpState, got: {:?}", other),
    }
}

#[test]
fn get_state_actions_dialog_target_round_trip() {
    // getState with an ActionsDialog target should parse correctly.
    let json = serde_json::json!({
        "type": "getState",
        "requestId": "gs-actions-1",
        "target": { "type": "kind", "kind": "actionsDialog" }
    });
    let msg: Message = serde_json::from_value(json).expect("parse");
    match msg {
        Message::GetState { request_id, target } => {
            assert_eq!(request_id, "gs-actions-1");
            let target = target.expect("target should be present");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::ActionsDialog);
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected GetState, got: {:?}", other),
    }
}

#[test]
fn get_layout_info_actions_dialog_target_round_trip() {
    // getLayoutInfo with an ActionsDialog target should parse correctly.
    let json = serde_json::json!({
        "type": "getLayoutInfo",
        "requestId": "li-actions-1",
        "target": { "type": "kind", "kind": "actionsDialog" }
    });
    let msg: Message = serde_json::from_value(json).expect("parse");
    match msg {
        Message::GetLayoutInfo { request_id, target } => {
            assert_eq!(request_id, "li-actions-1");
            let target = target.expect("target should be present");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::ActionsDialog);
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected GetLayoutInfo, got: {:?}", other),
    }
}

#[test]
fn get_elements_with_actions_dialog_target_round_trip() {
    // getElements with an ActionsDialog target should parse correctly.
    let json = serde_json::json!({
        "type": "getElements",
        "requestId": "ge-actions-1",
        "limit": 20,
        "target": { "type": "kind", "kind": "actionsDialog" }
    });
    let msg: Message = serde_json::from_value(json).expect("parse");
    match msg {
        Message::GetElements {
            request_id,
            limit,
            target,
        } => {
            assert_eq!(request_id, "ge-actions-1");
            assert_eq!(limit, Some(20));
            let target = target.expect("target should be present");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::ActionsDialog);
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

#[test]
fn capture_screenshot_with_actions_dialog_target_round_trip() {
    // captureScreenshot with an ActionsDialog target should parse correctly.
    let json = serde_json::json!({
        "type": "captureScreenshot",
        "requestId": "ss-actions-1",
        "target": { "type": "kind", "kind": "actionsDialog" }
    });
    let msg: Message = serde_json::from_value(json).expect("parse");
    match msg {
        Message::CaptureScreenshot {
            request_id, target, ..
        } => {
            assert_eq!(request_id, "ss-actions-1");
            let target = target.expect("target should be present");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::ActionsDialog);
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected CaptureScreenshot, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Actions dialog semantic element contract tests
// ---------------------------------------------------------------------------

#[test]
fn actions_dialog_collector_has_search_input_element() {
    // Verify the collector source defines the search input semantic ID
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"input:actions-search\""),
        "Actions dialog collector must define input:actions-search element"
    );
}

#[test]
fn actions_dialog_collector_has_list_element() {
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"list:actions\""),
        "Actions dialog collector must define list:actions element"
    );
}

#[test]
fn actions_dialog_collector_emits_choice_elements() {
    // The collector must use choice:N:id format for individual action rows
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("ElementType::Choice"),
        "Actions dialog collector must emit Choice-typed elements for actions"
    );
}

#[test]
fn actions_dialog_fallback_preserves_panel_only_warning() {
    // When entity is unavailable, must still emit the panel_only warning
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"panel_only_actions_dialog\""),
        "Actions dialog must preserve panel_only_actions_dialog fallback warning"
    );
}

#[test]
fn prompt_popup_collector_tries_known_popup_types() {
    // The PromptPopup collector must try mention, model selector, and confirm
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("collect_mention_picker_snapshot"),
        "PromptPopup collector must try mention picker"
    );
    assert!(
        source.contains("collect_model_selector_snapshot"),
        "PromptPopup collector must try model selector"
    );
    assert!(
        source.contains("collect_confirm_popup_snapshot"),
        "PromptPopup collector must try confirm popup"
    );
}

#[test]
fn prompt_popup_fallback_preserves_panel_only_warning() {
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"panel_only_prompt_popup\""),
        "PromptPopup must preserve panel_only_prompt_popup fallback warning"
    );
}

#[test]
fn confirm_popup_collector_has_button_elements() {
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"button:0:confirm\""),
        "Confirm popup must define button:0:confirm"
    );
    assert!(
        source.contains("\"button:1:cancel\""),
        "Confirm popup must define button:1:cancel"
    );
}

#[test]
fn mention_picker_collector_uses_item_id_in_semantic_ids() {
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    // Mention picker uses item.id for semantic IDs
    assert!(
        source.contains("format!(\"choice:{}:{}\", idx, item.id)"),
        "Mention picker must use item.id in choice semantic IDs"
    );
}

#[test]
fn model_selector_collector_uses_entry_id_in_semantic_ids() {
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("format!(\"choice:{}:{}\", idx, entry.id)"),
        "Model selector must use entry.id in choice semantic IDs"
    );
}
