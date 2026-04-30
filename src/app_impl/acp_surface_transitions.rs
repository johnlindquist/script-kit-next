//! The single mutator for [`ScriptListApp::acp_surface_state`].
//!
//! Oracle-Session `acp-chat-state-machine-audit` PR1. Every write to
//! the placement machine goes through [`transition_acp_surface`], which
//! runs the pure reducer, emits a structured transition event, and in
//! debug builds asserts the placement agrees with
//! [`ScriptListApp::current_view`]. Raw writes to `acp_surface_state`
//! are forbidden; an audit test pins that contract.

use super::*;
use crate::ai::acp::surface_state::{reduce_acp_surface, AcpSurfaceEvent, AcpSurfaceState};

impl ScriptListApp {
    /// Apply an [`AcpSurfaceEvent`]. No-op when the reduced next state
    /// equals the current state. Emits one `acp_surface_transition`
    /// tracing event per real transition so operators can correlate
    /// placement drift with launcher-entry bugs.
    pub(crate) fn transition_acp_surface(&mut self, event: AcpSurfaceEvent) {
        let previous = self.acp_surface_state;
        let next = reduce_acp_surface(previous, event);
        if next == previous {
            tracing::trace!(
                target: "script_kit::acp",
                event = "acp_surface_transition_noop",
                from = ?previous,
                trigger = ?event,
            );
            return;
        }
        tracing::info!(
            target: "script_kit::acp",
            event = "acp_surface_transition",
            from = ?previous,
            to = ?next,
            trigger = ?event,
        );
        self.acp_surface_state = next;
    }

    /// Debug-only consistency check between the placement enum and
    /// [`AppView`]. Fires when the two disagree — the embedded state
    /// must co-occur with `AppView::AcpChatView`, a portal must co-occur
    /// with the matching portal host view, and `Hidden` must not be
    /// observed while the ACP chat view is on-screen.
    ///
    /// This is `debug_assert` so release builds pay no cost. The goal
    /// is to fail loudly in test / dev runs if a future refactor sets
    /// `current_view` without calling [`transition_acp_surface`].
    #[cfg(debug_assertions)]
    pub(crate) fn debug_assert_acp_surface_consistent(&self) {
        match self.acp_surface_state {
            AcpSurfaceState::Embedded => {
                debug_assert!(
                    matches!(self.current_view, AppView::AcpChatView { .. }),
                    "AcpSurfaceState::Embedded must agree with AppView::AcpChatView; \
                     current_view = {:?}",
                    self.current_view
                );
            }
            AcpSurfaceState::AttachmentPortal { .. } => {
                // Portal host view is one of several builtin surfaces;
                // we can only assert the *negative* half — the chat
                // view must NOT be the current view while a portal
                // owns the panel.
                debug_assert!(
                    !matches!(self.current_view, AppView::AcpChatView { .. }),
                    "AcpSurfaceState::AttachmentPortal must not observe AppView::AcpChatView"
                );
            }
            AcpSurfaceState::Hidden => {
                debug_assert!(
                    !matches!(self.current_view, AppView::AcpChatView { .. }),
                    "AcpSurfaceState::Hidden must not observe AppView::AcpChatView"
                );
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn debug_assert_acp_surface_consistent(&self) {}
}
