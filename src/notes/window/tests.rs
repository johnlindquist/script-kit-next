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
