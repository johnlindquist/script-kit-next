//! Source-level contract test for the ACP agent-switching draft-preservation flow.
//!
//! Backs `lat.md/acp-chat#ACP Chat#Agent switching` and user story
//! `acp-agent-switch-preserves-draft`. When the user switches agents from the
//! ACP actions menu, the relaunch path must stage a retry payload that carries
//! the current draft (input text, cursor, pending inline context parts) and
//! the current session's capability requirements, then force a fresh ACP open
//! (not a cached-view reuse) so the retry payload gets consumed. On the
//! re-open side, the payload must restore the draft before the new session
//! goes live. Source-level assertions are the tightest regression gate we can
//! build without a live multi-agent fixture.

const VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const HANDLE_ACTION_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");

// @lat: [[lat.md/acp-chat#ACP Chat#Agent switching]]
#[test]
fn agent_switch_dispatch_stages_retry_payload_before_relaunch() {
    assert!(
        HANDLE_ACTION_SOURCE.contains("crate::actions::acp_switch_agent_id_from_action(action_id)"),
        "handle_action must dispatch on the `acp_switch_agent:<id>` prefix"
    );
    assert!(
        HANDLE_ACTION_SOURCE.contains("view.stage_agent_switch_retry(next_agent_id.clone(), cx);"),
        "handle_action must stage the retry payload on the ACP view before relaunch"
    );
    assert!(
        HANDLE_ACTION_SOURCE.contains("self.open_tab_ai_acp_with_entry_intent(None, cx);"),
        "handle_action must re-open ACP with a None entry intent so the relaunch \
         does not inject a fresh entry-intent auto-submit on top of the preserved draft"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#Agent switching]]
#[test]
fn stage_agent_switch_retry_captures_draft_and_launch_requirements() {
    assert!(
        VIEW_SOURCE.contains("pub(crate) fn stage_agent_switch_retry("),
        "AcpView::stage_agent_switch_retry must exist as the entry point"
    );
    assert!(
        VIEW_SOURCE
            .contains("let launch_requirements = self.current_retry_launch_requirements(cx);"),
        "stage_agent_switch_retry must snapshot the current launch requirements"
    );
    assert!(
        VIEW_SOURCE.contains("let draft_state = self.current_retry_draft_state(cx);"),
        "stage_agent_switch_retry must capture the current draft (input, cursor, pending parts)"
    );
    assert!(
        VIEW_SOURCE.contains("self.pending_retry_request = Some(AcpRetryRequest {"),
        "stage_agent_switch_retry must persist the retry request on the view"
    );
    assert!(
        VIEW_SOURCE.contains("preferred_agent_id: Some(next_agent_id.clone()),"),
        "retry payload must carry the selected agent id"
    );
    assert!(
        VIEW_SOURCE.contains("event = \"acp_switch_agent_retry_payload_staged\","),
        "payload staging must emit the telemetry span other tests depend on"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#Agent switching]]
#[test]
fn agent_switch_relaunch_restores_input_cursor_and_context_parts() {
    assert!(
        VIEW_SOURCE.contains("\"acp_switch_agent_retry_restore\","),
        "replace_pending_context_parts must be called with the retry-restore reason tag"
    );
    assert!(
        VIEW_SOURCE.contains("thread.input.set_text(input_text.clone());"),
        "draft input text must be restored onto the new session's composer"
    );
    assert!(
        VIEW_SOURCE.contains("thread.input.set_cursor(input_cursor);"),
        "draft cursor position must be restored onto the new session's composer"
    );
    assert!(
        VIEW_SOURCE.contains("self.sync_inline_mentions(cx);"),
        "inline @mentions must re-sync so preserved context parts render as chips"
    );
    assert!(
        VIEW_SOURCE.contains("event = \"acp_switch_agent_retry_draft_restored\","),
        "draft restore must emit the telemetry span confirming the payload consumed"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#Agent switching]]
#[test]
fn rapid_agent_switch_churn_never_orphans_pending_retry_state() {
    // Story: `rapid-agent-switch-churn` — 3 back-to-back switches within 2s.
    // The single-switch contract test above already proves one stage + one
    // restore works. This test exercises the stage/restore/consume state
    // machine under repeated churn: the key invariants are that each new
    // stage call OVERWRITES the pending retry (last-wins, never appends),
    // that the restore-then-stage cycle reads the just-restored draft as
    // the new baseline (no amnesia), and that the final take_retry_request
    // call is the single consumption point that clears pending state to
    // None. A source-level assertion is the tightest gate we can build
    // without a live multi-agent fixture capable of sub-second agent
    // switches.

    assert!(
        VIEW_SOURCE.contains(
            "pub(crate) fn stage_agent_switch_retry(\n        &mut self,\n        next_agent_id: String,\n        cx: &mut Context<Self>,\n    ) {"
        ),
        "stage_agent_switch_retry signature must remain stable for churn — each switch \
         must enter through this single entry point"
    );
    assert!(
        VIEW_SOURCE.contains("self.pending_retry_request = Some(AcpRetryRequest {"),
        "stage_agent_switch_retry must assign (overwrite) pending_retry_request on every \
         call — never append — so a rapid switch sequence leaves only the latest payload"
    );

    // The fresh draft capture on every stage call is what lets each hop
    // carry the latest composer state forward. Without this, a rapid
    // second switch would stage a stale snapshot from before the first
    // restore completed. Pin the production helper to the live-thread
    // read path explicitly.
    assert!(
        VIEW_SOURCE.contains("fn current_retry_draft_state(&self, cx: &App)")
            && VIEW_SOURCE.contains("AcpChatSession::Live(thread) => {"),
        "current_retry_draft_state must read draft from the live thread every call so \
         each stage captures the current (possibly just-restored) composer state"
    );
    assert!(
        VIEW_SOURCE.contains("input_text: thread.input.text().to_string(),")
            && VIEW_SOURCE.contains("input_cursor: thread.input.cursor(),"),
        "draft capture must pull input_text and cursor from the thread's live composer \
         on each stage call — otherwise churn loses state between hops"
    );

    // Consumption must be single-shot and exhaustive. If take_retry_request
    // ever returned Some(...) twice for the same stage, churn could produce
    // ghost restores. `.take()` on Option guarantees Some once, then None.
    assert!(
        VIEW_SOURCE.contains(
            "pub(crate) fn take_retry_request(&mut self) -> Option<AcpRetryRequest> {\n        self.pending_retry_request.take()\n    }"
        ),
        "take_retry_request must use Option::take to guarantee single-consumption — \
         repeated take() after a single stage() must return None, preventing orphans \
         when churn races with consumption"
    );

    // After churn settles, the final open's take_retry_request call removes
    // the last pending request. If has_retry_request() is ever true when
    // take should have consumed, restore and churn desynchronize.
    assert!(
        VIEW_SOURCE.contains(
            "pub(crate) fn has_retry_request(&self) -> bool {\n        self.pending_retry_request.is_some()\n    }"
        ),
        "has_retry_request must be the mirror of take_retry_request — both read the \
         same field so reuse-gate and consumption agree on state"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#Agent switching]]
#[test]
fn cached_retry_request_forces_fresh_open_not_view_reuse() {
    assert!(
        TAB_AI_MODE_SOURCE.contains(
            "fn should_reuse_embedded_acp_view_for_open(\n        entry_intent: Option<&str>,\n        has_cached_retry_request: bool,\n    ) -> bool {\n        entry_intent.is_some() && !has_cached_retry_request\n    }"
        ),
        "view-reuse decision must skip reuse when a retry request is staged, otherwise \
         the staged draft payload is never consumed and the new agent's session starts empty"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("entity.read(cx).has_retry_request()"),
        "reuse check must consult has_retry_request on the embedded chat view"
    );
    assert!(
        VIEW_SOURCE.contains("pub(crate) fn has_retry_request(&self) -> bool {")
            && VIEW_SOURCE.contains("self.pending_retry_request.is_some()"),
        "has_retry_request must be the public predicate wrapping pending_retry_request"
    );
    assert!(
        VIEW_SOURCE
            .contains("pub(crate) fn take_retry_request(&mut self) -> Option<AcpRetryRequest> {"),
        "take_retry_request must exist as the single consumption point"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#Agent switching]]
#[test]
fn explicit_agent_switch_resolution_does_not_fallback_to_ready_agent() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("resolve_explicit_acp_launch_with_requirements"),
        "retry-backed agent switches must use explicit resolution so selecting Codex \
         cannot silently fall back to OpenCode"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("if retry_request.is_some()"),
        "the ACP open path must distinguish explicit retry/switch launches from \
         ordinary preference-based launches"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("resolve_acp_launch_with_requirements"),
        "ordinary preference launches should keep the capability-aware fallback resolver"
    );
}
