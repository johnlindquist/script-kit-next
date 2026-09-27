//! Source-level contract test for the Agent Chat cancel-midstream flow.
//!
//! User story `agent_chat-cancel-midstream` requires: while a turn is streaming,
//! the user can stop it and return Agent Chat to `Idle` without leaving an orphan
//! stream task. `AgentChatThread::stop_streaming` owns that stop; the thread's
//! status follows the reliability phase, and a user stop lands in `Cancelled`,
//! which reads as `Idle`. This test pins the streaming guard and the state-reset
//! contract so a future refactor cannot silently drop either of them.

const THREAD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/thread.rs");

fn fn_body(signature: &str) -> &'static str {
    let start = THREAD_SOURCE
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} must exist"));
    let open = start + THREAD_SOURCE[start..].find('{').expect("fn body");
    let mut depth = 0;
    for (offset, byte) in THREAD_SOURCE.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &THREAD_SOURCE[open..=open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("no matching brace for {signature}");
}

#[test]
fn cancel_streaming_is_gated_on_streaming_status() {
    let body = fn_body("pub(crate) fn stop_streaming(&mut self, cx: &mut Context<Self>)");
    assert!(
        body.contains("if !matches!(self.status, AgentChatThreadStatus::Streaming)"),
        "stop_streaming must early-return when not currently Streaming (idempotent)"
    );
}

#[test]
fn cancel_streaming_resets_stream_task_and_status() {
    let body = fn_body("pub(crate) fn stop_streaming(&mut self, cx: &mut Context<Self>)");
    assert!(
        body.contains("self.stream_task = None;"),
        "stop_streaming must drop the in-flight stream task"
    );
    assert!(
        body.contains("self.stream_started_at = None;"),
        "stop_streaming must clear the stream-start timestamp"
    );
    assert!(
        body.contains("AiOperationEvent::RuntimeStopped"),
        "stop_streaming must move the reliability phase to a stopped turn"
    );

    let sync = fn_body("fn sync_status_from_reliability(&mut self)");
    let cancelled = sync
        .find("AiPhase::Cancelled { .. }")
        .expect("a cancelled turn must map to a thread status");
    let idle = sync
        .find("=> AgentChatThreadStatus::Idle")
        .expect("some phases must map to Idle");
    let next_arm = sync[cancelled..]
        .find("=>")
        .map(|offset| cancelled + offset)
        .expect("cancelled arm");
    assert_eq!(
        next_arm, idle,
        "a cancelled turn must return the thread to Idle"
    );
}
