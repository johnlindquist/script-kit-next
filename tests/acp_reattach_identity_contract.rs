//! Source-level contract test for the ACP detached-to-panel reattach flow.
//!
//! Backs `removed-docs Chat#Detached window behavior` and user story
//! `detach-then-reattach-identity`. When the user fires "Return to Panel" from
//! a detached ACP chat window, the handle_action dispatcher closes the
//! detached window and MUST reuse the cached embedded AcpChatView (which
//! still holds a strong reference to the same AcpThread entity that the
//! detached view was rendering). Reusing the embedded view preserves the
//! thread's message history, pending context parts, and view identity across
//! the round trip. A fresh `open_tab_ai_acp_with_entry_intent(None, ...)`
//! falls through the reuse gate (entry_intent=None) and constructs a brand
//! new thread, silently discarding the prior conversation — that was the
//! bug this test pins against.
//!
//! Live sub-second detach/reattach verification requires a multi-agent ACP
//! fixture with a real provider reachable, which the audit substrate cannot
//! guarantee. A source-level contract test is the tightest regression gate
//! we can build without that fixture.

const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const HANDLE_ACTION_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");

#[test]
fn reattach_method_exists_and_reuses_cached_embedded_view_first() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("pub(crate) fn reattach_embedded_acp_from_detached("),
        "ScriptListApp must expose reattach_embedded_acp_from_detached as the single \
         entry point for the 'Return to Panel' flow"
    );
    let reattach_start = TAB_AI_MODE_SOURCE
        .find("pub(crate) fn reattach_embedded_acp_from_detached(")
        .expect("reattach_embedded_acp_from_detached should exist");
    let reattach_body =
        &TAB_AI_MODE_SOURCE[reattach_start..TAB_AI_MODE_SOURCE.len().min(reattach_start + 1500)];

    assert!(
        reattach_body.contains("if self.try_reuse_embedded_acp_view(\n            None,"),
        "reattach_embedded_acp_from_detached must call try_reuse_embedded_acp_view with \
         None entry_intent first — that reuses the cached embedded view (same thread \
         entity, same identity, full history) instead of launching a fresh session"
    );
    assert!(
        reattach_body.contains("event = \"acp_reattach_embedded_reused\",")
            && reattach_body.contains("event = \"acp_reattach_embedded_cache_miss_fresh_launch\","),
        "both reuse-success and cache-miss branches must emit distinct telemetry \
         spans so the audit log can distinguish a preserved-identity reattach from \
         a fresh fallback launch"
    );
}

#[test]
fn handle_action_reattach_arm_routes_to_the_preserving_helper() {
    let Some(arm_start) = HANDLE_ACTION_SOURCE.find("\"acp_reattach_panel\" => {") else {
        panic!("handle_action must keep an acp_reattach_panel arm");
    };
    let arm = &HANDLE_ACTION_SOURCE[arm_start..];
    let close_pos = arm
        .find("crate::ai::acp::chat_window::close_chat_window(cx);")
        .expect("acp_reattach_panel must close the detached chat window");
    let reattach_pos = arm
        .find("self.reattach_embedded_acp_from_detached(cx);")
        .expect("acp_reattach_panel must route through the preserving reattach helper");

    assert!(
        close_pos < reattach_pos,
        "handle_action's acp_reattach_panel arm must close the detached window then \
         route to the preserving reattach helper, not to open_tab_ai_acp_with_entry_intent \
         directly (which would fall through the reuse gate with entry_intent=None and \
         create a fresh thread, discarding history)"
    );
    assert!(
        !HANDLE_ACTION_SOURCE.contains("\"acp_reattach_panel\" => {\n                crate::ai::acp::chat_window::close_chat_window(cx);\n                self.open_tab_ai_acp_with_entry_intent(None, cx);"),
        "the pre-fix sequence (close_chat_window followed by open_tab_ai_acp_with_entry_intent(None)) \
         must NOT reappear in the acp_reattach_panel arm — that was the bug (entry_intent=None \
         skips the reuse gate at try_reuse_embedded_acp_view, forcing a fresh harness launch \
         with a new thread)"
    );
}

#[test]
fn try_reuse_embedded_acp_view_handles_none_intent_without_submit() {
    // The preserving reattach path calls try_reuse_embedded_acp_view(None, cx).
    // The helper must gracefully handle a None entry_intent: set current_view
    // to the cached embedded entity without submitting anything. If the
    // submission gate were ever inverted (submit on None), reattach would
    // fire a spurious empty turn on reuse. Pin the submit guard.
    assert!(
        TAB_AI_MODE_SOURCE.contains(
            "if let Some(intent) = normalized_intent.clone().filter(|_| !is_setup_mode) {"
        ),
        "try_reuse_embedded_acp_view must only submit when normalized_intent is Some \
         — None must fall through the if-let so a reattach (entry_intent=None) leaves \
         the cached thread state untouched"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("self.enter_embedded_acp_chat_surface(entity.clone(), cx);"),
        "try_reuse_embedded_acp_view must enter AcpChatView with \
         the CACHED entity — not a freshly constructed one — so the preserved thread \
         entity inside the entity becomes the visible, addressable chat again"
    );
}

#[test]
fn embedded_cache_is_populated_on_detach_so_reattach_has_a_target() {
    // The whole reattach-preserves-history story rests on this: when the
    // user detaches, close_acp_chat_to_script_list writes the current
    // embedded entity into self.embedded_acp_chat BEFORE clearing the
    // current view. That cached entity is what the reattach helper reuses
    // below. If the detach path ever stops populating this cache, the
    // reuse helper silently falls through to the fresh-launch branch and
    // identity is lost — so pin the assignment directly.
    assert!(
        TAB_AI_MODE_SOURCE.contains("if let AppView::AcpChatView { entity } = &self.current_view {\n            self.embedded_acp_chat = Some(entity.clone());"),
        "close_acp_chat_to_script_list (the detach exit path) must populate \
         self.embedded_acp_chat with a clone of the current AcpChatView entity \
         BEFORE flipping current_view away — this is what lets the reattach \
         helper reuse the same thread and preserve message history"
    );
}
