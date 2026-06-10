use super::*;

impl NotesApp {
    const SELECTED_NOTE_NOT_FOUND_FEEDBACK: &'static str = "Selected note could not be found";

    pub(super) fn resolve_selected_note(
        selected_note_id: Option<NoteId>,
        notes: &[Note],
    ) -> Option<(NoteId, &Note)> {
        let selected_note_id = selected_note_id?;
        notes
            .iter()
            .find(|note| note.id == selected_note_id)
            .map(|note| (selected_note_id, note))
    }

    pub(super) fn show_selected_note_missing_feedback(
        &mut self,
        action: &'static str,
        cx: &mut Context<Self>,
    ) {
        tracing::warn!(
            action,
            selected_note_id = ?self.selected_note_id,
            notes_len = self.notes.len(),
            "notes_action_selected_note_not_found",
        );
        self.show_action_feedback(Self::SELECTED_NOTE_NOT_FOUND_FEEDBACK, true);
        cx.notify();
    }

    pub(super) fn selected_note_for_action(
        &mut self,
        action: &'static str,
        cx: &mut Context<Self>,
    ) -> Option<(NoteId, &Note)> {
        let Some(selected_note_id) = self.selected_note_id else {
            self.show_selected_note_missing_feedback(action, cx);
            return None;
        };

        if !self.notes.iter().any(|note| note.id == selected_note_id) {
            self.show_selected_note_missing_feedback(action, cx);
            return None;
        }

        Self::resolve_selected_note(Some(selected_note_id), &self.notes)
    }

    /// Cycle sort mode: Updated → Created → Alphabetical → Updated
    pub(super) fn cycle_sort_mode(&mut self, cx: &mut Context<Self>) {
        self.sort_mode = match self.sort_mode {
            NotesSortMode::Updated => NotesSortMode::Created,
            NotesSortMode::Created => NotesSortMode::Alphabetical,
            NotesSortMode::Alphabetical => NotesSortMode::Updated,
        };
        self.apply_sort(cx);
        info!(sort_mode = ?self.sort_mode, "Cycled sort mode");
    }

    /// Apply current sort mode to the notes list
    pub(super) fn apply_sort(&mut self, cx: &mut Context<Self>) {
        match self.sort_mode {
            NotesSortMode::Updated => {
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.updated_at.cmp(&a.updated_at),
                });
            }
            NotesSortMode::Created => {
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.created_at.cmp(&a.created_at),
                });
            }
            NotesSortMode::Alphabetical => {
                self.notes
                    .sort_by_cached_key(|n| (!n.is_pinned, n.title.to_lowercase()));
            }
        }
        cx.notify();
    }

    /// Empty the entire trash — permanently deletes all trashed notes
    pub(super) fn empty_trash(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<NoteId> = self.deleted_notes.iter().map(|n| n.id).collect();
        for id in &ids {
            if let Err(e) = storage::delete_note_permanently(*id) {
                tracing::error!(error = %e, note_id = %id, "Failed to permanently delete note");
            }
        }
        self.deleted_notes.clear();
        self.selected_note_id = None;
        info!(count = ids.len(), "Emptied trash");
        cx.notify();
    }

    /// Copy the current note content to clipboard
    pub(super) fn copy_note_to_clipboard(&self, cx: &Context<Self>) {
        let content = self.editor_state.read(cx).value().to_string();
        self.copy_text_to_clipboard(&content);
    }

    pub(super) fn copy_text_to_clipboard(&self, content: &str) {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let _ = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(content.as_bytes())?;
                    }
                    child.wait()
                });
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = content; // Avoid unused warning
        }
    }

    pub(super) fn note_deeplink(&self, id: NoteId) -> String {
        format!("scriptkit://notes/{}", id.as_str())
    }

    /// Follow the `[[wiki link]]` under the cursor (Cmd+Shift+Enter).
    ///
    /// Resolves the link target against note titles/aliases. A unique match
    /// opens that note; no match creates a new note titled after the target
    /// (Obsidian-style); an ambiguous match shows feedback instead of guessing.
    ///
    /// Returns true when the cursor was inside a wiki link.
    pub(super) fn follow_wiki_link_at_cursor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let (value, cursor) = {
            let state = self.editor_state.read(cx);
            (state.value().to_string(), state.selection().start)
        };

        let Some(target) = Self::wiki_link_target_at(&value, cursor) else {
            return false;
        };

        match storage::resolve_note_ref(&target) {
            Ok(storage::NoteRefResolution::Unique(note_id)) => {
                if self.has_unsaved_changes {
                    self.save_current_note();
                }
                self.select_note(note_id, window, cx);
            }
            Ok(storage::NoteRefResolution::NotFound) => {
                if self.has_unsaved_changes {
                    self.save_current_note();
                }
                if let Err(error) =
                    self.create_note_with_content(format!("{target}\n\n"), window, cx)
                {
                    tracing::warn!(error = %error, target, "Failed to create note from wiki link");
                    self.show_action_feedback("Could not create linked note", true);
                }
            }
            Ok(storage::NoteRefResolution::Ambiguous) => {
                self.show_action_feedback(format!("Multiple notes match \"{target}\""), true);
            }
            Err(error) => {
                tracing::warn!(error = %error, target, "Failed to resolve wiki link");
                self.show_action_feedback("Could not resolve link", true);
            }
        }
        cx.notify();
        true
    }

    /// Extract the `[[target]]` whose span contains `cursor`, if any.
    fn wiki_link_target_at(value: &str, cursor: usize) -> Option<String> {
        let mut scan = 0;
        while let Some(relative_start) = value[scan..].find("[[") {
            let start = scan + relative_start;
            let content_start = start + 2;
            let Some(relative_end) = value[content_start..].find("]]") else {
                return None;
            };
            let end = content_start + relative_end + 2;
            if cursor >= start && cursor <= end {
                let inner = value[content_start..content_start + relative_end].trim();
                if inner.is_empty() {
                    return None;
                }
                let target = inner
                    .split_once('|')
                    .map(|(t, _)| t.trim())
                    .unwrap_or(inner);
                return (!target.is_empty()).then(|| target.to_string());
            }
            if end <= cursor {
                scan = end;
            } else {
                return None;
            }
        }
        None
    }

    pub(super) fn copy_note_as_markdown(&mut self, cx: &mut Context<Self>) {
        self.export_note(ExportFormat::Markdown, cx);
    }

    pub(super) fn copy_note_deeplink(&mut self, cx: &mut Context<Self>) {
        let Some((id, _)) = self.selected_note_for_action("copy_note_deeplink", cx) else {
            return;
        };
        let deeplink = self.note_deeplink(id);
        self.copy_text_to_clipboard(&deeplink);
    }

    pub(super) fn create_note_quicklink(&mut self, cx: &mut Context<Self>) {
        let Some((id, note)) = self.selected_note_for_action("create_note_quicklink", cx) else {
            return;
        };
        let title = if note.title.is_empty() {
            "Untitled Note".to_string()
        } else {
            note.title.clone()
        };
        let deeplink = self.note_deeplink(id);
        let quicklink = format!("[{}]({})", title, deeplink);
        self.copy_text_to_clipboard(&quicklink);
    }

    pub(super) fn copy_note_backlinks(&mut self, cx: &mut Context<Self>) {
        let Some((id, _)) = self.selected_note_for_action("copy_note_backlinks", cx) else {
            return;
        };

        match storage::get_note_backlinks(id) {
            Ok(backlinks) if backlinks.is_empty() => {
                self.copy_text_to_clipboard("No backlinks");
                self.show_action_feedback("No backlinks", false);
            }
            Ok(backlinks) => {
                let markdown = backlinks
                    .iter()
                    .map(|note| {
                        let title = if note.title.trim().is_empty() {
                            "Untitled Note"
                        } else {
                            note.title.trim()
                        };
                        format!("- [{}]({})", title, self.note_deeplink(note.id))
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                self.copy_text_to_clipboard(&markdown);
                self.show_action_feedback("Copied backlinks", false);
            }
            Err(error) => {
                tracing::warn!(error = %error, note_id = %id, "Failed to copy note backlinks");
                self.show_action_feedback("Backlinks unavailable", true);
            }
        }

        cx.notify();
    }

    pub(super) fn duplicate_selected_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((_id, note)) = self.selected_note_for_action("duplicate_selected_note", cx) else {
            return;
        };

        let content = note.content.clone();
        let duplicate = Note::with_content(content);
        if let Err(e) = storage::save_note(&duplicate) {
            tracing::error!(error = %e, "Failed to duplicate note");
            return;
        }

        self.notes.insert(0, duplicate.clone());
        self.show_action_feedback("Duplicated", false);
        self.select_note(duplicate.id, window, cx);
    }
}
