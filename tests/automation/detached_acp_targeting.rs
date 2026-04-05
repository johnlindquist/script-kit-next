//! Detached ACP window targeting regression tests.
//!
//! Proves that the automation registry can track one or more detached
//! ACP chat windows, resolve them by kind + index, and distinguish them
//! from the main window for screenshot and element targeting.

use script_kit_gpui::protocol::{
    AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget, Message, SimulatedGpuiEvent,
};
use script_kit_gpui::stdin_commands::KeyModifier;
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(30_000);
fn prefix() -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("acp{n}")
}

fn cleanup(prefix: &str, ids: &[&str]) {
    for id in ids {
        script_kit_gpui::windows::remove_automation_window(&format!("{prefix}:{id}"));
    }
}

#[test]
fn detached_acp_targeting_flow() {
    let p = prefix();

    // Register main
    let main = AutomationWindowInfo {
        id: format!("{p}:main"),
        kind: AutomationWindowKind::Main,
        title: Some("Script Kit".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("scriptList".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(main);

    // Register detached ACP
    let acp = AutomationWindowInfo {
        id: format!("{p}:acp-thread-1"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("Script Kit AI".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(acp);

    // Target by kind → ACP
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(0),
        }))
        .expect("should resolve ACP");
    assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);
    assert_eq!(resolved.semantic_surface.as_deref(), Some("acpChat"));
    assert_eq!(resolved.title.as_deref(), Some("Script Kit AI"));

    // Target by ID → ACP
    let resolved_id =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:acp-thread-1"),
        }))
        .expect("resolve by ID");
    assert_eq!(resolved_id.kind, AutomationWindowKind::AcpDetached);

    // Screenshot routing: ACP title differs from main
    assert_ne!(
        resolved.title.as_deref(),
        Some("Script Kit"),
        "must not screenshot the main window"
    );

    // Focused → ACP (since ACP has focused=true)
    let focused =
        script_kit_gpui::windows::resolve_automation_window(None).expect("resolve focused");
    assert_eq!(focused.kind, AutomationWindowKind::AcpDetached);

    cleanup(&p, &["main", "acp-thread-1"]);
}

#[test]
fn multiple_detached_acp_windows_indexed() {
    let p = prefix();

    let acp0 = AutomationWindowInfo {
        id: format!("{p}:acp-0"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("ACP Thread 0".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(acp0);

    let acp1 = AutomationWindowInfo {
        id: format!("{p}:acp-1"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("ACP Thread 1".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(acp1);

    // Index 0 → first registered
    let first =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(0),
        }))
        .expect("resolve index 0");

    // Index 1 → second registered
    let second =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(1),
        }))
        .expect("resolve index 1");

    assert_ne!(first.id, second.id, "index 0 and 1 must differ");

    // Index 2 → error
    let err =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(2),
        }));
    assert!(err.is_err());

    cleanup(&p, &["acp-0", "acp-1"]);
}

#[test]
fn acp_simulate_gpui_event_request_round_trip() {
    // Verify a full ACP-targeted simulateGpuiEvent request parses and
    // serializes with all fields intact.
    let msg = Message::simulate_gpui_event(
        "acp-sim-1".into(),
        SimulatedGpuiEvent::KeyDown {
            key: "k".into(),
            modifiers: vec![KeyModifier::Cmd],
            text: None,
        },
        Some(AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(0),
        }),
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    let back: Message = serde_json::from_str(&json).expect("deserialize");
    match back {
        Message::SimulateGpuiEvent {
            request_id,
            target,
            event,
        } => {
            assert_eq!(request_id, "acp-sim-1");
            assert!(target.is_some());
            match event {
                SimulatedGpuiEvent::KeyDown { key, modifiers, .. } => {
                    assert_eq!(key, "k");
                    assert!(modifiers.contains(&KeyModifier::Cmd));
                }
                other => panic!("Expected KeyDown, got: {:?}", other),
            }
        }
        other => panic!("Expected SimulateGpuiEvent, got: {:?}", other),
    }
}

#[test]
fn acp_window_close_removes_from_registry() {
    let p = prefix();

    let acp = AutomationWindowInfo {
        id: format!("{p}:acp-close"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("Closing ACP".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(acp);

    // Verify it resolves
    assert!(script_kit_gpui::windows::resolve_automation_window(Some(
        &AutomationWindowTarget::Id {
            id: format!("{p}:acp-close"),
        },
    ))
    .is_ok());

    // Close
    script_kit_gpui::windows::remove_automation_window(&format!("{p}:acp-close"));

    // No longer resolvable
    assert!(script_kit_gpui::windows::resolve_automation_window(Some(
        &AutomationWindowTarget::Id {
            id: format!("{p}:acp-close"),
        },
    ))
    .is_err());
}

#[test]
fn acp_visibility_toggle() {
    let p = prefix();

    let acp = AutomationWindowInfo {
        id: format!("{p}:acp-vis"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("Visibility ACP".into()),
        focused: false,
        visible: true,
        semantic_surface: None,
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(acp);

    // Hide
    script_kit_gpui::windows::set_automation_visibility(&format!("{p}:acp-vis"), false);
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:acp-vis"),
        }))
        .expect("resolve");
    assert!(!resolved.visible);

    // Show
    script_kit_gpui::windows::set_automation_visibility(&format!("{p}:acp-vis"), true);
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:acp-vis"),
        }))
        .expect("resolve");
    assert!(resolved.visible);

    cleanup(&p, &["acp-vis"]);
}
