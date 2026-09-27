//! Source-level contract for Oracle-Session `agent_chat-chat-state-machine-audit` PR1.
//!
//! Background: the "where does Agent Chat live right now" question was previously
//! answered by a 4-field conjunction across `ScriptListApp`:
//!   - `current_view == AppView::AgentChatView { .. }`
//!   - `embedded_agent_chat.is_some()`
//!   - `attachment_portal_return_view.is_some()`
//!   - `active_attachment_portal_kind.is_some()`
//!
//! Launcher-entry guards inferred their answer from different subsets
//! of these fields and drifted under refactor. PR1 collapses the
//! cross-product into one explicit enum —
//! `AgentChatSurfaceState { Hidden, Embedded, AttachmentPortal { kind } }` —
//! reduced by a tiny event machine in `src/ai/agent_chat/ui/surface_state.rs` and
//! mutated through the single `transition_agent_chat_surface` entry point in
//! `src/app_impl/agent_chat_surface_transitions.rs`.
//!
//! This contract pins the integration shape that the pure reducer's
//! unit tests cannot pin:
//!   1. No call site outside the mutator assigns the field directly.
//!      (`self.agent_chat_surface_state = ...` must only exist in the mutator.)
//!   2. Every Agent Chat open / close site in `agent_handoff.rs` fires the
//!      matching `EmbeddedOpened` / `EmbeddedClosed` event.
//!   3. Every attachment-portal open / close site in
//!      `attachment_portal.rs` fires `PortalOpened { kind }` /
//!      `PortalClosed`, and `is_in_attachment_portal()` reads from
//!      the app-owned machine instead of the legacy return-view probe.
//!   4. The public shape of the module stays stable — a silent rename
//!      that drops `blocks_launcher_ai_entry` or the reducer is caught
//!      here, not after the next refactor breaks both launcher guards.
