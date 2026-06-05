//! App-owned ACP placement machine.
//!
//! Collapses the previously-implicit "where does ACP live right now"
//! cross-product (`current_view == AcpChatView` + `embedded_acp_chat` +
//! `attachment_portal_return_view.is_some()` + `active_attachment_portal_kind`)
//! into one explicit enum reduced by a tiny event machine.
//!
//! This is intentionally small: it is PR1 from Oracle-Session
//! `acp-chat-state-machine-audit`. Overlay (picker / history /
//! permission), thread turn status, portal contract, and resume
//! lifecycle all stay where they are until PR2. The only purpose of
//! this first slice is to give `tab_ai_mode.rs` and `attachment_portal.rs`
//! *one* predicate to answer "should a launcher Tab / global Cmd+Enter
//! route into ACP right now?" — today that predicate is inferred from
//! three different field shapes and drifts under refactor.
//!
//! ```text
//!                EmbeddedOpened
//!           Hidden ───────────► Embedded ◄──────┐
//!             ▲                    │            │ PortalClosed
//!             │ EmbeddedClosed     │ PortalOpened{kind}
//!             │                    ▼            │
//!             │              AttachmentPortal { kind }
//!             │                    │
//!             └────────────────────┘   EmbeddedClosed
//! ```
//!
//! `EmbeddedClosed` from any state always returns to `Hidden` — an
//! app-level hide/close is a hard ejector. `PortalOpened` only makes
//! sense while embedded; from `Hidden` it is a no-op (the reducer
//! simply stays `Hidden`). `PortalClosed` from a non-portal state is
//! also a no-op — mirrors the "one portal at a time" guard in
//! `open_attachment_portal`.

use crate::ai::window::context_picker::types::PortalKind;

/// Where the ACP surface physically lives right now.
///
/// Detached-window placement is deliberately NOT a variant here. The
/// detached popup lifecycle lives in `src/ai/acp/chat_window.rs`; the
/// app observes it externally via `is_chat_window_open()`. Oracle's PR1
/// scope keeps that observer model — merging it in would change
/// invariants the portal flow relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcpSurfaceState {
    /// ACP chat is not the active main-panel subview.
    #[default]
    Hidden,
    /// `current_view == AppView::AcpChatView { .. }`.
    Embedded,
    /// An attachment portal has temporarily claimed the main panel on
    /// behalf of the embedded ACP view. The portal kind is preserved so
    /// the close-cancel path can route the cancel back through the
    /// right `AcpChatView::cancel_pending_portal_session` arm.
    AttachmentPortal { kind: PortalKind },
}

/// Events that move the placement machine.
///
/// These are the *minimum* events needed by launcher-entry guards that
/// need to know whether ACP is embedded or in an attachment portal.
/// Every additional event is a future invariant we would have to
/// maintain — keep this set small.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpSurfaceEvent {
    /// ACP is now the main-panel subview (fresh launch, reuse, setup
    /// card, or the not-ready path — they all end in the same
    /// observable state).
    EmbeddedOpened,
    /// ACP is no longer the main-panel subview. Always drops to
    /// `Hidden` regardless of the previous state; an attachment portal
    /// that is cancelled by a host hide gets forced-closed together
    /// with its parent.
    EmbeddedClosed,
    /// An attachment portal opened on behalf of the embedded ACP view.
    /// Only valid while embedded — from `Hidden` it is a no-op.
    PortalOpened { kind: PortalKind },
    /// The attachment portal closed (attach or cancel). Only valid
    /// from `AttachmentPortal { .. }`.
    PortalClosed,
}

/// Pure reducer for the placement machine. Returned separately from
/// [`ScriptListApp::transition_acp_surface`] so the table can be
/// exhaustively unit-tested without a running app.
#[must_use]
pub fn reduce_acp_surface(current: AcpSurfaceState, event: AcpSurfaceEvent) -> AcpSurfaceState {
    use AcpSurfaceEvent::*;
    use AcpSurfaceState::*;
    match (current, event) {
        // An open event from any state lands in Embedded. A reopen
        // while already embedded is idempotent.
        (_, EmbeddedOpened) => Embedded,
        // Only a currently-embedded surface can host a portal.
        (Embedded, PortalOpened { kind }) => AttachmentPortal { kind },
        // Portal close drops back to embedded; from elsewhere it is a
        // no-op (e.g. a stray close event after the host already hid).
        (AttachmentPortal { .. }, PortalClosed) => Embedded,
        // A hard close ejects from any state, including portal — the
        // portal cannot outlive its embedded parent.
        (_, EmbeddedClosed) => Hidden,
        // Fall-through: event does not apply to this state.
        (state, _) => state,
    }
}

impl AcpSurfaceState {
    /// Shared predicate used by both launcher-entry guards.
    ///
    /// Launcher `Tab` and global `Cmd+Enter` must not route into ACP
    /// while an attachment portal is on-screen: the portal is hosted
    /// on what looks like a launcher view (`ScriptList`, clipboard
    /// history, etc.), but keyboard routing there belongs to the
    /// portal, not to the launcher.
    pub fn blocks_launcher_ai_entry(self) -> bool {
        matches!(self, Self::AttachmentPortal { .. })
    }

    /// Replacement for the legacy `attachment_portal_return_view.is_some()`
    /// probe. Kept on the enum so future callers don't read the
    /// snapshot fields directly.
    pub fn is_attachment_portal(self) -> bool {
        matches!(self, Self::AttachmentPortal { .. })
    }

    /// The portal kind, when in the `AttachmentPortal` state. Returns
    /// `None` from `Hidden` / `Embedded`.
    pub fn attachment_portal_kind(self) -> Option<PortalKind> {
        match self {
            Self::AttachmentPortal { kind } => Some(kind),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_hidden() {
        assert_eq!(AcpSurfaceState::default(), AcpSurfaceState::Hidden);
    }

    #[test]
    fn hidden_to_embedded_via_open() {
        assert_eq!(
            reduce_acp_surface(AcpSurfaceState::Hidden, AcpSurfaceEvent::EmbeddedOpened),
            AcpSurfaceState::Embedded
        );
    }

    #[test]
    fn embedded_open_is_idempotent() {
        assert_eq!(
            reduce_acp_surface(AcpSurfaceState::Embedded, AcpSurfaceEvent::EmbeddedOpened),
            AcpSurfaceState::Embedded
        );
    }

    #[test]
    fn embedded_to_portal_carries_kind() {
        let next = reduce_acp_surface(
            AcpSurfaceState::Embedded,
            AcpSurfaceEvent::PortalOpened {
                kind: PortalKind::FileSearch,
            },
        );
        assert_eq!(
            next,
            AcpSurfaceState::AttachmentPortal {
                kind: PortalKind::FileSearch
            }
        );
    }

    #[test]
    fn portal_open_from_hidden_is_noop() {
        // A portal without an embedded parent should never happen at
        // runtime, but the reducer must not invent an illegal state.
        let next = reduce_acp_surface(
            AcpSurfaceState::Hidden,
            AcpSurfaceEvent::PortalOpened {
                kind: PortalKind::FileSearch,
            },
        );
        assert_eq!(next, AcpSurfaceState::Hidden);
    }

    #[test]
    fn portal_close_returns_to_embedded() {
        let next = reduce_acp_surface(
            AcpSurfaceState::AttachmentPortal {
                kind: PortalKind::ClipboardHistory,
            },
            AcpSurfaceEvent::PortalClosed,
        );
        assert_eq!(next, AcpSurfaceState::Embedded);
    }

    #[test]
    fn portal_close_from_embedded_is_noop() {
        // The portal module already enforces a "one portal at a time"
        // guard; a stray PortalClosed from the embedded state must not
        // teleport the machine to Hidden.
        let next = reduce_acp_surface(AcpSurfaceState::Embedded, AcpSurfaceEvent::PortalClosed);
        assert_eq!(next, AcpSurfaceState::Embedded);
    }

    #[test]
    fn embedded_close_from_portal_drops_to_hidden() {
        // A host-level hide during an open portal is a real path: the
        // prompt handler or hotkey can force-close the main panel. The
        // portal must not outlive its embedded parent.
        let next = reduce_acp_surface(
            AcpSurfaceState::AttachmentPortal {
                kind: PortalKind::AcpHistory,
            },
            AcpSurfaceEvent::EmbeddedClosed,
        );
        assert_eq!(next, AcpSurfaceState::Hidden);
    }

    #[test]
    fn blocks_launcher_ai_entry_only_in_portal() {
        assert!(!AcpSurfaceState::Hidden.blocks_launcher_ai_entry());
        assert!(!AcpSurfaceState::Embedded.blocks_launcher_ai_entry());
        assert!(AcpSurfaceState::AttachmentPortal {
            kind: PortalKind::FileSearch
        }
        .blocks_launcher_ai_entry());
    }

    #[test]
    fn attachment_portal_kind_roundtrip() {
        assert_eq!(AcpSurfaceState::Hidden.attachment_portal_kind(), None);
        assert_eq!(AcpSurfaceState::Embedded.attachment_portal_kind(), None);
        assert_eq!(
            AcpSurfaceState::AttachmentPortal {
                kind: PortalKind::BrowserHistory
            }
            .attachment_portal_kind(),
            Some(PortalKind::BrowserHistory)
        );
    }
}
