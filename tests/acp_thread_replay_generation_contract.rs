//! Source-level contract for ACP replay generation guards.

const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");

fn fn_body(source: &str, signature: &str, next_signature: &str) -> String {
    let start = source.find(signature).expect("signature should exist");
    let rest = &source[start..];
    let end = rest.find(next_signature).unwrap_or(rest.len());
    rest[..end].to_string()
}

// doc-anchor-removed: [[tests/notes-acp#ACP transcript replay generation#Replay resets transient stream state]]
#[test]
fn acp_thread_has_transcript_generation_guard() {
    assert!(
        ACP_THREAD_SOURCE.contains("transcript_generation: u64"),
        "AcpThread must store a transcript_generation field"
    );
    assert!(
        ACP_THREAD_SOURCE.contains("transcript_generation: 0"),
        "new/test AcpThread constructors must initialize transcript_generation to 0"
    );
    assert!(
        ACP_THREAD_SOURCE
            .contains("fn bump_transcript_generation(&mut self, reason: &'static str)"),
        "AcpThread must expose a traced generation bump helper"
    );
}

// doc-anchor-removed: [[tests/notes-acp#ACP transcript replay generation#Stale stream events are discarded]]
#[test]
fn bind_stream_captures_and_checks_generation_before_apply_event() {
    let body = fn_body(
        ACP_THREAD_SOURCE,
        "fn bind_stream(&mut self, rx: AcpEventRx, cx: &mut Context<Self>)",
        "fn bump_transcript_generation",
    );
    let capture = body
        .find("let generation = self.transcript_generation;")
        .expect("bind_stream must capture the current generation before spawning");
    let compare = body
        .find("if this.transcript_generation != generation")
        .expect("bind_stream must compare the live generation before applying events");
    let apply = body
        .find("this.apply_event(event, cx);")
        .expect("bind_stream must still apply current-generation events");
    assert!(
        capture < compare && compare < apply,
        "stream generation capture/check must happen before apply_event"
    );
}

// doc-anchor-removed: [[tests/notes-acp#ACP transcript replay generation#Replay resets transient stream state]]
#[test]
fn load_saved_messages_bumps_generation_and_clears_transient_state() {
    let body = fn_body(
        ACP_THREAD_SOURCE,
        "pub(crate) fn load_saved_messages(",
        "fn reset_pending_context_for_new_entry_intent",
    );
    for needle in [
        "self.bump_transcript_generation(\"load_saved_messages\");",
        "self.stream_task = None;",
        "self.stream_started_at = None;",
        "self.pending_permission = None;",
        "self.status = AcpThreadStatus::Idle;",
        "self.active_plan_entries.clear();",
        "self.active_tool_calls.clear();",
        "self.tool_call_lookup.clear();",
        "self.active_mode_id = None;",
        "self.available_commands.clear();",
        "self.usage_tokens = None;",
        "self.usage_cost_usd = None;",
        "self.next_message_id = 1;",
        "self.clear_all_pending_context(\"load_saved_messages\");",
        "self.messages.clear();",
    ] {
        assert!(
            body.contains(needle),
            "load_saved_messages must contain {needle}"
        );
    }
}
