//! Integration tests for the `waitFor` / `waitForResult` protocol messages.
//!
//! Validates wire-level compatibility: parsing incoming requests and producing
//! correctly shaped responses, covering success and timeout result shapes.

use script_kit_gpui::protocol::Message;

// ---------- waitFor request parsing from raw JSON ----------

#[test]
fn wait_for_named_condition_from_raw_json() {
    let raw = r#"{"type":"waitFor","requestId":"w-1","condition":"choicesRendered","timeout":1000,"pollInterval":25}"#;
    let msg: Message = serde_json::from_str(raw).expect("should parse waitFor from raw JSONL");

    let reserialized = serde_json::to_value(&msg).expect("should reserialize");
    assert_eq!(reserialized["type"], "waitFor");
    assert_eq!(reserialized["requestId"], "w-1");
    assert_eq!(reserialized["condition"], "choicesRendered");
    assert_eq!(reserialized["timeout"], 1000);
    assert_eq!(reserialized["pollInterval"], 25);
}

#[test]
fn wait_for_detailed_condition_from_raw_json() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-2",
        "condition": {
            "type": "stateMatch",
            "state": {
                "promptType": "arg",
                "inputValue": "apple",
                "windowVisible": true
            }
        },
        "timeout": 3000
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse detailed waitFor");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["condition"]["type"], "stateMatch");
    assert_eq!(reserialized["condition"]["state"]["promptType"], "arg");
    assert_eq!(reserialized["condition"]["state"]["inputValue"], "apple");
    assert_eq!(reserialized["condition"]["state"]["windowVisible"], true);
    // selectedValue should be absent (not serialized when None)
    assert!(
        reserialized["condition"]["state"].get("selectedValue").is_none(),
        "selectedValue should be omitted"
    );
}

// ---------- waitForResult response shapes ----------

#[test]
fn wait_for_result_success_shape() {
    let msg = Message::wait_for_result("w-1".to_string(), true, 17, None);
    let json = serde_json::to_value(&msg).expect("serialize success result");

    assert_eq!(json["type"], "waitForResult");
    assert_eq!(json["requestId"], "w-1");
    assert_eq!(json["success"], true);
    assert_eq!(json["elapsed"], 17);
    assert!(
        json.get("error").is_none(),
        "error field should be absent on success"
    );

    // Verify it round-trips
    let back: Message = serde_json::from_value(json).expect("round-trip success result");
    let re = serde_json::to_value(&back).expect("re-serialize");
    assert_eq!(re["success"], true);
    assert_eq!(re["elapsed"], 17);
}

#[test]
fn wait_for_result_timeout_shape() {
    let msg = Message::wait_for_result(
        "w-timeout".to_string(),
        false,
        5000,
        Some("Timeout after 5000ms".to_string()),
    );
    let json = serde_json::to_value(&msg).expect("serialize timeout result");

    assert_eq!(json["type"], "waitForResult");
    assert_eq!(json["requestId"], "w-timeout");
    assert_eq!(json["success"], false);
    assert_eq!(json["elapsed"], 5000);
    assert_eq!(json["error"], "Timeout after 5000ms");
}

// ---------- waitFor with all named conditions ----------

#[test]
fn wait_for_all_named_conditions_parse() {
    let conditions = ["choicesRendered", "inputEmpty", "windowVisible", "windowFocused"];

    for cond_str in &conditions {
        let raw = serde_json::json!({
            "type": "waitFor",
            "requestId": format!("w-{cond_str}"),
            "condition": cond_str,
        });
        let msg: Message = serde_json::from_value(raw)
            .unwrap_or_else(|e| panic!("should parse waitFor with condition '{cond_str}': {e}"));
        let reserialized = serde_json::to_value(&msg).expect("should reserialize");
        assert_eq!(
            reserialized["condition"], *cond_str,
            "condition should preserve camelCase name: {cond_str}"
        );
    }
}

// ---------- waitFor with all detailed conditions ----------

#[test]
fn wait_for_all_detailed_conditions_parse() {
    let conditions = [
        serde_json::json!({"type": "elementExists", "semanticId": "choice:0:apple"}),
        serde_json::json!({"type": "elementVisible", "semanticId": "input:filter"}),
        serde_json::json!({"type": "elementFocused", "semanticId": "input:filter"}),
        serde_json::json!({"type": "stateMatch", "state": {"promptType": "arg"}}),
    ];

    for (i, cond) in conditions.iter().enumerate() {
        let raw = serde_json::json!({
            "type": "waitFor",
            "requestId": format!("w-detail-{i}"),
            "condition": cond,
        });
        let msg: Message = serde_json::from_value(raw)
            .unwrap_or_else(|e| panic!("should parse detailed condition {i}: {e}"));
        let reserialized = serde_json::to_value(&msg).expect("should reserialize");
        assert_eq!(
            reserialized["condition"]["type"],
            cond["type"],
            "detailed condition type should preserve"
        );
    }
}
