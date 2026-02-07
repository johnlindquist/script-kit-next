use super::*;

impl NotesApp {
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
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                });
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

    pub(super) fn copy_note_as_markdown(&self) {
        self.export_note(ExportFormat::Markdown);
    }

    pub(super) fn copy_note_deeplink(&self) {
        if let Some(id) = self.selected_note_id {
            let deeplink = self.note_deeplink(id);
            self.copy_text_to_clipboard(&deeplink);
        }
    }

    pub(super) fn create_note_quicklink(&self) {
        if let Some(id) = self.selected_note_id {
            let title = self
                .notes
                .iter()
                .find(|note| note.id == id)
                .map(|note| {
                    if note.title.is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        note.title.clone()
                    }
                })
                .unwrap_or_else(|| "Untitled Note".to_string());
            let deeplink = self.note_deeplink(id);
            let quicklink = format!("[{}]({})", title, deeplink);
            self.copy_text_to_clipboard(&quicklink);
        }
    }

    pub(super) fn duplicate_selected_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_note_id else {
            return;
        };
        let Some(note) = self.notes.iter().find(|note| note.id == id) else {
            return;
        };

        let duplicate = Note::with_content(note.content.clone());
        if let Err(e) = storage::save_note(&duplicate) {
            tracing::error!(error = %e, "Failed to duplicate note");
            return;
        }

        self.notes.insert(0, duplicate.clone());
        self.select_note(duplicate.id, window, cx);
        self.show_action_feedback("Duplicated", false);
    }
}
