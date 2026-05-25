const FOCUSED_TEXT_ENTRY: &str =
    include_str!("../../src/app_impl/tab_ai_mode/focused_text_entry.rs");
const ACP_VIEW: &str = include_str!("../../src/ai/acp/view.rs");
const ACP_THREAD: &str = include_str!("../../src/ai/acp/thread.rs");
const SIMULATE_KEY_DISPATCH: &str = include_str!("../../src/app_impl/simulate_key_dispatch.rs");

fn source_between<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing start marker: {start_marker}"));
    let rest = &source[start..];
    let end = rest
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing end marker: {end_marker}"));
    &rest[..end]
}

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

#[test]
fn focused_text_result_followup_enter_expands_and_submits_chat_semantics() {
    for required in [
        "submit_focused_text_from_enter",
        "FocusedTextMiniPhase::Result",
        "expand_focused_text_to_full_chat",
        "FocusedTextEditSemantics::Chat",
        "set_ui_variant(AcpChatUiVariant::Standard",
        "on_focused_text_expand_requested",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text result follow-up expansion contract: {required}"
        );
    }
}

#[test]
fn simulate_key_enter_uses_focused_text_submit_path() {
    for required in [
        "chat.has_focused_text_context()",
        "chat.submit_focused_text_from_enter(cx)",
        "thread.submit_input(cx)",
    ] {
        assert!(
            SIMULATE_KEY_DISPATCH.contains(required),
            "simulateKey Enter must preserve focused-text follow-up handoff: {required}"
        );
    }
}

#[test]
fn focused_text_initial_enter_remains_replace_semantics() {
    for required in [
        "FocusedTextMiniPhase::InputOnly",
        "FocusedTextEditSemantics::Replace",
        "submit_focused_text_turn",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text initial submit contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_result_phase_requires_assistant_output() {
    let phase_fn = source_between(
        ACP_VIEW,
        "fn focused_text_mini_phase_for_thread",
        "fn focused_text_mini_footer_visible_for_thread",
    );

    assert!(phase_fn
        .contains("let has_output = Self::latest_assistant_response_text(thread).is_some();"));
    assert!(!phase_fn.contains("has_user_turn"));
    assert!(
        phase_fn.contains("(true, false) => Some(FocusedTextMiniPhase::InputOnly)"),
        "streaming without assistant text must stay compact/input-only"
    );
}
