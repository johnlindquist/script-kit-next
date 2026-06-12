use super::*;

use crate::components::notes_editor::NotesEditor;

impl NotesApp {
    pub(super) fn delete_current_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.delete_current_line(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn indent_at_cursor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.indent_at_cursor(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn outdent_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.outdent_line(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn toggle_bullet_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.toggle_bullet_list(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn toggle_numbered_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.toggle_numbered_list(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn numbered_list_prefix_len(line: &str) -> usize {
        NotesEditor::numbered_list_prefix_len(line)
    }

    pub(super) fn detect_next_list_number(value: &str, current_line_start: usize) -> usize {
        NotesEditor::detect_next_list_number(value, current_line_start)
    }

    pub(super) fn join_lines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.join_lines(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn transform_case(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.transform_case(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn to_title_case(s: &str) -> String {
        NotesEditor::to_title_case(s)
    }
}
