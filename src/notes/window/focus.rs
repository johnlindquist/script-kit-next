use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NotesFocusSurface {
    Editor,
    ActionsPanel,
    BrowsePanel,
    Dialog,
}

impl NotesApp {
    pub(super) fn current_focus_surface(&self) -> NotesFocusSurface {
        if self.command_bar.is_open() || self.show_actions_panel {
            NotesFocusSurface::ActionsPanel
        } else if self.note_switcher.is_open() || self.show_browse_panel {
            NotesFocusSurface::BrowsePanel
        } else {
            NotesFocusSurface::Editor
        }
    }

    pub(super) fn request_focus_surface(
        &mut self,
        surface: NotesFocusSurface,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "notes",
            requested_surface = ?surface,
            current_surface = ?self.current_focus_surface(),
            command_bar_open = self.command_bar.is_open(),
            note_switcher_open = self.note_switcher.is_open(),
            "notes_focus_surface_requested"
        );
        self.pending_focus_surface = Some(surface);
        cx.notify();
    }

    pub(super) fn restore_primary_focus_after_dialog(&mut self, cx: &mut Context<Self>) {
        let surface = if self.command_bar.is_open() || self.show_actions_panel {
            NotesFocusSurface::ActionsPanel
        } else if self.note_switcher.is_open() || self.show_browse_panel {
            NotesFocusSurface::BrowsePanel
        } else {
            NotesFocusSurface::Editor
        };

        tracing::info!(
            target: "notes",
            restore_surface = ?surface,
            "notes_focus_surface_restored_after_dialog"
        );

        self.pending_focus_surface = Some(surface);
        cx.notify();
    }

    pub(super) fn apply_pending_focus_surface(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(surface) = self.pending_focus_surface.take() else {
            return;
        };

        match surface {
            NotesFocusSurface::Editor => {
                self.editor_state
                    .update(cx, |state, cx| state.focus(window, cx));
            }
            NotesFocusSurface::ActionsPanel | NotesFocusSurface::BrowsePanel => {
                self.focus_handle.focus(window, cx);
            }
            NotesFocusSurface::Dialog => {
                // Dialog manages its own focus — no action needed
            }
        }

        tracing::info!(
            target: "notes",
            applied_surface = ?surface,
            has_active_dialog = window.has_active_dialog(cx),
            command_bar_open = self.command_bar.is_open(),
            note_switcher_open = self.note_switcher.is_open(),
            "notes_focus_surface_applied"
        );
    }
}
