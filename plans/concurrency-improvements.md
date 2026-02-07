# Concurrency Improvements Audit

Date: 2026-02-07
Agent: codex-concurrency
Scope: `src/**/*.rs`

## Summary

This audit reviewed async/await usage, channels, mutexes, thread lifecycles, dispatch-queue usage, and cancellation/shutdown behavior.

Top priorities:

1. Remove lock-held-over-I/O in Claude session manager (`src/ai/session.rs`).
2. Make scheduler shutdown interruptible (`src/scheduler.rs`).
3. Replace startup/background polling bridges with event-driven receives (`src/app_impl.rs`).

## Method

- Static code audit of concurrency-sensitive paths in:
  - `src/ai/session.rs`
  - `src/scheduler.rs`
  - `src/app_impl.rs`
  - `src/file_search.rs`
  - `src/hotkeys.rs`
  - `src/hotkey_pollers.rs`
  - `src/watcher.rs`
  - `src/stdin_commands.rs`
  - `src/camera.rs`
  - `src/terminal/alacritty.rs`
  - `src/execute_script.rs`

## Findings

### 1) Critical: Global session mutex is held during blocking Claude I/O

Evidence:

- `src/ai/session.rs:230-287` acquires `sessions: Mutex<HashMap<...>>` and calls `session.send_message(...)` while still holding the lock.
- `src/ai/session.rs:107-139` shows `send_message` may block up to 120s with repeated `recv_timeout(100ms)`.

Risk:

- Head-of-line blocking across all sessions.
- Any concurrent send/create/cleanup path must wait for the slowest in-flight request.
- Increases deadlock surface for future callback paths.

Recommendation:

- Store per-session handles as `Arc<Mutex<ClaudeSession>>` (or equivalent), not raw sessions in a globally locked map.
- Lock the map only long enough to get/insert session handle, then drop map lock before sending.
- Add explicit cancellation token support to `ClaudeSession::send_message` so callers can abort before 120s timeout.

Validation:

- Add stress test with N concurrent session sends; assert parallel completion and no serialization.
- Add cancellation test: cancel mid-response and assert prompt return + process health.

### 2) High: Scheduler stop can block up to full check interval

Evidence:

- `src/scheduler.rs:235-297` uses `thread::sleep(Duration::from_secs(30))` in loop.
- `src/scheduler.rs:217-225` `stop()` sets a mutex bool then `join()`s thread.

Risk:

- Shutdown/stop latency up to 30s.
- UI/app teardown can appear hung.

Recommendation:

- Replace `Mutex<bool>` with `AtomicBool` and interruptible wait.
- Use a wake channel (`recv_timeout`) or condvar so `stop()` can notify immediately.
- Keep periodic check behavior, but make sleep abortable.

Validation:

- Add unit/integration test measuring `stop()` latency (target <200ms).

### 3) High: Background init/rebuild uses polling loops instead of event-driven receives

Evidence:

- App scan bridge: `src/app_impl.rs:71-134` (thread + mpsc + `Timer::after(50ms)` + `try_recv`).
- Provider registry startup: `src/app_impl.rs:472-507`.
- Provider registry rebuild: `src/app_impl.rs:7344-7377`.

Risk:

- Unnecessary wakeups and latency jitter.
- Repeated polling tasks increase background churn.

Recommendation:

- Use async-capable channels and await `recv()` directly in spawned GPUI async tasks.
- If staying with std mpsc, move blocking receive to background thread and forward one-shot result via async channel/UI callback.

Validation:

- Compare startup idle wakeups and time-to-ready for apps/providers before/after.

### 4) Medium: File-search streaming UI loops poll every 16ms and rely on detached task lifecycle

Evidence:

- Directory stream consumer: `src/app_impl.rs:2873-2967` (`Timer::after(16ms)` + `try_recv`).
- Search stream consumer: `src/app_impl.rs:2998-3113`.
- Producer completion paths do emit `SearchEvent::Done`: `src/file_search.rs:445`, `src/file_search.rs:549`, `src/file_search.rs:576`, `src/file_search.rs:584`, `src/file_search.rs:592`, `src/file_search.rs:659`.

Risk:

- Periodic polling adds wakeups even when no results are incoming.
- Detached consumer tasks can overlap in high-churn input scenarios; generation guards prevent stale writes but still consume work.

Recommendation:

- Convert consumers to receive-driven batching (event-triggered drain, timeout-based flush only when needed).
- Keep explicit cancel token (already present) but also abort/replace previous UI task handles deterministically.

Validation:

- Add perf test for high-frequency input changes; track CPU and stale-task count.

### 5) Medium: Critical user/control events can be dropped under backpressure

Evidence:

- Cancel exit message dropped if channel full: `src/app_impl.rs:6286-6291`.
- Prompt submit dropped if channel full: `src/app_impl.rs:7108-7118`.
- Hotkey/script/logs events dropped on full channels: `src/hotkeys.rs:1286`, `src/hotkeys.rs:1335`, `src/hotkeys.rs:1351`, plus notes/AI fallback paths `src/hotkeys.rs:868`, `src/hotkeys.rs:900`.

Risk:

- Lost user actions during load or blocked consumers.
- Hard-to-reproduce UX regressions.

Recommendation:

- Instrument per-channel dropped-event counters (structured logs + telemetry fields).
- For high-priority control events (`cancel`, `submit`), use limited retry or small blocking timeout off UI thread.
- For hotkeys, consider latest-wins coalescing strategy for repeated identical triggers.

Validation:

- Add channel-pressure tests asserting no silent loss for critical flows.

### 6) Medium: Poisoned mutex panic risk in hotkey dispatch path

Evidence:

- `src/hotkeys.rs:855-860` and `src/hotkeys.rs:887-891` use `.lock().unwrap()` on handler storage.

Risk:

- One panic while holding mutex can permanently poison lock and cause follow-on panic on hotkey use.

Recommendation:

- Use poison-tolerant recovery: `.lock().unwrap_or_else(|e| e.into_inner())`.
- Emit structured warning with context.

Validation:

- Unit test simulating poisoned mutex recovery behavior.

## Cancellation/Shutdown Assessment

Good patterns already present:

- Watchers use stop flags + interruptible control loops + thread joins in `Drop` (`src/watcher.rs:319-329`, `src/watcher.rs:551-559`, `src/watcher.rs:1176-1184`, `src/watcher.rs:1632-1640`).
- Camera capture performs ordered shutdown and dispatch-queue drain (`src/camera.rs:40-67`).
- Terminal handle signals reader shutdown on drop (`src/terminal/alacritty.rs:951-957`).
- Hotkey pollers are event-driven (recv-await) instead of timer polling (`src/hotkey_pollers.rs:24-30`, `src/hotkey_pollers.rs:257`, `src/hotkey_pollers.rs:319`, `src/hotkey_pollers.rs:348`).
- Stdin listener uses bounded channel and explicit thread exit on receiver drop (`src/stdin_commands.rs:172-205`).

Gaps:

- Claude session messaging has timeout but no caller-provided cancellation path.
- Scheduler stop responsiveness is tied to sleep interval.
- Some UI bridges still use timer polling where event-driven delivery is available.

## Suggested Implementation Plan

### Phase 1 (reliability, low API blast radius)

- Refactor `ClaudeSessionManager` to avoid manager-lock-held-over-send.
- Make scheduler loop interruptible and stop-fast.
- Replace `unwrap()` hotkey mutex locks with poison-tolerant handling.

### Phase 2 (parallelism and efficiency)

- Convert app/provider background bridges to event-driven receives.
- Convert file-search consumer loops to receive-driven batching.

### Phase 3 (observability and backpressure hardening)

- Add dropped-event counters for all bounded `try_send` paths.
- Add structured fields for queue length, drop count, and correlation IDs around critical flows.

## Regression Test Matrix (to add alongside changes)

- `test_ai_sessions_do_not_serialize_when_multiple_sessions_active`
- `test_scheduler_stop_returns_quickly_when_idle`
- `test_hotkey_handler_mutex_poison_recovery`
- `test_prompt_submit_not_silently_dropped_under_channel_pressure`
- `test_file_search_cancels_previous_task_without_stale_updates`

## Notes

- This task delivered analysis/reporting only; no runtime behavior was changed.
