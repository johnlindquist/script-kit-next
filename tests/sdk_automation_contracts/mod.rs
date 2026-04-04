//! Contract tests verifying that wait/batch error, trace, and SDK-reference
//! contracts are normalized to the repo's structured diagnostics pattern.

use script_kit_gpui::protocol::{
    BatchCommand, BatchOptions, Message, StateMatchSpec, TransactionError, TransactionErrorCode,
    TransactionTraceMode, UiStateSnapshot, WaitCondition, WaitDetailedCondition,
    WaitNamedCondition,
};
use script_kit_gpui::protocol::transaction_executor::{
    execute_batch, execute_wait_for, TransactionStateProvider,
};

// ---------------------------------------------------------------------------
// Mock state provider
// ---------------------------------------------------------------------------

struct MockProvider {
    input_value: String,
    selected_value: Option<String>,
    window_visible: bool,
    window_focused: bool,
    choice_count: usize,
    visible_semantic_ids: Vec<String>,
    focused_semantic_id: Option<String>,
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
        if self.visible_semantic_ids.iter().any(|id| id.contains(value)) {
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
}

// ============================================================
// TransactionErrorCode — stable wire codes
// ============================================================

#[test]
fn error_code_wait_condition_timeout_serializes_to_snake_case() {
    let code = TransactionErrorCode::WaitConditionTimeout;
    let json = serde_json::to_value(&code).expect("serialize");
    assert_eq!(json, "wait_condition_timeout");
}

#[test]
fn error_code_element_not_found_serializes_to_snake_case() {
    let code = TransactionErrorCode::ElementNotFound;
    let json = serde_json::to_value(&code).expect("serialize");
    assert_eq!(json, "element_not_found");
}

#[test]
fn error_code_unsupported_prompt_serializes_to_snake_case() {
    let code = TransactionErrorCode::UnsupportedPrompt;
    let json = serde_json::to_value(&code).expect("serialize");
    assert_eq!(json, "unsupported_prompt");
}

#[test]
fn error_code_selection_not_found_serializes_to_snake_case() {
    let code = TransactionErrorCode::SelectionNotFound;
    let json = serde_json::to_value(&code).expect("serialize");
    assert_eq!(json, "selection_not_found");
}

#[test]
fn error_code_unsupported_command_serializes_to_snake_case() {
    let code = TransactionErrorCode::UnsupportedCommand;
    let json = serde_json::to_value(&code).expect("serialize");
    assert_eq!(json, "unsupported_command");
}

#[test]
fn error_code_action_failed_serializes_to_snake_case() {
    let code = TransactionErrorCode::ActionFailed;
    let json = serde_json::to_value(&code).expect("serialize");
    assert_eq!(json, "action_failed");
}

#[test]
fn all_error_codes_round_trip() {
    let codes = vec![
        TransactionErrorCode::WaitConditionTimeout,
        TransactionErrorCode::ElementNotFound,
        TransactionErrorCode::SelectionNotFound,
        TransactionErrorCode::InvalidCondition,
        TransactionErrorCode::UnsupportedCommand,
        TransactionErrorCode::UnsupportedPrompt,
        TransactionErrorCode::ActionFailed,
    ];
    for code in codes {
        let json = serde_json::to_value(&code).expect("serialize");
        let back: TransactionErrorCode = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, code);
    }
}

// ============================================================
// TransactionError helper constructors
// ============================================================

#[test]
fn error_helper_element_not_found_has_correct_shape() {
    let err = TransactionError::element_not_found("choice:99:missing");
    assert_eq!(err.code, TransactionErrorCode::ElementNotFound);
    assert!(err.message.contains("choice:99:missing"));
    assert!(err.suggestion.is_some());
}

#[test]
fn error_helper_unsupported_prompt_has_correct_shape() {
    let err = TransactionError::unsupported_prompt("div prompt does not support selection");
    assert_eq!(err.code, TransactionErrorCode::UnsupportedPrompt);
    assert!(err.message.contains("div prompt"));
    assert!(err.suggestion.is_some());
}

#[test]
fn error_helper_wait_timeout_has_correct_shape() {
    let err = TransactionError::wait_timeout("Timeout after 5000ms");
    assert_eq!(err.code, TransactionErrorCode::WaitConditionTimeout);
    assert!(err.suggestion.is_none());
}

// ============================================================
// Batch trace stays at message top level, not inside BatchOptions
// ============================================================

#[test]
fn batch_message_trace_is_top_level() {
    let json = serde_json::json!({
        "type": "batch",
        "requestId": "b-trace",
        "commands": [{"type": "setInput", "text": "x"}],
        "options": {"stopOnError": true, "timeout": 1000},
        "trace": "onFailure"
    });
    let msg: Message = serde_json::from_value(json).expect("parse batch with top-level trace");
    match msg {
        Message::Batch { trace, options, .. } => {
            assert_eq!(trace, TransactionTraceMode::OnFailure);
            // options should NOT contain trace
            let opts_json = serde_json::to_value(&options).expect("serialize options");
            assert!(
                opts_json.get("trace").is_none(),
                "BatchOptions must not contain trace"
            );
        }
        other => panic!("Expected Batch, got: {other:?}"),
    }
}

#[test]
fn batch_options_serializes_only_execution_fields() {
    let opts = BatchOptions {
        stop_on_error: true,
        rollback_on_error: false,
        timeout: 3000,
    };
    let json = serde_json::to_value(&opts).expect("serialize");
    assert!(json.get("stopOnError").is_some());
    assert!(json.get("timeout").is_some());
    assert!(
        json.get("trace").is_none(),
        "trace must not be in BatchOptions"
    );
}

// ============================================================
// waitFor — success path produces correct result shape
// ============================================================

#[test]
fn wait_for_success_has_no_error_and_no_trace_when_off() {
    let mut provider = MockProvider::default();
    provider.input_value = "match".to_string();

    let output = execute_wait_for(
        &mut provider,
        "wf-ok".to_string(),
        &WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
            state: StateMatchSpec {
                input_value: Some("match".to_string()),
                ..Default::default()
            },
        }),
        Some(100),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute");

    assert!(output.success);
    assert!(output.error.is_none());
    assert!(output.trace.is_none());
}

// ============================================================
// waitFor — failure path produces structured error with code
// ============================================================

#[test]
fn wait_for_timeout_returns_structured_error_with_code() {
    let mut provider = MockProvider::default();
    provider.input_value = "wrong".to_string();

    let output = execute_wait_for(
        &mut provider,
        "wf-fail".to_string(),
        &WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
            state: StateMatchSpec {
                input_value: Some("never".to_string()),
                ..Default::default()
            },
        }),
        Some(30),
        Some(10),
        TransactionTraceMode::Off,
    )
    .expect("execute");

    assert!(!output.success);
    let err = output.error.expect("error on timeout");
    assert_eq!(err.code, TransactionErrorCode::WaitConditionTimeout);
    assert!(!err.message.is_empty());

    // Verify wire shape
    let json = serde_json::to_value(&err).expect("serialize");
    assert_eq!(json["code"], "wait_condition_timeout");
    assert!(json["message"].is_string());
}

// ============================================================
// waitFor — trace-on-failure includes trace only on failure
// ============================================================

#[test]
fn wait_for_trace_on_failure_includes_trace_on_timeout() {
    let mut provider = MockProvider::default();

    let output = execute_wait_for(
        &mut provider,
        "wf-trace".to_string(),
        &WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
            state: StateMatchSpec {
                input_value: Some("never".to_string()),
                ..Default::default()
            },
        }),
        Some(30),
        Some(10),
        TransactionTraceMode::OnFailure,
    )
    .expect("execute");

    assert!(!output.success);
    assert!(output.trace.is_some(), "trace should be present on failure");
}

#[test]
fn wait_for_trace_on_failure_omits_trace_on_success() {
    let mut provider = MockProvider::default();
    provider.input_value = "match".to_string();

    let output = execute_wait_for(
        &mut provider,
        "wf-trace-ok".to_string(),
        &WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
            state: StateMatchSpec {
                input_value: Some("match".to_string()),
                ..Default::default()
            },
        }),
        Some(100),
        Some(10),
        TransactionTraceMode::OnFailure,
    )
    .expect("execute");

    assert!(output.success);
    assert!(output.trace.is_none(), "trace should be absent on success with onFailure mode");
}

// ============================================================
// batch — failure includes structured error with code
// ============================================================

#[test]
fn batch_selection_failure_returns_structured_error() {
    let mut provider = MockProvider::default();

    let commands = vec![BatchCommand::SelectByValue {
        value: "nonexistent".to_string(),
        submit: false,
    }];

    let output = execute_batch(
        &mut provider,
        "b-sel-fail".to_string(),
        &commands,
        None,
        TransactionTraceMode::Off,
    )
    .expect("execute");

    assert!(!output.success);
    assert_eq!(output.failed_at, Some(0));
    let err = output.results[0].error.as_ref().expect("error on selection failure");
    assert_eq!(err.code, TransactionErrorCode::SelectionNotFound);
}

#[test]
fn batch_unsupported_command_returns_structured_error() {
    let mut provider = MockProvider::default();

    let commands = vec![BatchCommand::TypeAndSubmit {
        text: "hello".to_string(),
    }];

    let output = execute_batch(
        &mut provider,
        "b-unsup".to_string(),
        &commands,
        None,
        TransactionTraceMode::Off,
    )
    .expect("execute");

    assert!(!output.success);
    let err = output.results[0].error.as_ref().expect("error");
    assert_eq!(err.code, TransactionErrorCode::UnsupportedCommand);
}

// ============================================================
// batch — trace-on-failure behavior
// ============================================================

#[test]
fn batch_trace_on_failure_includes_trace_and_commands() {
    let mut provider = MockProvider::default();

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
        "b-trace-fail".to_string(),
        &commands,
        None,
        TransactionTraceMode::OnFailure,
    )
    .expect("execute");

    assert!(!output.success);
    let trace = output.trace.expect("trace on failure");
    assert_eq!(trace.request_id, "b-trace-fail");
    assert!(!trace.commands.is_empty());
    assert!(trace.failed_at.is_some());
}

#[test]
fn batch_trace_on_failure_omits_trace_on_success() {
    let mut provider = MockProvider::default();

    let commands = vec![BatchCommand::SetInput {
        text: "ok".to_string(),
    }];

    let output = execute_batch(
        &mut provider,
        "b-trace-ok".to_string(),
        &commands,
        None,
        TransactionTraceMode::OnFailure,
    )
    .expect("execute");

    assert!(output.success);
    assert!(output.trace.is_none(), "trace should be absent on success with onFailure mode");
}

// ============================================================
// batch — success path shape
// ============================================================

#[test]
fn batch_success_has_correct_result_shape() {
    let mut provider = MockProvider::default();

    let commands = vec![
        BatchCommand::SetInput {
            text: "test".to_string(),
        },
        BatchCommand::WaitFor {
            condition: WaitCondition::Detailed(WaitDetailedCondition::StateMatch {
                state: StateMatchSpec {
                    input_value: Some("test".to_string()),
                    ..Default::default()
                },
            }),
            timeout: Some(100),
            poll_interval: Some(10),
        },
    ];

    let output = execute_batch(
        &mut provider,
        "b-shape".to_string(),
        &commands,
        None,
        TransactionTraceMode::Off,
    )
    .expect("execute");

    assert!(output.success);
    assert_eq!(output.results.len(), 2);
    assert!(output.failed_at.is_none());
    assert!(output.total_elapsed < 1000);

    // Each result entry has the expected fields
    assert_eq!(output.results[0].command, "setInput");
    assert!(output.results[0].elapsed.is_some());
    assert!(output.results[0].error.is_none());

    assert_eq!(output.results[1].command, "waitFor");
    assert!(output.results[1].elapsed.is_some());
    assert!(output.results[1].error.is_none());
}

// ============================================================
// kit://sdk-reference — descriptions match wire contract
// ============================================================

#[test]
fn sdk_reference_wait_for_description_documents_error_codes() {
    let content =
        script_kit_gpui::mcp_resources::read_resource("kit://sdk-reference", &[], &[], None)
            .expect("should resolve");
    let doc: serde_json::Value = serde_json::from_str(&content.text).expect("parse JSON");

    let functions = doc["functions"].as_array().expect("functions array");
    let wait_for = functions
        .iter()
        .find(|f| f["name"] == "waitFor")
        .expect("waitFor in sdk-reference");

    let desc = wait_for["description"].as_str().expect("description string");
    assert!(
        desc.contains("wait_condition_timeout"),
        "waitFor description must document wait_condition_timeout code"
    );
    assert!(
        desc.contains("element_not_found"),
        "waitFor description must document element_not_found code"
    );
    assert!(
        desc.contains("unsupported_prompt"),
        "waitFor description must document unsupported_prompt code"
    );
    assert!(
        desc.contains("trace"),
        "waitFor description must mention trace"
    );
}

#[test]
fn sdk_reference_batch_description_documents_error_codes_and_trace() {
    let content =
        script_kit_gpui::mcp_resources::read_resource("kit://sdk-reference", &[], &[], None)
            .expect("should resolve");
    let doc: serde_json::Value = serde_json::from_str(&content.text).expect("parse JSON");

    let functions = doc["functions"].as_array().expect("functions array");
    let batch = functions
        .iter()
        .find(|f| f["name"] == "batch")
        .expect("batch in sdk-reference");

    let desc = batch["description"].as_str().expect("description string");
    assert!(
        desc.contains("failedAt"),
        "batch description must document failedAt"
    );
    assert!(
        desc.contains("totalElapsed"),
        "batch description must document totalElapsed"
    );
    assert!(
        desc.contains("trace"),
        "batch description must mention trace"
    );
    assert!(
        desc.contains("not inside options"),
        "batch description must clarify trace is top-level, not inside options"
    );
    assert!(
        desc.contains("action_failed"),
        "batch description must document action_failed code"
    );
}
