//! Integration tests for targeted ACP state and test-probe reads.
//!
//! Verifies that:
//! - `getAcpState` and `getAcpTestProbe` with an `AcpDetached` target resolve
//!   through the automation registry (registry-level, not runtime-level, since
//!   we cannot open real GPUI windows in unit tests).
//! - Non-ACP secondary targets produce structured `target_unsupported` warnings.
//! - Protocol messages round-trip correctly with target fields.
//! - Main-window (no target) messages still parse and serialize correctly.

use script_kit_gpui::protocol::{
    AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(50_000);
fn prefix() -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("acpread{n}")
}

fn cleanup(prefix: &str, ids: &[&str]) {
    for id in ids {
        script_kit_gpui::windows::remove_automation_window(&format!("{prefix}:{id}"));
    }
}

// ── Registry-level targeting ────────────────────────────────────────

#[test]
fn acp_detached_target_resolves_to_correct_kind() {
    let p = prefix();

    script_kit_gpui::windows::upsert_automation_window(AutomationWindowInfo {
        id: format!("{p}:main"),
        kind: AutomationWindowKind::Main,
        title: Some("Script Kit".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("scriptList".into()),
        bounds: None,
        parent_window_id: None,
        parent_kind: None,
    });
    script_kit_gpui::windows::upsert_automation_window(AutomationWindowInfo {
        id: format!("{p}:acp-1"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("Script Kit ACP".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
        parent_window_id: None,
        parent_kind: None,
    });

    // Target by ID for deterministic resolution (avoids global registry index collisions)
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:acp-1"),
        }))
        .expect("should resolve AcpDetached target by ID");

    assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);
    assert_eq!(resolved.id, format!("{p}:acp-1"));

    let main =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:main"),
        }))
        .expect("should resolve main by ID");
    assert_eq!(main.kind, AutomationWindowKind::Main);

    cleanup(&p, &["main", "acp-1"]);
}

#[test]
fn non_acp_secondary_target_does_not_resolve_as_acp() {
    let p = prefix();

    script_kit_gpui::windows::upsert_automation_window(AutomationWindowInfo {
        id: format!("{p}:notes"),
        kind: AutomationWindowKind::Notes,
        title: Some("Script Kit Notes".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("notes".into()),
        bounds: None,
        parent_window_id: None,
        parent_kind: None,
    });

    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::Notes,
            index: None,
        }))
        .expect("should resolve notes");
    assert_eq!(resolved.kind, AutomationWindowKind::Notes);
    assert_ne!(resolved.kind, AutomationWindowKind::AcpDetached);

    cleanup(&p, &["notes"]);
}

// ── Protocol message round-trip ─────────────────────────────────────

#[test]
fn get_acp_state_with_detached_target_round_trips() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "getAcpState",
        "requestId": "acp-state-detached-1",
        "target": {
            "type": "kind",
            "kind": "acpDetached",
            "index": 0
        }
    });

    let msg: Message = serde_json::from_value(json.clone())
        .expect("should parse getAcpState with AcpDetached target");

    // Round-trip: serialize back and compare key fields
    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("getAcpState"));
    assert_eq!(
        re_serialized["requestId"].as_str(),
        Some("acp-state-detached-1")
    );
    assert_eq!(
        re_serialized["target"]["kind"].as_str(),
        Some("acpDetached")
    );
}

#[test]
fn get_acp_test_probe_with_detached_target_round_trips() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "getAcpTestProbe",
        "requestId": "acp-probe-detached-1",
        "tail": 10,
        "target": {
            "type": "kind",
            "kind": "acpDetached",
            "index": 0
        }
    });

    let msg: Message =
        serde_json::from_value(json).expect("should parse getAcpTestProbe with AcpDetached target");

    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("getAcpTestProbe"));
    assert_eq!(
        re_serialized["requestId"].as_str(),
        Some("acp-probe-detached-1")
    );
    assert_eq!(
        re_serialized["target"]["kind"].as_str(),
        Some("acpDetached")
    );
    assert_eq!(re_serialized["tail"].as_u64(), Some(10));
}

#[test]
fn get_acp_state_without_target_round_trips() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "getAcpState",
        "requestId": "acp-state-main-1"
    });

    let msg: Message =
        serde_json::from_value(json).expect("should parse getAcpState without target");

    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("getAcpState"));
    assert_eq!(
        re_serialized["requestId"].as_str(),
        Some("acp-state-main-1")
    );
}

#[test]
fn get_acp_test_probe_without_target_round_trips() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "getAcpTestProbe",
        "requestId": "acp-probe-main-1"
    });

    let msg: Message =
        serde_json::from_value(json).expect("should parse getAcpTestProbe without target");

    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("getAcpTestProbe"));
    assert_eq!(
        re_serialized["requestId"].as_str(),
        Some("acp-probe-main-1")
    );
}

// ── ACP state/probe result with warnings round-trip ──────────────────

#[test]
fn acp_state_result_with_target_unsupported_warning_round_trips() {
    use script_kit_gpui::protocol::{AcpStateSnapshot, Message};

    let mut state = AcpStateSnapshot::default();
    state.warnings = vec![
        "target_unsupported: getAcpState supports only Main and AcpDetached targets; resolved notes:1 (Notes)".to_string(),
    ];

    let response = Message::acp_state_result("acp-state-notes-1".to_string(), state);
    let json = serde_json::to_value(&response).expect("serialize");

    assert_eq!(json["type"].as_str(), Some("acpStateResult"));
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0]
        .as_str()
        .expect("warning str")
        .contains("target_unsupported"));
}

#[test]
fn acp_test_probe_result_with_target_unsupported_warning_round_trips() {
    use script_kit_gpui::protocol::{AcpTestProbeSnapshot, Message};

    let mut probe = AcpTestProbeSnapshot::default();
    probe.warnings = vec![
        "target_unsupported: getAcpTestProbe supports only Main and AcpDetached targets; resolved notes:1 (Notes)".to_string(),
    ];

    let response = Message::acp_test_probe_result("acp-probe-notes-1".to_string(), probe);
    let json = serde_json::to_value(&response).expect("serialize");

    assert_eq!(json["type"].as_str(), Some("acpTestProbeResult"));
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0]
        .as_str()
        .expect("warning str")
        .contains("target_unsupported"));
}

// ── Target-by-ID for detached ACP ───────────────────────────────────

#[test]
fn acp_detached_target_by_id_resolves_correctly() {
    let p = prefix();

    script_kit_gpui::windows::upsert_automation_window(AutomationWindowInfo {
        id: format!("{p}:acp-thread-42"),
        kind: AutomationWindowKind::AcpDetached,
        title: Some("Script Kit ACP".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("acpChat".into()),
        bounds: None,
        parent_window_id: None,
        parent_kind: None,
    });

    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:acp-thread-42"),
        }))
        .expect("should resolve by ID");

    assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);
    assert_eq!(resolved.id, format!("{p}:acp-thread-42"));

    cleanup(&p, &["acp-thread-42"]);
}

// ── Multiple ACP windows with index targeting ────────────────────────

#[test]
fn multiple_acp_detached_windows_indexed_targeting() {
    let p = prefix();

    for i in 0..3 {
        script_kit_gpui::windows::upsert_automation_window(AutomationWindowInfo {
            id: format!("{p}:acp-{i}"),
            kind: AutomationWindowKind::AcpDetached,
            title: Some(format!("ACP Thread {i}")),
            focused: i == 1,
            visible: true,
            semantic_surface: Some("acpChat".into()),
            bounds: None,
            parent_window_id: None,
            parent_kind: None,
        });
    }

    let r0 =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(0),
        }))
        .expect("index 0");
    assert_eq!(r0.kind, AutomationWindowKind::AcpDetached);

    let r2 =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(2),
        }))
        .expect("index 2");
    assert_eq!(r2.kind, AutomationWindowKind::AcpDetached);
    assert_ne!(r0.id, r2.id);

    let err =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(99),
        }));
    assert!(err.is_err());

    cleanup(&p, &["acp-0", "acp-1", "acp-2"]);
}

// ── resetAcpTestProbe with target ──────────────────────────────────

#[test]
fn reset_acp_test_probe_with_detached_target_round_trips() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "resetAcpTestProbe",
        "requestId": "probe-reset-detached-1",
        "target": {
            "type": "kind",
            "kind": "acpDetached",
            "index": 0
        }
    });

    let msg: Message = serde_json::from_value(json)
        .expect("should parse resetAcpTestProbe with AcpDetached target");

    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("resetAcpTestProbe"));
    assert_eq!(
        re_serialized["requestId"].as_str(),
        Some("probe-reset-detached-1")
    );
    assert_eq!(
        re_serialized["target"]["kind"].as_str(),
        Some("acpDetached")
    );
}

#[test]
fn reset_acp_test_probe_without_target_backward_compatible() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "resetAcpTestProbe",
        "requestId": "probe-reset-legacy"
    });

    let msg: Message =
        serde_json::from_value(json).expect("should parse legacy resetAcpTestProbe without target");

    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("resetAcpTestProbe"));
    assert_eq!(
        re_serialized["requestId"].as_str(),
        Some("probe-reset-legacy")
    );
    // target should not be present in serialized output
    assert!(
        re_serialized.get("target").is_none() || re_serialized["target"].is_null(),
        "target should be omitted for legacy requests"
    );
}

// ── AcpResolvedTarget in state responses ──────────────────────────────

#[test]
fn acp_state_result_with_resolved_target_round_trips() {
    use script_kit_gpui::protocol::{AcpResolvedTarget, AcpStateSnapshot, Message};

    let mut state = AcpStateSnapshot::default();
    state.resolved_target = Some(AcpResolvedTarget {
        window_id: "acpDetached:thread-1".to_string(),
        window_kind: "acpDetached".to_string(),
        title: Some("Script Kit ACP".to_string()),
    });

    let response = Message::acp_state_result("acp-state-resolved-1".to_string(), state);
    let json = serde_json::to_value(&response).expect("serialize");

    assert_eq!(json["type"].as_str(), Some("acpStateResult"));
    assert_eq!(
        json["resolvedTarget"]["windowId"].as_str(),
        Some("acpDetached:thread-1")
    );
    assert_eq!(
        json["resolvedTarget"]["windowKind"].as_str(),
        Some("acpDetached")
    );
    assert_eq!(
        json["resolvedTarget"]["title"].as_str(),
        Some("Script Kit ACP")
    );

    // Schema version should be 2 with resolved_target support
    assert_eq!(
        json["schemaVersion"].as_u64(),
        Some(script_kit_gpui::protocol::ACP_STATE_SCHEMA_VERSION as u64)
    );
}

#[test]
fn acp_state_result_without_resolved_target_omits_field() {
    use script_kit_gpui::protocol::{AcpStateSnapshot, Message};

    let state = AcpStateSnapshot::default();
    let response = Message::acp_state_result("acp-state-main-compat".to_string(), state);
    let json = serde_json::to_value(&response).expect("serialize");

    assert!(
        json.get("resolvedTarget").is_none() || json["resolvedTarget"].is_null(),
        "resolvedTarget should be absent when None"
    );
}

// ── waitFor with ACP detached target round-trips ─────────────────────

#[test]
fn wait_for_with_acp_detached_target_round_trips() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "waitFor",
        "requestId": "acp-wait-detached-1",
        "condition": {
            "type": "acpAcceptedViaKey",
            "key": "enter"
        },
        "target": {
            "type": "kind",
            "kind": "acpDetached",
            "index": 0
        },
        "timeout": 3000,
        "pollInterval": 25,
        "trace": "onFailure"
    });

    let msg: Message =
        serde_json::from_value(json).expect("should parse waitFor with AcpDetached target");

    let re_serialized = serde_json::to_value(&msg).expect("re-serialize");
    assert_eq!(re_serialized["type"].as_str(), Some("waitFor"));
    assert_eq!(
        re_serialized["target"]["kind"].as_str(),
        Some("acpDetached")
    );
    assert_eq!(re_serialized["timeout"].as_u64(), Some(3000));
}
