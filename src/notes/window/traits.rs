use super::*;

impl Focusable for NotesApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Drop for NotesApp {
    fn drop(&mut self) {
        // Save any unsaved changes before closing
        if self.has_unsaved_changes {
            if let Some(id) = self.selected_note_id {
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    if let Err(e) = storage::save_note(note) {
                        tracing::error!(error = %e, "Failed to save note on close");
                    } else {
                        debug!(note_id = %id, "Note saved on window close");
                    }
                }
            }
        }

        // Clear the global window handle when NotesApp is dropped
        // This ensures is_notes_window_open() returns false after the window closes
        // regardless of how it was closed (Cmd+W, traffic light, toggle, etc.)
        if let Some(window_handle) = NOTES_WINDOW.get() {
            if let Ok(mut guard) = window_handle.lock() {
                *guard = None;
                debug!("NotesApp dropped - cleared global window handle");
            }
        }

        // Clear the global app entity handle
        if let Some(app_entity) = NOTES_APP_ENTITY.get() {
            if let Ok(mut guard) = app_entity.lock() {
                *guard = None;
                debug!("NotesApp dropped - cleared global app entity handle");
            }
        }
    }
}
