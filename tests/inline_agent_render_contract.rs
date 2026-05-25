use script_kit_gpui::ai::inline_agent::history::InlineAgentTurn;
use script_kit_gpui::ai::inline_agent::types::InlineAgentEditSemantics;
use script_kit_gpui::inline_agent::render_compact::{
    compact_view_model, InlineAgentSupportChip, THINKING_LABEL,
};
use script_kit_gpui::inline_agent::render_expanded::{expanded_header_label, expanded_view_model};
use script_kit_gpui::inline_agent::{
    InlineAgentMutationReceipt, InlineAgentOutputAction, InlineAgentRunState,
    INLINE_AGENT_INPUT_PLACEHOLDER,
};
use script_kit_gpui::platform::accessibility::focused_text::focused_text_snapshot_for_tests;

fn snapshot_for_tests() -> script_kit_gpui::inline_agent::InlineAgentSnapshot {
    let mut focused = focused_text_snapshot_for_tests("hello world");
    focused.app.name = "Slack".to_string();
    script_kit_gpui::inline_agent::InlineAgentSnapshot {
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
fn compact_view_model_exposes_required_header_prompt_and_actions() {
    let snapshot = snapshot_for_tests();
    let view = compact_view_model(
        &snapshot,
        &InlineAgentRunState::Completed {
            output: "tightened copy".to_string(),
        },
    );

    assert_eq!(view.app_badge, "Slack");
    assert_eq!(view.metrics_label, "11 chars · ~3 tokens");
    assert_eq!(view.support_chip, InlineAgentSupportChip::Editable);
    assert_eq!(view.instruction_text, "");
    assert_eq!(view.input_placeholder, INLINE_AGENT_INPUT_PLACEHOLDER);
    assert_eq!(view.output_preview.as_deref(), Some("tightened copy"));
    assert!(view
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Replace && action.enabled));
    assert!(view
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Chat && action.enabled));
}

#[test]
fn compact_view_model_shows_thinking_state_without_output_preview() {
    let snapshot = snapshot_for_tests();
    let view = compact_view_model(
        &snapshot,
        &InlineAgentRunState::Thinking {
            request_id: "req-1".to_string(),
            started_at_ms: 1,
        },
    );

    assert!(view.thinking_visible);
    assert_eq!(view.thinking_label, Some(THINKING_LABEL));
    assert_eq!(view.output_preview, None);
    assert!(view.stop_enabled);
    assert!(!view.retry_enabled);
}

#[test]
fn compact_view_model_preserves_output_preview_after_apply_or_error() {
    let snapshot = snapshot_for_tests();
    let applied = compact_view_model(
        &snapshot,
        &InlineAgentRunState::Applied {
            action: InlineAgentOutputAction::Replace,
            output: "done".to_string(),
            receipt: InlineAgentMutationReceipt {
                action: InlineAgentOutputAction::Replace,
                success: true,
                changed_text: true,
                copied_to_clipboard: false,
                message: None,
            },
        },
    );

    assert_eq!(applied.output_preview.as_deref(), Some("done"));
    assert!(applied
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Copy && action.enabled));

    let error = compact_view_model(
        &snapshot,
        &InlineAgentRunState::Error {
            message: "target disappeared".to_string(),
            retryable: true,
            latest_output: Some("done".to_string()),
        },
    );

    assert_eq!(error.output_preview.as_deref(), Some("done"));
    assert!(!error.stop_enabled);
    assert!(error.retry_enabled);
    assert!(error
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Copy && action.enabled));
}

#[test]
fn compact_view_model_keeps_applying_preview_but_disables_actions() {
    let snapshot = snapshot_for_tests();
    let view = compact_view_model(
        &snapshot,
        &InlineAgentRunState::Applying {
            action: InlineAgentOutputAction::Copy,
            latest_output: Some("done".to_string()),
        },
    );

    assert_eq!(view.output_preview.as_deref(), Some("done"));
    assert!(view.actions.iter().all(|action| !action.enabled));
}

#[test]
fn expanded_view_model_projects_turns_and_latest_output() {
    let snapshot = snapshot_for_tests();
    let turns = vec![InlineAgentTurn {
        instruction: "make it shorter".to_string(),
        semantics: InlineAgentEditSemantics::Replace,
        assistant_output: Some("short".to_string()),
    }];
    let view = expanded_view_model(&snapshot, &turns, &InlineAgentRunState::Idle);

    assert_eq!(expanded_header_label(1), "Cue - 1 turn");
    assert_eq!(expanded_header_label(2), "Cue - 2 turns");
    assert_eq!(view.header_label, "Cue - 1 turn");
    assert_eq!(view.session_id, snapshot.session_id.to_string());
    assert_eq!(view.input_placeholder, INLINE_AGENT_INPUT_PLACEHOLDER);
    assert_eq!(view.instruction_text, "");
    assert_eq!(view.turns[0].user_instruction, "make it shorter");
    assert_eq!(view.latest_output.as_deref(), Some("short"));
    assert!(!view.stop_enabled);
    assert!(!view.retry_enabled);
    assert!(view
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Replace && action.enabled));
    assert!(view
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Append && action.enabled));
    assert!(view
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Copy && action.enabled));
    assert!(!view
        .actions
        .iter()
        .any(|action| action.action == InlineAgentOutputAction::Chat));
}
