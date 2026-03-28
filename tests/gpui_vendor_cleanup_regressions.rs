//! Regression tests locking in GPUI vendor upgrade cleanups.
//!
//! These tests use source-string assertions to verify that already-landed
//! migrations (hover suppression removal, native follow-tail adoption) remain
//! in place and do not regress.

const AI_ACTIONS_SOURCE: &str = include_str!("../src/ai/window/render_message_actions.rs");
const CHAT_STATE_SOURCE: &str = include_str!("../src/prompts/chat/state.rs");
const CHAT_RENDER_CORE_SOURCE: &str = include_str!("../src/prompts/chat/render_core.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const LIST_ITEM_SOURCE: &str = include_str!("../src/list_item/mod.rs");

// ---------------------------------------------------------------------------
// AI action strip: unconditional GPUI hover
// ---------------------------------------------------------------------------

#[test]
fn ai_action_buttons_use_unconditional_gpui_hover() {
    assert!(
        AI_ACTIONS_SOURCE
            .contains(".hover(|s| s.bg(muted_bg.opacity(OPACITY_HOVER)).text_color(muted_fg))"),
        "action_btn_base should always apply GPUI hover styling"
    );
    assert!(
        !AI_ACTIONS_SOURCE.contains("mouse_mode"),
        "AI action buttons should not gate hover on a local mouse_mode flag"
    );
    assert!(
        !AI_ACTIONS_SOURCE.contains("InputMode::Mouse"),
        "AI action buttons should not gate hover on InputMode::Mouse"
    );
}

// ---------------------------------------------------------------------------
// ChatPrompt: native follow-tail via ListState::set_follow_tail
// ---------------------------------------------------------------------------

#[test]
fn chat_prompt_uses_native_follow_tail() {
    assert!(
        CHAT_STATE_SOURCE.contains("self.turns_list_state.set_follow_tail(false);"),
        "empty chat should explicitly disable follow-tail"
    );
    assert!(
        CHAT_STATE_SOURCE.contains(".set_follow_tail(!self.user_has_scrolled_up);"),
        "chat append path should drive follow-tail from manual-scroll state"
    );
    assert!(
        CHAT_STATE_SOURCE.contains("self.turns_list_state.set_follow_tail(item_count > 0);"),
        "force-scroll path should use follow-tail instead of manual last-item scrolling"
    );
    assert!(
        CHAT_RENDER_CORE_SOURCE
            .contains(".set_follow_tail(has_turns && !this.user_has_scrolled_up);"),
        "wheel reconciliation should write back to GPUI follow-tail"
    );
}

#[test]
fn chat_prompt_follow_paths_do_not_use_manual_item_scrolling() {
    assert!(
        !CHAT_STATE_SOURCE.contains("scroll_to_item(")
            && !CHAT_STATE_SOURCE.contains("scroll_to_reveal_item("),
        "state.rs follow-tail paths should not use manual item scrolling"
    );
    assert!(
        !CHAT_RENDER_CORE_SOURCE.contains("scroll_to_item(")
            && !CHAT_RENDER_CORE_SOURCE.contains("scroll_to_reveal_item("),
        "render_core.rs follow-tail paths should not use manual item scrolling"
    );
}

// ---------------------------------------------------------------------------
// ActionsDialog: direct GPUI hover without local state machine
// ---------------------------------------------------------------------------

#[test]
fn actions_dialog_rows_use_direct_gpui_hover() {
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("let hover_row_bg = if is_destructive {"),
        "actions dialog should compute a direct hover background"
    );
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("row.hover("),
        "actions dialog rows should use GPUI hover styling directly"
    );
}

#[test]
fn actions_dialog_does_not_gate_hover_through_local_state() {
    for needle in [
        "current_input_mode == InputMode::Mouse",
        "this.input_mode = InputMode::Mouse",
        "this.hovered_index = Some(ix)",
        "self.input_mode = InputMode::Keyboard;",
        "self.hovered_index = None;",
    ] {
        assert!(
            !ACTIONS_DIALOG_SOURCE.contains(needle),
            "unexpected legacy hover-state pattern still present: {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// ListItem: GPUI hover without custom suppression toggle
// ---------------------------------------------------------------------------

#[test]
fn list_item_relies_on_gpui_hover_without_custom_toggle() {
    assert!(
        !LIST_ITEM_SOURCE.contains("enable_hover_effect"),
        "ListItem should not carry a custom hover suppression field"
    );
    assert!(
        !LIST_ITEM_SOURCE.contains("with_hover_effect"),
        "ListItem should not expose a custom hover suppression builder"
    );
    assert!(
        LIST_ITEM_SOURCE.contains("if !self.selected {"),
        "non-selected rows should keep hover styling"
    );
    assert!(
        LIST_ITEM_SOURCE
            .contains("inner_content = inner_content.hover(move |s| s.bg(hover_bg));"),
        "hover styling should be unconditional for non-selected rows"
    );
}
