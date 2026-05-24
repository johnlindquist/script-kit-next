use super::state::InlineAgentRunState;
use super::types::InlineAgentOutputAction;

pub fn is_action_enabled(action: InlineAgentOutputAction, state: &InlineAgentRunState) -> bool {
    match (action, state.latest_complete_output()) {
        (InlineAgentOutputAction::Copy, Some(output)) => !output.is_empty(),
        (InlineAgentOutputAction::Chat, _) => {
            !matches!(state, InlineAgentRunState::Applying { .. })
        }
        (_, Some(output)) => {
            !output.is_empty() && !matches!(state, InlineAgentRunState::Applying { .. })
        }
        _ => false,
    }
}
