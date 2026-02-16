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
