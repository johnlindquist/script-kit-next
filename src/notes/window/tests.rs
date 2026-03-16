use super::{Note, NoteId, NotesApp};

#[test]
fn formatting_replacement_wraps_selected_text() {
    let value = "hello world";
    let selection = 6..11;

    let (replacement, new_selection) =
        NotesApp::formatting_replacement(value, selection.clone(), "**", "**");

    let new_value = format!(
        "{}{}{}",
        &value[..selection.start],
        replacement,
        &value[selection.end..]
    );

    assert_eq!(new_value, "hello **world**");
    assert_eq!(new_selection, 8..13);
}

#[test]
fn formatting_replacement_inserts_and_positions_cursor() {
    let value = "hello";
    let selection = 2..2;

    let (replacement, new_selection) =
        NotesApp::formatting_replacement(value, selection.clone(), "**", "**");

    let new_value = format!(
        "{}{}{}",
        &value[..selection.start],
        replacement,
        &value[selection.end..]
    );

    assert_eq!(new_value, "he****llo");
    assert_eq!(new_selection, 4..4);
}

#[test]
fn test_format_search_match_counter_uses_selected_position_when_available() {
    let counter = NotesApp::format_search_match_counter(Some((3, 8)), 8);
    assert_eq!(counter, "3/8");
}

#[test]
fn test_format_search_match_counter_uses_zero_when_selection_missing() {
    let counter = NotesApp::format_search_match_counter(None, 6);
    assert_eq!(counter, "0/6");
}

#[test]
fn test_resolve_selected_note_returns_none_when_selection_is_missing() {
    let notes = vec![Note::with_content("one"), Note::with_content("two")];

    let selected = NotesApp::resolve_selected_note(None, &notes);

    assert!(selected.is_none());
}

#[test]
fn test_resolve_selected_note_returns_none_when_selection_is_stale() {
    let notes = vec![Note::with_content("one"), Note::with_content("two")];

    let selected = NotesApp::resolve_selected_note(Some(NoteId::new()), &notes);

    assert!(selected.is_none());
}

#[test]
fn test_resolve_selected_note_returns_note_when_selection_exists() {
    let notes = vec![Note::with_content("one"), Note::with_content("two")];
    let selected_id = notes[1].id;

    let selected = NotesApp::resolve_selected_note(Some(selected_id), &notes);

    assert_eq!(
        selected.map(|(id, note)| (id, note.id)),
        Some((selected_id, selected_id))
    );
}

#[test]
fn test_cmd_f_dispatches_search_on_window_when_notes_shortcut_runs() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("window.dispatch_action(Box::new(Search), cx);"),
        "Notes cmd+f shortcut should dispatch Search through the current window"
    );
    assert!(
        !KEYBOARD_SOURCE.contains("cx.dispatch_action(&Search);"),
        "Notes cmd+f shortcut should not dispatch Search through app context"
    );
}

#[test]
fn test_find_in_note_action_dispatches_search_on_window_when_action_executes() {
    const PANELS_SOURCE: &str = include_str!("panels.rs");
    assert!(
        PANELS_SOURCE.contains("window.dispatch_action(Box::new(Search), cx);"),
        "Notes Find in Note action should dispatch Search through the current window"
    );
    assert!(
        !PANELS_SOURCE.contains("cx.dispatch_action(&Search);"),
        "Notes Find in Note action should not dispatch Search through app context"
    );
}

#[test]
fn test_platform_arrow_shortcuts_only_run_note_navigation_when_editor_not_focused() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("focus_handle(cx)")
            && KEYBOARD_SOURCE.contains(".is_focused(window);"),
        "Platform arrow shortcuts must check editor focus before note navigation"
    );
    assert!(
        KEYBOARD_SOURCE.contains("if !editor_is_focused {"),
        "Platform arrow shortcuts must skip note navigation when editor is focused"
    );
}

#[test]
fn test_show_selected_note_missing_feedback_notifies_after_feedback_state_update() {
    const NOTES_ACTIONS_SOURCE: &str = include_str!("notes_actions.rs");
    assert!(
        NOTES_ACTIONS_SOURCE.contains(
            "self.show_action_feedback(Self::SELECTED_NOTE_NOT_FOUND_FEEDBACK, true);\n        cx.notify();"
        ),
        "Missing-note feedback should notify after updating action feedback state"
    );
}

#[test]
fn test_duplicate_selected_note_sets_feedback_before_select_note() {
    const NOTES_ACTIONS_SOURCE: &str = include_str!("notes_actions.rs");
    let feedback_idx = NOTES_ACTIONS_SOURCE
        .find("self.show_action_feedback(\"Duplicated\", false);")
        .expect("Expected duplicate feedback call in notes_actions.rs");
    let select_idx = NOTES_ACTIONS_SOURCE
        .find("self.select_note(duplicate.id, window, cx);")
        .expect("Expected duplicate select_note call in notes_actions.rs");

    assert!(
        feedback_idx < select_idx,
        "Duplicate feedback should be set before select_note triggers notify"
    );
}

#[test]
fn test_copy_as_markdown_notifies_after_feedback_state_update() {
    const CLIPBOARD_OPS_SOURCE: &str = include_str!("clipboard_ops.rs");
    assert!(
        CLIPBOARD_OPS_SOURCE
            .contains("self.show_action_feedback(\"Copied\", false);\n        cx.notify();"),
        "Copy-as-markdown should notify after updating action feedback state"
    );
}

#[test]
fn test_notes_keyboard_handles_named_bracket_keys_when_platform_navigation_shortcuts_run() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("fn is_key_left_bracket(key: &str) -> bool"),
        "Notes keyboard should define a left bracket key helper"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key == \"[\" || key.eq_ignore_ascii_case(\"bracketleft\")"),
        "Left bracket helper should match '[' and 'bracketleft'"
    );
    assert!(
        KEYBOARD_SOURCE.contains("fn is_key_right_bracket(key: &str) -> bool"),
        "Notes keyboard should define a right bracket key helper"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key == \"]\" || key.eq_ignore_ascii_case(\"bracketright\")"),
        "Right bracket helper should match ']' and 'bracketright'"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key if is_key_left_bracket(key) => {"),
        "Notes keyboard should use left bracket helper for navigate_back"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key if is_key_right_bracket(key) => {"),
        "Notes keyboard should use right bracket helper for navigate_forward"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_when_escape_closes_actions_panel() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let escape_branch =
        "if is_key_escape(key) || (modifiers.platform && key.eq_ignore_ascii_case(\"k\")) {";
    let branch_start = KEYBOARD_SOURCE
        .find(escape_branch)
        .expect("Expected actions panel escape branch in keyboard.rs");
    let branch_slice =
        &KEYBOARD_SOURCE[branch_start..(branch_start + 256).min(KEYBOARD_SOURCE.len())];

    let close_idx = branch_slice
        .find("self.close_actions_panel(window, cx);")
        .expect("Expected close_actions_panel call in actions panel escape branch");
    let stop_idx = branch_slice
        .find("cx.stop_propagation();")
        .expect("Expected cx.stop_propagation call in actions panel escape branch");
    let return_idx = branch_slice
        .find("return;")
        .expect("Expected return in actions panel escape branch");

    assert!(
        close_idx < stop_idx && stop_idx < return_idx,
        "Actions panel escape branch should stop propagation before returning"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_at_start_of_global_escape_chain() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("if is_key_escape(key) {\n            cx.stop_propagation();"),
        "Global escape chain should stop propagation before handling escape branches"
    );
}

#[test]
fn test_notes_actions_panel_uses_shared_disabled_opacity_constant() {
    const ACTIONS_PANEL_SOURCE: &str = include_str!("../actions_panel.rs");
    assert!(
        ACTIONS_PANEL_SOURCE.contains("use super::window::OPACITY_DISABLED;"),
        "Actions panel should use the shared Notes disabled opacity constant"
    );
    assert!(
        !ACTIONS_PANEL_SOURCE.contains("const OPACITY_DISABLED: f32"),
        "Actions panel should not define a duplicate disabled opacity constant"
    );
}

#[test]
fn test_notes_keyboard_delete_shortcut_routes_through_confirmation_helper() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("pub(super) fn handle_platform_delete_shortcut")
            && KEYBOARD_SOURCE.contains("self.request_delete_selected_note(window, cx);"),
        "Notes keyboard delete shortcut should route through the confirmation helper"
    );
    assert!(
        KEYBOARD_SOURCE.contains("notes_delete_shortcut_requesting_confirmation"),
        "Delete shortcut helper should emit a structured confirmation-request log"
    );
}

#[test]
fn test_notes_keyboard_delete_shortcut_works_in_trash_view() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    // The trash-view guard was removed so the delete shortcut routes through
    // request_delete_selected_note which already handles both modes.
    assert!(
        !KEYBOARD_SOURCE.contains("trash_view_requires_dedicated_delete_flow"),
        "Delete shortcut should not block trash view — request_delete_selected_note handles both modes"
    );
}

#[test]
fn test_delete_dialog_cancel_restores_editor_focus() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("this.focus_editor(window, cx);"),
        "Cancelling the delete dialog should restore editor focus"
    );
}

#[test]
fn test_delete_note_by_id_restores_editor_focus_when_no_notes_remain() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("self.focus_editor(window, cx);"),
        "Deleting the last note should restore editor focus"
    );
}

#[test]
fn test_delete_dialog_width_uses_viewport_size() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("window.viewport_size().width"),
        "Dialog width should prefer viewport_size"
    );
    assert!(
        NOTES_SOURCE.contains("window.bounds().size.width"),
        "Dialog width should fall back to bounds when viewport width is zero"
    );
}

#[test]
fn test_on_search_change_uses_reload_helper_instead_of_silently_swallowing_errors() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("pub(super) fn refresh_notes_for_search_query")
            && NOTES_SOURCE.contains("Failed to reload all notes while clearing the notes search"),
        "Notes search should use a dedicated helper with actionable reload errors"
    );
    assert!(
        !NOTES_SOURCE.contains("storage::get_all_notes().unwrap_or_default()"),
        "Notes search should not silently swallow errors when clearing the search query"
    );
    assert!(
        NOTES_SOURCE.contains("notes_search_refresh_started")
            && NOTES_SOURCE.contains("notes_search_refresh_completed")
            && NOTES_SOURCE.contains("notes_search_refresh_failed"),
        "Notes search refresh should emit structured logs for start, completion, and failure"
    );
}

#[test]
fn test_request_delete_selected_note_emits_structured_confirmation_logs() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("notes_delete_confirmation_requested")
            && NOTES_SOURCE.contains("notes_delete_confirmation_opened")
            && NOTES_SOURCE.contains("notes_delete_cancelled"),
        "Delete confirmation flow should emit structured request/open/cancel logs"
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_prefers_viewport() {
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(360.0, 520.0),
        360.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_falls_back_to_bounds() {
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(0.0, 520.0),
        520.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_uses_default_when_sizes_missing() {
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(0.0, 0.0),
        472.0
    );
}

#[test]
fn test_delete_dialog_width_prefers_viewport_but_falls_back_when_zero() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("window.viewport_size().width"),
        "Dialog width should still prefer viewport_size"
    );
    assert!(
        NOTES_SOURCE.contains("window.bounds().size.width"),
        "Dialog width should fall back to bounds when viewport width is unavailable"
    );
}

#[test]
fn test_permanent_delete_accepts_window_and_restores_selection_or_focus() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("fn permanently_delete_note(")
            && NOTES_SOURCE.contains("window: &mut Window,"),
        "Permanent delete should accept window so it can restore editor state"
    );
    assert!(
        NOTES_SOURCE.contains("self.select_note(next_note.id, window, cx);")
            && NOTES_SOURCE.contains("self.focus_editor(window, cx);"),
        "Permanent delete should either select the next trashed note or refocus the editor when empty"
    );
}
