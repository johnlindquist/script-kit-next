//! Source-level contract test for the Agent Chat detached-to-panel reattach flow.
//!
//! Backs user story `detach-then-reattach-identity` (2026-06 revision: the
//! detached window is an independent workspace). When the user fires
//! "Return to Panel" — from the panel actions, from automation, or from the
//! detached window's own Cmd+K menu — the conversation must survive the
//! round trip:
//!
//! 1. The reattach helper pulls the CURRENT live thread out of the detached
//!    view BEFORE closing the window (the detached window owns the thread
//!    and may have switched threads since the detach), then rebuilds the
//!    embedded view around that same thread entity.
//! 2. The detach exit path must NOT alias the detached thread into
//!    `self.embedded_agent_chat` — main-window Cmd+Enter starts a fresh chat
//!    while a detached window is open, so the cache must stay empty for the
//!    handed-off thread. Reattach therefore cannot rely on the cache; it
//!    reads the thread from the detached view itself.
//! 3. The detach exit path must release the Pi warm lease WITHOUT cancelling
//!    the in-flight turn (the detached window keeps streaming on that
//!    connection).
//!
//! Live sub-second detach/reattach verification requires a multi-agent Agent
//! Chat fixture with a real provider reachable, which the audit substrate
//! cannot guarantee. A source-level contract test is the tightest regression
//! gate we can build without that fixture.

const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/agent_handoff/mod.rs");
const HANDLE_ACTION_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/chat_window.rs");

fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("signature not found: {signature}"));
    let rest = &source[start..];
    let end = rest[1..]
        .find("\n    pub")
        .or_else(|| rest[1..].find("\n    fn "))
        .map(|ix| ix + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn reattach_pulls_live_thread_from_detached_view_before_closing_the_window() {
    let body = fn_body(
        TAB_AI_MODE_SOURCE,
        "pub(crate) fn reattach_embedded_agent_chat_from_detached(",
    );

    let read_pos = body
        .find("get_detached_agent_chat_view_entity()")
        .expect("reattach must read the live thread from the detached view");
    let close_pos = body
        .find("close_chat_window(cx);")
        .expect("reattach must close the detached window");
    assert!(
        read_pos < close_pos,
        "reattach must capture the detached view's thread BEFORE closing the window — \
         closing first drops the only strong path to the conversation"
    );
    assert!(
        body.contains("AgentChatView::new(thread, cx)")
            && body.contains("wire_embedded_agent_chat_footer_callbacks")
            && body.contains("enter_embedded_agent_chat_surface"),
        "reattach must rebuild the embedded view around the SAME thread entity and wire \
         the embedded host callbacks so footer actions keep working after the round trip"
    );
    assert!(
        body.contains("event = \"agent_chat_reattach_embedded_from_detached_thread\",")
            && body.contains("event = \"agent_chat_reattach_embedded_reused\",")
            && body.contains("event = \"agent_chat_reattach_embedded_cache_miss_fresh_launch\","),
        "thread-handoff, cached-view fallback, and fresh-launch fallback must emit distinct \
         telemetry spans so the audit log can distinguish them"
    );
}

#[test]
fn handle_action_reattach_arm_routes_to_the_preserving_helper_without_preclosing() {
    let Some(arm_start) = HANDLE_ACTION_SOURCE.find("\"agent_chat_reattach_panel\" => {") else {
        panic!("handle_action must keep an agent_chat_reattach_panel arm");
    };
    let arm = &HANDLE_ACTION_SOURCE[arm_start..];
    let reattach_pos = arm
        .find("self.reattach_embedded_agent_chat_from_detached(cx);")
        .expect("agent_chat_reattach_panel must route through the preserving reattach helper");
    if let Some(close_pos) = arm.find("close_chat_window(cx);") {
        assert!(
            close_pos > reattach_pos,
            "handle_action's agent_chat_reattach_panel arm must NOT close the detached window \
             before the reattach helper runs — the helper reads the live thread out of the \
             detached view, so a pre-close discards the conversation hand-off"
        );
    }
}

#[test]
fn detached_window_menu_reattach_routes_into_the_main_panel() {
    let arm_start = CHAT_WINDOW_SOURCE
        .find("\"agent_chat_reattach_panel\" => {")
        .expect("dispatch_detached_action must keep an agent_chat_reattach_panel arm");
    let arm = &CHAT_WINDOW_SOURCE[arm_start..arm_start + 300];
    assert!(
        arm.contains("reattach_detached_chat_into_main_panel(cx)"),
        "the detached window's own 'Return to Panel' action must route through \
         reattach_detached_chat_into_main_panel — a bare close_chat_window silently \
         drops the conversation instead of restoring it to the main panel"
    );
    let hook_body = fn_body(
        TAB_AI_MODE_SOURCE,
        "pub(crate) fn reattach_detached_chat_hook(",
    );
    assert!(
        hook_body.contains("reattach_embedded_agent_chat_from_detached(cx)")
            && hook_body.contains("WindowEvent::ShowMain { activate_app: true }"),
        "the binary-side reattach hook must call the app-side reattach helper and reveal \
         the main window so the restored conversation is actually visible"
    );
}

#[test]
fn detach_exit_path_does_not_alias_the_detached_thread_into_the_embedded_cache() {
    let body = fn_body(
        TAB_AI_MODE_SOURCE,
        "pub(crate) fn close_agent_chat_to_script_list(",
    );
    assert!(
        body.contains("detached_owns_thread"),
        "close_agent_chat_to_script_list (the detach exit path) must detect whether the \
         detached window now owns the closing view's thread"
    );
    assert!(
        body.contains("self.embedded_agent_chat = None;"),
        "when the detached window owns the thread, the embedded cache must stay empty so \
         main-window Cmd+Enter opens a fresh chat instead of aliasing the detached \
         conversation into the panel"
    );
    assert!(
        body.contains("release_agent_chat_warm_lease_for_detach_handoff"),
        "the detach exit path must release the warm lease via the no-cancel hand-off — \
         dismiss_reset_background would cancel_turn the connection the detached window is \
         still streaming on"
    );
}

#[test]
fn try_reuse_embedded_agent_chat_view_handles_none_intent_without_submit() {
    // The reattach fallback path calls try_reuse_embedded_agent_chat_view(None, cx).
    // The helper must gracefully handle a None entry_intent: set current_view
    // to the cached embedded entity without submitting anything. If the
    // submission gate were ever inverted (submit on None), reattach would
    // fire a spurious empty turn on reuse. Pin the submit guard.
    assert!(
        TAB_AI_MODE_SOURCE.contains(
            "if let Some(intent) = normalized_intent.clone().filter(|_| !is_setup_mode) {"
        ),
        "try_reuse_embedded_agent_chat_view must only submit when normalized_intent is Some \
         — None must fall through the if-let so a reattach (entry_intent=None) leaves \
         the cached thread state untouched"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("self.enter_embedded_agent_chat_surface(entity.clone(), cx);"),
        "try_reuse_embedded_agent_chat_view must enter AgentChatView with \
         the CACHED entity — not a freshly constructed one — so the preserved thread \
         entity inside the entity becomes the visible, addressable chat again"
    );
}
