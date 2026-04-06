//! Contract tests for waitFor wire shape.
//!
//! Locks the serialization contract: canonical `semanticId` on output,
//! accepts both `semanticId` and legacy `semantic_id` on input.

use script_kit_gpui::protocol::Message;
use serde_json::{json, Value};

fn wait_for_message(condition: Value) -> Value {
    json!({
        "type": "waitFor",
        "requestId": "wait-1",
        "condition": condition,
        "timeout": 1000,
        "pollInterval": 25
    })
}

fn round_trip(message_json: Value) -> Value {
    let parsed: Message =
        serde_json::from_value(message_json).expect("waitFor message should parse");
    serde_json::to_value(parsed).expect("waitFor message should serialize")
}

#[test]
fn wait_for_accepts_canonical_semantic_id_and_emits_canonical_output() {
    for condition in [
        json!({"type": "elementExists", "semanticId": "choice:0:apple"}),
        json!({"type": "elementVisible", "semanticId": "input:filter"}),
        json!({"type": "elementFocused", "semanticId": "input:filter"}),
    ] {
        let output = round_trip(wait_for_message(condition));
        let condition_out = output
            .get("condition")
            .expect("serialized message should include condition");

        assert!(
            condition_out.get("semanticId").is_some(),
            "serialized condition must include semanticId: {condition_out}"
        );
        assert!(
            condition_out.get("semantic_id").is_none(),
            "serialized condition must not include legacy semantic_id: {condition_out}"
        );
    }
}

#[test]
fn wait_for_accepts_legacy_semantic_id_but_canonicalizes_output() {
    for condition in [
        json!({"type": "elementExists", "semantic_id": "choice:0:apple"}),
        json!({"type": "elementVisible", "semantic_id": "input:filter"}),
        json!({"type": "elementFocused", "semantic_id": "input:filter"}),
    ] {
        let output = round_trip(wait_for_message(condition));
        let condition_out = output
            .get("condition")
            .expect("serialized message should include condition");

        assert!(
            condition_out.get("semanticId").is_some(),
            "serialized condition must include canonical semanticId: {condition_out}"
        );
        assert!(
            condition_out.get("semantic_id").is_none(),
            "serialized condition must not re-emit legacy semantic_id: {condition_out}"
        );
    }
}

#[test]
fn wait_for_state_match_round_trips_without_semantic_id_fields() {
    let output = round_trip(wait_for_message(json!({
        "type": "stateMatch",
        "state": {
            "promptType": "arg",
            "inputValue": "apple",
            "selectedValue": "apple",
            "windowVisible": true
        }
    })));

    let condition_out = output
        .get("condition")
        .expect("serialized message should include condition");

    assert_eq!(condition_out.get("type"), Some(&json!("stateMatch")));
    assert!(condition_out.get("semanticId").is_none());
    assert!(condition_out.get("semantic_id").is_none());
}

// ============================================================
// Cross-window target contracts
// ============================================================

#[test]
fn wait_for_with_acp_detached_target_round_trips() {
    let msg =
        wait_for_message(json!({"type": "elementExists", "semanticId": "input:acp-composer"}));
    let mut msg_with_target = msg.clone();
    msg_with_target.as_object_mut().unwrap().insert(
        "target".into(),
        json!({"type": "kind", "kind": "acpDetached"}),
    );

    let parsed: Message = serde_json::from_value(msg_with_target)
        .expect("waitFor with acpDetached target should parse");
    let output = serde_json::to_value(parsed).expect("should serialize");

    assert!(
        output.get("target").is_some(),
        "target field must survive round-trip"
    );
    let target = output.get("target").unwrap();
    assert_eq!(target["kind"], "acpDetached");
}

#[test]
fn wait_for_with_notes_target_round_trips() {
    let msg =
        wait_for_message(json!({"type": "elementExists", "semanticId": "input:notes-editor"}));
    let mut msg_with_target = msg.clone();
    msg_with_target
        .as_object_mut()
        .unwrap()
        .insert("target".into(), json!({"type": "kind", "kind": "notes"}));

    let parsed: Message =
        serde_json::from_value(msg_with_target).expect("waitFor with notes target should parse");
    let output = serde_json::to_value(parsed).expect("should serialize");

    assert!(
        output.get("target").is_some(),
        "target field must survive round-trip"
    );
    let target = output.get("target").unwrap();
    assert_eq!(target["kind"], "notes");
}

#[test]
fn wait_for_without_target_still_works() {
    // Ensure backward compatibility: no target = main window
    let output = round_trip(wait_for_message(json!("choicesRendered")));
    assert!(
        output.get("target").is_none() || output.get("target").unwrap().is_null(),
        "waitFor without target should not inject one"
    );
}

#[test]
fn wait_for_named_condition_round_trips_unchanged() {
    let output = round_trip(wait_for_message(json!("choicesRendered")));

    assert_eq!(output.get("type"), Some(&json!("waitFor")));
    assert_eq!(output.get("requestId"), Some(&json!("wait-1")));
    assert_eq!(output.get("condition"), Some(&json!("choicesRendered")));
    assert_eq!(output.get("timeout"), Some(&json!(1000)));
    assert_eq!(output.get("pollInterval"), Some(&json!(25)));
}
