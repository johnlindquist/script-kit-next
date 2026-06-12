use std::ops::Range;

use gpui::{Context, Window};
use tracing::info;

use super::NotesEditor;

impl NotesEditor {
    /// Compute replacement text and resulting selection for formatting insertion.
    pub fn formatting_replacement(
        value: &str,
        selection: Range<usize>,
        prefix: &str,
        suffix: &str,
    ) -> (String, Range<usize>) {
        let mut start = selection.start.min(value.len());
        let mut end = selection.end.min(value.len());
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        debug_assert!(value.is_char_boundary(start));
        debug_assert!(value.is_char_boundary(end));

        let selected_text = if start == end { "" } else { &value[start..end] };

        let replacement = format!("{}{}{}", prefix, selected_text, suffix);
        let selection_start = start + prefix.len();
        let selection_end = if selected_text.is_empty() {
            selection_start
        } else {
            selection_start + selected_text.len()
        };

        (replacement, selection_start..selection_end)
    }

    /// Insert markdown formatting at cursor position.
    ///
    /// Inserts prefix+suffix at cursor. If text is selected, it gets replaced
    /// with prefix+suffix via the replace() method.
    pub fn insert_formatting(
        &mut self,
        prefix: &str,
        suffix: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current_value = self.input_state.read(cx).value().to_string();

        self.input_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let (replacement, new_selection) =
                Self::formatting_replacement(&value, selection, prefix, suffix);

            state.replace(&replacement, window, cx);
            state.set_selection(new_selection.start, new_selection.end, window, cx);
        });

        let _ = current_value;
        info!(prefix = prefix, "Formatting inserted");
        cx.notify();
    }
}
