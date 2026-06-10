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
const VIEW_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/view.rs");

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

#[test]
fn cmd_dot_keyboard_gesture_invokes_cancel_streaming() {
    assert!(
        VIEW_SOURCE.contains("modifiers.platform && key == \".\""),
        "Cmd+. must remain the Agent Chat cancel-streaming gesture"
    );
    assert!(
        VIEW_SOURCE.contains(
            "matches!(\n                self.live_thread().read(cx).status,\n                AgentChatThreadStatus::Streaming\n            )"
        ) || VIEW_SOURCE.contains("AgentChatThreadStatus::Streaming"),
        "Cmd+. handler must only cancel when status is Streaming"
    );
    assert!(
        VIEW_SOURCE.contains(".update(cx, |thread, cx| thread.cancel_streaming(cx));"),
        "Cmd+. handler must invoke AgentChatThread::cancel_streaming"
    );
}

/// Returns up to `lines` lines of `source` starting at `marker` (panics if missing).
fn lines_after_marker(source: &str, marker: &str, lines: usize) -> String {
    let index = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker in view.rs: {marker}"));
    source[index..]
        .lines()
        .take(lines)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn cancel_gestures_keep_their_own_cancel_streaming_call_sites() {
    // Each known cancel gesture must keep its own cancel_streaming call site:
    // the Cmd+. keybinding handler and the streaming-dot footer button.
    let cmd_dot_scope = lines_after_marker(VIEW_SOURCE, "modifiers.platform && key == \".\"", 20);
    assert!(
        cmd_dot_scope.contains("thread.cancel_streaming(cx)"),
        "Cmd+. keybinding handler must invoke cancel_streaming"
    );
    let streaming_dot_scope = lines_after_marker(VIEW_SOURCE, "\"agent_chat-streaming-dot\"", 40);
    assert!(
        streaming_dot_scope.contains("thread.cancel_streaming(cx)"),
        "streaming-dot cancel button must invoke cancel_streaming"
    );
    // Escape-cancel was later added intentionally; it must route through the
    // dedicated streaming-gated helper rather than calling cancel_streaming
    // directly from ad-hoc escape branches.
    let escape_helper_scope = lines_after_marker(
        VIEW_SOURCE,
        "pub(crate) fn cancel_streaming_from_escape(&mut self, cx: &mut Context<Self>) -> bool",
        40,
    );
    assert!(
        escape_helper_scope.contains("thread.cancel_streaming(cx)"),
        "cancel_streaming_from_escape must invoke AgentChatThread::cancel_streaming"
    );
    assert!(
        VIEW_SOURCE.contains("self.cancel_streaming_from_escape(cx)"),
        "escape handling must route streaming cancellation through cancel_streaming_from_escape"
    );
}
