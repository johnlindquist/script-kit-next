use script_kit_gpui::inline_agent::{InlineAgentMode, InlineAgentRunState, InlineAgentState};

#[test]
fn expanded_collapse_preserves_latest_output() {
    let mut state = InlineAgentState::default();
    state.finish("final output".to_string());
    state.expand();
    state.collapse();

    assert_eq!(state.mode, InlineAgentMode::Compact);
    assert_eq!(state.latest_output.as_deref(), Some("final output"));
    assert_eq!(
        state.run_state,
        InlineAgentRunState::Completed {
            output: "final output".to_string()
        }
    );
}
