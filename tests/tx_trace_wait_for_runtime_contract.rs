use script_kit_gpui::protocol::transaction_executor::{
    execute_wait_for, matches_state_spec, TransactionStateProvider,
};
use script_kit_gpui::protocol::{
    AgentChatInputLayoutTelemetry, AgentChatPickerItemAcceptedTelemetry,
    AgentChatTestProbeSnapshot, StateMatchSpec, TransactionErrorCode, TransactionTraceMode,
    UiStateSnapshot, WaitCondition, WaitDetailedCondition,
};

#[derive(Clone, Default)]
struct Provider {
    snapshot: UiStateSnapshot,
    probe: AgentChatTestProbeSnapshot,
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

    fn agent_chat_test_probe(&self, _tail: usize) -> AgentChatTestProbeSnapshot {
        self.probe.clone()
    }
}

#[test]
fn state_match_checks_prompt_type() {
    let snapshot = UiStateSnapshot {
        prompt_type: Some("agentChatChat".to_string()),
        input_value: Some("/".to_string()),
        window_visible: true,
        ..Default::default()
    };

    assert!(matches_state_spec(
        &snapshot,
        &StateMatchSpec {
            prompt_type: Some("agentChatChat".to_string()),
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
fn agent_chat_input_match_and_contains_use_snapshot_input_value() {
    let mut provider = Provider {
        snapshot: UiStateSnapshot {
            input_value: Some("/new-script".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let exact = execute_wait_for(
        &mut provider,
        unique_request_id("agent_chat-input-match"),
        &WaitCondition::Detailed(WaitDetailedCondition::AgentChatInputMatch {
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
        unique_request_id("agent_chat-input-contains"),
        &WaitCondition::Detailed(WaitDetailedCondition::AgentChatInputContains {
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
            agent_chat_status: Some("idle".to_string()),
            agent_chat_context_ready: true,
            agent_chat_picker_open: true,
            agent_chat_cursor_index: Some(1),
            ..Default::default()
        },
        probe: AgentChatTestProbeSnapshot {
            accepted_items: vec![AgentChatPickerItemAcceptedTelemetry {
                item_label: "New Script".to_string(),
                item_id: "skill:new-script".to_string(),
                trigger: "/".to_string(),
                accepted_via_key: "tab".to_string(),
                cursor_after: 12,
                caused_submit: false,
            }],
            input_layout: Some(AgentChatInputLayoutTelemetry {
                char_count: 1,
                visible_start: 0,
                visible_end: 1,
                cursor_in_window: 1,
            }),
            ..Default::default()
        },
    };

    for condition in [
        WaitDetailedCondition::AgentChatReady,
        WaitDetailedCondition::AgentChatPickerOpen,
        WaitDetailedCondition::AgentChatStatus {
            status: "idle".to_string(),
        },
        WaitDetailedCondition::AgentChatCursorAt { index: 1 },
        WaitDetailedCondition::AgentChatInputMatch {
            text: "/".to_string(),
        },
        WaitDetailedCondition::AgentChatInputContains {
            substring: "/".to_string(),
        },
        WaitDetailedCondition::AgentChatItemAccepted,
        WaitDetailedCondition::AgentChatAcceptedViaKey {
            key: "tab".to_string(),
        },
        WaitDetailedCondition::AgentChatAcceptedLabel {
            label: "New Script".to_string(),
        },
        WaitDetailedCondition::AgentChatAcceptedCursorAt { index: 12 },
        WaitDetailedCondition::AgentChatInputLayoutMatch {
            visible_start: 0,
            visible_end: 1,
            cursor_in_window: 1,
        },
    ] {
        let result = execute_wait_for(
            &mut provider,
            unique_request_id("agent_chat-runtime"),
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
        unique_request_id("agent_chat-setup-unsupported"),
        &WaitCondition::Detailed(WaitDetailedCondition::AgentChatSetupVisible),
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
