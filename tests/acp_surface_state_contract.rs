//! Source-level contract for Oracle-Session `acp-chat-state-machine-audit` PR1.
//!
//! Background: the "where does ACP live right now" question was previously
//! answered by a 4-field conjunction across `ScriptListApp`:
//!   - `current_view == AppView::AcpChatView { .. }`
//!   - `embedded_acp_chat.is_some()`
//!   - `attachment_portal_return_view.is_some()`
//!   - `active_attachment_portal_kind.is_some()`
//!
//! Launcher-entry guards inferred their answer from different subsets
//! of these fields and drifted under refactor. PR1 collapses the
//! cross-product into one explicit enum —
//! `AcpSurfaceState { Hidden, Embedded, AttachmentPortal { kind } }` —
//! reduced by a tiny event machine in `src/ai/acp/surface_state.rs` and
//! mutated through the single `transition_acp_surface` entry point in
//! `src/app_impl/acp_surface_transitions.rs`.
//!
//! This contract pins the integration shape that the pure reducer's
//! unit tests cannot pin:
//!   1. No call site outside the mutator assigns the field directly.
//!      (`self.acp_surface_state = ...` must only exist in the mutator.)
//!   2. Every ACP open / close site in `tab_ai_mode.rs` fires the
//!      matching `EmbeddedOpened` / `EmbeddedClosed` event.
//!   3. Every attachment-portal open / close site in
//!      `attachment_portal.rs` fires `PortalOpened { kind }` /
//!      `PortalClosed`, and `is_in_attachment_portal()` reads from
//!      the app-owned machine instead of the legacy return-view probe.
//!   4. The public shape of the module stays stable — a silent rename
//!      that drops `blocks_launcher_ai_entry` or the reducer is caught
//!      here, not after the next refactor breaks both launcher guards.

const ACP_SURFACE_STATE: &str = include_str!("../src/ai/acp/surface_state.rs");
const ACP_SURFACE_TRANSITIONS: &str = include_str!("../src/app_impl/acp_surface_transitions.rs");
const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const ATTACHMENT_PORTAL: &str = include_str!("../src/app_impl/attachment_portal.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");

#[test]
fn acp_surface_state_raw_writes_only_in_mutator() {
    // `self.acp_surface_state = ...` assignments must appear in exactly
    // one place — the mutator. Any drift (a future refactor poking the
    // field directly from a tab_ai_mode / attachment_portal path)
    // defeats the tracing and debug-assert coverage the mutator adds.
    assert!(
        ACP_SURFACE_TRANSITIONS.contains("self.acp_surface_state = next;"),
        "acp_surface_transitions.rs must assign the reduced next state"
    );

    for (label, source) in [
        ("src/app_impl/tab_ai_mode/mod.rs", TAB_AI_MODE),
        ("src/app_impl/attachment_portal.rs", ATTACHMENT_PORTAL),
        ("src/app_impl/startup.rs", STARTUP),
    ] {
        assert!(
            !source.contains("self.acp_surface_state ="),
            "{label} must not assign `self.acp_surface_state` directly — \
             route through `transition_acp_surface` so tracing + debug \
             asserts fire"
        );
    }
}

#[test]
fn acp_embedded_open_sites_fire_transition() {
    // All four embedded-ACP open paths (fresh, reuse, setup card, not
    // ready) must fire `EmbeddedOpened`. The count guards against a
    // future refactor that splits one of them into a new entry path
    // without wiring the transition.
    let opens = TAB_AI_MODE
        .matches("crate::ai::acp::surface_state::AcpSurfaceEvent::EmbeddedOpened")
        .count();
    assert!(
        opens >= 4,
        "tab_ai_mode.rs must fire EmbeddedOpened from all four ACP open paths \
         (fresh launch, reuse, setup card, not-ready); found {opens}"
    );

    let closes = TAB_AI_MODE
        .matches("crate::ai::acp::surface_state::AcpSurfaceEvent::EmbeddedClosed")
        .count();
    assert!(
        closes >= 2,
        "tab_ai_mode.rs must fire EmbeddedClosed from both close-to-script-list \
         and harness-terminal-closing-chat paths; found {closes}"
    );
}

#[test]
fn acp_attachment_portal_fires_portal_transitions() {
    // Attachment portal open/close must flow through
    // `transition_acp_surface`, and `is_in_attachment_portal()` must
    // read from the app-owned machine instead of the legacy
    // `attachment_portal_return_view.is_some()` snapshot probe.
    assert!(
        ATTACHMENT_PORTAL
            .contains("crate::ai::acp::surface_state::AcpSurfaceEvent::PortalOpened { kind }"),
        "open_attachment_portal must fire PortalOpened with the portal kind"
    );
    let closes = ATTACHMENT_PORTAL
        .matches("crate::ai::acp::surface_state::AcpSurfaceEvent::PortalClosed")
        .count();
    assert!(
        closes >= 2,
        "attachment_portal.rs must fire PortalClosed from both the accept \
         (close_with_part) and cancel paths; found {closes}"
    );
    assert!(
        ATTACHMENT_PORTAL.contains("self.acp_surface_state.is_attachment_portal()"),
        "is_in_attachment_portal() must read from the app-owned state \
         machine, not from `attachment_portal_return_view.is_some()`"
    );
}

#[test]
fn acp_surface_state_reducer_and_mutator_are_wired() {
    // Keep the public shape of the surface_state module stable: the
    // enum name, the pure reducer, the launcher-entry predicate, and
    // the mutator entry point. A rename should come with an explicit
    // test update, not a silent drop of contract coverage.
    assert!(
        ACP_SURFACE_STATE.contains("pub enum AcpSurfaceState")
            && ACP_SURFACE_STATE.contains("pub enum AcpSurfaceEvent")
            && ACP_SURFACE_STATE.contains("pub fn reduce_acp_surface(")
            && ACP_SURFACE_STATE.contains("pub fn blocks_launcher_ai_entry("),
        "surface_state.rs must expose the AcpSurfaceState enum, event \
         enum, pure reducer, and launcher-entry predicate"
    );
    assert!(
        ACP_SURFACE_TRANSITIONS.contains("pub(crate) fn transition_acp_surface(")
            && ACP_SURFACE_TRANSITIONS.contains("reduce_acp_surface(previous, event)")
            && ACP_SURFACE_TRANSITIONS.contains("debug_assert_acp_surface_consistent"),
        "acp_surface_transitions.rs must host the single mutator that \
         delegates to the pure reducer and carries the debug consistency \
         check"
    );
}
