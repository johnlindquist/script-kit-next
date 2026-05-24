use script_kit_gpui::inline_agent::render_actions::is_action_enabled;
use script_kit_gpui::inline_agent::{InlineAgentOutputAction, InlineAgentRunState};

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
            action: InlineAgentOutputAction::Replace
        }
    ));
}
