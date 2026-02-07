use super::*;

impl NotesApp {
    pub(super) fn delete_current_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Calculate what to remove: include the trailing newline if present, or leading newline
            let (remove_start, remove_end) = if line_end < value.len() {
                // There's a newline after — remove line + newline
                (line_start, line_end + 1)
            } else if line_start > 0 {
                // Last line — remove leading newline + line
                (line_start - 1, line_end)
            } else {
                // Only line — clear everything
                (0, value.len())
            };

            let new_value = format!("{}{}", &value[..remove_start], &value[remove_end..]);
            let new_cursor = remove_start.min(new_value.len());
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Deleted current line");
        cx.notify();
    }

    /// Insert 2 spaces at cursor position (Tab key)
    pub(super) fn indent_at_cursor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());
            let indent = "  ";
            let new_value = format!("{}{}{}", &value[..cursor], indent, &value[cursor..]);
            let new_cursor = cursor + indent.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Remove up to 2 leading spaces from the current line (Shift+Tab)
    pub(super) fn outdent_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line = &value[line_start..];

            let remove_count = if line.starts_with("  ") {
                2
            } else if line.starts_with(' ') {
                1
            } else {
                return;
            };

            let new_value = format!(
                "{}{}",
                &value[..line_start],
                &value[line_start + remove_count..]
            );
            let new_cursor = cursor.saturating_sub(remove_count).max(line_start);
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Toggle bullet list prefix on current line (Cmd+Shift+8)
    ///
    /// Behavior:
    /// - Plain text → `- text`
    /// - `- text` → plain text (strip prefix)
    /// - `- [ ] text` or `- [x] text` → `- text` (strip checkbox, keep bullet)
    pub(super) fn toggle_bullet_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_line, cursor_delta): (String, isize) =
                if let Some(rest) = line.strip_prefix("- [x] ") {
                    // Checkbox → bullet only (remove checkbox, keep "- ")
                    (format!("- {}", rest), -4)
                } else if let Some(rest) = line.strip_prefix("- [ ] ") {
                    // Checkbox → bullet only
                    (format!("- {}", rest), -4)
                } else if let Some(rest) = line.strip_prefix("- ") {
                    // Bullet → plain (remove "- ")
                    (rest.to_string(), -2)
                } else {
                    // Plain → bullet (add "- ")
                    (format!("- {}", line), 2)
                };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(line_start as isize) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled bullet list on current line");
        cx.notify();
    }

    /// Toggle numbered list prefix on current line (Cmd+Shift+7)
    ///
    /// Behavior:
    /// - Plain text → `1. text` (auto-detects sequence from previous line)
    /// - `N. text` → plain text (strip numbered prefix)
    pub(super) fn toggle_numbered_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            // Check if line already has a numbered list prefix (e.g., "1. ", "12. ")
            let numbered_prefix_len = Self::numbered_list_prefix_len(line);

            let (new_line, cursor_delta): (String, isize) = if numbered_prefix_len > 0 {
                // Remove numbered prefix
                let rest = &line[numbered_prefix_len..];
                (rest.to_string(), -(numbered_prefix_len as isize))
            } else {
                // Add numbered prefix — detect number from previous line
                let num = Self::detect_next_list_number(&value, line_start);
                let prefix = format!("{}. ", num);
                let prefix_len = prefix.len() as isize;
                (format!("{}{}", prefix, line), prefix_len)
            };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(line_start as isize) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled numbered list on current line");
        cx.notify();
    }

    /// Get the length of a numbered list prefix (e.g., "1. " → 3, "12. " → 4, "abc" → 0)
    pub(super) fn numbered_list_prefix_len(line: &str) -> usize {
        let mut chars = line.chars().peekable();
        let mut digit_count = 0;

        // Count leading digits
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                digit_count += 1;
                chars.next();
            } else {
                break;
            }
        }

        if digit_count == 0 {
            return 0;
        }

        // Must be followed by ". "
        if chars.next() == Some('.') && chars.next() == Some(' ') {
            digit_count + 2 // digits + ". "
        } else {
            0
        }
    }

    /// Detect the next number for a numbered list by looking at the previous line
    pub(super) fn detect_next_list_number(value: &str, current_line_start: usize) -> usize {
        if current_line_start == 0 {
            return 1;
        }
        // Find previous line
        let prev_line_end = current_line_start - 1; // skip the \n
        let prev_line_start = value[..prev_line_end].rfind('\n').map_or(0, |p| p + 1);
        let prev_line = &value[prev_line_start..prev_line_end];

        // Check if previous line has a numbered prefix
        let prefix_len = Self::numbered_list_prefix_len(prev_line);
        if prefix_len > 0 {
            // Parse the number from the previous line
            let num_str: String = prev_line
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            num_str.parse::<usize>().unwrap_or(1) + 1
        } else {
            1
        }
    }

    /// Join the current line with the next line (Cmd+J)
    ///
    /// Replaces the newline between current and next line with a single space.
    pub(super) fn join_lines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find end of current line
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Can't join if on the last line
            if line_end >= value.len() {
                return;
            }

            // Find the start of actual content on the next line (skip leading whitespace)
            let next_content_start = value[line_end + 1..]
                .find(|c: char| !c.is_whitespace() || c == '\n')
                .map_or(value.len(), |p| line_end + 1 + p);

            // If next line is empty or only whitespace, just remove the newline
            let next_char = value.as_bytes().get(next_content_start);
            let (new_value, join_cursor) = if next_char == Some(&b'\n') || next_char.is_none() {
                // Next line is blank — remove it
                let end = if next_content_start < value.len() {
                    next_content_start + 1 // include the trailing \n
                } else {
                    next_content_start
                };
                let new_value = format!("{}{}", &value[..line_end], &value[end..]);
                (new_value, line_end)
            } else {
                // Join with a space
                let new_value = format!("{} {}", &value[..line_end], &value[next_content_start..]);
                (new_value, line_end)
            };

            state.set_value(&new_value, window, cx);
            state.set_selection(join_cursor, join_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Joined current line with next");
        cx.notify();
    }

    /// Cycle the case of selected text (Cmd+Shift+U)
    ///
    /// Behavior:
    /// - lowercase → UPPERCASE
    /// - UPPERCASE → Title Case
    /// - Title Case → lowercase
    /// - Mixed → lowercase (then cycles from there)
    pub(super) fn transform_case(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let start = selection.start.min(value.len());
            let end = selection.end.min(value.len());
            let (start, end) = if start > end {
                (end, start)
            } else {
                (start, end)
            };

            if start == end {
                return; // No selection, nothing to transform
            }

            let selected = &value[start..end];

            // Determine current case and cycle
            let transformed = if selected == selected.to_lowercase() {
                // All lowercase → UPPERCASE
                selected.to_uppercase()
            } else if selected == selected.to_uppercase() {
                // All UPPERCASE → Title Case
                Self::to_title_case(selected)
            } else {
                // Mixed/Title Case → lowercase
                selected.to_lowercase()
            };

            let new_value = format!("{}{}{}", &value[..start], transformed, &value[end..]);
            let new_end = start + transformed.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(start, new_end, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Transformed case of selected text");
        cx.notify();
    }

    /// Convert a string to Title Case (capitalize first letter of each word)
    pub(super) fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
