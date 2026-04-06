use super::*;

// ============================================================
// AutomationWindowTarget serde round-trips
// ============================================================

#[test]
fn automation_window_target_round_trip_main() {
    let target = AutomationWindowTarget::Main;
    let json = serde_json::to_string(&target).expect("serialize");
    let back: AutomationWindowTarget = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, target);
    assert!(json.contains(r#""type":"main"#));
}

#[test]
fn automation_window_target_round_trip_focused() {
    let target = AutomationWindowTarget::Focused;
    let json = serde_json::to_string(&target).expect("serialize");
    let back: AutomationWindowTarget = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, target);
}

#[test]
fn automation_window_target_round_trip_id() {
    let target = AutomationWindowTarget::Id {
        id: "notes:primary".into(),
    };
    let json = serde_json::to_string(&target).expect("serialize");
    let back: AutomationWindowTarget = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, target);
}

#[test]
fn automation_window_target_round_trip_kind() {
    let json = r#"{"type":"kind","kind":"acpDetached","index":0}"#;
    let parsed: AutomationWindowTarget =
        serde_json::from_str(json).expect("target should deserialize");
    match &parsed {
        AutomationWindowTarget::Kind { kind, index } => {
            assert_eq!(*kind, AutomationWindowKind::AcpDetached);
            assert_eq!(*index, Some(0));
        }
        other => panic!("Expected Kind, got: {:?}", other),
    }
    let out = serde_json::to_string(&parsed).expect("target should serialize");
    let back: AutomationWindowTarget = serde_json::from_str(&out).expect("roundtrip");
    assert_eq!(back, parsed);
}

#[test]
fn automation_window_target_kind_without_index() {
    let json = r#"{"type":"kind","kind":"notes"}"#;
    let parsed: AutomationWindowTarget = serde_json::from_str(json).expect("parse");
    match &parsed {
        AutomationWindowTarget::Kind { kind, index } => {
            assert_eq!(*kind, AutomationWindowKind::Notes);
            assert_eq!(*index, None);
        }
        other => panic!("Expected Kind, got: {:?}", other),
    }
}

#[test]
fn automation_window_target_title_contains() {
    let target = AutomationWindowTarget::TitleContains {
        text: "Script Kit".into(),
    };
    let json = serde_json::to_string(&target).expect("serialize");
    let back: AutomationWindowTarget = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, target);
}

// ============================================================
// AutomationWindowKind exhaustive serde
// ============================================================

#[test]
fn automation_window_kind_all_variants_round_trip() {
    let kinds = vec![
        (AutomationWindowKind::Main, "main"),
        (AutomationWindowKind::Notes, "notes"),
        (AutomationWindowKind::Ai, "ai"),
        (AutomationWindowKind::MiniAi, "miniAi"),
        (AutomationWindowKind::AcpDetached, "acpDetached"),
        (AutomationWindowKind::ActionsDialog, "actionsDialog"),
        (AutomationWindowKind::PromptPopup, "promptPopup"),
    ];
    for (kind, expected_str) in kinds {
        let json = serde_json::to_string(&kind).expect("serialize kind");
        assert_eq!(json, format!("\"{}\"", expected_str), "kind {:?}", kind);
        let back: AutomationWindowKind = serde_json::from_str(&json).expect("deserialize kind");
        assert_eq!(back, kind);
    }
}

// ============================================================
// AutomationWindowInfo serde
// ============================================================

#[test]
fn automation_window_info_round_trip() {
    let info = AutomationWindowInfo {
        id: "notes:primary".into(),
        kind: AutomationWindowKind::Notes,
        title: Some("Script Kit Notes".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("notes".into()),
        bounds: None,
        parent_window_id: None,
        parent_kind: None,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    let back: AutomationWindowInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, info);
}

#[test]
fn automation_window_info_minimal_fields() {
    let json = r#"{"id":"main","kind":"main","focused":false,"visible":true}"#;
    let info: AutomationWindowInfo = serde_json::from_str(json).expect("parse minimal");
    assert_eq!(info.id, "main");
    assert_eq!(info.kind, AutomationWindowKind::Main);
    assert!(!info.focused);
    assert!(info.visible);
    assert!(info.title.is_none());
    assert!(info.semantic_surface.is_none());
    assert!(info.bounds.is_none());
}

#[test]
fn automation_window_info_with_bounds() {
    let info = AutomationWindowInfo {
        id: "main".into(),
        kind: AutomationWindowKind::Main,
        title: Some("Script Kit".into()),
        focused: true,
        visible: true,
        semantic_surface: None,
        bounds: Some(AutomationWindowBounds {
            x: 100.0,
            y: 200.0,
            width: 800.0,
            height: 600.0,
        }),
        parent_window_id: None,
        parent_kind: None,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    assert!(json.contains("\"bounds\""));
    let back: AutomationWindowInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.bounds.as_ref().expect("bounds").width, 800.0);
}

// ============================================================
// SimulatedGpuiEvent serde
// ============================================================

#[test]
fn simulate_gpui_event_key_down_round_trip() {
    let event = SimulatedGpuiEvent::KeyDown {
        key: "k".into(),
        modifiers: vec![crate::stdin_commands::KeyModifier::Cmd],
        text: None,
    };
    let json = serde_json::to_string(&event).expect("serialize");
    assert!(json.contains(r#""type":"keyDown"#));
    assert!(json.contains(r#""key":"k"#));
    let back: SimulatedGpuiEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, event);
}

#[test]
fn simulate_gpui_event_mouse_move_round_trip() {
    let event = SimulatedGpuiEvent::MouseMove { x: 10.5, y: 20.0 };
    let json = serde_json::to_string(&event).expect("serialize");
    let back: SimulatedGpuiEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, event);
}

#[test]
fn simulate_gpui_event_mouse_down_round_trip() {
    let event = SimulatedGpuiEvent::MouseDown {
        x: 50.0,
        y: 100.0,
        button: Some("right".into()),
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let back: SimulatedGpuiEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, event);
}

#[test]
fn simulate_gpui_event_request_round_trip() {
    let json = r#"{
        "type": "simulateGpuiEvent",
        "requestId": "gpui-1",
        "target": {"type": "kind", "kind": "acpDetached"},
        "event": {"type": "keyDown", "key": "k", "modifiers": ["cmd"]}
    }"#;
    let msg: crate::protocol::Message =
        serde_json::from_str(json).expect("parse simulateGpuiEvent");
    match msg {
        crate::protocol::Message::SimulateGpuiEvent {
            request_id,
            target,
            event,
        } => {
            assert_eq!(request_id, "gpui-1");
            assert!(target.is_some());
            match event {
                SimulatedGpuiEvent::KeyDown { key, modifiers, .. } => {
                    assert_eq!(key, "k");
                    assert_eq!(modifiers.len(), 1);
                }
                other => panic!("Expected KeyDown, got: {:?}", other),
            }
        }
        other => panic!("Expected SimulateGpuiEvent, got: {:?}", other),
    }
}

// ============================================================
// ListAutomationWindows serde
// ============================================================

#[test]
fn list_automation_windows_request_round_trip() {
    let json = r#"{"type":"listAutomationWindows","requestId":"wins-1"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::ListAutomationWindows { request_id } => {
            assert_eq!(request_id, "wins-1");
        }
        other => panic!("Expected ListAutomationWindows, got: {:?}", other),
    }
}

#[test]
fn automation_window_list_result_round_trip() {
    let msg = crate::protocol::Message::automation_window_list_result(
        "wins-1".into(),
        vec![
            AutomationWindowInfo {
                id: "main".into(),
                kind: AutomationWindowKind::Main,
                title: Some("Script Kit".into()),
                focused: false,
                visible: true,
                semantic_surface: Some("scriptList".into()),
                bounds: None,
                parent_window_id: None,
                parent_kind: None,
            },
            AutomationWindowInfo {
                id: "acpDetached:thread-1".into(),
                kind: AutomationWindowKind::AcpDetached,
                title: Some("Script Kit AI".into()),
                focused: true,
                visible: true,
                semantic_surface: Some("acpChat".into()),
                bounds: None,
                parent_window_id: None,
                parent_kind: None,
            },
        ],
        Some("acpDetached:thread-1".into()),
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("automationWindowListResult"));
    assert!(json.contains("focusedWindowId"));
    let back: crate::protocol::Message = serde_json::from_str(&json).expect("deserialize");
    match back {
        crate::protocol::Message::AutomationWindowListResult {
            windows,
            focused_window_id,
            ..
        } => {
            assert_eq!(windows.len(), 2);
            assert_eq!(focused_window_id.as_deref(), Some("acpDetached:thread-1"));
        }
        other => panic!("Expected AutomationWindowListResult, got: {:?}", other),
    }
}

// ============================================================
// Backward compatibility: target omitted
// ============================================================

#[test]
fn legacy_get_elements_request_still_parses_without_target() {
    let json = r#"{"type":"getElements","requestId":"elm-1"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::GetElements {
            request_id,
            limit,
            target,
        } => {
            assert_eq!(request_id, "elm-1");
            assert_eq!(limit, None);
            assert!(target.is_none(), "target should default to None");
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

#[test]
fn legacy_get_state_request_still_parses_without_target() {
    let json = r#"{"type":"getState","requestId":"gs-1"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::GetState { request_id, target } => {
            assert_eq!(request_id, "gs-1");
            assert!(target.is_none());
        }
        other => panic!("Expected GetState, got: {:?}", other),
    }
}

#[test]
fn legacy_capture_screenshot_still_parses_without_target() {
    let json = r#"{"type":"captureScreenshot","requestId":"shot-1"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::CaptureScreenshot { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected CaptureScreenshot, got: {:?}", other),
    }
}

#[test]
fn legacy_wait_for_still_parses_without_target() {
    let json = r#"{"type":"waitFor","requestId":"wf-1","condition":"windowVisible"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::WaitFor { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected WaitFor, got: {:?}", other),
    }
}

#[test]
fn legacy_batch_still_parses_without_target() {
    let json = r#"{"type":"batch","requestId":"b-1","commands":[]}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::Batch { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected Batch, got: {:?}", other),
    }
}

#[test]
fn legacy_get_acp_state_still_parses_without_target() {
    let json = r#"{"type":"getAcpState","requestId":"acp-1"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::GetAcpState { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected GetAcpState, got: {:?}", other),
    }
}

#[test]
fn legacy_simulate_click_still_parses_without_target() {
    let json = r#"{"type":"simulateClick","requestId":"sc-1","x":10,"y":20}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::SimulateClick { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected SimulateClick, got: {:?}", other),
    }
}

// ============================================================
// Targeted requests: target included
// ============================================================

#[test]
fn get_elements_with_target_parses() {
    let json = r#"{"type":"getElements","requestId":"elm-notes","target":{"type":"kind","kind":"notes"},"limit":50}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::GetElements {
            request_id,
            limit,
            target,
        } => {
            assert_eq!(request_id, "elm-notes");
            assert_eq!(limit, Some(50));
            let t = target.expect("target should be present");
            match t {
                AutomationWindowTarget::Kind { kind, index } => {
                    assert_eq!(kind, AutomationWindowKind::Notes);
                    assert_eq!(index, None);
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

#[test]
fn capture_screenshot_with_target_parses() {
    let json = r#"{"type":"captureScreenshot","requestId":"shot-acp","target":{"type":"kind","kind":"acpDetached","index":0},"hiDpi":true}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    match msg {
        crate::protocol::Message::CaptureScreenshot {
            request_id,
            hi_dpi,
            target,
        } => {
            assert_eq!(request_id, "shot-acp");
            assert_eq!(hi_dpi, Some(true));
            assert!(target.is_some());
        }
        other => panic!("Expected CaptureScreenshot, got: {:?}", other),
    }
}

#[test]
fn inspect_automation_window_request_round_trip_and_request_id() {
    let json = r#"{
        "type":"inspectAutomationWindow",
        "requestId":"inspect-1",
        "target":{"type":"kind","kind":"acpDetached","index":0},
        "hiDpi":true,
        "probes":[{"x":24,"y":24},{"x":320,"y":180}]
    }"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse");
    assert_eq!(msg.request_id(), Some("inspect-1"));
    match msg {
        crate::protocol::Message::InspectAutomationWindow {
            request_id,
            target,
            hi_dpi,
            probes,
        } => {
            assert_eq!(request_id, "inspect-1");
            assert_eq!(hi_dpi, Some(true));
            assert_eq!(probes.len(), 2);
            match target.expect("target should be present") {
                AutomationWindowTarget::Kind { kind, index } => {
                    assert_eq!(kind, AutomationWindowKind::AcpDetached);
                    assert_eq!(index, Some(0));
                }
                other => panic!("Expected Kind target, got: {:?}", other),
            }
        }
        other => panic!("Expected InspectAutomationWindow, got: {:?}", other),
    }
}

#[test]
fn automation_inspect_result_round_trip_and_request_id() {
    let msg = crate::protocol::Message::automation_inspect_result(
        "inspect-1".into(),
        crate::protocol::AutomationInspectSnapshot {
            schema_version: crate::protocol::AUTOMATION_INSPECT_SCHEMA_VERSION,
            window_id: "acpDetached:thread-1".into(),
            window_kind: "AcpDetached".into(),
            title: Some("Script Kit AI".into()),
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
            pixel_probes: vec![crate::protocol::PixelProbeResult {
                x: 24,
                y: 24,
                r: 28,
                g: 28,
                b: 30,
                a: 255,
            }],
            os_window_id: Some(99),
            warnings: vec!["semantic_elements_detached_acp_pending".into()],
        },
    );
    assert_eq!(msg.request_id(), Some("inspect-1"));
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("automationInspectResult"));
    let back: crate::protocol::Message = serde_json::from_str(&json).expect("deserialize");
    match back {
        crate::protocol::Message::AutomationInspectResult {
            request_id,
            snapshot,
        } => {
            assert_eq!(request_id, "inspect-1");
            assert_eq!(snapshot.window_id, "acpDetached:thread-1");
            assert_eq!(snapshot.screenshot_width, Some(1440));
            assert_eq!(snapshot.pixel_probes.len(), 1);
        }
        other => panic!("Expected AutomationInspectResult, got: {:?}", other),
    }
}

// ============================================================
// Constructor tests
// ============================================================

#[test]
fn list_automation_windows_constructor() {
    let msg = crate::protocol::Message::list_automation_windows("test-1".into());
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("listAutomationWindows"));
    assert!(json.contains("test-1"));
}

#[test]
fn simulate_gpui_event_constructor() {
    let msg = crate::protocol::Message::simulate_gpui_event(
        "evt-1".into(),
        SimulatedGpuiEvent::KeyDown {
            key: "enter".into(),
            modifiers: vec![],
            text: None,
        },
        Some(AutomationWindowTarget::Main),
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("simulateGpuiEvent"));
    assert!(json.contains("enter"));
}

#[test]
fn simulate_gpui_event_result_success_constructor() {
    let msg = crate::protocol::Message::simulate_gpui_event_result_success(
        "evt-1".into(),
        Some("exact_handle".into()),
        Some("win-42".into()),
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains(r#""success":true"#));
    assert!(json.contains(r#""dispatchPath":"exact_handle""#));
    assert!(json.contains(r#""resolvedWindowId":"win-42""#));
}

#[test]
fn simulate_gpui_event_result_success_omits_none_fields() {
    let msg =
        crate::protocol::Message::simulate_gpui_event_result_success("evt-2".into(), None, None);
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains(r#""success":true"#));
    assert!(!json.contains("dispatchPath"));
    assert!(!json.contains("resolvedWindowId"));
}

#[test]
fn simulate_gpui_event_result_error_constructor() {
    let msg = crate::protocol::Message::simulate_gpui_event_result_error(
        "evt-1".into(),
        "target_not_found".into(),
        "Window not found".into(),
        None,
        None,
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains(r#""success":false"#));
    assert!(json.contains(r#""errorCode":"target_not_found""#));
    assert!(json.contains("Window not found"));
}
