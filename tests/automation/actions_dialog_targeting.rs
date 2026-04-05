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
