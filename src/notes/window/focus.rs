use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NotesFocusSurface {
    Editor,
    ActionsPanel,
    BrowsePanel,
    Dialog,
    /// Embedded ACP chat inside the Notes window.
    AcpChat,
}

impl NotesApp {
    pub(super) fn current_focus_surface(&self) -> NotesFocusSurface {
        if self.command_bar.is_open() || self.show_actions_panel {
            NotesFocusSurface::ActionsPanel
        } else if self.note_switcher.is_open() || self.show_browse_panel {
            NotesFocusSurface::BrowsePanel
        } else if self.surface_mode == NotesSurfaceMode::Acp {
            NotesFocusSurface::AcpChat
        } else {
            NotesFocusSurface::Editor
        }
    }

    /// Request and immediately apply a focus surface transition.
    ///
    /// Focus is applied synchronously so that GPUI's focus state is
    /// consistent before the next render — no deferred pending state.
    pub(super) fn request_focus_surface(
        &mut self,
        surface: NotesFocusSurface,
        window: &mut Window,
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

        self.apply_focus_surface(surface, window, cx);
        cx.notify();
    }

    /// Apply any deferred focus request that was set outside a window context
    /// (e.g., from an async action dispatch that only had `&mut App`).
    pub(super) fn drain_pending_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(surface) = self.pending_focus_surface.take() {
            self.apply_focus_surface(surface, window, cx);
        }
    }

    /// Restore keyboard focus to the appropriate surface after a dialog
    /// is dismissed (cancel or confirm).
    pub(super) fn restore_primary_focus_after_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let surface = if self.command_bar.is_open() || self.show_actions_panel {
            NotesFocusSurface::ActionsPanel
        } else if self.note_switcher.is_open() || self.show_browse_panel {
            NotesFocusSurface::BrowsePanel
        } else if self.surface_mode == NotesSurfaceMode::Acp {
            NotesFocusSurface::AcpChat
        } else {
            NotesFocusSurface::Editor
        };

        tracing::info!(
            target: "notes",
            restore_surface = ?surface,
            "notes_focus_surface_restored_after_dialog"
        );

        self.apply_focus_surface(surface, window, cx);
        cx.notify();
    }

    /// Apply a focus surface transition immediately.
    fn apply_focus_surface(
        &mut self,
        surface: NotesFocusSurface,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clear any stale pending value so render never re-applies.
        self.pending_focus_surface = None;

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
            NotesFocusSurface::AcpChat => {
                if let Some(acp_entity) = self.embedded_acp_chat.as_ref() {
                    let focus_handle = acp_entity.read(cx).focus_handle(cx);
                    window.focus(&focus_handle, cx);
                } else {
                    self.focus_handle.focus(window, cx);
                }
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
