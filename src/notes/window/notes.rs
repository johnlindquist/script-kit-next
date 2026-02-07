use super::*;

impl NotesApp {
    pub(super) fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();

        // If search is not empty, use FTS search
        if !query.trim().is_empty() {
            match storage::search_notes(&query) {
                Ok(results) => {
                    self.notes = results;
                    // Update selection if current note not in results
                    if let Some(id) = self.selected_note_id {
                        if !self.notes.iter().any(|n| n.id == id) {
                            self.selected_note_id = self.notes.first().map(|n| n.id);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Search failed");
                }
            }
        } else {
            // Reload all notes when search is cleared
            self.notes = storage::get_all_notes().unwrap_or_default();
        }

        cx.notify();
    }

    /// Create a new note
    pub(super) fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let note = Note::new();
        let id = note.id;

        // Save to storage
        if let Err(e) = storage::save_note(&note) {
            tracing::error!(error = %e, "Failed to create note");
            return;
        }

        // Add to cache and select it
        self.notes.insert(0, note);
        self.select_note(id, window, cx);

        info!(note_id = %id, "New note created");
    }

    /// Create a new note pre-filled with system clipboard content (Cmd+Shift+N)
    pub(super) fn create_note_from_clipboard(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let clipboard_content = Self::read_clipboard();
        if clipboard_content.is_empty() {
            // Nothing on clipboard, just create an empty note
            self.create_note(window, cx);
            return;
        }

        let note = Note::with_content(clipboard_content);
        let id = note.id;

        if let Err(e) = storage::save_note(&note) {
            tracing::error!(error = %e, "Failed to create note from clipboard");
            return;
        }

        self.notes.insert(0, note);
        self.select_note(id, window, cx);

        info!(note_id = %id, "New note created from clipboard");
    }

    /// Read text from system clipboard
    pub(super) fn read_clipboard() -> String {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("pbpaste")
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        }
        #[cfg(not(target_os = "macos"))]
        {
            String::new()
        }
    }

    /// Select a note for editing
    pub(super) fn select_note(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        // Save any unsaved changes to the current note before switching
        self.save_current_note();

        // Push current note onto history stack (unless navigating back/forward)
        if !self.navigating_history {
            if let Some(prev_id) = self.selected_note_id {
                if prev_id != id {
                    self.history_back.push(prev_id);
                    // Clear forward history on new navigation
                    self.history_forward.clear();
                }
            }
        }

        self.selected_note_id = Some(id);

        // Load content into editor
        let note_list = if self.view_mode == NotesViewMode::Trash {
            &self.deleted_notes
        } else {
            &self.notes
        };

        if let Some(note) = note_list.iter().find(|n| n.id == id) {
            let content_len = note.content.len();
            self.editor_state.update(cx, |state, cx| {
                state.set_value(&note.content, window, cx);
                // Move cursor to end of text (set selection to end..end = no selection, cursor at end)
                state.set_selection(content_len, content_len, window, cx);
            });
        }

        // Focus the editor after selecting a note
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    /// Delete the currently selected note (soft delete)
    pub(super) fn delete_selected_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.soft_delete();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to delete note");
                }

                // Move to deleted notes
                self.deleted_notes.insert(0, note.clone());
            }

            // Remove from visible list and select next
            self.notes.retain(|n| n.id != id);
            self.selected_note_id = self.notes.first().map(|n| n.id);

            self.show_action_feedback("Deleted · ⌘⇧T trash", false);
            cx.notify();
        }
    }

    /// Permanently delete the selected note from trash
    pub(super) fn permanently_delete_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Err(e) = storage::delete_note_permanently(id) {
                tracing::error!(error = %e, "Failed to permanently delete note");
                return;
            }

            self.deleted_notes.retain(|n| n.id != id);
            self.selected_note_id = self.deleted_notes.first().map(|n| n.id);

            info!(note_id = %id, "Note permanently deleted");
            cx.notify();
        }
    }

    /// Restore the selected note from trash
    pub(super) fn restore_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.deleted_notes.iter_mut().find(|n| n.id == id) {
                note.restore();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to restore note");
                    return;
                }

                // Move back to active notes
                self.notes.insert(0, note.clone());
            }

            self.deleted_notes.retain(|n| n.id != id);
            self.view_mode = NotesViewMode::AllNotes;
            self.selected_note_id = Some(id);
            self.select_note(id, window, cx);

            info!(note_id = %id, "Note restored");
            cx.notify();
        }
    }

    /// Switch view mode
    pub(super) fn set_view_mode(
        &mut self,
        mode: NotesViewMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.view_mode = mode;

        // Select first note in new view
        let notes = match mode {
            NotesViewMode::AllNotes => &self.notes,
            NotesViewMode::Trash => &self.deleted_notes,
        };

        if let Some(note) = notes.first() {
            self.select_note(note.id, window, cx);
        } else {
            self.selected_note_id = None;
            self.editor_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
        }

        cx.notify();
    }

    /// Export the current note
    pub(super) fn export_note(&self, format: ExportFormat) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                let content = match format {
                    ExportFormat::PlainText => note.content.clone(),
                    // For Markdown, just export the content as-is.
                    // The title is derived from the first line of content,
                    // so prepending it would cause duplication.
                    ExportFormat::Markdown => note.content.clone(),
                    ExportFormat::Html => {
                        // For HTML, we include proper structure with the title
                        // and render the content as preformatted text
                        format!(
                            "<!DOCTYPE html>\n<html>\n<head><title>{}</title></head>\n<body>\n<h1>{}</h1>\n<pre>{}</pre>\n</body>\n</html>",
                            note.title, note.title, note.content
                        )
                    }
                };

                // Copy to clipboard
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
                    info!(format = ?format, "Note exported to clipboard");
                }
            }
        }
    }
}
