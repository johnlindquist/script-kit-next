Implemented concurrency fixes from `plans/concurrency-improvements.md` in these files:

- `src/ai/session.rs:188`  
  - `ClaudeSessionManager.sessions` now stores `Arc<Mutex<ClaudeSession>>` handles.
  - Refactored `send_message` to avoid holding the global sessions map lock during blocking I/O (`session.send_message`).
  - Updated cleanup/close paths (`close_session`, `close_all_sessions`, `cleanup_stale_sessions`) to avoid long map-lock holds and to recover from poisoned per-session locks.
  - Added regression test `test_ai_sessions_do_not_serialize_when_multiple_sessions_active` at `src/ai/session.rs:747`.

- `src/scheduler.rs:62`  
  - Replaced `Arc<Mutex<bool>>` running flag with `Arc<AtomicBool>`.
  - Added interrupt channel (`stop_tx`) so `stop()` wakes the scheduler immediately instead of waiting for the 30s sleep window.
  - Added regression test `test_scheduler_stop_returns_quickly_when_idle` at `src/scheduler.rs:604`.

- `src/hotkeys.rs:779`  
  - Added `clone_hotkey_handler_with_poison_recovery` and switched notes/AI hotkey dispatch to use poison-tolerant lock recovery.
  - Added regression test `test_hotkey_handler_mutex_poison_recovery` at `src/hotkeys.rs:1576`.

Verification run:

- `cargo fmt` ✅
- `cargo check --lib` ✅
- `cargo check` ❌ blocked by unrelated parallel-agent errors:
  - `src/render_builtins.rs:83` and `src/render_builtins.rs:206` (`focus_main_filter` missing)
- `cargo test --lib test_scheduler_stop_returns_quickly_when_idle -- --exact` ❌ blocked by unrelated test compile error:
  - `src/ai/window.rs:8729` (`AiApp::message_body_content` missing)

How to test once workspace-wide blockers are resolved:

1. `cargo check`
2. `cargo test --lib test_ai_sessions_do_not_serialize_when_multiple_sessions_active -- --exact`
3. `cargo test --lib test_scheduler_stop_returns_quickly_when_idle -- --exact`
4. `cargo test --lib test_hotkey_handler_mutex_poison_recovery -- --exact`

Risks / known gaps:

- Full workspace compile/test is currently blocked by unrelated changes in other files.
- New Claude concurrency test is `#[cfg(unix)]` and uses a mock shell script.
- Same-session sends are still serialized by per-session mutex (intentional); only cross-session head-of-line blocking was removed.

Commits made: none.