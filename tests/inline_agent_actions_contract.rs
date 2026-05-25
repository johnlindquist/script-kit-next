use script_kit_gpui::inline_agent::render_actions::{
    apply_latest_output_action, is_action_enabled, is_action_enabled_for_snapshot,
    latest_output_mutation_for_action,
};
use script_kit_gpui::inline_agent::{InlineAgentAnchor, InlineAgentSnapshot};
use script_kit_gpui::inline_agent::{
    InlineAgentMutationReceipt, InlineAgentOutputAction, InlineAgentPlatformBridge,
    InlineAgentRunState, InlineAgentTextMutation,
};
use script_kit_gpui::platform::accessibility::focused_text::focused_text_snapshot_for_tests;

#[derive(Debug, Default)]
struct RecordingBridge {
    receipt: Option<InlineAgentMutationReceipt>,
}

impl InlineAgentPlatformBridge for RecordingBridge {
    fn capture_focused_text_snapshot(&self) -> anyhow::Result<InlineAgentSnapshot> {
        Ok(inline_snapshot_for_tests("hello"))
    }

    fn apply_text_mutation(
        &self,
        _anchor: &InlineAgentAnchor,
        mutation: InlineAgentTextMutation,
    ) -> anyhow::Result<InlineAgentMutationReceipt> {
        let action = match mutation {
            InlineAgentTextMutation::Replace { .. } => InlineAgentOutputAction::Replace,
            InlineAgentTextMutation::Append { .. } => InlineAgentOutputAction::Append,
            InlineAgentTextMutation::Copy { .. } => InlineAgentOutputAction::Copy,
        };
        Ok(self.receipt.clone().unwrap_or(InlineAgentMutationReceipt {
            action,
            success: true,
            message: None,
        }))
    }
}

fn inline_snapshot_for_tests(text: &str) -> InlineAgentSnapshot {
    let focused = focused_text_snapshot_for_tests(text);
    InlineAgentSnapshot {
        session_id: focused.session_id,
        app: focused.app,
        text: focused.text,
        metrics: focused.metrics,
        capabilities: focused.capabilities,
        anchor: script_kit_gpui::inline_agent::types::InlineAgentAnchor {
            geometry: focused.geometry,
        },
    }
}

#[test]
fn output_actions_require_latest_complete_output_except_chat() {
    assert!(!is_action_enabled(
        InlineAgentOutputAction::Replace,
        &InlineAgentRunState::Idle
    ));
    assert!(is_action_enabled(
        InlineAgentOutputAction::Copy,
        &InlineAgentRunState::Completed {
            output: "done".to_string()
        }
    ));
    assert!(is_action_enabled(
        InlineAgentOutputAction::Append,
        &InlineAgentRunState::Completed {
            output: "done".to_string()
        }
    ));
    assert!(!is_action_enabled(
        InlineAgentOutputAction::Chat,
        &InlineAgentRunState::Applying {
            action: InlineAgentOutputAction::Replace,
            latest_output: Some("done".to_string())
        }
    ));
}

#[test]
fn applied_and_error_states_preserve_latest_output_for_actions() {
    let snapshot = inline_snapshot_for_tests("hello");
    let applied = InlineAgentRunState::Applied {
        action: InlineAgentOutputAction::Replace,
        output: "rewritten text".to_string(),
    };

    assert_eq!(applied.latest_complete_output(), Some("rewritten text"));
    assert!(is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Replace,
        &applied,
        &snapshot
    ));
    assert!(is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Copy,
        &applied,
        &snapshot
    ));

    let error = InlineAgentRunState::Error {
        message: "target disappeared".to_string(),
        retryable: true,
        latest_output: Some("rewritten text".to_string()),
    };

    assert_eq!(error.latest_complete_output(), Some("rewritten text"));
    assert!(is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Copy,
        &error,
        &snapshot
    ));
}

#[test]
fn applying_preserves_preview_output_but_disables_actions() {
    let snapshot = inline_snapshot_for_tests("hello");
    let applying = InlineAgentRunState::Applying {
        action: InlineAgentOutputAction::Append,
        latest_output: Some("rewritten text".to_string()),
    };

    assert_eq!(applying.latest_complete_output(), Some("rewritten text"));
    assert!(!is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Copy,
        &applying,
        &snapshot
    ));
    assert!(!is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Chat,
        &applying,
        &snapshot
    ));
}

#[test]
fn output_actions_honor_focused_field_capabilities() {
    let focused = focused_text_snapshot_for_tests("hello");
    let mut snapshot = inline_snapshot_for_tests(&focused.text);
    snapshot.capabilities.can_replace = false;
    snapshot.capabilities.can_append = false;
    let state = InlineAgentRunState::Completed {
        output: "done".to_string(),
    };

    assert!(!is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Replace,
        &state,
        &snapshot
    ));
    assert!(!is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Append,
        &state,
        &snapshot
    ));
    assert!(is_action_enabled_for_snapshot(
        InlineAgentOutputAction::Copy,
        &state,
        &snapshot
    ));
}

#[test]
fn latest_output_actions_resolve_to_session_scoped_mutations() {
    let snapshot = inline_snapshot_for_tests("hello");
    let state = InlineAgentRunState::Completed {
        output: "done".to_string(),
    };

    assert_eq!(
        latest_output_mutation_for_action(InlineAgentOutputAction::Replace, &state, &snapshot),
        Some(InlineAgentTextMutation::Replace {
            session_id: snapshot.session_id.clone(),
            text: "done".to_string()
        })
    );
    assert_eq!(
        latest_output_mutation_for_action(InlineAgentOutputAction::Append, &state, &snapshot),
        Some(InlineAgentTextMutation::Append {
            session_id: snapshot.session_id.clone(),
            text: "done".to_string()
        })
    );
    assert_eq!(
        latest_output_mutation_for_action(InlineAgentOutputAction::Copy, &state, &snapshot),
        Some(InlineAgentTextMutation::Copy {
            text: "done".to_string()
        })
    );
    assert_eq!(
        latest_output_mutation_for_action(InlineAgentOutputAction::Chat, &state, &snapshot),
        None
    );
}

#[test]
fn apply_latest_output_action_routes_mutations_through_platform_bridge() {
    let snapshot = inline_snapshot_for_tests("hello");
    let state = InlineAgentRunState::Completed {
        output: "done".to_string(),
    };
    let bridge = RecordingBridge::default();

    let receipt =
        apply_latest_output_action(&bridge, InlineAgentOutputAction::Replace, &state, &snapshot)
            .expect("bridge apply should succeed")
            .expect("replace should produce a receipt");

    assert_eq!(receipt.action, InlineAgentOutputAction::Replace);
    assert!(receipt.success);
}
