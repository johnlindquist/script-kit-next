use super::*;

impl NotesApp {
    pub(super) fn toggle_checklist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find the start and end of the current line
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_line, cursor_delta): (String, isize) =
                if let Some(rest) = line.strip_prefix("- [x] ") {
                    // Checked → unchecked
                    (format!("- [ ] {}", rest), 0)
                } else if let Some(rest) = line.strip_prefix("- [ ] ") {
                    // Unchecked → checked
                    (format!("- [x] {}", rest), 0)
                } else if let Some(rest) = line.strip_prefix("- ") {
                    // List item without checkbox → add checkbox
                    (format!("- [ ] {}", rest), 4) // "[ ] " is 4 chars
                } else {
                    // Plain line → add full checkbox prefix
                    (format!("- [ ] {}", line), 6) // "- [ ] " is 6 chars
                };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(0) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled checklist on current line");
        cx.notify();
    }

    /// Insert a horizontal rule (---) at cursor position (Cmd+Shift+-)
    pub(super) fn insert_horizontal_rule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Ensure we're on a new line and add the rule
            let needs_newline =
                cursor > 0 && value.as_bytes().get(cursor - 1).is_none_or(|&b| b != b'\n');
            let rule = if needs_newline {
                "\n\n---\n\n"
            } else {
                "\n---\n\n"
            };

            let new_value = format!("{}{}{}", &value[..cursor], rule, &value[cursor..]);
            let new_cursor = cursor + rule.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Inserted horizontal rule");
        cx.notify();
    }

    /// Cycle heading level on the current line (Cmd+Shift+H)
    ///
    /// Behavior:
    /// - Plain text → `# text`
    /// - `# text` → `## text`
    /// - `## text` → `### text`
    /// - `### text` → plain text (strip heading)
    pub(super) fn cycle_heading(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find the start and end of the current line
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_line, cursor_delta): (String, isize) =
                if let Some(rest) = line.strip_prefix("### ") {
                    // ### → plain (remove 4 chars)
                    (rest.to_string(), -4)
                } else if let Some(rest) = line.strip_prefix("## ") {
                    // ## → ### (add 1 char)
                    (format!("### {}", rest), 1)
                } else if let Some(rest) = line.strip_prefix("# ") {
                    // # → ## (add 1 char)
                    (format!("## {}", rest), 1)
                } else {
                    // plain → # (add 2 chars)
                    (format!("# {}", line), 2)
                };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(line_start as isize) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Cycled heading level on current line");
        cx.notify();
    }

    /// Move the current line up (Alt+Up)
    pub(super) fn move_line_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Can't move up if already on first line
            if line_start == 0 {
                return;
            }

            // Find the previous line boundaries
            let prev_line_start = value[..line_start - 1].rfind('\n').map_or(0, |p| p + 1);

            let current_line = &value[line_start..line_end];
            let prev_line = &value[prev_line_start..line_start - 1]; // exclude the \n

            // Build new value: prev_line and current_line swapped
            let new_value = format!(
                "{}{}\n{}{}",
                &value[..prev_line_start],
                current_line,
                prev_line,
                &value[line_end..]
            );

            // Adjust cursor position: move it up by the length of prev_line + newline
            let offset_in_line = cursor - line_start;
            let new_cursor = prev_line_start + offset_in_line;

            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Move the current line down (Alt+Down)
    pub(super) fn move_line_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Can't move down if already on last line
            if line_end >= value.len() {
                return;
            }

            // Find the next line boundaries
            let next_line_end = value[line_end + 1..]
                .find('\n')
                .map_or(value.len(), |p| line_end + 1 + p);

            let current_line = &value[line_start..line_end];
            let next_line = &value[line_end + 1..next_line_end];

            // Build new value: next_line and current_line swapped
            let new_value = format!(
                "{}{}\n{}{}",
                &value[..line_start],
                next_line,
                current_line,
                &value[next_line_end..]
            );

            // Adjust cursor: it moves down by length of next_line + newline
            let offset_in_line = cursor - line_start;
            let new_line_start = line_start + next_line.len() + 1;
            let new_cursor = new_line_start + offset_in_line;

            state.set_value(&new_value, window, cx);
            state.set_selection(
                new_cursor.min(new_value.len()),
                new_cursor.min(new_value.len()),
                window,
                cx,
            );
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Select the entire current line (Cmd+L)
    pub(super) fn select_current_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            state.set_selection(line_start, line_end, window, cx);
        });
        cx.notify();
    }

    /// Smart paste: if text is selected and clipboard contains a URL, wrap as markdown link.
    /// Otherwise, fall through to normal paste behavior.
    /// Returns true if smart paste was handled, false to let default paste proceed.
    pub(super) fn try_smart_paste(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let clipboard = Self::read_clipboard();
        let trimmed = clipboard.trim();

        // Check if clipboard looks like a URL
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return false;
        }

        // Check if we have a text selection
        let selection = self.editor_state.read(cx).selection();
        if selection.start == selection.end {
            return false;
        }

        // We have a URL on clipboard and selected text — create a markdown link
        let value = self.editor_state.read(cx).value().to_string();
        let start = selection.start.min(value.len());
        let end = selection.end.min(value.len());
        let (start, end) = if start > end {
            (end, start)
        } else {
            (start, end)
        };
        let selected_text = &value[start..end];
        let link = format!("[{}]({})", selected_text, trimmed);
        let new_value = format!("{}{}{}", &value[..start], link, &value[end..]);
        let new_cursor = start + link.len();

        self.editor_state.update(cx, |state, cx| {
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Smart paste: wrapped selection as markdown link");
        cx.notify();
        true
    }

    /// Wrap selected lines as blockquote (Cmd+Shift+.)
    ///
    /// Prefixes each selected line (or current line if no selection) with "> ".
    /// If all target lines already start with "> ", remove the prefix instead (toggle).
    pub(super) fn toggle_blockquote(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let sel_start = selection.start.min(value.len());
            let sel_end = selection.end.min(value.len());
            let (sel_start, sel_end) = if sel_start > sel_end {
                (sel_end, sel_start)
            } else {
                (sel_start, sel_end)
            };

            // Expand to full lines
            let region_start = value[..sel_start].rfind('\n').map_or(0, |p| p + 1);
            let region_end = value[sel_end..]
                .find('\n')
                .map_or(value.len(), |p| sel_end + p);

            let region = &value[region_start..region_end];
            let lines: Vec<&str> = region.split('\n').collect();

            // Check if ALL lines already have blockquote prefix
            let all_quoted = lines.iter().all(|l| l.starts_with("> "));

            let new_lines: Vec<String> = if all_quoted {
                // Remove "> " prefix from all lines
                lines
                    .iter()
                    .map(|l| l.strip_prefix("> ").unwrap_or(l).to_string())
                    .collect()
            } else {
                // Add "> " prefix to all lines
                lines.iter().map(|l| format!("> {}", l)).collect()
            };

            let new_region = new_lines.join("\n");
            let new_value = format!(
                "{}{}{}",
                &value[..region_start],
                new_region,
                &value[region_end..]
            );

            // Place cursor at end of modified region
            let new_cursor = region_start + new_region.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled blockquote on selected lines");
        cx.notify();
    }

    /// Duplicate the current line below (Alt+Shift+Down) or above (Alt+Shift+Up)
    pub(super) fn duplicate_line(
        &mut self,
        direction_down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_value, new_cursor) = if direction_down {
                // Insert copy after current line
                let new_value = format!("{}\n{}{}", &value[..line_end], line, &value[line_end..]);
                // Move cursor to same offset in the duplicated line below
                let offset_in_line = cursor - line_start;
                let new_cursor = line_end + 1 + offset_in_line;
                (new_value, new_cursor)
            } else {
                // Insert copy before current line
                let new_value =
                    format!("{}{}\n{}", &value[..line_start], line, &value[line_start..]);
                // Cursor stays at same absolute position (now on the original line pushed down)
                let offset_in_line = cursor - line_start;
                let new_cursor = line_start + offset_in_line;
                (new_value, new_cursor)
            };

            state.set_value(&new_value, window, cx);
            state.set_selection(
                new_cursor.min(new_value.len()),
                new_cursor.min(new_value.len()),
                window,
                cx,
            );
        });
        self.has_unsaved_changes = true;
        info!(
            direction = if direction_down { "down" } else { "up" },
            "Duplicated current line"
        );
        cx.notify();
    }
}
