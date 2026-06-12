use super::*;

impl NotesApp {
    pub(super) fn toggle_checklist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.toggle_checklist(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn toggle_task_marker_at(
        &mut self,
        marker_range: std::ops::Range<usize>,
        currently_checked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let toggled = self.notes_editor.update(cx, |editor, cx| {
            editor.toggle_task_marker_at(marker_range, currently_checked, window, cx)
        });
        if toggled {
            self.has_unsaved_changes = true;
        }
    }

    pub(super) fn insert_horizontal_rule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.insert_horizontal_rule(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn cycle_heading(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.cycle_heading(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn move_line_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.move_line_up(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn move_line_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.move_line_down(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn select_current_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.select_current_line(window, cx);
        });
    }

    pub(super) fn try_smart_paste(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let clipboard = Self::read_clipboard();
        let handled = self.notes_editor.update(cx, |editor, cx| {
            editor.try_smart_paste(&clipboard, window, cx)
        });
        if handled {
            self.has_unsaved_changes = true;
        }
        handled
    }

    pub(super) fn toggle_blockquote(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.toggle_blockquote(window, cx);
        });
        self.has_unsaved_changes = true;
    }

    pub(super) fn duplicate_line(
        &mut self,
        direction_down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.duplicate_line(direction_down, window, cx);
        });
        self.has_unsaved_changes = true;
    }
}
