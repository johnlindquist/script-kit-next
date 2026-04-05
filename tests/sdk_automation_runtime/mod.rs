//! Integration tests verifying that the `getState`, `getElements`, `waitFor`,
//! and `batch` automation APIs have correct protocol wire shapes, round-trip
//! cleanly, and that the transaction executor produces the expected results.

use script_kit_gpui::protocol::transaction_executor::{
    execute_batch, execute_wait_for, TransactionStateProvider,
};
use script_kit_gpui::protocol::{
    AcpInputLayoutTelemetry, AcpPickerItemAcceptedTelemetry, AcpTestProbeSnapshot, BatchCommand,
    Message, StateMatchSpec, TransactionError, TransactionErrorCode, TransactionTraceMode,
    UiStateSnapshot, WaitCondition, WaitDetailedCondition, WaitNamedCondition,
};

// ---------------------------------------------------------------------------
// Mock state provider for executor tests
// ---------------------------------------------------------------------------

struct MockProvider {
    input_value: String,
    selected_value: Option<String>,
    window_visible: bool,
    window_focused: bool,
    choice_count: usize,
    visible_semantic_ids: Vec<String>,
    focused_semantic_id: Option<String>,
    acp_probe: AcpTestProbeSnapshot,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            input_value: String::new(),
            selected_value: None,
            window_visible: true,
            window_focused: true,
            choice_count: 3,
            visible_semantic_ids: vec![
                "choice:0:alpha".to_string(),
                "choice:1:beta".to_string(),
                "choice:2:gamma".to_string(),
            ],
            focused_semantic_id: Some("choice:0:alpha".to_string()),
            acp_probe: AcpTestProbeSnapshot::default(),
        }
    }
}

impl TransactionStateProvider for MockProvider {
    fn snapshot(&self) -> UiStateSnapshot {
        UiStateSnapshot {
            window_visible: self.window_visible,
            window_focused: self.window_focused,
            input_value: Some(self.input_value.clone()),
            selected_value: self.selected_value.clone(),
            focused_semantic_id: self.focused_semantic_id.clone(),
            visible_semantic_ids: self.visible_semantic_ids.clone(),
            choice_count: self.choice_count,
        }
    }

    fn set_input(&mut self, text: &str) -> anyhow::Result<()> {
        self.input_value = text.to_string();
        Ok(())
    }

    fn select_by_value(&mut self, value: &str, _submit: bool) -> anyhow::Result<Option<String>> {
        if self
            .visible_semantic_ids
            .iter()
            .any(|id| id.contains(value))
        {
            self.selected_value = Some(value.to_string());
            Ok(Some(value.to_string()))
        } else {
            Ok(None)
        }
    }

    fn select_by_semantic_id(
        &mut self,
        semantic_id: &str,
        _submit: bool,
    ) -> anyhow::Result<Option<String>> {
        if self.visible_semantic_ids.contains(&semantic_id.to_string()) {
            self.selected_value = Some(semantic_id.to_string());
            Ok(Some(semantic_id.to_string()))
        } else {
            Ok(None)
        }
    }

    fn acp_test_probe(&self, _tail: usize) -> AcpTestProbeSnapshot {
        self.acp_probe.clone()
    }
}

// ============================================================
// getState protocol shape
// ============================================================

#[test]
fn get_state_request_parses() {
    let json = serde_json::json!({"type": "getState", "requestId": "gs-1"});
    let msg: Message = serde_json::from_value(json).expect("parse getState");
    match msg {
        Message::GetState { request_id, .. } => assert_eq!(request_id, "gs-1"),
        other => panic!("Expected GetState, got: {other:?}"),
    }
}

#[test]
fn state_result_serializes_all_fields() {
    let msg = Message::state_result(
        "gs-1".to_string(),
        "arg".to_string(),
        Some("prompt-1".to_string()),
        Some("Pick one".to_string()),
        "alpha".to_string(),
        3,
        2,
        0,
        Some("alpha".to_string()),
        true,
        true,
    );
    let json = serde_json::to_value(&msg).expect("serialize stateResult");
    assert_eq!(json["type"], "stateResult");
    assert_eq!(json["requestId"], "gs-1");
    assert_eq!(json["promptType"], "arg");
    assert_eq!(json["inputValue"], "alpha");
    assert_eq!(json["choiceCount"], 3);
    assert_eq!(json["visibleChoiceCount"], 2);
    assert_eq!(json["selectedIndex"], 0);
    assert_eq!(json["selectedValue"], "alpha");
    assert_eq!(json["isFocused"], true);
    assert_eq!(json["windowVisible"], true);
}

#[test]
fn state_result_round_trips() {
    let msg = Message::state_result(
        "gs-rt".to_string(),
        "none".to_string(),
        None,
        None,
        String::new(),
        0,
        0,
        -1,
        None,
        false,
        true,
    );
    let serialized = serde_json::to_string(&msg).expect("serialize");
    let back: Message = serde_json::from_str(&serialized).expect("deserialize");
    match back {
        Message::StateResult {
            request_id,
            prompt_type,
            input_value,
            selected_index,
            window_visible,
            ..
        } => {
            assert_eq!(request_id, "gs-rt");
            assert_eq!(prompt_type, "none");
            assert_eq!(input_value, "");
            assert_eq!(selected_index, -1);
            assert!(window_visible);
        }
        other => panic!("Expected StateResult, got: {other:?}"),
    }
}

// ============================================================
// getElements protocol shape
// ============================================================

#[test]
fn get_elements_request_parses_with_limit() {
    let json = serde_json::json!({"type": "getElements", "requestId": "ge-1", "limit": 10});
    let msg: Message = serde_json::from_value(json).expect("parse getElements");
    match msg {
        Message::GetElements { request_id, limit, .. } => {
            assert_eq!(request_id, "ge-1");
            assert_eq!(limit, Some(10));
        }
        other => panic!("Expected GetElements, got: {other:?}"),
    }
}

#[test]
fn get_elements_request_parses_without_limit() {
    let json = serde_json::json!({"type": "getElements", "requestId": "ge-2"});
    let msg: Message = serde_json::from_value(json).expect("parse getElements");
    match msg {
        Message::GetElements { limit, .. } => {
            assert_eq!(limit, None);
        }
        other => panic!("Expected GetElements, got: {other:?}"),
    }
}

// ============================================================
// waitFor protocol shape + executor
// ============================================================

#[test]
fn wait_for_request_parses_named_condition() {
    let json = serde_json::json!({
        "type": "waitFor",
        "requestId": "wf-1",
        "condition": "choicesRendered",
        "timeout": 1000,
        "pollInterval": 25
    });
    let msg: Message = serde_json::from_value(json).expect("parse waitFor");
    match msg {
        Message::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
            ..
        } => {
            assert_eq!(request_id, "wf-1");
            assert_eq!(
                condition,
                WaitCondition::Named(WaitNamedCondition::ChoicesRendered)
            );
            assert_eq!(timeout, Some(1000));
            assert_eq!(poll_interval, Some(25));
        }
        other => panic!("Expected WaitFor, got: {other:?}"),
    }
}

#[test]
fn wait_for_request_parses_state_match_condition() {
    let json = serde_json::json!({
        "type": "waitFor",
        "requestId": "wf-2",
        "condition": {
            "type": "stateMatch",
            "state": { "inputValue": "alpha" }
        }
    });
    let msg: Message = serde_json::from_value(json).expect("parse waitFor stateMatch");
    match msg {
        Message::WaitFor { condition, .. } => {
            assert_eq!(
                condition,
                WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
                    state: StateMatchSpec {
                        prompt_type: None,
                        input_value: Some("alpha".to_string()),
                        selected_value: None,
                        window_visible: None,
                    }
                })
            );
        }
        other => panic!("Expected WaitFor, got: {other:?}"),
    }
}

#[test]
fn wait_for_executor_succeeds_immediately() {
    let mut provider = MockProvider::default();
    provider.input_value = "alpha".to_string();

    let condition = WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
        state: StateMatchSpec {
            input_value: Some("alpha".to_string()),
            ..Default::default()
        },
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-imm".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(output.success);
    assert_eq!(output.elapsed, 0);
    assert!(output.error.is_none());
    assert!(output.trace.is_none());
}

#[test]
fn wait_for_executor_times_out() {
    let mut provider = MockProvider::default();
    provider.input_value = "wrong".to_string();

    let condition = WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
        state: StateMatchSpec {
            input_value: Some("never-gonna-match".to_string()),
            ..Default::default()
        },
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-to".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(!output.success);
    assert!(output.elapsed >= 50);
    let err = output.error.expect("should have error on timeout");
    assert_eq!(err.code, TransactionErrorCode::WaitConditionTimeout);
}

#[test]
fn wait_for_element_exists_succeeds() {
    let mut provider = MockProvider::default();

    let condition = WaitCondition::Detailed(WaitDetailedCondition::ElementExists {
        semantic_id: "choice:1:beta".to_string(),
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-elem".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(output.success);
}

// ============================================================
// batch protocol shape + executor
// ============================================================

#[test]
fn batch_set_input_then_wait_succeeds() {
    let mut provider = MockProvider::default();

    let commands = vec![
        BatchCommand::SetInput {
            text: "beta".to_string(),
        },
        BatchCommand::WaitFor {
            condition: WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
                state: StateMatchSpec {
                    input_value: Some("beta".to_string()),
                    ..Default::default()
                },
            }),
            timeout: Some(100),
            poll_interval: Some(10),
        },
    ];

    let output = execute_batch(
        &mut provider,
        "batch-1".to_string(),
        &commands,
        None,
        TransactionTraceMode::Off,
    )
    .expect("execute_batch");

    assert!(output.success);
    assert_eq!(output.results.len(), 2);
    assert!(output.results[0].success);
    assert!(output.results[1].success);
    assert!(output.failed_at.is_none());
}

#[test]
fn batch_stops_on_error_by_default() {
    let mut provider = MockProvider::default();

    let commands = vec![
        BatchCommand::SelectByValue {
            value: "nonexistent".to_string(),
            submit: false,
        },
        BatchCommand::SetInput {
            text: "should not reach".to_string(),
        },
    ];

    let output = execute_batch(
        &mut provider,
        "batch-err".to_string(),
        &commands,
        None,
        TransactionTraceMode::Off,
    )
    .expect("execute_batch");

    assert!(!output.success);
    assert_eq!(output.failed_at, Some(0));
    // Second command should not have run
    assert_eq!(output.results.len(), 1);
}

#[test]
fn batch_force_submit_is_unsupported_in_executor() {
    let mut provider = MockProvider::default();

    let commands = vec![BatchCommand::ForceSubmit {
        value: serde_json::json!("hello"),
    }];

    let output = execute_batch(
        &mut provider,
        "batch-fs".to_string(),
        &commands,
        None,
        TransactionTraceMode::Off,
    )
    .expect("execute_batch");

    // forceSubmit is unsupported in the test executor
    assert!(!output.success);
    assert_eq!(output.results.len(), 1);
    let err = output.results[0].error.as_ref().expect("should have error");
    assert_eq!(err.code, TransactionErrorCode::UnsupportedCommand);
}

#[test]
fn batch_force_submit_serde_round_trips() {
    let cmd = BatchCommand::ForceSubmit {
        value: serde_json::json!("test-value"),
    };
    let json = serde_json::to_value(&cmd).expect("serialize forceSubmit");
    assert_eq!(json["type"], "forceSubmit");
    assert_eq!(json["value"], "test-value");

    let back: BatchCommand = serde_json::from_value(json).expect("deserialize forceSubmit");
    assert_eq!(back, cmd);
}

#[test]
fn batch_request_with_force_submit_parses() {
    let json = serde_json::json!({
        "type": "batch",
        "requestId": "txn-fs",
        "commands": [
            {"type": "setInput", "text": "hello"},
            {"type": "forceSubmit", "value": "world"}
        ]
    });
    let msg: Message = serde_json::from_value(json).expect("parse batch with forceSubmit");
    match msg {
        Message::Batch {
            commands,
            request_id,
            ..
        } => {
            assert_eq!(request_id, "txn-fs");
            assert_eq!(commands.len(), 2);
            assert!(matches!(&commands[0], BatchCommand::SetInput { text } if text == "hello"));
            assert!(
                matches!(&commands[1], BatchCommand::ForceSubmit { value } if value == "world")
            );
        }
        other => panic!("Expected Batch, got: {other:?}"),
    }
}

#[test]
fn batch_with_trace_on_failure_includes_trace() {
    let mut provider = MockProvider::default();
    provider.input_value = "wrong".to_string();

    let commands = vec![BatchCommand::WaitFor {
        condition: WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
            state: StateMatchSpec {
                input_value: Some("never".to_string()),
                ..Default::default()
            },
        }),
        timeout: Some(30),
        poll_interval: Some(10),
    }];

    let output = execute_batch(
        &mut provider,
        "batch-trace".to_string(),
        &commands,
        None,
        TransactionTraceMode::OnFailure,
    )
    .expect("execute_batch");

    assert!(!output.success);
    let trace = output.trace.expect("trace should be present on failure");
    assert_eq!(trace.request_id, "batch-trace");
    assert!(!trace.commands.is_empty());
}

// ============================================================
// TransactionError shape
// ============================================================

#[test]
fn transaction_error_serializes_with_correct_wire_shape() {
    let err = TransactionError {
        code: TransactionErrorCode::WaitConditionTimeout,
        message: "Timeout after 5000ms".to_string(),
        suggestion: Some("Try a longer timeout.".to_string()),
    };
    let json = serde_json::to_value(&err).expect("serialize");
    assert_eq!(json["code"], "wait_condition_timeout");
    assert_eq!(json["message"], "Timeout after 5000ms");
    assert_eq!(json["suggestion"], "Try a longer timeout.");
}

#[test]
fn transaction_error_omits_null_suggestion() {
    let err = TransactionError::action_failed("something broke");
    let json = serde_json::to_value(&err).expect("serialize");
    assert_eq!(json["code"], "action_failed");
    assert!(
        json.get("suggestion").is_none(),
        "suggestion should be omitted when None"
    );
}

// ============================================================
// SDK-TS error shape alignment
// ============================================================

#[test]
fn transaction_error_matches_sdk_ts_interface() {
    // The TS TransactionErrorData interface expects:
    //   { code?: string; message: string; suggestion?: string; }
    // Verify the Rust serialization matches this shape.
    let err = TransactionError {
        code: TransactionErrorCode::ElementNotFound,
        message: "Element not found: choice:99:missing".to_string(),
        suggestion: Some("Check getElements() output.".to_string()),
    };
    let json = serde_json::to_value(&err).expect("serialize");

    // code is a string
    assert!(json["code"].is_string());
    // message is a string
    assert!(json["message"].is_string());
    // suggestion is a string (not "details")
    assert!(json["suggestion"].is_string());
    // No "details" field
    assert!(
        json.get("details").is_none(),
        "Rust TransactionError should not have a 'details' field"
    );
}

// ============================================================
// ACP proof conditions: acpAcceptedViaKey
// ============================================================

fn make_accepted_item(key: &str) -> AcpPickerItemAcceptedTelemetry {
    AcpPickerItemAcceptedTelemetry {
        trigger: "@".to_string(),
        item_label: "Context".to_string(),
        item_id: "built_in:context".to_string(),
        accepted_via_key: key.to_string(),
        cursor_after: 9,
        caused_submit: false,
    }
}

#[test]
fn acp_accepted_via_enter_succeeds_immediately() {
    let mut provider = MockProvider::default();
    provider.acp_probe.accepted_items = vec![make_accepted_item("enter")];

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedViaKey {
        key: "enter".to_string(),
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-enter".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(output.success, "acpAcceptedViaKey enter should match");
    assert_eq!(output.elapsed, 0);
    assert!(output.error.is_none());
}

#[test]
fn acp_accepted_via_tab_succeeds_immediately() {
    let mut provider = MockProvider::default();
    provider.acp_probe.accepted_items = vec![make_accepted_item("tab")];

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedViaKey {
        key: "tab".to_string(),
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-tab".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(output.success, "acpAcceptedViaKey tab should match");
    assert_eq!(output.elapsed, 0);
}

#[test]
fn acp_accepted_via_key_wrong_key_times_out() {
    let mut provider = MockProvider::default();
    provider.acp_probe.accepted_items = vec![make_accepted_item("tab")];

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedViaKey {
        key: "enter".to_string(),
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-wrong-key".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(!output.success, "wrong key should not match");
    let err = output.error.expect("should have timeout error");
    assert_eq!(err.code, TransactionErrorCode::WaitConditionTimeout);
}

#[test]
fn acp_accepted_via_key_empty_probe_times_out() {
    let mut provider = MockProvider::default();
    // acp_probe is default (empty)

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedViaKey {
        key: "enter".to_string(),
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-empty-probe".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(!output.success, "empty probe should not satisfy condition");
}

// ============================================================
// ACP proof conditions: acpAcceptedCursorAt
// ============================================================

#[test]
fn acp_accepted_cursor_at_succeeds() {
    let mut provider = MockProvider::default();
    provider.acp_probe.accepted_items = vec![make_accepted_item("enter")];

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedCursorAt {
        index: 9, // matches cursor_after in make_accepted_item
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-cursor-at".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(output.success, "cursor_after=9 should match index=9");
}

#[test]
fn acp_accepted_cursor_at_wrong_index_times_out() {
    let mut provider = MockProvider::default();
    provider.acp_probe.accepted_items = vec![make_accepted_item("enter")];

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedCursorAt {
        index: 42, // does not match cursor_after=9
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-cursor-wrong".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(!output.success);
}

#[test]
fn acp_accepted_cursor_at_empty_probe_times_out() {
    let mut provider = MockProvider::default();

    let condition =
        WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedCursorAt { index: 9 });

    let output = execute_wait_for(
        &mut provider,
        "wf-cursor-empty".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(
        !output.success,
        "empty probe should not satisfy cursor condition"
    );
}

// ============================================================
// ACP proof conditions: acpInputLayoutMatch
// ============================================================

#[test]
fn acp_input_layout_match_succeeds() {
    let mut provider = MockProvider::default();
    provider.acp_probe.input_layout = Some(AcpInputLayoutTelemetry {
        char_count: 20,
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 9,
    });

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpInputLayoutMatch {
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 9,
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-layout-ok".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(output.success, "matching layout should succeed");
}

#[test]
fn acp_input_layout_match_partial_mismatch_times_out() {
    let mut provider = MockProvider::default();
    provider.acp_probe.input_layout = Some(AcpInputLayoutTelemetry {
        char_count: 20,
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 9,
    });

    // cursor_in_window differs
    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpInputLayoutMatch {
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 5,
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-layout-mismatch".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(!output.success);
}

#[test]
fn acp_input_layout_match_no_layout_times_out() {
    let mut provider = MockProvider::default();
    // input_layout is None (default)

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpInputLayoutMatch {
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 9,
    });

    let output = execute_wait_for(
        &mut provider,
        "wf-layout-none".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute_wait_for");

    assert!(
        !output.success,
        "no layout data should not satisfy layout condition"
    );
}

// ============================================================
// Stale probe state after reset must not satisfy assertions
// ============================================================

#[test]
fn stale_probe_state_does_not_satisfy_after_reset() {
    let mut provider = MockProvider::default();

    // Simulate a previous session: probe has accepted items and layout
    provider.acp_probe.event_seq = 5;
    provider.acp_probe.accepted_items = vec![make_accepted_item("enter")];
    provider.acp_probe.input_layout = Some(AcpInputLayoutTelemetry {
        char_count: 20,
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 9,
    });

    // Verify the condition matches BEFORE reset
    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedViaKey {
        key: "enter".to_string(),
    });
    let before_reset = execute_wait_for(
        &mut provider,
        "wf-pre-reset".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("pre-reset check");
    assert!(before_reset.success, "should match before reset");

    // Simulate probe reset (what resetAcpTestProbe does)
    provider.acp_probe = AcpTestProbeSnapshot::default();

    // Verify the same condition does NOT match after reset
    let after_reset = execute_wait_for(
        &mut provider,
        "wf-post-reset".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("post-reset check");
    assert!(
        !after_reset.success,
        "reset probe must not satisfy a fresh assertion"
    );

    // Also verify layout condition doesn't match
    let layout_condition = WaitCondition::Detailed(WaitDetailedCondition::AcpInputLayoutMatch {
        visible_start: 0,
        visible_end: 15,
        cursor_in_window: 9,
    });
    let layout_after_reset = execute_wait_for(
        &mut provider,
        "wf-layout-post-reset".to_string(),
        &layout_condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("layout post-reset check");
    assert!(
        !layout_after_reset.success,
        "reset probe layout must not satisfy a fresh layout assertion"
    );
}

#[test]
fn stale_probe_accepted_label_does_not_satisfy_after_reset() {
    let mut provider = MockProvider::default();
    provider.acp_probe.accepted_items = vec![make_accepted_item("enter")];

    let condition = WaitCondition::Detailed(WaitDetailedCondition::AcpAcceptedLabel {
        label: "Context".to_string(),
    });

    let before = execute_wait_for(
        &mut provider,
        "wf-label-pre".to_string(),
        &condition,
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("pre-reset");
    assert!(before.success);

    provider.acp_probe = AcpTestProbeSnapshot::default();

    let after = execute_wait_for(
        &mut provider,
        "wf-label-post".to_string(),
        &condition,
        Some(50),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("post-reset");
    assert!(
        !after.success,
        "reset probe must not satisfy label assertion"
    );
}
