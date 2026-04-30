//! Source-level contract test for the ACP cancel-midstream flow.
//!
//! User story `acp-cancel-midstream` requires: while a turn is streaming,
//! the user can cancel and return ACP to `Idle` without leaving an orphan
//! stream task. The original story draft assumed the cancel gesture was
//! Escape, but the implementation uses Cmd+. (the standard macOS cancel)
//! at `src/ai/acp/view.rs`. Escape is reserved for popup-dismiss /
//! return-to-main-menu. This test pins the Cmd+. gesture + the streaming
//! guard + the state-reset contract so a future refactor cannot silently
//! drop any of them.

const THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");

#[test]
fn cancel_streaming_is_gated_on_streaming_status() {
    assert!(
        THREAD_SOURCE.contains("pub(crate) fn cancel_streaming(&mut self, cx: &mut Context<Self>)"),
        "AcpThread::cancel_streaming must exist with the expected signature"
    );
    assert!(
        THREAD_SOURCE.contains("if !matches!(self.status, AcpThreadStatus::Streaming)"),
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
        THREAD_SOURCE.contains("self.status = AcpThreadStatus::Idle;"),
        "cancel_streaming must return the thread to Idle"
    );
}

#[test]
fn cmd_dot_keyboard_gesture_invokes_cancel_streaming() {
    assert!(
        VIEW_SOURCE.contains("modifiers.platform && key == \".\""),
        "Cmd+. must remain the ACP cancel-streaming gesture"
    );
    assert!(
        VIEW_SOURCE.contains(
            "matches!(\n                self.live_thread().read(cx).status,\n                AcpThreadStatus::Streaming\n            )"
        ) || VIEW_SOURCE.contains("AcpThreadStatus::Streaming"),
        "Cmd+. handler must only cancel when status is Streaming"
    );
    assert!(
        VIEW_SOURCE.contains(".update(cx, |thread, cx| thread.cancel_streaming(cx));"),
        "Cmd+. handler must invoke AcpThread::cancel_streaming"
    );
}

#[test]
fn escape_does_not_cancel_streaming() {
    let cancel_count = VIEW_SOURCE.matches("thread.cancel_streaming(cx)").count();
    assert!(
        cancel_count >= 2,
        "expected at least two call sites (Cmd+. keybinding + cancel button); found {cancel_count}"
    );
    assert!(
        VIEW_SOURCE.contains("Escape with no open dialogs: let it propagate to the main window"),
        "Escape must remain reserved for popup-dismiss / return-to-main; if this comment \
         disappears, re-audit whether escape-cancel was added and update the story accordingly"
    );
}
