const NOTES_BROWSE: &str = include_str!("../src/render_builtins/notes_browse.rs");

#[test]
fn notes_browse_empty_state_copy_is_modeled() {
    assert!(
        NOTES_BROWSE.contains("enum NotesBrowseEmptyState")
            && NOTES_BROWSE.contains("NoNotesYet")
            && NOTES_BROWSE.contains("NoFilteredMatches"),
        "Notes Browse empty-state copy should use named states"
    );
    assert!(
        NOTES_BROWSE.contains("fn from_filter(filter: &str) -> Self")
            && NOTES_BROWSE.contains("fn message(self) -> &'static str"),
        "Notes Browse empty states should own filter classification and visible copy"
    );
    assert!(
        NOTES_BROWSE.contains("NotesBrowseEmptyState::from_filter(&filter).message()"),
        "Notes Browse renderer should derive empty-state copy from the model"
    );
    assert!(
        !NOTES_BROWSE.contains("child(if filter.is_empty()"),
        "Notes Browse empty-state copy must not regress to inline filter-empty branching"
    );
}

#[test]
fn notes_browse_untitled_note_copy_has_single_owner() {
    assert!(
        NOTES_BROWSE.contains("fn notes_browse_display_title(note: &crate::notes::Note) -> String"),
        "Notes Browse should own note title fallback in one helper"
    );
    assert!(
        NOTES_BROWSE.matches("\"Untitled Note\"").count() == 1,
        "Untitled Note fallback copy should live in notes_browse_display_title only"
    );
    for required in [
        ".map(|note| Self::notes_browse_display_title(&note))",
        "let title = Self::notes_browse_display_title(note);",
        "let title = Self::notes_browse_display_title(&note);",
    ] {
        assert!(
            NOTES_BROWSE.contains(required),
            "Notes Browse title sites should use the shared display helper: {required}"
        );
    }
}
