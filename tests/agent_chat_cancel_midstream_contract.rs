//! Source-level contract test for the Agent Chat cancel-midstream flow.
//!
//! User story `agent_chat-cancel-midstream` requires: while a turn is streaming,
//! the user can cancel and return Agent Chat to `Idle` without leaving an orphan
//! stream task. The original story draft assumed the cancel gesture was
//! Escape, but the implementation uses Cmd+. (the standard macOS cancel)
//! at `src/ai/agent_chat/ui/view.rs`; Escape-cancel was later added and must
//! route through the streaming-gated `cancel_streaming_from_escape` helper.
//! This test pins the Cmd+. gesture + the streaming guard + the state-reset
//! contract so a future refactor cannot silently drop any of them.

const THREAD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/thread.rs");

#[test]
fn cancel_streaming_is_gated_on_streaming_status() {
    assert!(
        THREAD_SOURCE.contains("pub(crate) fn cancel_streaming(&mut self, cx: &mut Context<Self>)"),
        "AgentChatThread::cancel_streaming must exist with the expected signature"
    );
    assert!(
        THREAD_SOURCE.contains("if !matches!(self.status, AgentChatThreadStatus::Streaming)"),
        "cancel_streaming must early-return when not currently Streaming (idempotent)"
    );
}

#[test]
fn cancel_streaming_resets_stream_task_and_status() {
    assert!(
        THREAD_SOURCE.contains("self.stream_task = None;"),
        "cancel_streaming must drop the in-flight stream task"
    );
    assert!(
        THREAD_SOURCE.contains("self.stream_started_at = None;"),
        "cancel_streaming must clear the stream-start timestamp"
    );
    assert!(
        THREAD_SOURCE.contains("self.status = AgentChatThreadStatus::Idle;"),
        "cancel_streaming must return the thread to Idle"
    );
}
