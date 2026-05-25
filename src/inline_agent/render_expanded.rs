pub fn expanded_header_label(turn_count: usize) -> String {
    format!(
        "Cue - {turn_count} turn{}",
        if turn_count == 1 { "" } else { "s" }
    )
}

use crate::ai::inline_agent::history::InlineAgentTurn;

use super::render_actions::is_action_enabled_for_snapshot;
use super::render_compact::{is_retry_enabled, is_stop_enabled, InlineAgentActionViewModel};
use super::state::InlineAgentRunState;
use super::types::{InlineAgentOutputAction, InlineAgentSnapshot, INLINE_AGENT_INPUT_PLACEHOLDER};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentExpandedViewModel {
    pub header_label: String,
    pub session_id: String,
    pub app_badge: String,
    pub input_placeholder: &'static str,
    pub instruction_text: String,
    pub turns: Vec<InlineAgentTurnViewModel>,
    pub latest_output: Option<String>,
    pub actions: Vec<InlineAgentActionViewModel>,
    pub stop_enabled: bool,
    pub retry_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentTurnViewModel {
    pub user_instruction: String,
    pub assistant_output: Option<String>,
}

pub fn expanded_view_model(
    snapshot: &InlineAgentSnapshot,
    turns: &[InlineAgentTurn],
    state: &InlineAgentRunState,
) -> InlineAgentExpandedViewModel {
    let latest_output = latest_complete_output(state, turns);
    let action_state = action_state_with_latest_output(state, latest_output.as_deref());

    InlineAgentExpandedViewModel {
        header_label: expanded_header_label(turns.len()),
        session_id: snapshot.session_id.to_string(),
        app_badge: snapshot.app.name.clone(),
        input_placeholder: INLINE_AGENT_INPUT_PLACEHOLDER,
        instruction_text: String::new(),
        turns: turns
            .iter()
            .map(|turn| InlineAgentTurnViewModel {
                user_instruction: turn.instruction.clone(),
                assistant_output: turn.assistant_output.clone(),
            })
            .collect(),
        latest_output,
        actions: [
            InlineAgentOutputAction::Replace,
            InlineAgentOutputAction::Append,
            InlineAgentOutputAction::Copy,
        ]
        .into_iter()
        .map(|action| InlineAgentActionViewModel {
            action,
            enabled: is_action_enabled_for_snapshot(action, &action_state, snapshot),
        })
        .collect(),
        stop_enabled: is_stop_enabled(state),
        retry_enabled: is_retry_enabled(state),
    }
}

fn latest_complete_output(
    state: &InlineAgentRunState,
    turns: &[InlineAgentTurn],
) -> Option<String> {
    state
        .latest_complete_output()
        .map(str::to_string)
        .or_else(|| {
            turns
                .iter()
                .rev()
                .find_map(|turn| turn.assistant_output.clone())
        })
}

fn action_state_with_latest_output(
    state: &InlineAgentRunState,
    latest_output: Option<&str>,
) -> InlineAgentRunState {
    if state.latest_complete_output().is_some() {
        return state.clone();
    }

    latest_output
        .map(|output| InlineAgentRunState::Completed {
            output: output.to_string(),
        })
        .unwrap_or_else(|| state.clone())
}
