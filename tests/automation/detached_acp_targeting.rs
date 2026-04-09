//! Detached ACP window targeting regression tests.
//!
//! Proves that the automation registry can track one or more detached
//! ACP chat windows, resolve them by kind + index, and distinguish them
//! from the main window for screenshot and element targeting.

use script_kit_gpui::protocol::{
    AcpResolvedTarget, AcpSetupActionKind, AcpStateSnapshot, AcpTestProbeSnapshot,
    AutomationInspectSnapshot, AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
    Message, SemanticQuality, SimulatedGpuiEvent, AUTOMATION_INSPECT_SCHEMA_VERSION,
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
        parent_window_id: None,
        parent_kind: None,
    };
    script_kit_gpui::windows::upsert_automation_window(main);

    // Register detached ACP
    let acp = AutomationWindowInfo {
        id: format!("{p}:acp-thread-1"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("Script Kit ACP".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
        parent_window_id: None,
        parent_kind: None,
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
    assert_eq!(resolved.title.as_deref(), Some("Script Kit ACP"));

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
        parent_window_id: None,
        parent_kind: None,
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
        parent_window_id: None,
        parent_kind: None,
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
        parent_window_id: None,
        parent_kind: None,
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
        parent_window_id: None,
        parent_kind: None,
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

// ============================================================
// Protocol-level targeted ACP regression tests
// ============================================================

#[test]
fn reset_acp_test_probe_targeted_round_trip() {
    // Verify resetAcpTestProbe with a target field serializes and deserializes correctly.
    let msg = Message::reset_acp_test_probe_targeted(
        "probe-reset-det-1".into(),
        AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(0),
        },
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    let back: Message = serde_json::from_str(&json).expect("deserialize");
    match back {
        Message::ResetAcpTestProbe { request_id, target } => {
            assert_eq!(request_id, "probe-reset-det-1");
            let target = target.expect("target must be present");
            match target {
                AutomationWindowTarget::Kind { kind, index } => {
                    assert_eq!(kind, AutomationWindowKind::AcpDetached);
                    assert_eq!(index, Some(0));
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected ResetAcpTestProbe, got: {:?}", other),
    }
}

#[test]
fn reset_acp_test_probe_without_target_backward_compatible() {
    // Legacy global reset (no target) must still parse.
    let json = r#"{"type":"resetAcpTestProbe","requestId":"probe-reset-legacy"}"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::ResetAcpTestProbe { request_id, target } => {
            assert_eq!(request_id, "probe-reset-legacy");
            assert!(target.is_none(), "legacy reset must have no target");
        }
        other => panic!("Expected ResetAcpTestProbe, got: {:?}", other),
    }
}

#[test]
fn acp_test_probe_result_carries_resolved_target() {
    // Build a probe result with resolvedTarget and verify it round-trips.
    let mut probe = AcpTestProbeSnapshot::default();
    probe.state.resolved_target = Some(AcpResolvedTarget {
        window_id: "acpDetached:thread-1".to_string(),
        window_kind: "acpDetached".to_string(),
        title: Some("Script Kit ACP".to_string()),
    });
    let msg = Message::acp_test_probe_result("probe-read-det-1".into(), probe);
    let json = serde_json::to_value(&msg).expect("serialize");

    // Verify resolvedTarget is nested inside state
    let state = json.get("state").expect("state field");
    let rt = state
        .get("resolvedTarget")
        .expect("resolvedTarget in state");
    assert_eq!(rt["windowKind"], "acpDetached");
    assert_eq!(rt["windowId"], "acpDetached:thread-1");
    assert_eq!(rt["title"], "Script Kit ACP");

    // Round-trip
    let back: Message = serde_json::from_value(json).expect("deserialize");
    match back {
        Message::AcpTestProbeResult { request_id, probe } => {
            assert_eq!(request_id, "probe-read-det-1");
            let rt = probe.state.resolved_target.expect("resolvedTarget");
            assert_eq!(rt.window_kind, "acpDetached");
        }
        other => panic!("Expected AcpTestProbeResult, got: {:?}", other),
    }
}

#[test]
fn acp_state_result_carries_resolved_target() {
    // Build an ACP state result with resolvedTarget and verify round-trip.
    let mut state = AcpStateSnapshot::default();
    state.resolved_target = Some(AcpResolvedTarget {
        window_id: "acpDetached:thread-1".to_string(),
        window_kind: "acpDetached".to_string(),
        title: Some("Script Kit ACP".to_string()),
    });
    let msg = Message::acp_state_result("acp-state-det-1".into(), state);
    let json = serde_json::to_value(&msg).expect("serialize");

    // resolvedTarget is flattened into the top-level AcpStateResult
    let rt = json.get("resolvedTarget").expect("resolvedTarget");
    assert_eq!(rt["windowKind"], "acpDetached");

    let back: Message = serde_json::from_value(json).expect("deserialize");
    match back {
        Message::AcpStateResult { request_id, state } => {
            assert_eq!(request_id, "acp-state-det-1");
            let rt = state.resolved_target.expect("resolvedTarget");
            assert_eq!(rt.window_kind, "acpDetached");
            assert_eq!(rt.window_id, "acpDetached:thread-1");
        }
        other => panic!("Expected AcpStateResult, got: {:?}", other),
    }
}

#[test]
fn perform_acp_setup_action_targeted_round_trip() {
    // Verify performAcpSetupAction with detached target parses correctly.
    let json = serde_json::json!({
        "type": "performAcpSetupAction",
        "requestId": "setup-det-1",
        "action": "openAgentPicker",
        "target": {
            "type": "kind",
            "kind": "acpDetached",
            "index": 0
        }
    });
    let msg: Message = serde_json::from_value(json).expect("parse");
    match msg {
        Message::PerformAcpSetupAction {
            request_id,
            action,
            agent_id,
            target,
        } => {
            assert_eq!(request_id, "setup-det-1");
            assert_eq!(action, AcpSetupActionKind::OpenAgentPicker);
            assert!(agent_id.is_none());
            let target = target.expect("target must be present");
            match target {
                AutomationWindowTarget::Kind { kind, index } => {
                    assert_eq!(kind, AutomationWindowKind::AcpDetached);
                    assert_eq!(index, Some(0));
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected PerformAcpSetupAction, got: {:?}", other),
    }
}

#[test]
fn acp_setup_action_result_with_resolved_target_round_trips() {
    // Verify acpSetupActionResult carries resolvedTarget through state.
    let mut state = AcpStateSnapshot::default();
    state.status = "setup".to_string();
    state.resolved_target = Some(AcpResolvedTarget {
        window_id: "acpDetached:thread-1".to_string(),
        window_kind: "acpDetached".to_string(),
        title: Some("Script Kit ACP".to_string()),
    });
    let msg = Message::AcpSetupActionResult {
        request_id: "setup-det-result".into(),
        success: true,
        error: None,
        state: Some(state),
    };
    let json = serde_json::to_value(&msg).expect("serialize");
    assert_eq!(json["success"], true);

    let state_json = json.get("state").expect("state");
    let rt = state_json.get("resolvedTarget").expect("resolvedTarget");
    assert_eq!(rt["windowKind"], "acpDetached");

    let back: Message = serde_json::from_value(json).expect("deserialize");
    match back {
        Message::AcpSetupActionResult {
            request_id,
            success,
            state,
            ..
        } => {
            assert_eq!(request_id, "setup-det-result");
            assert!(success);
            let state = state.expect("state");
            let rt = state.resolved_target.expect("resolvedTarget");
            assert_eq!(rt.window_kind, "acpDetached");
        }
        other => panic!("Expected AcpSetupActionResult, got: {:?}", other),
    }
}

#[test]
fn get_acp_state_targeted_from_json() {
    // Verify getAcpState with detached target parses.
    let json = r#"{"type":"getAcpState","requestId":"acp-s-1","target":{"type":"kind","kind":"acpDetached","index":0}}"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::GetAcpState { request_id, target } => {
            assert_eq!(request_id, "acp-s-1");
            let target = target.expect("target");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::AcpDetached);
                }
                other => panic!("Expected Kind, got: {:?}", other),
            }
        }
        other => panic!("Expected GetAcpState, got: {:?}", other),
    }
}

#[test]
// ============================================================
// Cross-window transaction contract: ACP not rejected as main-only
// ============================================================

fn get_elements_with_acp_detached_target_parses() {
    let json = serde_json::json!({
        "type": "getElements",
        "requestId": "elm-acp-det",
        "target": {"type": "kind", "kind": "acpDetached"},
        "limit": 20
    });
    let msg: Message =
        serde_json::from_value(json).expect("getElements with acpDetached target should parse");
    match msg {
        Message::GetElements {
            request_id,
            target,
            limit,
        } => {
            assert_eq!(request_id, "elm-acp-det");
            assert!(target.is_some());
            assert_eq!(limit, Some(20));
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

#[test]
fn acp_detached_get_elements_not_rejected_as_main_only() {
    // Verify that getElements routes ACP targets through surface collector,
    // not the main-only reject path.
    let source = include_str!("../../src/prompt_handler/mod.rs");
    // getElements must use resolve_automation_window (not resolve_main_only_target)
    assert!(
        source.contains("collect_surface_snapshot"),
        "getElements must delegate non-main targets to surface collector"
    );
    // getElements parse test
    get_elements_with_acp_detached_target_parses();
}

#[test]
fn acp_detached_batch_not_rejected_as_main_only() {
    let source = include_str!("../../src/prompt_handler/mod.rs");
    assert!(
        source.contains("resolve_automation_read_target(&rid, \"batch\""),
        "batch handler must use resolve_automation_read_target (accepts AcpDetached)"
    );
    assert!(
        !source.contains("batch currently supports only the main automation window"),
        "batch must not reject AcpDetached with main-only error"
    );
}

#[test]
fn acp_detached_wait_for_not_rejected_as_main_only() {
    let source = include_str!("../../src/prompt_handler/mod.rs");
    assert!(
        source.contains("resolve_acp_read_target(&request_id, \"waitFor\"")
            || source.contains("resolve_acp_read_target(&rid, \"waitFor\""),
        "waitFor handler must use resolve_acp_read_target (accepts AcpDetached)"
    );
    assert!(
        !source.contains(
            "waitFor currently supports only the main automation window; resolved acpDetached"
        ),
        "waitFor must not reject AcpDetached with main-only error"
    );
}

#[test]
fn acp_detached_transaction_provider_builds_snapshot() {
    let source = include_str!("../../src/windows/automation_transaction_provider.rs");
    assert!(
        source.contains("fn snapshot(&self) -> UiStateSnapshot"),
        "provider must build UiStateSnapshot from detached ACP state"
    );
    assert!(
        source.contains("collect_acp_state_snapshot"),
        "provider snapshot must use live ACP state"
    );
    assert!(
        source.contains("collect_acp_detached_elements"),
        "provider snapshot must include semantic IDs from shared collector"
    );
}

#[test]
fn get_acp_test_probe_targeted_from_json() {
    // Verify getAcpTestProbe with detached target parses.
    let json = r#"{"type":"getAcpTestProbe","requestId":"acp-p-1","tail":16,"target":{"type":"kind","kind":"acpDetached","index":0}}"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::GetAcpTestProbe {
            request_id,
            tail,
            target,
        } => {
            assert_eq!(request_id, "acp-p-1");
            assert_eq!(tail, Some(16));
            let target = target.expect("target");
            match target {
                AutomationWindowTarget::Kind { kind, .. } => {
                    assert_eq!(kind, AutomationWindowKind::AcpDetached);
                }
                other => panic!("Expected Kind, got: {:?}", other),
            }
        }
        other => panic!("Expected GetAcpTestProbe, got: {:?}", other),
    }
}

// ============================================================
// Non-main semantic proof: AcpDetached inspect receipts
// ============================================================

#[test]
fn acp_detached_collector_never_returns_non_main_pending_warning() {
    // The collector source must NOT emit semantic_elements_non_main_pending for AcpDetached.
    // Instead it must fall back to panel_only_acp_detached.
    let source = include_str!("../../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"panel_only_acp_detached\""),
        "AcpDetached must use panel_only_acp_detached fallback, not semantic_elements_non_main_pending"
    );
    // The match arm for AcpDetached must use unwrap_or_else, not ?.
    assert!(
        !source.contains("collect_acp_detached_snapshot(resolved, cx)?"),
        "AcpDetached collector must not use ? (which returns None → triggers non_main_pending)"
    );
}

#[test]
fn acp_detached_inspect_receipt_has_semantic_quality_field() {
    // The handler must set semantic_quality on all inspect receipts
    let source = include_str!("../../src/prompt_handler/mod.rs");
    assert!(
        source.contains("semantic_quality: Some(semantic_quality)"),
        "inspect handler must set semantic_quality on the snapshot"
    );
}

#[test]
fn inspect_receipt_schema_version_is_v3() {
    assert_eq!(
        AUTOMATION_INSPECT_SCHEMA_VERSION, 3,
        "schema version must be 3 after adding semantic_quality"
    );
}

#[test]
fn semantic_quality_serde_contract() {
    // Full
    let json = serde_json::to_string(&SemanticQuality::Full).expect("serialize");
    assert_eq!(json, "\"full\"");
    // PanelOnly
    let json = serde_json::to_string(&SemanticQuality::PanelOnly).expect("serialize");
    assert_eq!(json, "\"panel_only\"");
    // Unavailable
    let json = serde_json::to_string(&SemanticQuality::Unavailable).expect("serialize");
    assert_eq!(json, "\"unavailable\"");
}

#[test]
fn acp_detached_inspect_result_carries_semantic_quality() {
    // Build an inspect result with semantic_quality and verify it round-trips.
    let snapshot = AutomationInspectSnapshot {
        schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
        window_id: "acpDetached:thread-1".into(),
        window_kind: "AcpDetached".into(),
        title: Some("Script Kit ACP".into()),
        resolved_bounds: None,
        target_bounds_in_screenshot: None,
        surface_hit_point: None,
        suggested_hit_points: Vec::new(),
        elements: Vec::new(),
        total_count: 0,
        focused_semantic_id: None,
        selected_semantic_id: None,
        screenshot_width: Some(1440),
        screenshot_height: Some(900),
        pixel_probes: Vec::new(),
        os_window_id: Some(99),
        semantic_quality: Some(SemanticQuality::PanelOnly),
        warnings: vec!["panel_only_acp_detached".into()],
    };
    let msg = Message::automation_inspect_result("inspect-acp-1".into(), snapshot);
    let json = serde_json::to_value(&msg).expect("serialize");

    // Snapshot fields are flattened into top-level JSON (serde flatten)
    assert_eq!(json["semanticQuality"], "panel_only");
    assert_eq!(json["schemaVersion"], 3);

    // Round-trip
    let back: Message = serde_json::from_value(json).expect("deserialize");
    match back {
        Message::AutomationInspectResult { snapshot, .. } => {
            assert_eq!(snapshot.semantic_quality, Some(SemanticQuality::PanelOnly));
        }
        other => panic!("Expected AutomationInspectResult, got: {:?}", other),
    }
}

#[test]
fn acp_detached_full_quality_inspect_receipt() {
    // Full quality when entity is available
    let snapshot = AutomationInspectSnapshot {
        schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
        window_id: "acpDetached:thread-2".into(),
        window_kind: "AcpDetached".into(),
        title: Some("Script Kit ACP".into()),
        resolved_bounds: None,
        target_bounds_in_screenshot: None,
        surface_hit_point: None,
        suggested_hit_points: Vec::new(),
        elements: vec![script_kit_gpui::protocol::ElementInfo {
            semantic_id: "input:acp-composer".into(),
            element_type: script_kit_gpui::protocol::ElementType::Input,
            text: None,
            value: Some("hello".into()),
            selected: None,
            focused: Some(true),
            index: None,
        }],
        total_count: 1,
        focused_semantic_id: Some("input:acp-composer".into()),
        selected_semantic_id: None,
        screenshot_width: Some(800),
        screenshot_height: Some(600),
        pixel_probes: Vec::new(),
        os_window_id: None,
        semantic_quality: Some(SemanticQuality::Full),
        warnings: Vec::new(),
    };
    let json = serde_json::to_value(&snapshot).expect("serialize");
    assert_eq!(json["semanticQuality"], "full");
    assert!(json.get("warnings").is_none()); // empty vec skipped
    assert_eq!(json["elements"].as_array().expect("elements").len(), 1);
}
