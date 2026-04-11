use itertools::Itertools;

use super::*;

impl NotesApp {
    fn byte_offset_to_char_index(text: &str, byte_offset: usize) -> usize {
        text[..byte_offset.min(text.len())].chars().count()
    }

    fn char_index_to_byte_offset(text: &str, char_index: usize) -> usize {
        text.char_indices()
            .nth(char_index)
            .map(|(byte, _)| byte)
            .unwrap_or(text.len())
    }

    fn char_range_to_byte_range(
        text: &str,
        range: std::ops::Range<usize>,
    ) -> std::ops::Range<usize> {
        Self::char_index_to_byte_offset(text, range.start)
            ..Self::char_index_to_byte_offset(text, range.end)
    }

    fn note_portal_query_from_token(token: &str) -> Option<String> {
        let (prefix, value) = crate::ai::context_mentions::typed_mention_token_parts(token)?;
        (prefix == "note").then_some(value)
    }

    pub(super) fn focused_note_inline_token_span(
        &self,
        cx: &Context<Self>,
    ) -> Option<crate::ai::context_mentions::InlineTokenSpan> {
        let editor = self.editor_state.read(cx);
        let value = editor.value().to_string();
        let cursor_char = Self::byte_offset_to_char_index(&value, editor.cursor());
        crate::ai::context_mentions::inline_token_at_cursor(&value, cursor_char)
    }

    pub(super) fn focused_note_mention_preview(
        &self,
        cx: &Context<Self>,
    ) -> Option<(String, String)> {
        let span = self.focused_note_inline_token_span(cx)?;
        let detail = if let Some(query) = Self::note_portal_query_from_token(&span.token) {
            if query.trim().is_empty() {
                "notes portal • Cmd+Shift+O replace".to_string()
            } else {
                format!(
                    "notes portal for \"{}\" • Cmd+Shift+O replace",
                    query.trim()
                )
            }
        } else if let Some((prefix, value)) =
            crate::ai::context_mentions::typed_mention_token_parts(&span.token)
        {
            if value.trim().is_empty() {
                format!("@{prefix} token • open in ACP to replace")
            } else {
                format!("@{prefix} \"{}\" • open in ACP to replace", value.trim())
            }
        } else {
            "ACP token • open in ACP to replace".to_string()
        };

        Some((span.token, detail))
    }

    pub(super) fn open_focused_note_mention_portal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(span) = self.focused_note_inline_token_span(cx) else {
            return false;
        };
        let Some(query) = Self::note_portal_query_from_token(&span.token) else {
            return false;
        };

        let value = self.editor_state.read(cx).value().to_string();
        self.mention_portal_edit = Some(NotesMentionPortalEditSession {
            mention_range: Self::char_range_to_byte_range(&value, span.range),
            original_token: span.token,
        });
        self.open_browse_panel(window, cx);
        if let Some(dialog) = self.note_switcher.dialog() {
            dialog.update(cx, |d, cx| {
                d.set_context_title(Some("Replace @note".to_string()));
                d.set_search_text(query, cx);
            });
        }
        cx.notify();
        true
    }

    pub(super) fn replace_active_note_mention_with_note(
        &mut self,
        id: NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(edit) = self.mention_portal_edit.take() else {
            return false;
        };
        let Some(note) = self.notes.iter().find(|note| note.id == id) else {
            self.show_selected_note_missing_feedback("replace_active_note_mention_with_note", cx);
            self.close_browse_panel(window, cx);
            return true;
        };

        let title = if note.title.trim().is_empty() {
            "Untitled Note"
        } else {
            note.title.trim()
        };
        let token = crate::ai::context_mentions::format_typed_label_mention_token("note", title);
        let current_value = self.editor_state.read(cx).value().to_string();
        let suffix = &current_value[edit.mention_range.end.min(current_value.len())..];
        let needs_space = suffix
            .chars()
            .next()
            .map(|ch| !ch.is_whitespace() && !matches!(ch, ',' | '.' | ';' | ':' | ')' | ']' | '}'))
            .unwrap_or(false);
        let replacement = if needs_space {
            format!("{token} ")
        } else {
            token.clone()
        };
        let next_value = format!(
            "{}{}{}",
            &current_value[..edit.mention_range.start.min(current_value.len())],
            replacement,
            suffix,
        );
        let next_cursor = edit.mention_range.start + replacement.len();

        tracing::info!(
            target: "script_kit::notes",
            event = "notes_mention_portal_replaced",
            old_token = %edit.original_token,
            new_token = %token,
            note_id = %id.as_str(),
        );

        self.editor_state.update(cx, |state, cx| {
            state.set_value(next_value, window, cx);
            state.set_selection(next_cursor, next_cursor, window, cx);
        });
        self.close_browse_panel(window, cx);
        cx.notify();
        true
    }

    /// Get filtered notes based on search query
    pub(super) fn get_visible_notes(&self) -> &[Note] {
        match self.view_mode {
            NotesViewMode::AllNotes => &self.notes,
            NotesViewMode::Trash => &self.deleted_notes,
        }
    }

    /// Get the character count of the current note
    pub(super) fn get_character_count(&self, cx: &Context<Self>) -> usize {
        self.editor_state.read(cx).value().chars().count()
    }

    /// Get the word count of the current note
    pub(super) fn get_word_count(&self, cx: &Context<Self>) -> usize {
        self.editor_state
            .read(cx)
            .value()
            .split_whitespace()
            .count()
    }

    /// Get the 1-based index position of the current note in the visible list
    /// Returns (current_position, total_count) or None if no note selected
    pub(super) fn get_note_position(&self) -> Option<(usize, usize)> {
        let notes = self.get_visible_notes();
        let total = notes.len();
        if total == 0 {
            return None;
        }
        self.selected_note_id.and_then(|id| {
            notes
                .iter()
                .position(|n| n.id == id)
                .map(|idx| (idx + 1, total))
        })
    }

    /// Get the 1-based line number at cursor position, plus total line count
    pub(super) fn get_cursor_line_info(&self, cx: &Context<Self>) -> Option<(usize, usize)> {
        let value = self.editor_state.read(cx).value().to_string();
        if value.is_empty() {
            return None;
        }
        let selection = self.editor_state.read(cx).selection();
        let cursor = selection.start.min(value.len());
        let current_line = value[..cursor].matches('\n').count() + 1;
        let total_lines = value.lines().count().max(1);
        Some((current_line, total_lines))
    }

    /// Check if the currently selected note is pinned
    pub(super) fn is_current_note_pinned(&self) -> bool {
        self.selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|n| n.is_pinned)
            .unwrap_or(false)
    }

    /// Navigate to the previous note in the list
    pub(super) fn select_prev_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if notes.is_empty() {
            return;
        }
        if let Some(id) = self.selected_note_id {
            if let Some(idx) = notes.iter().position(|n| n.id == id) {
                if idx > 0 {
                    let prev_id = notes[idx - 1].id;
                    self.select_note(prev_id, window, cx);
                }
            }
        }
    }

    /// Navigate to the next note in the list
    pub(super) fn select_next_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if notes.is_empty() {
            return;
        }
        if let Some(id) = self.selected_note_id {
            if let Some(idx) = notes.iter().position(|n| n.id == id) {
                if idx + 1 < notes.len() {
                    let next_id = notes[idx + 1].id;
                    self.select_note(next_id, window, cx);
                }
            }
        }
    }

    /// Jump to the first note in the list (Cmd+Shift+Up)
    pub(super) fn select_first_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if let Some(note) = notes.first() {
            let id = note.id;
            self.select_note(id, window, cx);
        }
    }

    /// Jump to the last note in the list (Cmd+Shift+Down)
    pub(super) fn select_last_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if let Some(note) = notes.last() {
            let id = note.id;
            self.select_note(id, window, cx);
        }
    }

    /// Navigate back in history (Cmd+[)
    pub(super) fn navigate_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(prev_id) = self.history_back.pop() {
            // Only navigate if the note still exists
            if self.notes.iter().any(|n| n.id == prev_id) {
                // Push current note onto forward stack
                if let Some(current_id) = self.selected_note_id {
                    self.history_forward.push(current_id);
                }
                self.navigating_history = true;
                self.select_note(prev_id, window, cx);
                self.navigating_history = false;
            }
        }
    }

    /// Navigate forward in history (Cmd+])
    pub(super) fn navigate_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next_id) = self.history_forward.pop() {
            // Only navigate if the note still exists
            if self.notes.iter().any(|n| n.id == next_id) {
                // Push current note onto back stack
                if let Some(current_id) = self.selected_note_id {
                    self.history_back.push(current_id);
                }
                self.navigating_history = true;
                self.select_note(next_id, window, cx);
                self.navigating_history = false;
            }
        }
    }

    /// Toggle pin state of the currently selected note (Cmd+Shift+I)
    pub(super) fn toggle_pin_current_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            let mut was_pinned = false;
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.is_pinned = !note.is_pinned;
                let pinned = note.is_pinned;
                was_pinned = pinned;
                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to toggle pin state");
                    return;
                }
                info!(note_id = %id, pinned = pinned, "Toggled pin state");
            }
            // Re-sort notes: pinned first, then by updated_at descending
            self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.updated_at.cmp(&a.updated_at),
            });
            self.show_action_feedback(if was_pinned { "● Pinned" } else { "Unpinned" }, was_pinned);
            cx.notify();
        }
    }

    /// Get relative time description for when a note was last updated
    pub(super) fn get_relative_time(&self) -> Option<String> {
        self.selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|note| {
                let now = chrono::Utc::now();
                let diff = now - note.updated_at;

                if diff.num_seconds() < 5 {
                    "just now".to_string()
                } else if diff.num_seconds() < 60 {
                    format!("{}s ago", diff.num_seconds())
                } else if diff.num_minutes() < 60 {
                    let mins = diff.num_minutes();
                    format!("{}m ago", mins)
                } else if diff.num_hours() < 24 {
                    let hours = diff.num_hours();
                    format!("{}h ago", hours)
                } else if diff.num_days() < 7 {
                    let days = diff.num_days();
                    format!("{}d ago", days)
                } else {
                    note.updated_at.format("%b %d").to_string()
                }
            })
    }

    /// Select a pinned note by its ordinal position (Cmd+1 through Cmd+9)
    pub(super) fn select_pinned_note_by_index(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pinned_notes: Vec<NoteId> = self
            .notes
            .iter()
            .filter(|n| n.is_pinned)
            .map(|n| n.id)
            .collect();

        if let Some(&note_id) = pinned_notes.get(index) {
            self.select_note(note_id, window, cx);
        }
    }

    /// Toggle focus mode (Cmd+.) — hides titlebar icons, footer, toolbar for distraction-free writing
    pub(super) fn toggle_focus_mode(&mut self, cx: &mut Context<Self>) {
        self.focus_mode = !self.focus_mode;
        if self.focus_mode {
            // Also hide search and formatting toolbar in focus mode
            self.show_search = false;
            self.show_format_toolbar = false;
        }
        info!(focus_mode = self.focus_mode, "Toggled focus mode");
        cx.notify();
    }

    /// Get estimated reading time in minutes based on word count (200 wpm average)
    pub(super) fn get_reading_time(&self, cx: &Context<Self>) -> String {
        let words = self.get_word_count(cx);
        if words < 30 {
            return String::new(); // Too short for meaningful estimate
        }
        let minutes = (words as f64 / 200.0).ceil() as usize;
        if minutes <= 1 {
            "~1 min read".to_string()
        } else {
            format!("~{} min read", minutes)
        }
    }

    /// Get the selected text range stats, if any text is selected
    /// Returns (selected_words, selected_chars) or None if no selection
    pub(super) fn get_selection_stats(&self, cx: &Context<Self>) -> Option<(usize, usize)> {
        let selection = self.editor_state.read(cx).selection();
        if selection.start == selection.end {
            return None;
        }
        let value = self.editor_state.read(cx).value().to_string();
        let start = selection.start.min(value.len());
        let end = selection.end.min(value.len());
        let selected_text = &value[start..end];
        let words = selected_text.split_whitespace().count();
        let chars = selected_text.chars().count();
        if chars == 0 {
            return None;
        }
        Some((words, chars))
    }

    /// Format a DateTime as a relative time string for the note switcher
    pub(super) fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let diff = now - dt;

        if diff.num_seconds() < 5 {
            "just now".to_string()
        } else if diff.num_seconds() < 60 {
            format!("{}s ago", diff.num_seconds())
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            dt.format("%b %d").to_string()
        }
    }

    /// Strip markdown syntax from a preview string for clean display in the note switcher
    pub(super) fn strip_markdown_for_preview(s: &str) -> String {
        let mut result = s.to_string();
        // Strip common markdown inline formatting
        result = result.replace("**", "");
        result = result.replace("__", "");
        result = result.replace("~~", "");
        // Strip heading markers
        while result.starts_with('#') {
            result = result.trim_start_matches('#').to_string();
        }
        // Strip list markers and blockquotes
        result = result
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed
                    .strip_prefix("- [ ] ")
                    .or_else(|| trimmed.strip_prefix("- [x] "))
                {
                    rest
                } else if let Some(rest) = trimmed.strip_prefix("- ") {
                    rest
                } else if let Some(rest) = trimmed.strip_prefix("> ") {
                    rest
                } else {
                    trimmed
                }
            })
            .join(" ");
        // Collapse whitespace
        result.split_whitespace().join(" ").trim().to_string()
    }

    /// Welcome note content for first-time users.
    /// Teaches markdown syntax and key shortcuts through the product itself.
    pub(super) fn welcome_note_content() -> String {
        [
            "# Welcome to Notes",
            "",
            "A fast, keyboard-first notes app with markdown support.",
            "",
            "## Formatting",
            "",
            "- **Bold** with ⌘B",
            "- *Italic* with ⌘I",
            "- `Code` with ⌘E",
            "- ~~Strikethrough~~ with ⌘⇧X",
            "",
            "## Lists",
            "",
            "- [ ] Checklist item (⌘⇧L)",
            "- Bullet point (⌘⇧8)",
            "1. Numbered list (⌘⇧7)",
            "",
            "## Quick shortcuts",
            "",
            "- ⌘N  new note",
            "- ⌘P  switch notes",
            "- ⌘K  actions",
            "- ⌘.  focus mode",
            "- ⌘/  all shortcuts",
            "",
            "Start typing to make this note your own!",
        ]
        .join("\n")
    }

    /// Show a brief action feedback message in the footer (auto-clears after 2s)
    /// If `accent` is true, the message renders in accent color; otherwise muted.
    pub(super) fn show_action_feedback(&mut self, msg: impl Into<String>, accent: bool) {
        self.action_feedback = Some((msg.into(), accent, Instant::now()));
    }

    /// Check if action feedback should still be visible (within 2s window)
    pub(super) fn get_action_feedback(&self) -> Option<(&str, bool)> {
        self.action_feedback.as_ref().and_then(|(msg, accent, t)| {
            if t.elapsed() < Duration::from_millis(ACTION_FEEDBACK_MS) {
                Some((msg.as_str(), *accent))
            } else {
                None
            }
        })
    }

    /// Toggle keyboard shortcuts help overlay (Cmd+/)
    pub(super) fn toggle_shortcuts_help(&mut self, cx: &mut Context<Self>) {
        self.show_shortcuts_help = !self.show_shortcuts_help;
        cx.notify();
    }
}
