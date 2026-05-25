use super::state::InlineAgentRunState;
use super::types::{
    InlineAgentMutationReceipt, InlineAgentOutputAction, InlineAgentSnapshot,
    InlineAgentTextMutation,
};
use super::InlineAgentPlatformBridge;

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

pub fn is_action_enabled_for_snapshot(
    action: InlineAgentOutputAction,
    state: &InlineAgentRunState,
    snapshot: &InlineAgentSnapshot,
) -> bool {
    if !is_action_enabled(action, state) {
        return false;
    }

    match action {
        InlineAgentOutputAction::Replace => snapshot.capabilities.can_replace,
        InlineAgentOutputAction::Append => snapshot.capabilities.can_append,
        InlineAgentOutputAction::Copy => snapshot.capabilities.can_copy,
        InlineAgentOutputAction::Chat => true,
    }
}

pub fn latest_output_mutation_for_action(
    action: InlineAgentOutputAction,
    state: &InlineAgentRunState,
    snapshot: &InlineAgentSnapshot,
) -> Option<InlineAgentTextMutation> {
    if !is_action_enabled_for_snapshot(action, state, snapshot) {
        return None;
    }

    let output = state.latest_complete_output()?.to_string();
    match action {
        InlineAgentOutputAction::Replace => Some(InlineAgentTextMutation::Replace {
            session_id: snapshot.session_id.clone(),
            text: output,
        }),
        InlineAgentOutputAction::Append => Some(InlineAgentTextMutation::Append {
            session_id: snapshot.session_id.clone(),
            text: output,
        }),
        InlineAgentOutputAction::Copy => Some(InlineAgentTextMutation::Copy { text: output }),
        InlineAgentOutputAction::Chat => None,
    }
}

pub fn apply_latest_output_action(
    bridge: &dyn InlineAgentPlatformBridge,
    action: InlineAgentOutputAction,
    state: &InlineAgentRunState,
    snapshot: &InlineAgentSnapshot,
) -> anyhow::Result<Option<InlineAgentMutationReceipt>> {
    let Some(mutation) = latest_output_mutation_for_action(action, state, snapshot) else {
        return Ok(None);
    };

    bridge
        .apply_text_mutation(&snapshot.anchor, mutation)
        .map(Some)
}
