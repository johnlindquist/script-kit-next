//! Source-level contract test for the `notes-hosted-acp-replaces-staging`
//! user story.
//!
//! The story wants a live receipt: open Notes, trigger ACP from a note, and
//! confirm `getAcpState.host="notes"` with any prior `acp_host=main` staging
//! cleared. Two structural gaps block a live run: (1) `AcpState` carries no
//! `host` field today, so `getAcpState.host` is structurally unverifiable
//! (same class of gap as Pass #12's missing `dictationStatus`); (2) the
//! story's "main-host staging is cleared" wording is misleading — the actual
//! design invariant (per `removed-docs transitions`)
//! is that staged portal sessions SURVIVE host transitions. What the story
//! is really asking for is *host isolation*: the Notes ACP surface must not
//! inherit the main launcher's ACP state, and vice versa.
//!
//! Rather than block on the missing `AcpState.host` field, pin the
//! structural isolation that makes host swaps safe. If any of these
//! invariants regress, the two hosts start sharing ACP view state and the
//! story's underlying guarantee collapses — well before the missing-field
//! gap is closed.
//!
//! Invariants pinned:
//!
//! 1. `NotesApp.embedded_acp_chat` and `ScriptListApp.embedded_acp_chat`
//!    are distinct struct fields on distinct types. Each host owns its
//!    own cached `Entity<AcpChatView>` — there is no shared-static view.
//!
//! 2. `spawn_hosted_view` (in `src/ai/acp/hosted.rs`) ALWAYS constructs a
//!    fresh view via `cx.new(|cx| AcpChatView::new(thread, cx))` — a host
//!    that spawns never inherits another host's view state.
//!
//! 3. `AcpChatView::new` initializes `pending_portal_session: None`, so a
//!    freshly spawned Notes-hosted view starts with no staged portal —
//!    even if the main-host view had one staged moments earlier.
//!
//! 4. `NotesApp::open_or_focus_embedded_acp` emits the tracing event
//!    `notes_acp_surface_opened` — this is the structural stand-in for
//!    the missing `acp_host=notes` receipt. Removing the event would lose
//!    the only audit-visible signal of the host swap.
//!
//! 5. `AcpChatView::prepare_for_host_hide` clears ephemeral popup state
//!    (attach menu, model selector, permission options, mention session,
//!    history menu, setup agent picker) but does NOT clear
//!    `pending_portal_session`. This matches
//!    `removed-docs transitions#Host hide keeps
//!    the staged session` — the staged portal contract outlives host
//!    hides so reattach can deliver the token.

const NOTES_WINDOW_SOURCE: &str = include_str!("../src/notes/window.rs");
const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");
const SCRIPT_LIST_APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");
const HOSTED_SOURCE: &str = include_str!("../src/ai/acp/hosted.rs");
const VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");

fn body<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("Expected start marker: {start_marker}"));
    let tail = &source[start..];
    let end = tail
        .find(end_marker)
        .unwrap_or_else(|| panic!("Expected end marker after {start_marker}: {end_marker}"));
    &tail[..end]
}

fn prepare_for_host_hide_slice() -> &'static str {
    let start = VIEW_SOURCE
        .find("pub(crate) fn prepare_for_host_hide(&mut self, cx: &mut Context<Self>) {")
        .expect("prepare_for_host_hide must exist in ai/acp/view.rs");
    let tail = &VIEW_SOURCE[start..];
    let body_start = tail
        .find('{')
        .expect("prepare_for_host_hide must have a body");
    let mut depth: i32 = 0;
    let mut end = body_start;
    for (idx, ch) in tail[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + idx + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    &tail[..end]
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior#Reattach preserves embedded view identity]]
#[test]
fn notes_and_script_list_have_distinct_embedded_acp_chat_fields() {
    assert!(
        NOTES_WINDOW_SOURCE.contains(
            "embedded_acp_chat: Option<Entity<crate::ai::agent_chat::ui::AgentChatView>>,"
        ),
        "NotesApp must declare its own embedded_acp_chat field — host \
         isolation depends on Notes and the main launcher holding \
         SEPARATE cached ACP view entities so neither can observe or \
         mutate the other's state"
    );
    assert!(
        SCRIPT_LIST_APP_STATE_SOURCE.contains(
            "pub(crate) embedded_acp_chat: Option<Entity<crate::ai::acp::view::AcpChatView>>,"
        ),
        "ScriptListApp must keep its own embedded_acp_chat field — if this \
         regresses into a shared static or moves to a host-neutral cache, \
         the two hosts start sharing view state and host isolation breaks"
    );
}

#[test]
fn notes_embedded_acp_registers_notes_parented_ai_identity() {
    assert!(NOTES_ACP_HOST_SOURCE.contains("NOTES_EMBEDDED_AI_AUTOMATION_ID"));
    assert!(NOTES_ACP_HOST_SOURCE.contains("\"notes:ai\""));
    assert!(NOTES_ACP_HOST_SOURCE.contains("fn sync_notes_embedded_acp_automation_window("));
    assert!(NOTES_ACP_HOST_SOURCE.contains("AutomationWindowKind::Ai"));
    assert!(NOTES_ACP_HOST_SOURCE.contains("parent_window_id: Some(\"notes\".to_string())"));
    assert!(NOTES_ACP_HOST_SOURCE
        .contains("parent_kind: Some(crate::protocol::AutomationWindowKind::Notes)"));
    assert!(NOTES_ACP_HOST_SOURCE.contains("semantic_surface: Some(\"notesAcpChat\".to_string())"));
    assert!(NOTES_ACP_HOST_SOURCE.contains("focused: false"));
    assert!(!NOTES_ACP_HOST_SOURCE.contains("ensure_embedded_ai_window(true)"));
}

#[test]
fn notes_embedded_acp_lifecycle_syncs_child_identity() {
    let open = body(
        NOTES_ACP_HOST_SOURCE,
        "pub(crate) fn open_or_focus_embedded_acp(",
        "cx.notify();",
    );
    assert!(open.contains("self.sync_notes_embedded_acp_automation_window(true)"));

    let close = body(
        NOTES_ACP_HOST_SOURCE,
        "fn close_embedded_acp_via_host(",
        "tracing::info!(",
    );
    assert!(close.contains("self.sync_notes_embedded_acp_automation_window(false)"));

    let prepare = body(
        NOTES_ACP_HOST_SOURCE,
        "pub(super) fn prepare_embedded_acp_for_window_close(",
        "tracing::info!(",
    );
    assert!(prepare.contains("self.sync_notes_embedded_acp_automation_window(false)"));
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior#Reattach preserves embedded view identity]]
#[test]
fn spawn_hosted_view_always_constructs_a_fresh_view() {
    assert!(
        HOSTED_SOURCE.contains("pub(crate) fn spawn_hosted_view("),
        "spawn_hosted_view must remain the single host-neutral factory \
         for hosted ACP views — Notes and the main launcher both route \
         through it so fresh-view semantics are enforced in one place"
    );
    assert!(
        HOSTED_SOURCE.contains("let view = cx.new(|cx| AcpChatView::new(thread, cx));"),
        "spawn_hosted_view must construct a NEW AcpChatView every call \
         (never return a cached one) — that is what guarantees a host \
         cannot inherit another host's view state at spawn time"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior#Reattach preserves embedded view identity]]
#[test]
fn freshly_constructed_acp_chat_view_has_no_pending_portal_session() {
    let occurrences = VIEW_SOURCE.matches("pending_portal_session: None,").count();
    assert!(
        occurrences >= 2,
        "AcpChatView::new must initialize pending_portal_session to None \
         in BOTH constructor arms (Setup and Live) — found {occurrences} \
         occurrences, expected >=2. A freshly spawned Notes-hosted view \
         must not observe a portal staged in the main-host view a moment \
         earlier."
    );
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior#Reattach preserves embedded view identity]]
#[test]
fn open_or_focus_embedded_acp_emits_host_swap_tracing_event() {
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("pub(crate) fn open_or_focus_embedded_acp("),
        "NotesApp::open_or_focus_embedded_acp must remain the entry point \
         for the Notes host swap — it is where the host-identity tracing \
         event is emitted"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("event = \"notes_acp_surface_opened\","),
        "open_or_focus_embedded_acp must emit the `notes_acp_surface_opened` \
         tracing event — this is the audit-visible stand-in for the \
         missing `acp_host=notes` receipt (AcpState carries no `host` \
         field today). Removing the event loses the only structural \
         signal that a Notes-hosted ACP surface came up."
    );
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior#Reattach preserves embedded view identity]]
#[test]
fn prepare_for_host_hide_clears_popups_but_not_pending_portal_session() {
    let slice = prepare_for_host_hide_slice();
    for field in [
        "self.attach_menu_open = false;",
        "self.model_selector_open = false;",
        "self.permission_options_open = false;",
        "self.clear_composer_picker(AcpComposerPickerDismissReason::HostHide, cx);",
        "self.history_menu = None;",
        "view.set_agent_picker(None, cx)",
    ] {
        assert!(
            slice.contains(field),
            "prepare_for_host_hide must clear `{field}` so ephemeral popup \
             state does not leak across a host hide — losing any of these \
             causes an orphaned popup when the other host takes over"
        );
    }
    assert!(
        !slice.contains("pending_portal_session"),
        "prepare_for_host_hide MUST NOT touch pending_portal_session — \
         staged portal contracts outlive host hides per \
         removed-docs transitions. If this \
         regresses (the field appears anywhere in the function body), a \
         reattach after a host swap silently drops the staged token \
         before the user can deliver it."
    );
}
