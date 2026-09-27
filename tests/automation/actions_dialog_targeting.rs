//! Automation window targeting wire-format regression tests.
//!
//! Legacy requests without a target, parent identity fields, and inspect
//! receipts must keep their serde shape.

use script_kit_gpui::protocol::{
    AutomationInspectSnapshot, AutomationWindowInfo, AutomationWindowKind, Message,
    SemanticQuality, AUTOMATION_INSPECT_SCHEMA_VERSION,
};

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
fn backward_compat_get_agent_chat_state_without_target() {
    let json = r#"{"type":"getAgentChatState","requestId":"legacy-agent_chat-1"}"#;
    let msg: Message = serde_json::from_str(json).expect("parse");
    match msg {
        Message::GetAgentChatState { target, .. } => {
            assert!(target.is_none());
        }
        other => panic!("Expected GetAgentChatState, got: {:?}", other),
    }
}

#[test]
fn parent_identity_round_trips_through_serde() {
    let info = AutomationWindowInfo {
        id: "test-overlay".into(),
        kind: AutomationWindowKind::SnapOverlay,
        title: Some("Snap".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("snapOverlay".into()),
        bounds: None,
        parent_window_id: Some("main".into()),
        parent_window_generation: None,
        parent_kind: Some(AutomationWindowKind::Main),
        pid: None,
        generation: None,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    assert!(json.contains("\"parentWindowId\":\"main\""));
    assert!(json.contains("\"parentKind\":\"main\""));
    let deserialized: AutomationWindowInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.parent_window_id.as_deref(), Some("main"));
    assert_eq!(deserialized.parent_kind, Some(AutomationWindowKind::Main));
}

#[test]
fn parent_identity_omitted_when_none_in_serde() {
    let info = AutomationWindowInfo {
        id: "test-no-parent".into(),
        kind: AutomationWindowKind::Main,
        title: None,
        focused: false,
        visible: true,
        semantic_surface: None,
        bounds: None,
        parent_window_id: None,
        parent_window_generation: None,
        parent_kind: None,
        pid: None,
        generation: None,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    assert!(!json.contains("parentWindowId"));
    assert!(!json.contains("parentKind"));
}

#[test]
fn inspect_result_with_panel_only_quality() {
    let snapshot = AutomationInspectSnapshot {
        schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
        window_id: "snapOverlay:0".into(),
        window_kind: "SnapOverlay".into(),
        surface_kind: None,
        app_view_variant: None,
        native_footer_surface: None,
        target_generation: Some(1),
        surface_generation: Some(1),
        data_generation: Some(1),
        title: Some("Snap".into()),
        resolved_bounds: None,
        target_bounds_in_screenshot: None,
        surface_hit_point: None,
        suggested_hit_points: Vec::new(),
        elements: Vec::new(),
        total_count: 0,
        focused_semantic_id: None,
        selected_semantic_id: None,
        screenshot_width: Some(800),
        screenshot_height: Some(600),
        pixel_probes: Vec::new(),
        os_window_id: None,
        semantic_quality: Some(SemanticQuality::PanelOnly),
        warnings: Vec::new(),
        pid: None,
    };
    let json = serde_json::to_value(&snapshot).expect("serialize");
    assert_eq!(json["semanticQuality"], "panel_only");
    assert_eq!(json["windowKind"], "SnapOverlay");
    assert_eq!(json["schemaVersion"], 4);
}

#[test]
fn inspect_result_with_full_quality() {
    let snapshot = AutomationInspectSnapshot {
        schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
        window_id: "snapOverlay:0".into(),
        window_kind: "SnapOverlay".into(),
        surface_kind: None,
        app_view_variant: None,
        native_footer_surface: None,
        target_generation: Some(1),
        surface_generation: Some(1),
        data_generation: Some(1),
        title: Some("Confirm".into()),
        resolved_bounds: None,
        target_bounds_in_screenshot: None,
        surface_hit_point: None,
        suggested_hit_points: Vec::new(),
        elements: vec![script_kit_gpui::protocol::ElementInfo {
            element_type: script_kit_gpui::protocol::ElementType::Button,
            text: Some("Confirm".into()),
            value: Some("confirm".into()),
            content: None,
            selected: None,
            focused: Some(true),
            index: Some(0),
            role: None,
            kind: None,
            source: None,
            source_name: None,
            selectable: None,
            status_kind: None,
            action_disabled: None,
            style: None,
            semantic_id: "button:0:confirm".into(),
        }],
        total_count: 1,
        focused_semantic_id: Some("button:0:confirm".into()),
        selected_semantic_id: None,
        screenshot_width: Some(800),
        screenshot_height: Some(600),
        pixel_probes: Vec::new(),
        os_window_id: None,
        semantic_quality: Some(SemanticQuality::Full),
        warnings: Vec::new(),
        pid: None,
    };
    let json = serde_json::to_value(&snapshot).expect("serialize");
    assert_eq!(json["semanticQuality"], "full");
    assert_eq!(json["focusedSemanticId"], "button:0:confirm");
}

#[test]
fn backward_compat_inspect_receipt_without_semantic_quality() {
    // v2 callers (no semanticQuality) must still parse correctly
    let json = r#"{
        "schemaVersion": 2,
        "windowId": "main",
        "windowKind": "Main",
        "totalCount": 0
    }"#;
    let parsed: AutomationInspectSnapshot =
        serde_json::from_str(json).expect("should parse without semanticQuality");
    assert_eq!(parsed.semantic_quality, None);
    assert_eq!(parsed.window_id, "main");
}
