//! Source audit tests verifying symmetric ACP turn lifecycle instrumentation.
//!
//! Both ACP turn entry points (`handle_prompt_turn` and `handle_stream_prompt`)
//! must carry a `#[tracing::instrument]` span named `acp_turn` that records
//! `session` and `stop_reason` fields. They must emit a matching `acp_turn_start`
//! edge at the top of the function and record the span fields before returning.
//!
//! Rationale: ACP turn traces are frequently debugged across session + model +
//! stop_reason dimensions. Flat `tracing::info!` events scatter those fields
//! across separate log lines; spans collapse them into a single correlated unit.

use super::read_source as read;

const CLIENT_PATH: &str = "src/ai/acp/client.rs";

fn fn_body<'a>(content: &'a str, needle: &str) -> &'a str {
    let fn_start = content
        .find(needle)
        .unwrap_or_else(|| panic!("Expected function signature `{needle}` in {CLIENT_PATH}"));
    let fn_end = content[fn_start + needle.len()..]
        .find("\nasync fn ")
        .or_else(|| content[fn_start + needle.len()..].find("\nfn "))
        .map(|p| fn_start + needle.len() + p)
        .unwrap_or(content.len());
    &content[fn_start..fn_end]
}

fn attr_block<'a>(content: &'a str, needle: &str) -> &'a str {
    let fn_start = content
        .find(needle)
        .unwrap_or_else(|| panic!("Expected function signature `{needle}` in {CLIENT_PATH}"));
    let before = &content[..fn_start];
    let last_blank = before.rfind("\n\n").unwrap_or(0);
    &before[last_blank..]
}

// ---------------------------------------------------------------------------
// handle_prompt_turn — streaming path
// ---------------------------------------------------------------------------

#[test]
fn handle_prompt_turn_has_tracing_instrument_attribute() {
    let content = read(CLIENT_PATH);
    let attrs = attr_block(&content, "async fn handle_prompt_turn(");

    assert!(
        attrs.contains("#[tracing::instrument("),
        "handle_prompt_turn must be annotated with #[tracing::instrument(...)]"
    );
    assert!(
        attrs.contains(r#"name = "acp_turn""#),
        "handle_prompt_turn span must be named \"acp_turn\""
    );
    assert!(
        attrs.contains("skip_all"),
        "handle_prompt_turn span must use skip_all to avoid logging non-Display args"
    );
    assert!(
        attrs.contains("session = tracing::field::Empty"),
        "handle_prompt_turn span must declare an empty `session` field for later recording"
    );
    assert!(
        attrs.contains("stop_reason = tracing::field::Empty"),
        "handle_prompt_turn span must declare an empty `stop_reason` field for later recording"
    );
}

#[test]
fn handle_prompt_turn_emits_start_edge() {
    let content = read(CLIENT_PATH);
    let body = fn_body(&content, "async fn handle_prompt_turn(");

    assert!(
        body.contains(r#""acp_turn_start""#),
        "handle_prompt_turn must emit a `acp_turn_start` tracing event at the top of the span"
    );
}

#[test]
fn handle_prompt_turn_records_session_field_after_resolution() {
    let content = read(CLIENT_PATH);
    let body = fn_body(&content, "async fn handle_prompt_turn(");

    assert!(
        body.contains(r#"Span::current().record("session","#),
        "handle_prompt_turn must record the `session` field on its span once the id is known"
    );
}

#[test]
fn handle_prompt_turn_records_stop_reason_before_completion_event() {
    let content = read(CLIENT_PATH);
    let body = fn_body(&content, "async fn handle_prompt_turn(");

    let record_pos = body
        .find(r#"record("stop_reason""#)
        .expect("handle_prompt_turn must record `stop_reason` on its span");
    let completion_pos = body
        .find(r#""acp_turn_completed""#)
        .expect("handle_prompt_turn must still emit `acp_turn_completed` as the end edge");

    assert!(
        record_pos < completion_pos,
        "handle_prompt_turn must record stop_reason BEFORE emitting acp_turn_completed so the span carries the field"
    );
}

// ---------------------------------------------------------------------------
// handle_stream_prompt — legacy path
// ---------------------------------------------------------------------------

#[test]
fn handle_stream_prompt_has_tracing_instrument_attribute() {
    let content = read(CLIENT_PATH);
    let attrs = attr_block(&content, "async fn handle_stream_prompt(");

    assert!(
        attrs.contains("#[tracing::instrument("),
        "handle_stream_prompt must be annotated with #[tracing::instrument(...)]"
    );
    assert!(
        attrs.contains(r#"name = "acp_turn""#),
        "handle_stream_prompt span must share the `acp_turn` name with the streaming path"
    );
    assert!(
        attrs.contains("session = tracing::field::Empty"),
        "handle_stream_prompt span must declare an empty `session` field"
    );
    assert!(
        attrs.contains("stop_reason = tracing::field::Empty"),
        "handle_stream_prompt span must declare an empty `stop_reason` field"
    );
}

#[test]
fn handle_stream_prompt_emits_start_edge() {
    let content = read(CLIENT_PATH);
    let body = fn_body(&content, "async fn handle_stream_prompt(");

    assert!(
        body.contains(r#""acp_turn_start""#),
        "handle_stream_prompt must emit `acp_turn_start` symmetric with handle_prompt_turn"
    );
}

#[test]
fn handle_stream_prompt_records_session_and_stop_reason() {
    let content = read(CLIENT_PATH);
    let body = fn_body(&content, "async fn handle_stream_prompt(");

    assert!(
        body.contains(r#"record("session","#),
        "handle_stream_prompt must record `session` on its span"
    );
    assert!(
        body.contains(r#"record("stop_reason""#),
        "handle_stream_prompt must record `stop_reason` on its span"
    );
}

// ---------------------------------------------------------------------------
// End-edge events are preserved so downstream parsers keep working
// ---------------------------------------------------------------------------

#[test]
fn end_edge_event_names_are_preserved() {
    let content = read(CLIENT_PATH);

    assert!(
        content.contains(r#""acp_turn_completed""#),
        "handle_prompt_turn must keep emitting `acp_turn_completed` — downstream log parsers depend on it"
    );
    assert!(
        content.contains(r#""acp_prompt_completed""#),
        "handle_stream_prompt must keep emitting `acp_prompt_completed` — downstream log parsers depend on it"
    );
}
