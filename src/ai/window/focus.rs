use super::*;

/// Focus surface for the AI window — exactly one surface is active at a time.
///
/// This replaces the ad-hoc boolean flags (`needs_focus_input`, `needs_command_bar_focus`)
/// with a single pending-focus enum, mirroring the main window's `FocusCoordinator` pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiFocusSurface {
    /// The chat composer input field (default surface).
    Composer,
    /// The sidebar search input field.
    Search,
    /// The command bar overlay (Cmd+K) — routes keyboard events to the window's key handler.
    CommandBar,
}

impl AiApp {
    /// Record the last non-CommandBar surface the user was on.
    /// Called whenever the user explicitly moves to Composer or Search.
    pub(super) fn note_primary_focus_surface(&mut self, surface: AiFocusSurface) {
        if surface != AiFocusSurface::CommandBar {
            self.last_primary_focus_surface = surface;
        }
    }

    /// Returns the current logical focus surface based on command bar state
    /// and any pending focus request.
    pub(super) fn current_focus_surface(&self) -> AiFocusSurface {
        if self.command_bar.is_open() {
            AiFocusSurface::CommandBar
        } else {
            self.pending_focus_surface
                .unwrap_or(self.last_primary_focus_surface)
        }
    }

    /// Request a focus surface change. The actual focus call happens in render via
    /// `apply_pending_focus_surface`, ensuring GPUI's render contract is respected.
    pub(super) fn request_focus_surface(
        &mut self,
        surface: AiFocusSurface,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "ai",
            requested_surface = ?surface,
            current_surface = ?self.current_focus_surface(),
            command_bar_open = self.command_bar.is_open(),
            "ai_focus_surface_requested"
        );
        self.pending_focus_surface = Some(surface);
        cx.notify();
    }

    /// Apply any pending focus surface change. Called once per render frame from
    /// `process_render_focus_requests`.
    pub(super) fn apply_pending_focus_surface(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(surface) = self.pending_focus_surface.take() else {
            return;
        };

        match surface {
            AiFocusSurface::Composer => {
                // In setup mode, focus main handle for keyboard navigation instead of input.
                let in_setup_mode =
                    self.available_models.is_empty() && !self.showing_api_key_input;
                if in_setup_mode {
                    self.focus_handle.focus(window, cx);
                } else {
                    self.focus_input(window, cx);
                }
                self.note_primary_focus_surface(AiFocusSurface::Composer);
            }
            AiFocusSurface::Search => {
                self.focus_search(window, cx);
                self.note_primary_focus_surface(AiFocusSurface::Search);
            }
            AiFocusSurface::CommandBar => {
                self.focus_handle.focus(window, cx);
            }
        }

        tracing::info!(
            target: "ai",
            applied_surface = ?surface,
            command_bar_open = self.command_bar.is_open(),
            "ai_focus_surface_applied"
        );
    }
}
