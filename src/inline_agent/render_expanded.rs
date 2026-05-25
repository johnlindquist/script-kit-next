pub fn expanded_header_label(turn_count: usize) -> String {
    format!(
        "Cue - {turn_count} turn{}",
        if turn_count == 1 { "" } else { "s" }
    )
}

use crate::ai::inline_agent::history::InlineAgentTurn;

use super::state::InlineAgentRunState;
use super::types::{InlineAgentSnapshot, INLINE_AGENT_INPUT_PLACEHOLDER};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentExpandedViewModel {
    pub header_label: String,
    pub session_id: String,
    pub app_badge: String,
    pub input_placeholder: &'static str,
    pub turns: Vec<InlineAgentTurnViewModel>,
    pub latest_output: Option<String>,
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
    InlineAgentExpandedViewModel {
        header_label: expanded_header_label(turns.len()),
        session_id: snapshot.session_id.to_string(),
        app_badge: snapshot.app.name.clone(),
        input_placeholder: INLINE_AGENT_INPUT_PLACEHOLDER,
        turns: turns
            .iter()
            .map(|turn| InlineAgentTurnViewModel {
                user_instruction: turn.instruction.clone(),
                assistant_output: turn.assistant_output.clone(),
            })
            .collect(),
        latest_output: latest_complete_output(state, turns),
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
