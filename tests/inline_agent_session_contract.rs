use script_kit_gpui::ai::inline_agent::mock::MockInlineAgentExecutor;
use script_kit_gpui::ai::inline_agent::{
    InlineAgentEditSemantics, InlineAgentPhase, InlineAgentSession,
};
use script_kit_gpui::platform::accessibility::focused_text::focused_text_snapshot_for_tests;

#[test]
fn mocked_turn_records_latest_complete_output_and_history() {
    let snapshot = focused_text_snapshot_for_tests("Hello world");
    let executor = MockInlineAgentExecutor;
    let mut session = InlineAgentSession::new(snapshot);

    let (events, audit) = session
        .begin_turn(
            "Translate to French",
            InlineAgentEditSemantics::Replace,
            &executor,
        )
        .expect("mocked inline-agent turn should start");
    assert_eq!(audit.turn_count, 1);
    assert_eq!(session.stream.phase, InlineAgentPhase::Thinking);

    session.drain_provider_events(events);

    assert_eq!(session.stream.phase, InlineAgentPhase::Complete);
    assert_eq!(
        session.latest_complete_output(),
        Some("Translate to French")
    );
    assert_eq!(session.history.len(), 1);
    assert_eq!(session.history[0].instruction, "Translate to French");
    assert_eq!(
        session.history[0].semantics,
        InlineAgentEditSemantics::Replace
    );
    assert_eq!(
        session.history[0].assistant_output.as_deref(),
        Some("Translate to French")
    );
}

#[test]
fn followup_turn_prompt_audit_counts_previous_turns_and_preserves_capture() {
    let snapshot = focused_text_snapshot_for_tests("Original focused field");
    let executor = MockInlineAgentExecutor;
    let mut session = InlineAgentSession::new(snapshot);

    let (first_events, _) = session
        .begin_turn(
            "Make it shorter",
            InlineAgentEditSemantics::Replace,
            &executor,
        )
        .expect("first turn should start");
    session.drain_provider_events(first_events);

    let (second_events, second_audit) = session
        .begin_turn("Make it warmer", InlineAgentEditSemantics::Chat, &executor)
        .expect("second turn should start");
    assert_eq!(second_audit.turn_count, 2);
    assert_eq!(
        second_audit.capture_char_count,
        session.snapshot.metrics.chars
    );
    session.drain_provider_events(second_events);

    assert_eq!(session.history.len(), 2);
    assert_eq!(session.history[1].semantics, InlineAgentEditSemantics::Chat);
    assert_eq!(session.latest_complete_output(), Some("Make it warmer"));
    assert_eq!(session.snapshot.text, "Original focused field");
}

#[test]
fn cancelling_active_turn_clears_partial_output_and_keeps_prior_latest_output() {
    let snapshot = focused_text_snapshot_for_tests("Hello world");
    let executor = MockInlineAgentExecutor;
    let mut session = InlineAgentSession::new(snapshot);

    let (events, _) = session
        .begin_turn(
            "First complete turn",
            InlineAgentEditSemantics::Replace,
            &executor,
        )
        .expect("first turn should start");
    session.drain_provider_events(events);

    let (_events, _) = session
        .begin_turn(
            "Second partial turn",
            InlineAgentEditSemantics::Replace,
            &executor,
        )
        .expect("second turn should start");
    session
        .cancel_active_turn(&executor)
        .expect("cancel should route");

    assert_eq!(session.stream.phase, InlineAgentPhase::Cancelling);
    assert_eq!(session.stream.visible_output, "");
    assert_eq!(
        session.latest_complete_output(),
        Some("First complete turn")
    );
    assert_eq!(session.history.len(), 1);
}
