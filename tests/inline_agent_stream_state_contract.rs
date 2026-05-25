use script_kit_gpui::ai::inline_agent::{
    InlineAgentPhase, InlineAgentProviderEvent, InlineAgentStreamState,
};

#[test]
fn message_delta_switches_from_thinking_to_streaming_and_finish_sets_latest_output() {
    let mut state = InlineAgentStreamState::default();
    state.start_turn();
    assert_eq!(state.phase, InlineAgentPhase::Thinking);

    state.apply_provider_event(InlineAgentProviderEvent::AgentThoughtDelta {
        text: "planning".to_string(),
    });
    assert_eq!(state.phase, InlineAgentPhase::Thinking);
    assert_eq!(state.thought_log, "planning");
    assert_eq!(state.latest_complete_output, None);

    state.apply_provider_event(InlineAgentProviderEvent::AgentMessageDelta {
        text: "Hello".to_string(),
    });
    state.apply_provider_event(InlineAgentProviderEvent::AgentMessageDelta {
        text: " world".to_string(),
    });
    assert_eq!(state.phase, InlineAgentPhase::Streaming);
    assert_eq!(state.visible_output, "Hello world");

    state.apply_provider_event(InlineAgentProviderEvent::TurnFinished);
    assert_eq!(state.phase, InlineAgentPhase::Complete);
    assert_eq!(state.latest_complete_output.as_deref(), Some("Hello world"));
}

#[test]
fn failed_turn_preserves_previous_latest_complete_output() {
    let mut state = InlineAgentStreamState::default();
    state.start_turn();
    state.apply_provider_event(InlineAgentProviderEvent::AgentMessageDelta {
        text: "First".to_string(),
    });
    state.apply_provider_event(InlineAgentProviderEvent::TurnFinished);

    state.start_turn();
    state.apply_provider_event(InlineAgentProviderEvent::AgentMessageDelta {
        text: "partial".to_string(),
    });
    state.apply_provider_event(InlineAgentProviderEvent::Failed {
        message: "provider failed".to_string(),
    });

    assert_eq!(state.phase, InlineAgentPhase::Error);
    assert_eq!(state.latest_complete_output.as_deref(), Some("First"));
    assert_eq!(state.error.as_deref(), Some("provider failed"));
}

#[test]
fn cancelling_disables_partial_output_but_keeps_latest_complete_output() {
    let mut state = InlineAgentStreamState::default();
    state.start_turn();
    state.apply_provider_event(InlineAgentProviderEvent::AgentMessageDelta {
        text: "Complete".to_string(),
    });
    state.apply_provider_event(InlineAgentProviderEvent::TurnFinished);

    state.start_turn();
    state.apply_provider_event(InlineAgentProviderEvent::AgentMessageDelta {
        text: "partial".to_string(),
    });
    state.cancel();

    assert_eq!(state.phase, InlineAgentPhase::Cancelling);
    assert_eq!(state.visible_output, "");
    assert_eq!(state.latest_complete_output.as_deref(), Some("Complete"));
}
