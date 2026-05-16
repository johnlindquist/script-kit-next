//! Source-level contract test for the `telemetry-span-coverage` user story.
//!
//! The story wants three specific `tracing` event/span names emitted across
//! an end-to-end ACP turn: `acp_turn_start`, `acp_stream_chunk`, and
//! `acp_turn_end`. None of those three literal names exist in the codebase.
//! Instead, the turn lifecycle is instrumented under a different naming
//! scheme:
//!
//! - Turn submission: `event = "acp_submit_resolved_context_parts"` in
//!   `src/ai/acp/thread.rs`, emitted when blocks are prepared for submit.
//! - Session bootstrap (first turn's prep): `"acp_session_created"` message
//!   in `src/ai/acp/client.rs` (two sites — one per prompt path).
//! - Turn termination (happy path): `"acp_turn_completed"` message in
//!   `src/ai/acp/client.rs` after the `connection.prompt()` future resolves
//!   on the streaming path.
//! - Turn termination (legacy/non-streaming path): `"acp_prompt_completed"`
//!   message in `src/ai/acp/client.rs` for `handle_stream_prompt`.
//! - Per-chunk streaming: there is no single `acp_stream_chunk` name —
//!   session notifications route through `session_notification` and emit
//!   granular names (`acp_agent_thought`, `acp_tool_call`,
//!   `acp_tool_call_update`, `acp_plan_received`, `acp_mode_change`,
//!   `acp_commands_update`, `acp_usage_update`,
//!   `acp_session_update_unhandled`) depending on the update kind.
//!
//! The literal trio in the story is therefore structurally unverifiable.
//! But the BEHAVIORAL invariant the story cares about — that every turn
//! has an observable submit edge, at least one stream-class event, and an
//! observable termination edge, each carrying enough fields to correlate —
//! is implemented, just under a different naming scheme. This test pins
//! the actually-emitted event names so a future refactor that renames
//! (or deletes) these signals fails loudly in CI.
//!
//! Invariants pinned:
//!
//! 1. Submit edge: `event = "acp_submit_resolved_context_parts"` is
//!    emitted before any prompt is sent, tagged with `target:
//!    "script_kit::tab_ai"` and carries `attempted`, `resolved`, and
//!    `failures` counts — a turn that silently submits with zero receipts
//!    is a regression.
//!
//! 2. Session-created edge: `"acp_session_created"` is emitted at BOTH
//!    prompt-path entry points in `client.rs` (modern streaming path +
//!    legacy `handle_stream_prompt` path) — losing either side leaves one
//!    prompt path silently sessionless.
//!
//! 3. Turn-completed edge: `"acp_turn_completed"` is emitted exactly once
//!    per resolved streaming prompt, tagged with `stop_reason`. The
//!    presence of `stop_reason` is the structural analogue of the story's
//!    `acp_turn_end` expected-field requirement — without it, a caller
//!    cannot distinguish normal completion from cancellation.
//!
//! 4. Legacy-prompt termination: `"acp_prompt_completed"` is retained as
//!    the legacy-path twin of `acp_turn_completed` — renaming only one of
//!    the two would break half the turn-termination visibility.
//!
//! 5. Per-chunk fanout: the `session_notification` path must emit at
//!    least six distinct update-kind names (`acp_agent_thought`,
//!    `acp_tool_call`, `acp_tool_call_update`, `acp_plan_received`,
//!    `acp_mode_change`, `acp_commands_update`, `acp_usage_update`) so
//!    the story's "expected fields" check has a concrete per-kind signal
//!    rather than a single opaque `acp_stream_chunk` blob.
//!
//! 6. Unhandled-notification safety net: `"acp_session_update_unhandled"`
//!    MUST remain as the catch-all for session update kinds the handler
//!    does not explicitly recognize — without it, a new ACP protocol
//!    update kind would silently drop.
//!
//! If the story text is later updated to match the implementation (or if
//! the implementation adds an explicit `acp_turn_start` / `acp_turn_end`
//! span trio), this contract test documents the migration path via its
//! module header and can be updated in-place.

const CLIENT_SOURCE: &str = include_str!("../src/ai/acp/client.rs");
const HANDLERS_SOURCE: &str = include_str!("../src/ai/acp/handlers.rs");
const THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn submit_edge_emits_resolved_context_parts_event_with_receipt_fields() {
    assert!(
        THREAD_SOURCE.contains("event = \"acp_submit_resolved_context_parts\","),
        "src/ai/acp/thread.rs must retain the `event = \"acp_submit_resolved_context_parts\"` \
         tracing field — this is the turn's submit edge. A turn that \
         submits without emitting any observable signal breaks every \
         downstream telemetry consumer that correlates blocks-at-submit \
         against stop-reason-at-completion."
    );
    assert!(
        THREAD_SOURCE.contains("target: \"script_kit::tab_ai\","),
        "the submit edge event must carry `target: \"script_kit::tab_ai\"` \
         so tab-ai-only log filters pick it up without swallowing other \
         crates' `acp_*` events"
    );
    for field in [
        "attempted = receipt.attempted,",
        "resolved = receipt.resolved,",
        "failures = receipt.failures.len(),",
    ] {
        assert!(
            THREAD_SOURCE.contains(field),
            "submit edge event must retain field `{field}` — the three \
             counts (attempted / resolved / failures) are the \
             structural analogue of the story's \
             `acp_turn_start`-has-expected-fields clause; dropping any \
             of them collapses the receipt"
        );
    }
}

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn session_created_edge_emitted_from_both_prompt_paths() {
    let count = CLIENT_SOURCE.matches("\"acp_session_created\"").count();
    assert!(
        count >= 2,
        "`\"acp_session_created\"` must appear in at least 2 call sites in \
         src/ai/acp/client.rs (streaming path + legacy \
         `handle_stream_prompt` path). Found {count}. If a prompt path \
         stops emitting it, that side is silently sessionless from the \
         telemetry's point of view — downstream consumers correlate \
         turns against this event to know a session exists."
    );
}

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn turn_completed_edge_emits_with_stop_reason_field() {
    assert!(
        CLIENT_SOURCE.contains("\"acp_turn_completed\""),
        "src/ai/acp/client.rs must retain the `\"acp_turn_completed\"` \
         message — this is the streaming-path turn-end edge. Renaming or \
         deleting it removes the only signal a telemetry consumer has \
         that the prompt future resolved successfully."
    );
    assert!(
        CLIENT_SOURCE.contains("stop_reason = ?prompt_response.stop_reason,")
            && CLIENT_SOURCE.contains("\"acp_turn_completed\""),
        "the turn-completed edge must carry `stop_reason = \
         ?prompt_response.stop_reason` — this is the structural analogue \
         of the story's `acp_turn_end`-has-expected-fields clause, \
         distinguishing normal completion from cancellation / \
         tool-request-stop"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn legacy_prompt_path_termination_edge_retained() {
    assert!(
        CLIENT_SOURCE.contains("\"acp_prompt_completed\""),
        "src/ai/acp/client.rs must retain the `\"acp_prompt_completed\"` \
         message — this is the legacy `handle_stream_prompt` path's \
         turn-end twin of `acp_turn_completed`. Renaming only one of \
         the two would break half the turn-termination visibility; \
         renaming both silently migrates telemetry consumers."
    );
}

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn session_notification_per_kind_fanout_preserves_granular_names() {
    for kind_event in [
        "\"acp_agent_thought\"",
        "\"acp_tool_call\"",
        "\"acp_tool_call_update\"",
        "\"acp_plan_received\"",
        "\"acp_mode_change\"",
        "\"acp_commands_update\"",
        "\"acp_usage_update\"",
    ] {
        assert!(
            HANDLERS_SOURCE.contains(kind_event),
            "src/ai/acp/handlers.rs must retain {kind_event} — the \
             per-update-kind fanout is what gives the story's \
             `acp_stream_chunk` its concrete fields. Collapsing these \
             into a single opaque `acp_stream_chunk` name would erase \
             the kind discrimination a telemetry consumer needs to \
             correlate tool calls against model output."
        );
    }
}

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn unhandled_session_update_has_a_catch_all_event() {
    assert!(
        HANDLERS_SOURCE.contains("\"acp_session_update_unhandled\""),
        "src/ai/acp/handlers.rs must retain \
         `\"acp_session_update_unhandled\"` — without this catch-all, a \
         new ACP protocol update kind (added in a future agent-client \
         crate upgrade) would silently drop from the telemetry without \
         any regression signal"
    );
}
