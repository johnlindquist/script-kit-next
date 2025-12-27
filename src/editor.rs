//! GPUI Editor Prompt Component
//!
//! A full-featured code editor for Script Kit with:
//! - Text editing (insert, delete, backspace)
//! - Cursor navigation (arrows, home/end, word movement)
//! - Selection (shift+arrows, cmd+a, mouse)
//! - Clipboard (cmd+c/v/x)
//! - Undo/redo (cmd+z, cmd+shift+z)
//! - Syntax highlighting
//! - Line numbers
//! - Monospace font

#![allow(dead_code)]

use gpui::{
    div, prelude::*, px, rgb, rgba, uniform_list, Context, FocusHandle, Focusable, Render,
    SharedString, UniformListScrollHandle, Window, ClipboardItem,
};
use ropey::Rope;
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use crate::logging;
use crate::syntax::{highlight_code_lines, HighlightedLine};
use crate::theme::Theme;

/// Callback for prompt submission
/// Signature: (id: String, value: Option<String>)
pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;

/// Character width in pixels (monospace)
const CHAR_WIDTH: f32 = 8.4;
/// Line height in pixels
const LINE_HEIGHT: f32 = 20.0;
/// Gutter width for line numbers
const GUTTER_WIDTH: f32 = 50.0;
/// Maximum undo history size
const MAX_UNDO_HISTORY: usize = 100;

/// Cursor position in the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// Line index (0-based)
    pub line: usize,
    /// Column index (0-based, in characters)
    pub column: usize,
}

impl CursorPosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    
    pub fn start() -> Self {
        Self { line: 0, column: 0 }
    }
}

/// Selection range in the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Anchor position (where selection started)
    pub anchor: CursorPosition,
    /// Head position (where cursor is, end of selection)
    pub head: CursorPosition,
}

impl Selection {
    pub fn new(anchor: CursorPosition, head: CursorPosition) -> Self {
        Self { anchor, head }
    }
    
    pub fn caret(pos: CursorPosition) -> Self {
        Self { anchor: pos, head: pos }
    }
    
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }
    
    /// Get the selection as an ordered range (start, end)
    pub fn ordered(&self) -> (CursorPosition, CursorPosition) {
        if self.anchor.line < self.head.line || 
           (self.anchor.line == self.head.line && self.anchor.column <= self.head.column) {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

/// Undo/redo state snapshot
#[derive(Debug, Clone)]
struct EditorSnapshot {
    content: String,
    cursor: CursorPosition,
    selection: Selection,
}

/// EditorPrompt - Full-featured code editor
pub struct EditorPrompt {
    // Identity
    pub id: String,

    // Content - using ropey for efficient text manipulation
    rope: Rope,
    language: String,

    // Cursor and selection
    cursor: CursorPosition,
    selection: Selection,
    cursor_visible: bool,

    // Display cache
    highlighted_lines: Vec<HighlightedLine>,
    needs_rehighlight: bool,
    scroll_handle: UniformListScrollHandle,

    // Undo/redo
    undo_stack: VecDeque<EditorSnapshot>,
    redo_stack: VecDeque<EditorSnapshot>,

    // GPUI
    focus_handle: FocusHandle,
    on_submit: SubmitCallback,
    theme: Arc<Theme>,
}

impl EditorPrompt {
    /// Create a new EditorPrompt
    pub fn new(
        id: String,
        content: String,
        language: String,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<Theme>,
    ) -> Self {
        logging::log(
            "EDITOR",
            &format!(
                "EditorPrompt::new id={}, lang={}, content_len={}",
                id, language, content.len()
            ),
        );

        let rope = Rope::from_str(&content);
        let highlighted_lines = highlight_code_lines(&content, &language);

        logging::log(
            "EDITOR",
            &format!(
                "Highlighted {} lines for language '{}'",
                highlighted_lines.len(),
                language
            ),
        );

        Self {
            id,
            rope,
            language,
            cursor: CursorPosition::start(),
            selection: Selection::caret(CursorPosition::start()),
            cursor_visible: true,
            highlighted_lines,
            needs_rehighlight: false,
            scroll_handle: UniformListScrollHandle::new(),
            undo_stack: VecDeque::with_capacity(MAX_UNDO_HISTORY),
            redo_stack: VecDeque::new(),
            focus_handle,
            on_submit,
            theme,
        }
    }

    /// Get the current content as a String
    pub fn content(&self) -> String {
        self.rope.to_string()
    }

    /// Get the language
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    /// Get a specific line's content
    fn get_line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            let line = self.rope.line(line_idx);
            // Remove trailing newline if present
            let s = line.to_string();
            Some(s.trim_end_matches('\n').to_string())
        } else {
            None
        }
    }

    /// Get the length of a specific line (in characters)
    fn line_len(&self, line_idx: usize) -> usize {
        if line_idx < self.rope.len_lines() {
            let line = self.rope.line(line_idx);
            let len = line.len_chars();
            // Don't count the newline character
            if len > 0 && line.char(len - 1) == '\n' {
                len - 1
            } else {
                len
            }
        } else {
            0
        }
    }

    /// Convert cursor position to rope char index
    fn cursor_to_char_idx(&self, pos: CursorPosition) -> usize {
        if pos.line >= self.rope.len_lines() {
            return self.rope.len_chars();
        }
        let line_start = self.rope.line_to_char(pos.line);
        let line_len = self.line_len(pos.line);
        line_start + pos.column.min(line_len)
    }

    /// Convert rope char index to cursor position
    fn char_idx_to_cursor(&self, char_idx: usize) -> CursorPosition {
        let char_idx = char_idx.min(self.rope.len_chars());
        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let column = char_idx - line_start;
        CursorPosition::new(line, column)
    }

    /// Save current state for undo
    fn save_undo_state(&mut self) {
        let snapshot = EditorSnapshot {
            content: self.rope.to_string(),
            cursor: self.cursor,
            selection: self.selection,
        };
        
        if self.undo_stack.len() >= MAX_UNDO_HISTORY {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(snapshot);
        self.redo_stack.clear();
    }

    /// Undo last action
    fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop_back() {
            // Save current state for redo
            let current = EditorSnapshot {
                content: self.rope.to_string(),
                cursor: self.cursor,
                selection: self.selection,
            };
            self.redo_stack.push_back(current);
            
            // Restore previous state
            self.rope = Rope::from_str(&snapshot.content);
            self.cursor = snapshot.cursor;
            self.selection = snapshot.selection;
            self.needs_rehighlight = true;
            logging::log("EDITOR", "Undo performed");
        }
    }

    /// Redo last undone action
    fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop_back() {
            // Save current state for undo
            let current = EditorSnapshot {
                content: self.rope.to_string(),
                cursor: self.cursor,
                selection: self.selection,
            };
            self.undo_stack.push_back(current);
            
            // Restore redo state
            self.rope = Rope::from_str(&snapshot.content);
            self.cursor = snapshot.cursor;
            self.selection = snapshot.selection;
            self.needs_rehighlight = true;
            logging::log("EDITOR", "Redo performed");
        }
    }

    /// Rehighlight the content if needed
    fn rehighlight_if_needed(&mut self) {
        if self.needs_rehighlight {
            self.highlighted_lines = highlight_code_lines(&self.rope.to_string(), &self.language);
            self.needs_rehighlight = false;
        }
    }

    /// Insert text at cursor position
    fn insert_text(&mut self, text: &str) {
        self.save_undo_state();
        
        // Delete selection first if any
        if !self.selection.is_empty() {
            self.delete_selection_internal();
        }
        
        let char_idx = self.cursor_to_char_idx(self.cursor);
        self.rope.insert(char_idx, text);
        
        // Move cursor after inserted text
        let new_idx = char_idx + text.chars().count();
        self.cursor = self.char_idx_to_cursor(new_idx);
        self.selection = Selection::caret(self.cursor);
        self.needs_rehighlight = true;
        
        logging::log("EDITOR", &format!("Inserted {} chars at {:?}", text.len(), self.cursor));
    }

    /// Insert a single character
    fn insert_char(&mut self, ch: char) {
        self.insert_text(&ch.to_string());
    }

    /// Insert a newline
    fn insert_newline(&mut self) {
        self.insert_text("\n");
    }

    /// Delete selection without saving undo (internal use)
    fn delete_selection_internal(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        
        let (start, end) = self.selection.ordered();
        let start_idx = self.cursor_to_char_idx(start);
        let end_idx = self.cursor_to_char_idx(end);
        
        self.rope.remove(start_idx..end_idx);
        self.cursor = start;
        self.selection = Selection::caret(start);
    }

    /// Delete selected text or character before cursor (backspace)
    fn backspace(&mut self) {
        self.save_undo_state();
        
        if !self.selection.is_empty() {
            self.delete_selection_internal();
        } else if self.cursor.line > 0 || self.cursor.column > 0 {
            let char_idx = self.cursor_to_char_idx(self.cursor);
            if char_idx > 0 {
                self.rope.remove((char_idx - 1)..char_idx);
                self.cursor = self.char_idx_to_cursor(char_idx - 1);
                self.selection = Selection::caret(self.cursor);
            }
        }
        self.needs_rehighlight = true;
    }

    /// Delete selected text or character after cursor
    fn delete(&mut self) {
        self.save_undo_state();
        
        if !self.selection.is_empty() {
            self.delete_selection_internal();
        } else {
            let char_idx = self.cursor_to_char_idx(self.cursor);
            if char_idx < self.rope.len_chars() {
                self.rope.remove(char_idx..(char_idx + 1));
            }
        }
        self.needs_rehighlight = true;
    }

    /// Move cursor left
    fn move_left(&mut self, extend_selection: bool) {
        let char_idx = self.cursor_to_char_idx(self.cursor);
        if char_idx > 0 {
            self.cursor = self.char_idx_to_cursor(char_idx - 1);
        }
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor right
    fn move_right(&mut self, extend_selection: bool) {
        let char_idx = self.cursor_to_char_idx(self.cursor);
        if char_idx < self.rope.len_chars() {
            self.cursor = self.char_idx_to_cursor(char_idx + 1);
        }
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor up
    fn move_up(&mut self, extend_selection: bool) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            let line_len = self.line_len(self.cursor.line);
            self.cursor.column = self.cursor.column.min(line_len);
        }
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor down
    fn move_down(&mut self, extend_selection: bool) {
        if self.cursor.line < self.line_count() - 1 {
            self.cursor.line += 1;
            let line_len = self.line_len(self.cursor.line);
            self.cursor.column = self.cursor.column.min(line_len);
        }
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor to start of line
    fn move_to_line_start(&mut self, extend_selection: bool) {
        self.cursor.column = 0;
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor to end of line
    fn move_to_line_end(&mut self, extend_selection: bool) {
        self.cursor.column = self.line_len(self.cursor.line);
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor to start of document
    fn move_to_document_start(&mut self, extend_selection: bool) {
        self.cursor = CursorPosition::start();
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor to end of document
    fn move_to_document_end(&mut self, extend_selection: bool) {
        let last_line = self.line_count().saturating_sub(1);
        self.cursor = CursorPosition::new(last_line, self.line_len(last_line));
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor by word (Option/Alt + arrow)
    fn move_word_left(&mut self, extend_selection: bool) {
        let char_idx = self.cursor_to_char_idx(self.cursor);
        if char_idx == 0 {
            return;
        }
        
        // Find the start of the previous word
        let text: String = self.rope.chars().take(char_idx).collect();
        let mut new_idx = char_idx;
        
        // Skip whitespace
        while new_idx > 0 {
            let ch = text.chars().nth(new_idx - 1).unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            new_idx -= 1;
        }
        
        // Skip word characters
        while new_idx > 0 {
            let ch = text.chars().nth(new_idx - 1).unwrap_or(' ');
            if ch.is_whitespace() || !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            new_idx -= 1;
        }
        
        self.cursor = self.char_idx_to_cursor(new_idx);
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Move cursor by word (Option/Alt + arrow)
    fn move_word_right(&mut self, extend_selection: bool) {
        let char_idx = self.cursor_to_char_idx(self.cursor);
        let total_chars = self.rope.len_chars();
        if char_idx >= total_chars {
            return;
        }
        
        let text: String = self.rope.chars().collect();
        let mut new_idx = char_idx;
        
        // Skip current word characters
        while new_idx < total_chars {
            let ch = text.chars().nth(new_idx).unwrap_or(' ');
            if ch.is_whitespace() || (!ch.is_alphanumeric() && ch != '_') {
                break;
            }
            new_idx += 1;
        }
        
        // Skip whitespace
        while new_idx < total_chars {
            let ch = text.chars().nth(new_idx).unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            new_idx += 1;
        }
        
        self.cursor = self.char_idx_to_cursor(new_idx);
        
        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::caret(self.cursor);
        }
    }

    /// Select all text
    fn select_all(&mut self) {
        self.selection.anchor = CursorPosition::start();
        let last_line = self.line_count().saturating_sub(1);
        self.cursor = CursorPosition::new(last_line, self.line_len(last_line));
        self.selection.head = self.cursor;
    }

    /// Get selected text
    fn get_selected_text(&self) -> String {
        if self.selection.is_empty() {
            return String::new();
        }
        
        let (start, end) = self.selection.ordered();
        let start_idx = self.cursor_to_char_idx(start);
        let end_idx = self.cursor_to_char_idx(end);
        
        self.rope.slice(start_idx..end_idx).to_string()
    }

    /// Copy selection to clipboard
    fn copy(&self, cx: &mut Context<Self>) {
        let text = self.get_selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            logging::log("EDITOR", "Copied to clipboard");
        }
    }

    /// Cut selection to clipboard
    fn cut(&mut self, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            return;
        }
        
        let text = self.get_selected_text();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        
        self.save_undo_state();
        self.delete_selection_internal();
        self.needs_rehighlight = true;
        logging::log("EDITOR", "Cut to clipboard");
    }

    /// Paste from clipboard
    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.insert_text(&text);
                logging::log("EDITOR", &format!("Pasted {} chars", text.len()));
            }
        }
    }

    /// Submit the current content
    fn submit(&self) {
        logging::log("EDITOR", &format!("Submit id={}", self.id));
        (self.on_submit)(self.id.clone(), Some(self.rope.to_string()));
    }

    /// Cancel - submit None
    fn cancel(&self) {
        logging::log("EDITOR", &format!("Cancel id={}", self.id));
        (self.on_submit)(self.id.clone(), None);
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.to_lowercase();
        let cmd = event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;
        let alt = event.keystroke.modifiers.alt;
        
        match (key.as_str(), cmd, shift, alt) {
            // Submit/Cancel
            ("enter", true, false, false) => self.submit(),
            ("escape", _, _, _) => self.cancel(),
            
            // Undo/Redo
            ("z", true, false, false) => self.undo(),
            ("z", true, true, false) => self.redo(),
            
            // Clipboard
            ("c", true, false, false) => self.copy(cx),
            ("x", true, false, false) => self.cut(cx),
            ("v", true, false, false) => self.paste(cx),
            
            // Select all
            ("a", true, false, false) => self.select_all(),
            
            // Navigation (basic arrow keys, with or without shift for selection)
            ("arrowleft", false, _, false) => self.move_left(shift),
            ("arrowright", false, _, false) => self.move_right(shift),
            ("arrowup", false, _, false) => self.move_up(shift),
            ("arrowdown", false, _, false) => self.move_down(shift),
            
            // Word navigation (Alt/Option + arrow)
            ("arrowleft", false, _, true) => self.move_word_left(shift),
            ("arrowright", false, _, true) => self.move_word_right(shift),
            
            // Line start/end (Cmd+Left/Right on Mac)
            ("arrowleft", true, _, false) => self.move_to_line_start(shift),
            ("arrowright", true, _, false) => self.move_to_line_end(shift),
            ("home", false, _, _) => self.move_to_line_start(shift),
            ("end", false, _, _) => self.move_to_line_end(shift),
            
            // Document start/end (Cmd+Up/Down on Mac, Cmd+Home/End)
            ("arrowup", true, _, false) => self.move_to_document_start(shift),
            ("arrowdown", true, _, false) => self.move_to_document_end(shift),
            ("home", true, _, _) => self.move_to_document_start(shift),
            ("end", true, _, _) => self.move_to_document_end(shift),
            
            // Editing
            ("backspace", _, _, _) => self.backspace(),
            ("delete", _, _, _) => self.delete(),
            ("enter", false, _, _) => self.insert_newline(),
            ("tab", false, false, false) => self.insert_text("    "), // 4 spaces for tab
            
            // Character input
            _ => {
                if let Some(ref key_char) = event.keystroke.key_char {
                    if let Some(ch) = key_char.chars().next() {
                        if !ch.is_control() && !cmd {
                            self.insert_char(ch);
                        }
                    }
                }
            }
        }
        
        cx.notify();
    }

    /// Render a range of lines for uniform_list virtualization
    fn render_lines(&mut self, range: Range<usize>, _cx: &mut Context<Self>) -> Vec<impl IntoElement> {
        self.rehighlight_if_needed();
        
        let colors = &self.theme.colors;
        let (sel_start, sel_end) = self.selection.ordered();
        
        range
            .map(|line_idx| {
                let line_content = self.get_line(line_idx).unwrap_or_default();
                let line_number = line_idx + 1;
                let highlighted_line = self.highlighted_lines.get(line_idx);
                
                // Check if cursor is on this line
                let cursor_on_line = self.cursor.line == line_idx && self.cursor_visible;
                let cursor_column = if cursor_on_line { Some(self.cursor.column) } else { None };
                
                // Check if this line has selection
                let line_has_selection = !self.selection.is_empty() &&
                    line_idx >= sel_start.line && line_idx <= sel_end.line;
                
                let selection_range = if line_has_selection {
                    let start_col = if line_idx == sel_start.line { sel_start.column } else { 0 };
                    let end_col = if line_idx == sel_end.line { sel_end.column } else { line_content.len() };
                    Some((start_col, end_col))
                } else {
                    None
                };
                
                self.render_line(line_idx, line_number, &line_content, highlighted_line, cursor_column, selection_range, colors)
            })
            .collect()
    }

    /// Render a single line with cursor and selection
    #[allow(clippy::too_many_arguments)]
    fn render_line(
        &self,
        line_idx: usize,
        line_number: usize,
        line_content: &str,
        highlighted_line: Option<&HighlightedLine>,
        cursor_column: Option<usize>,
        selection_range: Option<(usize, usize)>,
        colors: &crate::theme::ColorScheme,
    ) -> impl IntoElement {
        let line_height = px(LINE_HEIGHT);
        let gutter_width = px(GUTTER_WIDTH);
        
        // Build the line content with syntax highlighting, cursor, and selection
        let content_element = if let Some(hl_line) = highlighted_line {
            self.render_highlighted_line(line_content, hl_line, cursor_column, selection_range, colors)
        } else {
            self.render_plain_line(line_content, cursor_column, selection_range, colors)
        };
        
        div()
            .id(("editor-line", line_idx))
            .flex()
            .flex_row()
            .h(line_height)
            .w_full()
            .font_family("Menlo")
            .text_sm()
            .child(
                // Line number gutter
                div()
                    .w(gutter_width)
                    .flex_shrink_0()
                    .text_color(rgb(colors.text.muted))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(SharedString::from(format!("{}", line_number))),
            )
            .child(
                // Code content with cursor and selection
                div()
                    .flex_1()
                    .pl_2()
                    .flex()
                    .flex_row()
                    .items_center()
                    .overflow_hidden()
                    .child(content_element),
            )
    }

    /// Render a line with syntax highlighting
    fn render_highlighted_line(
        &self,
        _line_content: &str,
        hl_line: &HighlightedLine,
        cursor_column: Option<usize>,
        selection_range: Option<(usize, usize)>,
        colors: &crate::theme::ColorScheme,
    ) -> gpui::AnyElement {
        let mut elements: Vec<gpui::AnyElement> = Vec::new();
        let mut char_offset = 0;
        
        for span in &hl_line.spans {
            let span_len = span.text.chars().count();
            let span_start = char_offset;
            let span_end = char_offset + span_len;
            
            // Render this span, potentially with cursor and/or selection
            let span_element = self.render_span(
                &span.text,
                span.color,
                span_start,
                cursor_column,
                selection_range,
                colors,
            );
            elements.push(span_element.into_any_element());
            
            char_offset = span_end;
        }
        
        // If cursor is at the end of the line (after all content)
        if let Some(col) = cursor_column {
            if col >= char_offset {
                elements.push(self.render_cursor(colors).into_any_element());
            }
        }
        
        div().flex().flex_row().children(elements).into_any_element()
    }

    /// Render a plain line (no syntax highlighting)
    fn render_plain_line(
        &self,
        line_content: &str,
        cursor_column: Option<usize>,
        selection_range: Option<(usize, usize)>,
        colors: &crate::theme::ColorScheme,
    ) -> gpui::AnyElement {
        let span_element = self.render_span(
            line_content,
            colors.text.primary,
            0,
            cursor_column,
            selection_range,
            colors,
        );
        
        let mut elements: Vec<gpui::AnyElement> = vec![span_element.into_any_element()];
        
        // If cursor is at the end of the line
        if let Some(col) = cursor_column {
            if col >= line_content.chars().count() {
                elements.push(self.render_cursor(colors).into_any_element());
            }
        }
        
        div().flex().flex_row().children(elements).into_any_element()
    }

    /// Render a text span with potential cursor and selection
    fn render_span(
        &self,
        text: &str,
        text_color: u32,
        span_start: usize,
        cursor_column: Option<usize>,
        selection_range: Option<(usize, usize)>,
        colors: &crate::theme::ColorScheme,
    ) -> impl IntoElement {
        let span_len = text.chars().count();
        let span_end = span_start + span_len;
        
        // Check if cursor is within this span
        let cursor_in_span = cursor_column
            .map(|col| col >= span_start && col < span_end)
            .unwrap_or(false);
        
        // Check if selection overlaps this span
        let selection_in_span = selection_range
            .map(|(sel_start, sel_end)| sel_start < span_end && sel_end > span_start)
            .unwrap_or(false);
        
        if !cursor_in_span && !selection_in_span {
            // Simple case: no cursor or selection in this span
            return div()
                .text_color(rgb(text_color))
                .child(SharedString::from(text.to_string()))
                .into_any_element();
        }
        
        // Complex case: need to split the span
        let mut elements: Vec<gpui::AnyElement> = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        
        let mut i = 0;
        while i < chars.len() {
            let global_idx = span_start + i;
            
            // Check if cursor is at this position
            if cursor_column == Some(global_idx) {
                elements.push(self.render_cursor(colors).into_any_element());
            }
            
            // Determine if this character is selected
            let is_selected = selection_range
                .map(|(sel_start, sel_end)| global_idx >= sel_start && global_idx < sel_end)
                .unwrap_or(false);
            
            // Collect consecutive characters with same selection state
            let mut end_i = i + 1;
            while end_i < chars.len() {
                let next_global_idx = span_start + end_i;
                let next_selected = selection_range
                    .map(|(sel_start, sel_end)| next_global_idx >= sel_start && next_global_idx < sel_end)
                    .unwrap_or(false);
                
                // Break if selection state changes or cursor is here
                if next_selected != is_selected || cursor_column == Some(next_global_idx) {
                    break;
                }
                end_i += 1;
            }
            
            let chunk: String = chars[i..end_i].iter().collect();
            
            let chunk_element = if is_selected {
                div()
                    .bg(rgba(0x3399FF44))
                    .text_color(rgb(text_color))
                    .child(SharedString::from(chunk))
            } else {
                div()
                    .text_color(rgb(text_color))
                    .child(SharedString::from(chunk))
            };
            
            elements.push(chunk_element.into_any_element());
            i = end_i;
        }
        
        div().flex().flex_row().children(elements).into_any_element()
    }

    /// Render the cursor
    fn render_cursor(&self, colors: &crate::theme::ColorScheme) -> impl IntoElement {
        div()
            .w(px(2.0))
            .h(px(LINE_HEIGHT - 4.0))
            .bg(rgb(colors.accent.selected))
            .my(px(2.0))
    }

    /// Render the status bar at the bottom
    fn render_status_bar(&self) -> impl IntoElement {
        let colors = &self.theme.colors;
        let line_count = self.line_count();
        let cursor_info = format!("Ln {}, Col {}", self.cursor.line + 1, self.cursor.column + 1);

        div()
            .flex()
            .flex_row()
            .h(px(28.))
            .px_4()
            .items_center()
            .justify_between()
            .bg(rgb(colors.background.title_bar))
            .border_t_1()
            .border_color(rgb(colors.ui.border))
            .font_family("Menlo")
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .text_color(rgb(colors.text.secondary))
                            .text_xs()
                            .child(SharedString::from(format!("{} lines", line_count))),
                    )
                    .child(
                        div()
                            .text_color(rgb(colors.text.secondary))
                            .text_xs()
                            .child(SharedString::from(cursor_info)),
                    ),
            )
            .child(
                div()
                    .text_color(rgb(colors.text.muted))
                    .text_xs()
                    .child(SharedString::from(format!(
                        "{} | Cmd+Enter to submit, Escape to cancel",
                        self.language
                    ))),
            )
    }
}

impl Focusable for EditorPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;
        let line_count = self.line_count();

        // Keyboard handler
        let handle_key = cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
            this.handle_key_event(event, cx);
        });

        div()
            .id("editor-prompt")
            .key_context("EditorPrompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(colors.background.main))
            .font_family("Menlo")
            .child(
                // Editor content area with virtualized line rendering
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        uniform_list(
                            "editor-lines",
                            line_count,
                            cx.processor(|this, range: Range<usize>, _window, cx| {
                                this.render_lines(range, cx)
                            }),
                        )
                        .track_scroll(&self.scroll_handle)
                        .h_full(),
                    ),
            )
            .child(self.render_status_bar())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position() {
        let pos = CursorPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_selection_ordered() {
        let sel = Selection::new(
            CursorPosition::new(5, 10),
            CursorPosition::new(2, 5),
        );
        let (start, end) = sel.ordered();
        assert_eq!(start.line, 2);
        assert_eq!(end.line, 5);
    }

    #[test]
    fn test_selection_is_empty() {
        let pos = CursorPosition::new(3, 7);
        let sel = Selection::caret(pos);
        assert!(sel.is_empty());
        
        let sel2 = Selection::new(
            CursorPosition::new(0, 0),
            CursorPosition::new(0, 5),
        );
        assert!(!sel2.is_empty());
    }

    #[test]
    fn test_line_count_empty() {
        let content = "";
        let lines = highlight_code_lines(content, "text");
        assert!(lines.is_empty() || lines.len() == 1);
    }

    #[test]
    fn test_line_count_multiline() {
        let content = "line1\nline2\nline3";
        let lines = highlight_code_lines(content, "text");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_typescript_highlighting() {
        let content = "const x: number = 42;";
        let lines = highlight_code_lines(content, "typescript");
        assert!(!lines.is_empty());
        assert!(!lines[0].spans.is_empty());
    }
}
