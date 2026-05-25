pub const THINKING_LABEL: &str = "Thinking...";

use super::render_actions::is_action_enabled_for_snapshot;
use super::state::InlineAgentRunState;
use super::types::{InlineAgentOutputAction, InlineAgentSnapshot, INLINE_AGENT_INPUT_PLACEHOLDER};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentCompactViewModel {
    pub app_badge: String,
    pub metrics_label: String,
    pub support_chip: InlineAgentSupportChip,
    pub instruction_text: String,
    pub input_placeholder: &'static str,
    pub thinking_visible: bool,
    pub thinking_label: Option<&'static str>,
    pub output_preview: Option<String>,
    pub actions: Vec<InlineAgentActionViewModel>,
    pub stop_enabled: bool,
    pub retry_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineAgentSupportChip {
    Editable,
    CopyOnly,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineAgentActionViewModel {
    pub action: InlineAgentOutputAction,
    pub enabled: bool,
}

pub fn compact_view_model(
    snapshot: &InlineAgentSnapshot,
    state: &InlineAgentRunState,
) -> InlineAgentCompactViewModel {
    InlineAgentCompactViewModel {
        app_badge: snapshot.app.name.clone(),
        metrics_label: format!(
            "{} chars · ~{} tokens",
            snapshot.metrics.chars, snapshot.metrics.estimated_tokens
        ),
        support_chip: support_chip(snapshot),
        instruction_text: String::new(),
        input_placeholder: INLINE_AGENT_INPUT_PLACEHOLDER,
        thinking_visible: matches!(
            state,
            InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. }
        ),
        thinking_label: matches!(
            state,
            InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. }
        )
        .then_some(THINKING_LABEL),
        output_preview: latest_output_preview(state),
        actions: [
            InlineAgentOutputAction::Replace,
            InlineAgentOutputAction::Append,
            InlineAgentOutputAction::Copy,
            InlineAgentOutputAction::Chat,
        ]
        .into_iter()
        .map(|action| InlineAgentActionViewModel {
            action,
            enabled: is_action_enabled_for_snapshot(action, state, snapshot),
        })
        .collect(),
        stop_enabled: is_stop_enabled(state),
        retry_enabled: is_retry_enabled(state),
    }
}

fn support_chip(snapshot: &InlineAgentSnapshot) -> InlineAgentSupportChip {
    match (
        snapshot.capabilities.can_replace || snapshot.capabilities.can_append,
        snapshot.capabilities.can_copy,
    ) {
        (true, _) => InlineAgentSupportChip::Editable,
        (false, true) => InlineAgentSupportChip::CopyOnly,
        (false, false) => InlineAgentSupportChip::Unsupported,
    }
}

fn latest_output_preview(state: &InlineAgentRunState) -> Option<String> {
    match state {
        InlineAgentRunState::Streaming { partial_output, .. } if !partial_output.is_empty() => {
            Some(partial_output.clone())
        }
        _ => state.latest_complete_output().map(ToOwned::to_owned),
    }
}

pub fn is_stop_enabled(state: &InlineAgentRunState) -> bool {
    matches!(
        state,
        InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. }
    )
}

pub fn is_retry_enabled(state: &InlineAgentRunState) -> bool {
    matches!(
        state,
        InlineAgentRunState::Error {
            retryable: true,
            ..
        }
    )
}
