use script_kit_gpui::protocol::transaction_executor::{
    execute_wait_for, matches_state_spec, TransactionStateProvider,
};
use script_kit_gpui::protocol::{
    AcpInputLayoutTelemetry, AcpPickerItemAcceptedTelemetry, AcpTestProbeSnapshot, StateMatchSpec,
    TransactionErrorCode, TransactionTraceMode, UiStateSnapshot, WaitCondition,
    WaitDetailedCondition,
};

#[derive(Clone, Default)]
struct Provider {
    snapshot: UiStateSnapshot,
    probe: AcpTestProbeSnapshot,
}

impl TransactionStateProvider for Provider {
    fn snapshot(&self) -> UiStateSnapshot {
        self.snapshot.clone()
    }

    fn set_input(&mut self, _text: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn select_by_value(&mut self, _value: &str, _submit: bool) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    fn select_by_semantic_id(
        &mut self,
        _semantic_id: &str,
        _submit: bool,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    fn acp_test_probe(&self, _tail: usize) -> AcpTestProbeSnapshot {
        self.probe.clone()
    }
}

#[test]
fn state_match_checks_prompt_type() {
    let snapshot = UiStateSnapshot {
        prompt_type: Some("acpChat".to_string()),
        input_value: Some("/".to_string()),
        window_visible: true,
        ..Default::default()
    };

    assert!(matches_state_spec(
        &snapshot,
        &StateMatchSpec {
            prompt_type: Some("acpChat".to_string()),
            input_value: Some("/".to_string()),
            window_visible: Some(true),
            ..Default::default()
        }
    ));
    assert!(!matches_state_spec(
        &snapshot,
        &StateMatchSpec {
            prompt_type: Some("arg".to_string()),
            ..Default::default()
        }
    ));
}

#[test]
fn acp_input_match_and_contains_use_snapshot_input_value() {
    let mut provider = Provider {
        snapshot: UiStateSnapshot {
            input_value: Some("/new-script".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let exact = execute_wait_for(
        &mut provider,
        unique_request_id("acp-input-match"),
        &WaitCondition::Detailed(WaitDetailedCondition::AcpInputMatch {
            text: "/new-script".to_string(),
        }),
        Some(1),
        Some(1),
        TransactionTraceMode::Off,
    )
    .expect("exact wait should run");
    assert!(exact.success);

    let contains = execute_wait_for(
        &mut provider,
        unique_request_id("acp-input-contains"),
        &WaitCondition::Detailed(WaitDetailedCondition::AcpInputContains {
            substring: "new".to_string(),
        }),
        Some(1),
        Some(1),
        TransactionTraceMode::Off,
    )
    .expect("contains wait should run");
    assert!(contains.success);
}

#[test]
fn every_wait_detailed_condition_has_runtime_match_arm_or_invalid_error() {
    let mut provider = Provider {
        snapshot: UiStateSnapshot {
            input_value: Some("/".to_string()),
            acp_status: Some("idle".to_string()),
            acp_context_ready: true,
            acp_picker_open: true,
            acp_cursor_index: Some(1),
            ..Default::default()
        },
        probe: AcpTestProbeSnapshot {
            accepted_items: vec![AcpPickerItemAcceptedTelemetry {
                item_label: "New Script".to_string(),
                item_id: "skill:new-script".to_string(),
                trigger: "/".to_string(),
                accepted_via_key: "tab".to_string(),
                cursor_after: 12,
                caused_submit: false,
            }],
            input_layout: Some(AcpInputLayoutTelemetry {
                char_count: 1,
                visible_start: 0,
                visible_end: 1,
                cursor_in_window: 1,
            }),
            ..Default::default()
        },
    };

    for condition in [
        WaitDetailedCondition::AcpReady,
        WaitDetailedCondition::AcpPickerOpen,
        WaitDetailedCondition::AcpStatus {
            status: "idle".to_string(),
        },
        WaitDetailedCondition::AcpCursorAt { index: 1 },
        WaitDetailedCondition::AcpInputMatch {
            text: "/".to_string(),
        },
        WaitDetailedCondition::AcpInputContains {
            substring: "/".to_string(),
        },
        WaitDetailedCondition::AcpItemAccepted,
        WaitDetailedCondition::AcpAcceptedViaKey {
            key: "tab".to_string(),
        },
        WaitDetailedCondition::AcpAcceptedLabel {
            label: "New Script".to_string(),
        },
        WaitDetailedCondition::AcpAcceptedCursorAt { index: 12 },
        WaitDetailedCondition::AcpInputLayoutMatch {
            visible_start: 0,
            visible_end: 1,
            cursor_in_window: 1,
        },
    ] {
        let result = execute_wait_for(
            &mut provider,
            unique_request_id("acp-runtime"),
            &WaitCondition::Detailed(condition),
            Some(1),
            Some(1),
            TransactionTraceMode::Off,
        )
        .expect("condition should execute");
        assert!(result.success, "condition should be satisfied");
    }

    let unsupported = execute_wait_for(
        &mut provider,
        unique_request_id("acp-setup-unsupported"),
        &WaitCondition::Detailed(WaitDetailedCondition::AcpSetupVisible),
        Some(1),
        Some(1),
        TransactionTraceMode::Off,
    )
    .expect("unsupported setup wait should return a typed result");
    assert!(!unsupported.success);
    assert_eq!(
        unsupported.error.unwrap().code,
        TransactionErrorCode::InvalidCondition
    );

    let source = std::fs::read_to_string("src/protocol/transaction_executor.rs")
        .expect("read transaction executor");
    assert!(
        !source.contains("_ => (false, Vec::new())"),
        "accepted public conditions must not fall through to permanent false"
    );
}

fn unique_request_id(prefix: &str) -> String {
    format!("{prefix}-{}-{}", std::process::id(), uuidish())
}

fn uuidish() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
