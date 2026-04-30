//! Source-audit tests pinning the `acp_session_update` per-notification
//! span on `ScriptKitAcpClient::session_notification`.
//!
//! Background — `tool-acp-stream-chunk-span` was filed as a deferred
//! follow-up from Run 2 Pass that documented `session_notification`
//! (`src/ai/acp/handlers.rs:527`) ran in a different tokio task from the
//! turn handler's `acp_turn` span. The original deferred intent was
//! per-chunk spans that NESTED under `acp_turn`; that nesting requires
//! either (a) passing a `Span` clone through the event sink channel or
//! (b) using `tracing::Instrument::instrument(...)` on the turn side,
//! both of which are non-trivial refactors. Run 7 Pass #7 `Prompt:
//! Extend` ships the modest win instead: a standalone
//! `acp_session_update` span that gives per-notification duration +
//! correlation without attempting cross-task nesting, plus a stable
//! `kind` field so ops can filter by update variant.
//!
//! The structural invariants pinned here:
//! 1. `session_notification` is annotated with `#[tracing::instrument]`
//!    under the `acp_session_update` name (NOT reusing `acp_turn` —
//!    collision with the turn-span name would make log filters
//!    ambiguous and re-introduce the cross-task nesting confusion).
//! 2. The span declares `session_id` + `kind` as fields (`session_id`
//!    eagerly recorded from `args.session_id.0`, `kind` left Empty and
//!    recorded at the top of the fn body via
//!    `Span::current().record("kind", session_update_kind_name(...))`).
//! 3. The `kind` naming function `session_update_kind_name` exists and
//!    returns `&'static str` for every currently-matched `SessionUpdate`
//!    variant (user_message_chunk, agent_message_chunk,
//!    agent_thought_chunk, tool_call, tool_call_update, plan,
//!    current_mode_update, available_commands_update, usage_update) so
//!    the field is greppable per variant AND a future variant
//!    (catch-all) renders as "other".
//! 4. The doc comment above the instrument attribute documents the
//!    task-boundary limitation and points at `tool-acp-stream-chunk-span`
//!    so a future engineer reading the code understands WHY the span
//!    does not nest under `acp_turn` and where the parking-lot is.
//!
//! Refactor threats defended (REQUIRED for Pin subjects):
//! - "Someone promotes the per-kind tracing events to spans with
//!    `.entered()`" — silently creates root spans (because the task
//!    doesn't carry `acp_turn` context), loses correlation. Pin #1
//!    catches the absence of the wrapper-level span; Pin #4 keeps the
//!    `why` documented so the promoter pauses.
//! - "Someone renames the span to `acp_turn` to unify with client.rs"
//!    — would collide with the actual turn span and make a single log
//!    line's span stack ambiguous. Pin #1 asserts the literal name.
//! - "Someone drops the `kind` field from the span to shave bytes" —
//!    ops lose the per-variant filter. Pin #2 asserts field declarations.
//! - "Someone deletes `session_update_kind_name` and inlines the
//!    labels in each arm" — the mapping drifts per-arm instead of
//!    living in one enforceable place. Pin #3 asserts the helper.

use super::read_source as read;

const HANDLER_PATH: &str = "src/ai/acp/handlers.rs";

fn handlers_content() -> String {
    read(HANDLER_PATH)
}

fn session_notification_attr_block(content: &str) -> &str {
    let fn_start = content
        .find("async fn session_notification(")
        .expect("session_notification must exist in handlers.rs");
    let before = &content[..fn_start];
    let last_blank = before.rfind("\n\n").unwrap_or(0);
    &before[last_blank..fn_start]
}

fn session_notification_body(content: &str) -> &str {
    let fn_start = content
        .find("async fn session_notification(")
        .expect("session_notification must exist in handlers.rs");
    let rest = &content[fn_start..];
    // Body runs until the next top-level async fn in the impl.
    let end = rest
        .find("\n    async fn read_text_file(")
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn session_notification_has_acp_session_update_span_attribute() {
    let content = handlers_content();
    let attrs = session_notification_attr_block(&content);

    assert!(
        attrs.contains("#[tracing::instrument("),
        "session_notification MUST be annotated with \
         `#[tracing::instrument(...)]` so per-notification duration + \
         child-event correlation work. Without it, the existing \
         per-kind events (`acp_agent_thought`, `acp_tool_call`, etc.) \
         scatter across root-span log lines with no shared parent."
    );
    assert!(
        attrs.contains(r#"name = "acp_session_update""#),
        "The span MUST be named `acp_session_update`. Naming it \
         `acp_turn` would collide with `handle_prompt_turn` / \
         `handle_stream_prompt` in client.rs (pinned by \
         acp_turn_lifecycle_spans) and re-introduce the cross-task \
         nesting ambiguity that prompted this pass. Naming it \
         `acp_notification` would conflict with ACP protocol vocabulary \
         (notifications are any client-bound message, not just session \
         updates)."
    );
    assert!(
        attrs.contains("skip_all"),
        "The span attribute MUST include `skip_all` — `args: \
         SessionNotification` is a large, non-Display value and the \
         instrument macro would otherwise try to record it with \
         `?args`, bloating each log line with structured dumps. Matches \
         the `acp_turn` pattern in client.rs."
    );
    assert!(
        attrs.contains("session_id = %args.session_id.0"),
        "The span MUST record `session_id = %args.session_id.0` \
         eagerly in its `fields(...)` so every child event inherits the \
         session id without re-specifying it. Using `= ?args.session_id` \
         would emit the full SessionId tuple-struct debug print \
         instead of the bare id string."
    );
    assert!(
        attrs.contains("kind = tracing::field::Empty"),
        "The span MUST declare `kind = tracing::field::Empty` so the \
         field exists on the span at enter-time and can be recorded at \
         the top of the fn body via `Span::current().record(...)`. \
         Without the declaration, `record(\"kind\", ...)` is a no-op."
    );
}

#[test]
fn session_notification_records_kind_field_via_helper() {
    let content = handlers_content();
    let body = session_notification_body(&content);

    assert!(
        body.contains("Span::current().record(\"kind\", session_update_kind_name(&args.update));"),
        "session_notification MUST record the `kind` field on its span \
         via the `session_update_kind_name(&args.update)` helper BEFORE \
         the match block. Recording inline in each arm (e.g. `Span::\
         current().record(\"kind\", \"user_message_chunk\")` inside the \
         UserMessageChunk arm) would drift per-arm; recording AFTER the \
         match (post-processing) would miss the window where child \
         events nest under the span. Recording via the helper centralizes \
         the variant→label mapping into one place `session_update_kind_\
         name` can be audited exhaustively."
    );
}

#[test]
fn session_update_kind_name_helper_covers_all_current_variants() {
    let content = handlers_content();
    assert!(
        content.contains("fn session_update_kind_name(update: &SessionUpdate) -> &'static str"),
        "Helper `fn session_update_kind_name(update: &SessionUpdate) -> \
         &'static str` MUST exist in handlers.rs. A future refactor that \
         moves this into a separate module (e.g. `super::events`) is \
         fine as long as the signature + variant coverage below hold, \
         but the pin asserts the helper stays discoverable from \
         `session_notification`'s own file."
    );

    // Variants currently matched in session_notification (Run 7 Pass #7
    // snapshot). The catch-all `_ => "other"` handles future variants.
    let required_variants = [
        ("SessionUpdate::UserMessageChunk(_)", "user_message_chunk"),
        ("SessionUpdate::AgentMessageChunk(_)", "agent_message_chunk"),
        ("SessionUpdate::AgentThoughtChunk(_)", "agent_thought_chunk"),
        ("SessionUpdate::ToolCall(_)", "tool_call"),
        ("SessionUpdate::ToolCallUpdate(_)", "tool_call_update"),
        ("SessionUpdate::Plan(_)", "plan"),
        ("SessionUpdate::CurrentModeUpdate(_)", "current_mode_update"),
        (
            "SessionUpdate::AvailableCommandsUpdate(_)",
            "available_commands_update",
        ),
        ("SessionUpdate::UsageUpdate(_)", "usage_update"),
    ];
    for (pattern, label) in required_variants {
        let arm = format!("{pattern} => \"{label}\"");
        assert!(
            content.contains(&arm),
            "session_update_kind_name MUST map `{pattern}` to the \
             label `\"{label}\"`. Label drift (e.g. renaming to camelCase \
             or plural) breaks ops queries and dashboards that filter by \
             `kind`. Missing arm:\n  expected: {arm}"
        );
    }

    assert!(
        content.contains("_ => \"other\""),
        "session_update_kind_name MUST have a catch-all `_ => \"other\"` \
         arm. When the upstream `agent_client_protocol` crate adds a new \
         `SessionUpdate` variant, the helper returns a greppable \"other\" \
         label so the notification still carries a `kind` field (the \
         dispatcher's match below also falls through to `_ => \
         tracing::trace!(\"acp_session_update_unhandled\")`, so the two \
         catch-alls stay in lockstep)."
    );
}

#[test]
fn session_notification_documents_cross_task_nesting_limit() {
    let content = handlers_content();
    let attrs = session_notification_attr_block(&content);

    assert!(
        attrs.contains("tool-acp-stream-chunk-span"),
        "The doc comment above `#[tracing::instrument]` on \
         session_notification MUST reference the parking-lot story \
         `tool-acp-stream-chunk-span` in `audits/afk/stories.md`. \
         Without the reference, a future engineer reading the code has \
         no breadcrumb to the deferred-work discussion explaining why \
         the span does NOT nest under `acp_turn`. The reference is the \
         ONLY structural signal that the cross-task boundary is a \
         known, parked design decision rather than an oversight."
    );
    assert!(
        attrs.contains("acp_turn"),
        "The doc comment MUST name `acp_turn` so the reader \
         understands which span they might EXPECT to see as a parent \
         and why it is absent. Anchors the reader in the turn-lifecycle \
         context already pinned by tests/source_audits/acp_turn_\
         lifecycle_spans.rs."
    );
    assert!(
        attrs.contains("Instrument"),
        "The doc comment MUST mention `tracing::Instrument` as one of \
         the two documented cross-task nesting options. Without it the \
         'how would we actually do this?' question is unanswered and a \
         motivated future engineer has to re-derive the investigation."
    );
}
