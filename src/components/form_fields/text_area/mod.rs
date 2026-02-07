use gpui::*;

use crate::protocol::Field;

use super::helpers::{byte_idx_from_char_idx, char_len, drain_char_range, slice_by_char_range};
use super::{FormFieldColors, FormFieldState};

mod render;

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
        let key = event.keystroke.key.as_str().to_lowercase();
        let cmd = event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        match (key.as_str(), cmd, shift) {
            // Select all
            ("a", true, false) => {
                self.select_all();
                cx.notify();
            }
            // Clipboard
            ("c", true, false) => {
                self.copy(cx);
            }
            ("x", true, false) => {
                self.cut(cx);
                cx.notify();
            }
            ("v", true, false) => {
                self.paste(cx);
                cx.notify();
            }
            // Navigation with optional selection
            ("left" | "arrowleft", false, s) => {
                self.move_left(s);
                cx.notify();
            }
            ("right" | "arrowright", false, s) => {
                self.move_right(s);
                cx.notify();
            }
            ("home", false, s) => {
                self.move_home(s);
                cx.notify();
            }
            ("end", false, s) => {
                self.move_end(s);
                cx.notify();
            }
            // Editing
            ("backspace", false, _) => {
                self.backspace_char();
                cx.notify();
            }
            ("delete", false, _) => {
                self.delete_forward_char();
                cx.notify();
            }
            // Enter inserts newline
            ("enter", false, _) => {
                self.insert_text_at_cursor("\n");
                cx.notify();
            }
            _ => {}
        }
    }

    /// Unified key event handler called by form_prompt.rs
    ///
    /// Handles: Selection (Shift+Arrow), Clipboard (Cmd+C/X/V/A),
    /// Navigation (Arrow, Home, End), Editing (Backspace, Delete, Enter),
    /// and printable character input.
    pub fn handle_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str().to_lowercase();
        let cmd = event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        match (key.as_str(), cmd, shift) {
            // Select all
            ("a", true, false) => {
                self.select_all();
                cx.notify();
                return;
            }
            // Clipboard
            ("c", true, false) => {
                self.copy(cx);
                return;
            }
            ("x", true, false) => {
                self.cut(cx);
                cx.notify();
                return;
            }
            ("v", true, false) => {
                self.paste(cx);
                cx.notify();
                return;
            }
            // Navigation with optional selection
            ("left" | "arrowleft", false, s) => {
                self.move_left(s);
                cx.notify();
                return;
            }
            ("right" | "arrowright", false, s) => {
                self.move_right(s);
                cx.notify();
                return;
            }
            ("home", false, s) => {
                self.move_home(s);
                cx.notify();
                return;
            }
            ("end", false, s) => {
                self.move_end(s);
                cx.notify();
                return;
            }
            // Editing
            ("backspace", false, _) => {
                self.backspace_char();
                cx.notify();
                return;
            }
            ("delete", false, _) => {
                self.delete_forward_char();
                cx.notify();
                return;
            }
            // Enter inserts newline
            ("enter", false, _) => {
                self.insert_text_at_cursor("\n");
                cx.notify();
                return;
            }
            _ => {}
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
