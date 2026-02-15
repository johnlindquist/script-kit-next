use gpui::*;

use crate::protocol::Field;

use super::helpers::{byte_idx_from_char_idx, char_len, drain_char_range, slice_by_char_range};
use super::{FormFieldColors, FormFieldState};

mod render;

#[derive(Clone, Copy)]
enum VerticalCursorDirection {
    Up,
    Down,
}

fn line_start_for_char_position(value: &str, char_position: usize) -> usize {
    let mut line_start = 0;
    for (idx, ch) in value.chars().enumerate() {
        if idx >= char_position {
            break;
        }
        if ch == '\n' {
            line_start = idx + 1;
        }
    }
    line_start
}

fn line_end_for_line_start(value: &str, line_start: usize) -> usize {
    let text_len = char_len(value);
    for (idx, ch) in value.chars().enumerate().skip(line_start) {
        if ch == '\n' {
            return idx;
        }
    }
    text_len
}

fn move_cursor_vertical_preserve_column(
    value: &str,
    cursor_position: usize,
    direction: VerticalCursorDirection,
) -> usize {
    let text_len = char_len(value);
    let cursor_position = cursor_position.min(text_len);
    let current_line_start = line_start_for_char_position(value, cursor_position);
    let current_column = cursor_position.saturating_sub(current_line_start);

    match direction {
        VerticalCursorDirection::Up => {
            if current_line_start == 0 {
                return cursor_position;
            }
            let previous_line_end = current_line_start.saturating_sub(1);
            let previous_line_start = line_start_for_char_position(value, previous_line_end);
            let previous_line_len = previous_line_end.saturating_sub(previous_line_start);
            previous_line_start + current_column.min(previous_line_len)
        }
        VerticalCursorDirection::Down => {
            let current_line_end = line_end_for_line_start(value, current_line_start);
            if current_line_end >= text_len {
                return cursor_position;
            }
            let next_line_start = current_line_end + 1;
            let next_line_end = line_end_for_line_start(value, next_line_start);
            let next_line_len = next_line_end.saturating_sub(next_line_start);
            next_line_start + current_column.min(next_line_len)
        }
    }
}

pub struct FormTextArea {
    /// Field definition from protocol
    field: Field,
    /// Pre-computed colors
    colors: FormFieldColors,
    /// Current text value (lines)
    pub value: String,
    /// Cursor position in the text (char index)
    pub cursor_position: usize,
    /// Selection anchor (char index), None if no selection
    pub selection_anchor: Option<usize>,
    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,
    /// Number of visible rows
    rows: usize,
    /// Shared state for external access
    pub state: FormFieldState,
}

impl FormTextArea {
    /// Create a new text area from a Field definition
    pub fn new(field: Field, colors: FormFieldColors, rows: usize, cx: &mut App) -> Self {
        let initial_value = field.value.clone().unwrap_or_default();
        let cursor_pos = char_len(&initial_value);
        let state = FormFieldState::new(initial_value.clone());

        Self {
            field,
            colors,
            value: initial_value,
            cursor_position: cursor_pos,
            selection_anchor: None,
            focus_handle: cx.focus_handle(),
            rows,
            state,
        }
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get the field name
    pub fn name(&self) -> &str {
        &self.field.name
    }

    /// Set the value programmatically
    pub fn set_value(&mut self, value: String) {
        self.cursor_position = char_len(&value);
        self.selection_anchor = None;
        self.value = value.clone();
        self.state.set_value(value);
    }

    /// Get the focus handle for this text area
    ///
    /// This allows parent components to delegate focus to this field.
    /// Used by FormPromptState to implement the Focusable trait by returning
    /// the child's focus handle instead of its own, preventing focus stealing.
    pub fn get_focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    // ───── Selection helpers ─────

    /// Get selection range as (start, end) in char indices, ordered
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            let start = anchor.min(self.cursor_position);
            let end = anchor.max(self.cursor_position);
            (start, end)
        })
    }

    /// Check if there is an active selection
    fn has_selection(&self) -> bool {
        self.selection_anchor
            .is_some_and(|a| a != self.cursor_position)
    }

    /// Get selected text
    fn selected_text(&self) -> Option<String> {
        self.selection_range()
            .map(|(start, end)| slice_by_char_range(&self.value, start, end).to_string())
    }

    /// Delete selected text, collapse cursor to start
    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            drain_char_range(&mut self.value, start, end);
            self.cursor_position = start;
            self.selection_anchor = None;
            self.state.set_value(self.value.clone());
        }
    }

    /// Clear selection without deleting
    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Select all text
    fn select_all(&mut self) {
        let len = char_len(&self.value);
        if len > 0 {
            self.selection_anchor = Some(0);
            self.cursor_position = len;
        }
    }

    // ───── Clipboard ─────

    fn copy(&self, cx: &mut Context<Self>) {
        if let Some(text) = self.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn cut(&mut self, cx: &mut Context<Self>) {
        self.copy(cx);
        self.delete_selection();
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.insert_text_at_cursor(&text);
            }
        }
    }

    // ───── Cursor movement ─────

    fn move_left(&mut self, extend_selection: bool) {
        if !extend_selection {
            // If selection exists, collapse to start
            if let Some((start, _)) = self.selection_range() {
                self.cursor_position = start;
                self.clear_selection();
                return;
            }
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        }
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_right(&mut self, extend_selection: bool) {
        let len = char_len(&self.value);
        if !extend_selection {
            if let Some((_, end)) = self.selection_range() {
                self.cursor_position = end;
                self.clear_selection();
                return;
            }
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        }
        if self.cursor_position < len {
            self.cursor_position += 1;
        }
        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_home(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        }
        self.cursor_position = 0;
        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_end(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        }
        self.cursor_position = char_len(&self.value);
        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_up(&mut self, extend_selection: bool) {
        if !extend_selection {
            if let Some((start, _)) = self.selection_range() {
                self.cursor_position = start;
                self.clear_selection();
                return;
            }
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        }

        self.cursor_position = move_cursor_vertical_preserve_column(
            &self.value,
            self.cursor_position,
            VerticalCursorDirection::Up,
        );
        if !extend_selection {
            self.clear_selection();
        }
    }

    fn move_down(&mut self, extend_selection: bool) {
        if !extend_selection {
            if let Some((_, end)) = self.selection_range() {
                self.cursor_position = end;
                self.clear_selection();
                return;
            }
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        }

        self.cursor_position = move_cursor_vertical_preserve_column(
            &self.value,
            self.cursor_position,
            VerticalCursorDirection::Down,
        );
        if !extend_selection {
            self.clear_selection();
        }
    }

    // ───── Editing ─────

    fn insert_text_at_cursor(&mut self, text: &str) {
        if self.has_selection() {
            self.delete_selection();
        }
        let byte_idx = byte_idx_from_char_idx(&self.value, self.cursor_position);
        self.value.insert_str(byte_idx, text);
        self.cursor_position += char_len(text);
        self.state.set_value(self.value.clone());
    }

    fn backspace_char(&mut self) {
        if self.has_selection() {
            self.delete_selection();
        } else if self.cursor_position > 0 {
            drain_char_range(
                &mut self.value,
                self.cursor_position - 1,
                self.cursor_position,
            );
            self.cursor_position -= 1;
            self.state.set_value(self.value.clone());
        }
    }

    fn delete_forward_char(&mut self) {
        if self.has_selection() {
            self.delete_selection();
        } else if self.cursor_position < char_len(&self.value) {
            drain_char_range(
                &mut self.value,
                self.cursor_position,
                self.cursor_position + 1,
            );
            self.state.set_value(self.value.clone());
        }
    }

    /// Handle text input (legacy, kept for render callback)
    fn handle_input(&mut self, text: &str, _cx: &mut Context<Self>) {
        self.insert_text_at_cursor(text);
    }

    /// Handle key down events (legacy, kept for render callback)
    fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let cmd = event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        // Select all
        if cmd && !shift && key.eq_ignore_ascii_case("a") {
            self.select_all();
            cx.notify();
            return;
        }
        // Clipboard
        if cmd && !shift && key.eq_ignore_ascii_case("c") {
            self.copy(cx);
            return;
        }
        if cmd && !shift && key.eq_ignore_ascii_case("x") {
            self.cut(cx);
            cx.notify();
            return;
        }
        if cmd && !shift && key.eq_ignore_ascii_case("v") {
            self.paste(cx);
            cx.notify();
            return;
        }
        // Navigation with optional selection
        if !cmd && (key.eq_ignore_ascii_case("left") || key.eq_ignore_ascii_case("arrowleft")) {
            self.move_left(shift);
            cx.notify();
            return;
        }
        if !cmd && (key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright")) {
            self.move_right(shift);
            cx.notify();
            return;
        }
        if !cmd && (key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("arrowup")) {
            self.move_up(shift);
            cx.notify();
            return;
        }
        if !cmd && (key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("arrowdown")) {
            self.move_down(shift);
            cx.notify();
            return;
        }
        if !cmd && key.eq_ignore_ascii_case("home") {
            self.move_home(shift);
            cx.notify();
            return;
        }
        if !cmd && key.eq_ignore_ascii_case("end") {
            self.move_end(shift);
            cx.notify();
            return;
        }
        // Editing
        if !cmd && key.eq_ignore_ascii_case("backspace") {
            self.backspace_char();
            cx.notify();
            return;
        }
        if !cmd && key.eq_ignore_ascii_case("delete") {
            self.delete_forward_char();
            cx.notify();
            return;
        }
        // Enter inserts newline
        if !cmd && key.eq_ignore_ascii_case("enter") {
            self.insert_text_at_cursor("\n");
            cx.notify();
        }
    }

    /// Unified key event handler called by form_prompt.rs
    ///
    /// Handles: Selection (Shift+Arrow), Clipboard (Cmd+C/X/V/A),
    /// Navigation (Arrow, Home, End), Editing (Backspace, Delete, Enter),
    /// and printable character input.
    pub fn handle_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let cmd = event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        // Select all
        if cmd && !shift && key.eq_ignore_ascii_case("a") {
            self.select_all();
            cx.notify();
            return;
        }
        // Clipboard
        if cmd && !shift && key.eq_ignore_ascii_case("c") {
            self.copy(cx);
            return;
        }
        if cmd && !shift && key.eq_ignore_ascii_case("x") {
            self.cut(cx);
            cx.notify();
            return;
        }
        if cmd && !shift && key.eq_ignore_ascii_case("v") {
            self.paste(cx);
            cx.notify();
            return;
        }
        // Navigation with optional selection
        if !cmd && (key.eq_ignore_ascii_case("left") || key.eq_ignore_ascii_case("arrowleft")) {
            self.move_left(shift);
            cx.notify();
            return;
        }
        if !cmd && (key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright")) {
            self.move_right(shift);
            cx.notify();
            return;
        }
        if !cmd && (key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("arrowup")) {
            self.move_up(shift);
            cx.notify();
            return;
        }
        if !cmd && (key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("arrowdown")) {
            self.move_down(shift);
            cx.notify();
            return;
        }
        if !cmd && key.eq_ignore_ascii_case("home") {
            self.move_home(shift);
            cx.notify();
            return;
        }
        if !cmd && key.eq_ignore_ascii_case("end") {
            self.move_end(shift);
            cx.notify();
            return;
        }
        // Editing
        if !cmd && key.eq_ignore_ascii_case("backspace") {
            self.backspace_char();
            cx.notify();
            return;
        }
        if !cmd && key.eq_ignore_ascii_case("delete") {
            self.delete_forward_char();
            cx.notify();
            return;
        }
        // Enter inserts newline
        if !cmd && key.eq_ignore_ascii_case("enter") {
            self.insert_text_at_cursor("\n");
            cx.notify();
            return;
        }

        // Printable character input (ignore when cmd/ctrl held)
        if !cmd {
            if let Some(ref key_char) = event.keystroke.key_char {
                let s = key_char.to_string();
                if !s.is_empty() && !s.chars().all(|c| c.is_control()) {
                    self.insert_text_at_cursor(&s);
                    cx.notify();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{move_cursor_vertical_preserve_column, VerticalCursorDirection};

    #[test]
    fn test_move_cursor_vertical_preserve_column_clamps_column_when_target_line_is_shorter() {
        let text = "first line\ntiny\nlonger third";
        let moved = move_cursor_vertical_preserve_column(text, 8, VerticalCursorDirection::Down);
        assert_eq!(moved, 15);
    }

    #[test]
    fn test_move_cursor_vertical_preserve_column_moves_to_previous_line_when_up_requested() {
        let text = "first line\ntiny\nlonger third";
        let moved = move_cursor_vertical_preserve_column(text, 20, VerticalCursorDirection::Up);
        assert_eq!(moved, 15);
    }

    #[test]
    fn test_move_cursor_vertical_preserve_column_keeps_cursor_when_moving_up_on_first_line() {
        let text = "abc\ndef";
        let moved = move_cursor_vertical_preserve_column(text, 2, VerticalCursorDirection::Up);
        assert_eq!(moved, 2);
    }

    #[test]
    fn test_move_cursor_vertical_preserve_column_keeps_cursor_when_moving_down_on_last_line() {
        let text = "abc\ndef";
        let moved = move_cursor_vertical_preserve_column(text, 6, VerticalCursorDirection::Down);
        assert_eq!(moved, 6);
    }
}
