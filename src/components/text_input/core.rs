use gpui::{ClipboardItem, Context, Render};

/// Selection in a single-line text input
/// anchor = where selection started, cursor = current position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextSelection {
    /// Where selection started (fixed point)
    pub anchor: usize,
    /// Current cursor position (moves with arrows)
    pub cursor: usize,
}

impl TextSelection {
    pub fn caret(pos: usize) -> Self {
        Self {
            anchor: pos,
            cursor: pos,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.cursor
    }

    /// Get selection as ordered range (start, end)
    pub fn range(&self) -> (usize, usize) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }

    /// Get the length of the selection
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let (start, end) = self.range();
        end - start
    }
}

/// State for a single-line text input with selection support
#[derive(Debug, Clone)]
pub struct TextInputState {
    /// The text content
    text: String,
    /// Selection state (anchor and cursor positions)
    selection: TextSelection,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selection: TextSelection::caret(0),
        }
    }

    #[allow(dead_code)]
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let len = text.chars().count();
        Self {
            text,
            selection: TextSelection::caret(len), // Cursor at end
        }
    }

    // === Getters ===

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.selection.cursor
    }

    pub fn selection(&self) -> TextSelection {
        self.selection
    }

    pub fn has_selection(&self) -> bool {
        !self.selection.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get selected text, or empty string if no selection
    pub fn selected_text(&self) -> &str {
        if self.selection.is_empty() {
            return "";
        }
        let (start, end) = self.selection.range();
        let start_byte = self.char_to_byte(start);
        let end_byte = self.char_to_byte(end);
        &self.text[start_byte..end_byte]
    }

    /// Get display text (masked if secret)
    pub fn display_text(&self, is_secret: bool) -> String {
        if is_secret && !self.text.is_empty() {
            "•".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }

    // === Setters ===

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        let len = self.text.chars().count();
        self.selection = TextSelection::caret(len.min(self.selection.cursor));
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.selection = TextSelection::caret(0);
    }

    // === Text Manipulation ===

    /// Insert a character at cursor, replacing selection if any
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection();
        let byte_pos = self.char_to_byte(self.selection.cursor);
        self.text.insert(byte_pos, ch);
        self.selection = TextSelection::caret(self.selection.cursor + 1);
    }

    /// Insert a string at cursor, replacing selection if any
    pub fn insert_str(&mut self, s: &str) {
        self.delete_selection();
        let byte_pos = self.char_to_byte(self.selection.cursor);
        self.text.insert_str(byte_pos, s);
        let inserted_chars = s.chars().count();
        self.selection = TextSelection::caret(self.selection.cursor + inserted_chars);
    }

    /// Delete selection, or character before cursor if no selection
    pub fn backspace(&mut self) {
        if !self.selection.is_empty() {
            self.delete_selection();
        } else if self.selection.cursor > 0 {
            let new_pos = self.selection.cursor - 1;
            let byte_start = self.char_to_byte(new_pos);
            let byte_end = self.char_to_byte(self.selection.cursor);
            self.text.replace_range(byte_start..byte_end, "");
            self.selection = TextSelection::caret(new_pos);
        }
    }

    /// Delete selection, or character after cursor if no selection
    pub fn delete(&mut self) {
        if !self.selection.is_empty() {
            self.delete_selection();
        } else {
            let len = self.text.chars().count();
            if self.selection.cursor < len {
                let byte_start = self.char_to_byte(self.selection.cursor);
                let byte_end = self.char_to_byte(self.selection.cursor + 1);
                self.text.replace_range(byte_start..byte_end, "");
            }
        }
    }

    /// Delete the selected text (internal)
    fn delete_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        let (start, end) = self.selection.range();
        let byte_start = self.char_to_byte(start);
        let byte_end = self.char_to_byte(end);
        self.text.replace_range(byte_start..byte_end, "");
        self.selection = TextSelection::caret(start);
    }

    // === Cursor Movement ===

    /// Move cursor left, optionally extending selection
    pub fn move_left(&mut self, extend_selection: bool) {
        if !extend_selection && !self.selection.is_empty() {
            // Collapse to start of selection
            let (start, _) = self.selection.range();
            self.selection = TextSelection::caret(start);
        } else if self.selection.cursor > 0 {
            let new_pos = self.selection.cursor - 1;
            if extend_selection {
                self.selection.cursor = new_pos;
            } else {
                self.selection = TextSelection::caret(new_pos);
            }
        }
    }

    /// Move cursor right, optionally extending selection
    pub fn move_right(&mut self, extend_selection: bool) {
        let len = self.text.chars().count();
        if !extend_selection && !self.selection.is_empty() {
            // Collapse to end of selection
            let (_, end) = self.selection.range();
            self.selection = TextSelection::caret(end);
        } else if self.selection.cursor < len {
            let new_pos = self.selection.cursor + 1;
            if extend_selection {
                self.selection.cursor = new_pos;
            } else {
                self.selection = TextSelection::caret(new_pos);
            }
        }
    }

    /// Move cursor to start of line, optionally extending selection
    pub fn move_to_start(&mut self, extend_selection: bool) {
        if extend_selection {
            self.selection.cursor = 0;
        } else {
            self.selection = TextSelection::caret(0);
        }
    }

    /// Move cursor to end of line, optionally extending selection
    pub fn move_to_end(&mut self, extend_selection: bool) {
        let len = self.text.chars().count();
        if extend_selection {
            self.selection.cursor = len;
        } else {
            self.selection = TextSelection::caret(len);
        }
    }

    /// Move cursor to previous word boundary
    pub fn move_word_left(&mut self, extend_selection: bool) {
        let new_pos = self.find_word_boundary_left();
        if extend_selection {
            self.selection.cursor = new_pos;
        } else {
            self.selection = TextSelection::caret(new_pos);
        }
    }

    /// Move cursor to next word boundary
    pub fn move_word_right(&mut self, extend_selection: bool) {
        let new_pos = self.find_word_boundary_right();
        if extend_selection {
            self.selection.cursor = new_pos;
        } else {
            self.selection = TextSelection::caret(new_pos);
        }
    }

    /// Select all text
    pub fn select_all(&mut self) {
        let len = self.text.chars().count();
        self.selection = TextSelection {
            anchor: 0,
            cursor: len,
        };
    }

    // === Clipboard Operations ===

    /// Copy selected text to clipboard
    pub fn copy<T: Render>(&self, cx: &mut Context<T>) {
        if !self.selection.is_empty() {
            let text = self.selected_text().to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    /// Cut selected text to clipboard
    pub fn cut<T: Render>(&mut self, cx: &mut Context<T>) {
        if !self.selection.is_empty() {
            let text = self.selected_text().to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.delete_selection();
        }
    }

    /// Paste from clipboard
    pub fn paste<T: Render>(&mut self, cx: &mut Context<T>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                // Filter to single line (no newlines)
                let single_line: String =
                    text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
                self.insert_str(&single_line);
            }
        }
    }

    // === Key Handling ===

    /// Handle a key event. Returns true if the event was handled.
    pub fn handle_key<T: Render>(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        cmd: bool,
        alt: bool,
        shift: bool,
        cx: &mut Context<T>,
    ) -> bool {
        // Clipboard
        if cmd && !alt && key.eq_ignore_ascii_case("c") {
            self.copy(cx);
            return true;
        }
        if cmd && !alt && key.eq_ignore_ascii_case("x") {
            self.cut(cx);
            return true;
        }
        if cmd && !alt && key.eq_ignore_ascii_case("v") {
            self.paste(cx);
            return true;
        }
        if cmd && !alt && key.eq_ignore_ascii_case("a") {
            self.select_all();
            return true;
        }

        // Navigation
        if key.eq_ignore_ascii_case("left") || key.eq_ignore_ascii_case("arrowleft") {
            if cmd {
                self.move_to_start(shift);
            } else if alt {
                self.move_word_left(shift);
            } else {
                self.move_left(shift);
            }
            return true;
        }
        if key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright") {
            if cmd {
                self.move_to_end(shift);
            } else if alt {
                self.move_word_right(shift);
            } else {
                self.move_right(shift);
            }
            return true;
        }
        if key.eq_ignore_ascii_case("home") {
            self.move_to_start(shift);
            return true;
        }
        if key.eq_ignore_ascii_case("end") {
            self.move_to_end(shift);
            return true;
        }

        // Deletion
        if key.eq_ignore_ascii_case("backspace") {
            if cmd {
                // Cmd+Backspace: delete to start of line
                let (_, end) = self.selection.range();
                self.selection = TextSelection {
                    anchor: 0,
                    cursor: end,
                };
                self.delete_selection();
            } else if alt {
                // Alt+Backspace: delete word left
                let start = self.find_word_boundary_left();
                let end = self.selection.cursor;
                if start < end {
                    self.selection = TextSelection {
                        anchor: start,
                        cursor: end,
                    };
                    self.delete_selection();
                }
            } else {
                self.backspace();
            }
            return true;
        }
        if key.eq_ignore_ascii_case("delete") {
            if alt {
                // Alt+Delete: delete word right
                let start = self.selection.cursor;
                let end = self.find_word_boundary_right();
                if start < end {
                    self.selection = TextSelection {
                        anchor: start,
                        cursor: end,
                    };
                    self.delete_selection();
                }
            } else {
                self.delete();
            }
            return true;
        }

        // Character input (no cmd modifier)
        if !cmd {
            if let Some(key_char) = key_char {
                if let Some(ch) = key_char.chars().next() {
                    if !ch.is_control() {
                        self.insert_char(ch);
                        return true;
                    }
                }
            }
        }

        false
    }

    // === Helper Methods ===

    /// Convert character index to byte index
    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Find the previous word boundary from cursor
    fn find_word_boundary_left(&self) -> usize {
        if self.selection.cursor == 0 {
            return 0;
        }

        let chars: Vec<char> = self.text.chars().collect();
        let mut pos = self.selection.cursor - 1;

        // Skip whitespace
        while pos > 0 && chars[pos].is_whitespace() {
            pos -= 1;
        }

        // Skip word characters
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        pos
    }

    /// Find the next word boundary from cursor
    fn find_word_boundary_right(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let len = chars.len();
        let mut pos = self.selection.cursor;

        // Skip current word
        while pos < len && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip whitespace
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }

        pos
    }
}
