const NOTES_BROWSE: &str = include_str!("../src/render_builtins/notes_browse.rs");

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
