//! Regression tests locking in GPUI vendor upgrade cleanups.
//!
//! These tests use source-string assertions to verify that already-landed
//! migrations (hover suppression removal, native follow-tail adoption) remain
//! in place and do not regress.

const AI_ACTIONS_SOURCE: &str = include_str!("../src/ai/window/render_message_actions.rs");
const AI_MESSAGES_SOURCE: &str = include_str!("../src/ai/window/render_messages.rs");
const CHAT_STATE_SOURCE: &str = include_str!("../src/prompts/chat/state.rs");
const CHAT_RENDER_CORE_SOURCE: &str = include_str!("../src/prompts/chat/render_core.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const LIST_ITEM_SOURCE: &str = include_str!("../src/list_item/mod.rs");
const FILE_SEARCH_SOURCE: &str = include_str!("../src/render_builtins/file_search.rs");
const THEME_CHOOSER_SOURCE: &str = include_str!("../src/render_builtins/theme_chooser.rs");
const THEME_CHOOSER_HEADER_SOURCE: &str =
    include_str!("../src/render_builtins/theme_chooser_list_header.rs");
const KIT_STORE_SOURCE: &str = include_str!("../src/render_builtins/kit_store.rs");
const SELECT_RENDER_SOURCE: &str = include_str!("../src/prompts/select/render.rs");

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
// AI window: native follow-tail via ListState::set_follow_tail
// ---------------------------------------------------------------------------

#[test]
fn ai_window_uses_native_follow_tail() {
    assert!(
        AI_MESSAGES_SOURCE
            .contains(".set_follow_tail(!self.user_has_scrolled_up && item_count > 0);"),
        "AI window append/stream path should drive native follow-tail"
    );
    assert!(
        AI_MESSAGES_SOURCE.contains("self.messages_list_state.set_follow_tail(item_count > 0);"),
        "AI window force-scroll path should use native follow-tail"
    );
    assert!(
        !AI_MESSAGES_SOURCE.contains("scroll_to_item(")
            && !AI_MESSAGES_SOURCE.contains("scroll_to_reveal_item("),
        "AI window follow-tail paths should not use manual item scrolling"
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
        LIST_ITEM_SOURCE.contains("inner_content = inner_content.hover(move |s| s.bg(hover_bg));"),
        "hover styling should be unconditional for non-selected rows"
    );
}

// ---------------------------------------------------------------------------
// File search: direct GPUI hover without manual hover bookkeeping
// ---------------------------------------------------------------------------

#[test]
fn file_search_rows_use_direct_gpui_hover() {
    assert!(
        FILE_SEARCH_SOURCE.contains(".when(!is_selected, |d| d.hover(move |s| s.bg(hover_bg)))"),
        "file search rows should use direct GPUI hover styling"
    );
    for needle in [
        "file_input_mode == InputMode::Mouse",
        "let hover_entity_handle = cx.entity().downgrade();",
        ".on_hover(hover_handler)",
    ] {
        assert!(
            !FILE_SEARCH_SOURCE.contains(needle),
            "unexpected legacy file-search hover pattern still present: {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// Theme chooser: direct GPUI hover without manual hover bookkeeping
// ---------------------------------------------------------------------------

#[test]
fn theme_chooser_rows_use_direct_gpui_hover() {
    assert!(
        THEME_CHOOSER_SOURCE.contains("d.hover(move |s| s.bg(theme_row_hover_bg))"),
        "theme chooser rows should use direct GPUI hover styling"
    );
    for needle in [
        "current_input_mode == InputMode::Mouse",
        "let hover_entity_handle = entity_handle.clone();",
        ".on_hover(hover_handler)",
    ] {
        assert!(
            !THEME_CHOOSER_SOURCE.contains(needle),
            "unexpected legacy theme-chooser hover pattern still present: {needle}"
        );
    }
}

#[test]
fn theme_chooser_header_rows_use_direct_gpui_hover() {
    assert!(
        THEME_CHOOSER_HEADER_SOURCE.contains("d.hover(move |s| s.bg(hover_bg))"),
        "theme chooser header rows should use direct GPUI hover styling"
    );
    for needle in [
        "current_input_mode == InputMode::Mouse",
        "this.input_mode = InputMode::Mouse;",
        "this.hovered_index = Some(ix);",
        ".on_hover(hover_handler)",
    ] {
        assert!(
            !THEME_CHOOSER_HEADER_SOURCE.contains(needle),
            "unexpected legacy theme-chooser-header hover pattern still present: {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// Kit store: direct GPUI hover without manual hover bookkeeping
// ---------------------------------------------------------------------------

#[test]
fn kit_store_rows_use_direct_gpui_hover() {
    assert!(
        KIT_STORE_SOURCE
            .contains(".when(!is_selected, |row| row.hover(move |style| style.bg(row_bg)))"),
        "kit store rows should use direct GPUI hover styling"
    );
    for needle in [
        "hovered_row == Some(ix) && input_mode == InputMode::Mouse",
        ".on_hover(move |is_hovered, _window, cx|",
    ] {
        assert!(
            !KIT_STORE_SOURCE.contains(needle),
            "unexpected legacy kit-store hover pattern still present: {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// ListItem: modality-aware hover disclosure via Window input modality
// ---------------------------------------------------------------------------

#[test]
fn list_item_masks_hover_driven_disclosure_with_window_input_modality() {
    assert!(
        LIST_ITEM_SOURCE
            .contains("let hover_visible = self.hovered && !window.last_input_was_keyboard();"),
        "ListItem should derive a modality-aware hover flag from Window"
    );
    for needle in [
        "should_show_search_description(self.selected, hover_visible, has_description_match)",
        "should_show_search_shortcut(is_filtering, self.selected, hover_visible)",
        "} else if hover_visible {",
        "self.selected || hover_visible || is_filtering",
    ] {
        assert!(
            LIST_ITEM_SOURCE.contains(needle),
            "ListItem should route hover-driven disclosure through hover_visible: {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// SelectPrompt: modality-aware hover via visual_row_state_for_input_modality
// ---------------------------------------------------------------------------

#[test]
fn select_prompt_masks_stateful_hover_with_window_input_modality() {
    assert!(
        SELECT_RENDER_SOURCE.contains("window.last_input_was_keyboard()"),
        "SelectPrompt should read GPUI input modality from Window"
    );
    assert!(
        SELECT_RENDER_SOURCE.contains("visual_row_state_for_input_modality("),
        "SelectPrompt should normalize row state before painting hover-driven visuals"
    );
    assert!(
        !SELECT_RENDER_SOURCE.contains("let is_hovered = row_state.is_hovered;"),
        "SelectPrompt should not paint hover from raw row_state anymore"
    );
    assert!(
        SELECT_RENDER_SOURCE.contains("let is_hovered = visual_row_state.is_hovered;"),
        "SelectPrompt should drive UnifiedListItem hover state from modality-adjusted row state"
    );
}
