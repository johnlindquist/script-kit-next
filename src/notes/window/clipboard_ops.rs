use super::*;

impl NotesApp {
    /// Insert current date/time at cursor position (Cmd+Shift+D)
    pub(super) fn insert_date_time(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d %H:%M").to_string();
        self.editor_state.update(cx, |state, cx| {
            let selection = state.selection();
            let value = state.value().to_string();
            let start = selection.start.min(value.len());
            let end = selection.end.min(value.len());
            let new_value = format!("{}{}{}", &value[..start], date_str, &value[end..]);
            let new_cursor = start + date_str.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Inserted date/time at cursor");
        cx.notify();
    }

    /// Copy note content as markdown to clipboard (Cmd+Shift+C)
    pub(super) fn copy_as_markdown(&mut self, cx: &Context<Self>) {
        let content = self.editor_state.read(cx).value().to_string();
        self.copy_text_to_clipboard(&content);
        self.show_action_feedback("Copied", false);
        info!("Copied note as markdown to clipboard");
    }
}
