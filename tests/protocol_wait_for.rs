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
        reserialized["condition"]["state"]
            .get("selectedValue")
            .is_none(),
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
    assert!(
        json.get("trace").is_none(),
        "trace field should be absent when not requested"
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
        Some(script_kit_gpui::protocol::TransactionError::wait_timeout(
            "Timeout after 5000ms",
        )),
    );
    let json = serde_json::to_value(&msg).expect("serialize timeout result");

    assert_eq!(json["type"], "waitForResult");
    assert_eq!(json["requestId"], "w-timeout");
    assert_eq!(json["success"], false);
    assert_eq!(json["elapsed"], 5000);
    assert_eq!(json["error"]["code"], "wait_condition_timeout");
    assert_eq!(json["error"]["message"], "Timeout after 5000ms");
}

// ---------- waitFor with all named conditions ----------

#[test]
fn wait_for_all_named_conditions_parse() {
    let conditions = [
        "choicesRendered",
        "inputEmpty",
        "windowVisible",
        "windowFocused",
    ];

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
            reserialized["condition"]["type"], cond["type"],
            "detailed condition type should preserve"
        );
    }
}

// ---------- trace mode on waitFor requests ----------

#[test]
fn wait_for_with_trace_mode_parses_and_preserves() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-trace",
        "condition": "choicesRendered",
        "timeout": 1000,
        "trace": "on"
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse waitFor with trace");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["trace"], "on");
}

#[test]
fn wait_for_with_on_failure_trace_mode_parses() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-trace-fail",
        "condition": "windowVisible",
        "trace": "onFailure"
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse onFailure trace");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["trace"], "onFailure");
}

#[test]
fn wait_for_without_trace_omits_field() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-no-trace",
        "condition": "choicesRendered"
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse without trace");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert!(
        reserialized.get("trace").is_none(),
        "trace field should be omitted when off (default)"
    );
}

// ---------- waitForResult with trace receipt ----------

#[test]
fn wait_for_result_with_trace_receipt_serializes() {
    use script_kit_gpui::protocol::{
        TransactionCommandTrace, TransactionError, TransactionErrorCode, TransactionTrace,
        TransactionTraceStatus, UiStateSnapshot, WaitPollObservation,
    };

    let trace = TransactionTrace {
        request_id: "w-traced".to_string(),
        status: TransactionTraceStatus::Timeout,
        started_at_ms: 100,
        total_elapsed_ms: 1000,
        failed_at: Some(0),
        commands: vec![TransactionCommandTrace {
            index: 0,
            command: "waitFor".to_string(),
            started_at_ms: 100,
            elapsed_ms: 1000,
            before: UiStateSnapshot {
                window_visible: true,
                window_focused: true,
                choice_count: 0,
                ..Default::default()
            },
            after: UiStateSnapshot {
                window_visible: true,
                window_focused: true,
                choice_count: 0,
                ..Default::default()
            },
            polls: vec![WaitPollObservation {
                attempt: 1,
                elapsed_ms: 1000,
                condition_satisfied: false,
                snapshot: UiStateSnapshot {
                    window_visible: true,
                    window_focused: true,
                    choice_count: 0,
                    ..Default::default()
                },
                matched_semantic_ids: vec![],
            }],
            error: Some(TransactionError {
                code: TransactionErrorCode::WaitConditionTimeout,
                message: "Timeout after 1000ms waiting for choicesRendered".to_string(),
                suggestion: Some("No choices were visible at timeout.".to_string()),
            }),
        }],
    };

    let msg = Message::wait_for_result_with_trace(
        "w-traced".to_string(),
        false,
        1000,
        Some(TransactionError {
            code: TransactionErrorCode::WaitConditionTimeout,
            message: "Timeout after 1000ms waiting for choicesRendered".to_string(),
            suggestion: Some("No choices were visible at timeout.".to_string()),
        }),
        Some(trace),
    );
    let json = serde_json::to_value(&msg).expect("serialize result with trace");

    assert_eq!(json["type"], "waitForResult");
    assert_eq!(json["success"], false);
    assert_eq!(json["error"]["code"], "wait_condition_timeout");
    assert!(json["error"]["suggestion"].is_string());

    // Trace receipt structure
    let trace_json = &json["trace"];
    assert_eq!(trace_json["requestId"], "w-traced");
    assert_eq!(trace_json["status"], "timeout");
    assert_eq!(trace_json["failedAt"], 0);
    assert_eq!(trace_json["totalElapsedMs"], 1000);

    let cmd = &trace_json["commands"][0];
    assert_eq!(cmd["command"], "waitFor");
    assert_eq!(cmd["polls"][0]["conditionSatisfied"], false);
    assert_eq!(cmd["polls"][0]["snapshot"]["choiceCount"], 0);

    // Round-trip
    let back: Message = serde_json::from_value(json).expect("round-trip with trace");
    let re = serde_json::to_value(&back).expect("re-serialize");
    assert_eq!(re["trace"]["status"], "timeout");
}

// ---------- ACP proof condition wire shapes ----------

#[test]
fn wait_for_acp_accepted_via_key_enter_parses() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-acp-enter",
        "condition": {
            "type": "acpAcceptedViaKey",
            "key": "enter"
        },
        "timeout": 3000,
        "pollInterval": 25,
        "trace": "onFailure"
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse acpAcceptedViaKey enter");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["condition"]["type"], "acpAcceptedViaKey");
    assert_eq!(reserialized["condition"]["key"], "enter");
    assert_eq!(reserialized["timeout"], 3000);
    assert_eq!(reserialized["pollInterval"], 25);
    assert_eq!(reserialized["trace"], "onFailure");
}

#[test]
fn wait_for_acp_accepted_via_key_tab_parses() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-acp-tab",
        "condition": {
            "type": "acpAcceptedViaKey",
            "key": "tab"
        },
        "timeout": 3000
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse acpAcceptedViaKey tab");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["condition"]["type"], "acpAcceptedViaKey");
    assert_eq!(reserialized["condition"]["key"], "tab");
}

#[test]
fn wait_for_acp_accepted_cursor_at_parses() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-cursor",
        "condition": {
            "type": "acpAcceptedCursorAt",
            "index": 17
        },
        "timeout": 3000
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse acpAcceptedCursorAt");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["condition"]["type"], "acpAcceptedCursorAt");
    assert_eq!(reserialized["condition"]["index"], 17);
}

#[test]
fn wait_for_acp_input_layout_match_parses() {
    let raw = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-layout",
        "condition": {
            "type": "acpInputLayoutMatch",
            "visibleStart": 0,
            "visibleEnd": 15,
            "cursorInWindow": 9
        },
        "timeout": 3000
    });
    let msg: Message = serde_json::from_value(raw).expect("should parse acpInputLayoutMatch");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["condition"]["type"], "acpInputLayoutMatch");
    assert_eq!(reserialized["condition"]["visibleStart"], 0);
    assert_eq!(reserialized["condition"]["visibleEnd"], 15);
    assert_eq!(reserialized["condition"]["cursorInWindow"], 9);
}

#[test]
fn wait_for_acp_proof_conditions_round_trip() {
    let conditions = [
        serde_json::json!({"type": "acpAcceptedViaKey", "key": "enter"}),
        serde_json::json!({"type": "acpAcceptedViaKey", "key": "tab"}),
        serde_json::json!({"type": "acpAcceptedCursorAt", "index": 42}),
        serde_json::json!({"type": "acpInputLayoutMatch", "visibleStart": 5, "visibleEnd": 20, "cursorInWindow": 3}),
    ];

    for (i, cond) in conditions.iter().enumerate() {
        let raw = serde_json::json!({
            "type": "waitFor",
            "requestId": format!("w-proof-{i}"),
            "condition": cond,
        });
        let msg: Message = serde_json::from_value(raw.clone())
            .unwrap_or_else(|e| panic!("should parse ACP proof condition {i}: {e}"));
        let serialized = serde_json::to_string(&msg).expect("serialize");
        let back: Message =
            serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("round-trip {i}: {e}"));
        let reserialized = serde_json::to_value(&back).expect("re-serialize");
        assert_eq!(
            reserialized["condition"]["type"], cond["type"],
            "proof condition type should survive round-trip: {i}"
        );
    }
}

// ---------- waitForResult success receipt with trace ----------

#[test]
fn wait_for_result_success_receipt_with_trace() {
    use script_kit_gpui::protocol::{TransactionTrace, TransactionTraceStatus};

    let trace = TransactionTrace {
        request_id: "w-ok".to_string(),
        status: TransactionTraceStatus::Ok,
        started_at_ms: 50,
        total_elapsed_ms: 17,
        failed_at: None,
        commands: vec![],
    };

    let msg = Message::wait_for_result_with_trace("w-ok".to_string(), true, 17, None, Some(trace));
    let json = serde_json::to_value(&msg).expect("serialize success with trace");

    assert_eq!(json["success"], true);
    assert!(json.get("error").is_none());
    assert_eq!(json["trace"]["status"], "ok");
    assert_eq!(json["trace"]["totalElapsedMs"], 17);
}
