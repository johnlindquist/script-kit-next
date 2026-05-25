const FOCUSED_TEXT_ENTRY: &str =
    include_str!("../../src/app_impl/tab_ai_mode/focused_text_entry.rs");
const ACP_VIEW: &str = include_str!("../../src/ai/acp/view.rs");
const ACP_THREAD: &str = include_str!("../../src/ai/acp/thread.rs");

#[test]
fn focused_text_turns_use_focused_prompt_blocks_not_generic_submit_input() {
    for required in [
        "pub(crate) fn submit_focused_text_turn",
        "build_focused_text_prompt",
        "FocusedTextPromptRequest",
        "FocusedTextEditSemantics::Replace",
        "thread.submit_blocks(blocks, instruction, cx)",
        "focused_text_prompt_built",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text prompt submission contract: {required}"
        );
    }

    for required in [
        "pub(crate) fn submit_blocks",
        "display_user_text",
        "clear_all_pending_context(\"submit_blocks\")",
        "AcpThreadMessageRole::User",
        "start_turn(AgentChatTurnRequest",
    ] {
        assert!(
            ACP_THREAD.contains(required),
            "missing ACP explicit-block submit contract: {required}"
        );
    }

    assert!(
        FOCUSED_TEXT_ENTRY.contains("chat.submit_focused_text_turn("),
        "focused-text Pi fixture should submit through the focused-text prompt path"
    );
    assert!(
        !FOCUSED_TEXT_ENTRY.contains("thread.submit_input(cx)"),
        "focused-text entry must not submit captured text through generic ACP context"
    );
}

#[test]
fn focused_text_view_keeps_snapshot_in_memory_for_multiturn_prompting() {
    for required in [
        "snapshot: crate::platform::accessibility::FocusedTextSnapshot",
        "let snapshot = state.snapshot.clone();",
        "focused_text_previous_turns",
        "assistant_output",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text retained snapshot/multiturn contract: {required}"
        );
    }
}
