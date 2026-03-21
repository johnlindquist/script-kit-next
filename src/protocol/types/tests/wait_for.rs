use super::*;

// ============================================================
// WaitCondition serde — named conditions
// ============================================================

#[test]
fn wait_named_condition_choices_rendered_round_trips() {
    let cond = WaitCondition::Named(WaitNamedCondition::ChoicesRendered);
    let json = serde_json::to_value(&cond).expect("serialize named condition");
    assert_eq!(json, serde_json::json!("choicesRendered"));

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize named condition");
    assert_eq!(back, cond);
}

#[test]
fn wait_named_condition_input_empty_round_trips() {
    let cond = WaitCondition::Named(WaitNamedCondition::InputEmpty);
    let json = serde_json::to_value(&cond).expect("serialize");
    assert_eq!(json, serde_json::json!("inputEmpty"));

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, cond);
}

#[test]
fn wait_named_condition_window_visible_round_trips() {
    let cond = WaitCondition::Named(WaitNamedCondition::WindowVisible);
    let json = serde_json::to_value(&cond).expect("serialize");
    assert_eq!(json, serde_json::json!("windowVisible"));

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, cond);
}

#[test]
fn wait_named_condition_window_focused_round_trips() {
    let cond = WaitCondition::Named(WaitNamedCondition::WindowFocused);
    let json = serde_json::to_value(&cond).expect("serialize");
    assert_eq!(json, serde_json::json!("windowFocused"));

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, cond);
}

// ============================================================
// WaitCondition serde — detailed conditions
// ============================================================

#[test]
fn wait_detailed_element_exists_round_trips() {
    let cond = WaitCondition::Detailed(WaitDetailedCondition::ElementExists {
        semantic_id: "choice:0:apple".to_string(),
    });
    let json = serde_json::to_value(&cond).expect("serialize detailed condition");
    assert_eq!(json["type"], "elementExists");
    assert_eq!(json["semanticId"], "choice:0:apple");

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize detailed condition");
    assert_eq!(back, cond);
}

#[test]
fn wait_detailed_element_visible_round_trips() {
    let cond = WaitCondition::Detailed(WaitDetailedCondition::ElementVisible {
        semantic_id: "input:filter".to_string(),
    });
    let json = serde_json::to_value(&cond).expect("serialize");
    assert_eq!(json["type"], "elementVisible");
    assert_eq!(json["semanticId"], "input:filter");

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, cond);
}

#[test]
fn wait_detailed_element_focused_round_trips() {
    let cond = WaitCondition::Detailed(WaitDetailedCondition::ElementFocused {
        semantic_id: "input:filter".to_string(),
    });
    let json = serde_json::to_value(&cond).expect("serialize");
    assert_eq!(json["type"], "elementFocused");
    assert_eq!(json["semanticId"], "input:filter");

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, cond);
}

#[test]
fn wait_detailed_state_match_round_trips() {
    let cond = WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
        state: StateMatchSpec {
            prompt_type: Some("arg".to_string()),
            input_value: Some("apple".to_string()),
            selected_value: None,
            window_visible: Some(true),
        },
    });
    let json = serde_json::to_value(&cond).expect("serialize state match");
    assert_eq!(json["type"], "stateMatch");
    assert_eq!(json["state"]["promptType"], "arg");
    assert_eq!(json["state"]["inputValue"], "apple");
    assert!(json["state"].get("selectedValue").is_none());
    assert_eq!(json["state"]["windowVisible"], true);

    let back: WaitCondition = serde_json::from_value(json).expect("deserialize state match");
    assert_eq!(back, cond);
}

#[test]
fn state_match_spec_omits_none_fields() {
    let spec = StateMatchSpec {
        prompt_type: None,
        input_value: None,
        selected_value: None,
        window_visible: None,
    };
    let json = serde_json::to_value(&spec).expect("serialize empty spec");
    let obj = json.as_object().expect("should be object");
    assert!(obj.is_empty(), "All-None spec should serialize to empty object");
}

// ============================================================
// waitFor message — request parsing
// ============================================================

#[test]
fn wait_for_request_parses_with_named_condition() {
    let json = r#"{"type":"waitFor","requestId":"wait-1","condition":"choicesRendered","timeout":1000,"pollInterval":25}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse waitFor");

    match msg {
        crate::protocol::Message::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
        } => {
            assert_eq!(request_id, "wait-1");
            assert_eq!(condition, WaitCondition::Named(WaitNamedCondition::ChoicesRendered));
            assert_eq!(timeout, Some(1000));
            assert_eq!(poll_interval, Some(25));
        }
        other => panic!("Expected WaitFor, got: {:?}", other),
    }
}

#[test]
fn wait_for_request_parses_with_detailed_condition() {
    let json = serde_json::json!({
        "type": "waitFor",
        "requestId": "wait-2",
        "condition": {
            "type": "elementExists",
            "semanticId": "choice:0:apple"
        }
    });
    let msg: crate::protocol::Message = serde_json::from_value(json).expect("parse waitFor detailed");

    match msg {
        crate::protocol::Message::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
        } => {
            assert_eq!(request_id, "wait-2");
            assert_eq!(
                condition,
                WaitCondition::Detailed(WaitDetailedCondition::ElementExists {
                    semantic_id: "choice:0:apple".to_string(),
                })
            );
            assert_eq!(timeout, None);
            assert_eq!(poll_interval, None);
        }
        other => panic!("Expected WaitFor, got: {:?}", other),
    }
}

#[test]
fn wait_for_request_defaults_omitted_timeout_and_poll_interval() {
    let json = r#"{"type":"waitFor","requestId":"wait-3","condition":"inputEmpty"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("parse waitFor minimal");

    match msg {
        crate::protocol::Message::WaitFor {
            timeout,
            poll_interval,
            ..
        } => {
            assert_eq!(timeout, None);
            assert_eq!(poll_interval, None);
        }
        other => panic!("Expected WaitFor, got: {:?}", other),
    }
}

// ============================================================
// waitForResult message — response serialization
// ============================================================

#[test]
fn wait_for_result_success_serializes_correctly() {
    let msg = crate::protocol::Message::wait_for_result(
        "wait-1".to_string(),
        true,
        17,
        None,
    );
    let json = serde_json::to_value(&msg).expect("serialize waitForResult");

    assert_eq!(json["type"], "waitForResult");
    assert_eq!(json["requestId"], "wait-1");
    assert_eq!(json["success"], true);
    assert_eq!(json["elapsed"], 17);
    assert!(json.get("error").is_none(), "error should be omitted on success");
}

#[test]
fn wait_for_result_timeout_serializes_correctly() {
    let msg = crate::protocol::Message::wait_for_result(
        "wait-1".to_string(),
        false,
        5000,
        Some("Timeout after 5000ms".to_string()),
    );
    let json = serde_json::to_value(&msg).expect("serialize waitForResult timeout");

    assert_eq!(json["type"], "waitForResult");
    assert_eq!(json["requestId"], "wait-1");
    assert_eq!(json["success"], false);
    assert_eq!(json["elapsed"], 5000);
    assert_eq!(json["error"], "Timeout after 5000ms");
}

#[test]
fn wait_for_result_round_trips() {
    let msg = crate::protocol::Message::wait_for_result(
        "wait-rt".to_string(),
        true,
        42,
        None,
    );
    let json = serde_json::to_string(&msg).expect("serialize");
    let back: crate::protocol::Message = serde_json::from_str(&json).expect("deserialize");

    match back {
        crate::protocol::Message::WaitForResult {
            request_id,
            success,
            elapsed,
            error,
        } => {
            assert_eq!(request_id, "wait-rt");
            assert!(success);
            assert_eq!(elapsed, 42);
            assert_eq!(error, None);
        }
        other => panic!("Expected WaitForResult, got: {:?}", other),
    }
}
