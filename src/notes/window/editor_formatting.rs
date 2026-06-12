use super::*;

use crate::components::notes_editor::NotesEditor;

impl NotesApp {
    /// Compute replacement text and resulting selection for formatting insertion.
    pub(super) fn formatting_replacement(
        value: &str,
        selection: Range<usize>,
        prefix: &str,
        suffix: &str,
    ) -> (String, Range<usize>) {
        NotesEditor::formatting_replacement(value, selection, prefix, suffix)
    }

    /// Insert markdown formatting at cursor position
    ///
    /// Inserts prefix+suffix at cursor. If text is selected, it gets replaced
    /// with prefix+suffix via the replace() method.
    pub(super) fn insert_formatting(
        &mut self,
        prefix: &str,
        suffix: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.insert_formatting(prefix, suffix, window, cx);
        });
        self.has_unsaved_changes = true;
    }
}
