use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NotesFocusSurface {
    Editor,
    ActionsPanel,
    BrowsePanel,
}

impl NotesApp {
    pub(super) fn current_focus_surface(&self) -> NotesFocusSurface {
        if self.command_bar.is_open() {
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
        }

        tracing::info!(
            target: "notes",
            applied_surface = ?surface,
            command_bar_open = self.command_bar.is_open(),
            note_switcher_open = self.note_switcher.is_open(),
            "notes_focus_surface_applied"
        );
    }
}
