# Terminal Integration Audit

Date: 2026-02-07  
Audited crates: `portable-pty 0.9`, `alacritty_terminal 0.25`, `vte 0.15`, `bitflags 2.10`  
Audit scope: `Cargo.toml`, `src/terminal/*.rs`, `src/term_prompt.rs`

## Executive Summary

Terminal integration is generally solid on core rendering and ANSI parsing, but there is one high-impact protocol correctness gap:

1. `AlacrittyEvent::PtyWrite` is dropped instead of written back to the PTY. This breaks terminal response paths required by various control/query sequences.

There are also lifecycle/resource risks:

1. PTY reader thread is detached and never joined.
2. Shutdown ordering relies on best-effort behavior (`Drop` + blocking read unblocking) and can leave a background thread alive longer than intended.
3. Event/request features emitted by Alacritty (`ClipboardLoad`, `ColorRequest`, `TextAreaSizeRequest`) are currently ignored, reducing compatibility.

## Dependency Usage Check

- `portable-pty = "0.9"` is used in `src/terminal/pty.rs`.
- `alacritty_terminal = "0.25"` is used in `src/terminal/alacritty.rs`.
- `vte = "0.15"` is used via `vte::ansi::Processor` in `src/terminal/alacritty.rs`.
- `bitflags = "2.10"` is used for terminal cell presentation flags in `src/terminal/alacritty.rs`.

No obvious version skew issues were found among these four crates in current usage.

## Findings

### High: `PtyWrite` Events Are Ignored

Evidence:

- `src/terminal/alacritty.rs:116` matches `AlacrittyEvent::PtyWrite(text)` and returns `None` with comment “handled internally”.
- In `alacritty_terminal`, `Event::PtyWrite` is explicitly defined as “Write some text to the PTY” (`.../alacritty_terminal-0.25.1/src/event.rs:38-39`).
- Alacritty emits `PtyWrite` for device attributes/status/mode/size replies (`.../alacritty_terminal-0.25.1/src/term/mod.rs:1262`, `1337`, `1342`, `2090`, `2147`, `2270`).

Impact:

- Terminal-side queries that require emulator responses may fail or behave incorrectly.
- Reduces compatibility with applications expecting xterm/alacritty-style response sequences.

Recommendation:

- Treat `PtyWrite` as first-class output-to-PTY data, not as ignorable UI event.
- Minimal approach: add a thread-safe PTY writer sink and write `text.as_bytes()` when `PtyWrite` is received.
- Alternative: translate `PtyWrite` into an internal event and flush through `TerminalHandle::process()`.

### Medium: Reader Thread Is Detached (No Join / Deterministic Shutdown)

Evidence:

- Reader thread is spawned with `std::thread::spawn(...)` and handle is discarded (`src/terminal/alacritty.rs:464`).
- `TerminalHandle::Drop` only sets `reader_stop_flag` (`src/terminal/alacritty.rs:951-957`).
- Thread loop blocks in `reader.read(&mut buffer)` (`src/terminal/alacritty.rs:474`) and stop flag is checked only before read.

Impact:

- Shutdown is nondeterministic; thread may outlive `TerminalHandle` temporarily.
- If read does not unblock promptly (e.g., descendant process keeping slave open), thread can linger.
- Harder to reason about teardown and can accumulate resources in repeated create/drop cycles.

Recommendation:

- Store `JoinHandle<()>` in `TerminalHandle` and join during drop with timeout strategy.
- Ensure read unblocks before join by explicitly closing writer/master or signaling child termination in controlled order.

### Medium: PTY Lifecycle Works for Common Case, but Exit Accounting Is Weak

Evidence:

- `PtyManager::Drop` kills child if `is_running()` (`src/terminal/pty.rs:417-427`).
- `TermPrompt` falls back to `is_running()` and assigns synthetic exit code `0` if no explicit event (`src/term_prompt.rs:580-585`).
- `portable-pty` `Child::try_wait`/`wait` APIs are available (`.../portable-pty-0.9.0/src/lib.rs:131-138`), but normal terminal prompt flow does not call `wait()`.

Impact:

- Exit code fidelity can be lost (`0` used when unknown).
- Lifecycle state is “good enough” for UI closure but not robust for accurate process outcome reporting.

Recommendation:

- Capture real exit status path where possible (poll and store last observed status from PTY child).
- Consider explicit `wait`/reap path on terminal shutdown where it won’t block UI.

### Medium: Unimplemented Alacritty Request Events Reduce Feature Coverage

Evidence:

- `ClipboardStore`, `ClipboardLoad`, `ColorRequest`, `TextAreaSizeRequest` are all matched and dropped (`src/terminal/alacritty.rs:129-144`).
- Alacritty emits these to support OSC 52 clipboard operations, dynamic color queries, and text-area size requests (`.../alacritty_terminal-0.25.1/src/term/mod.rs:1679`, `1719`, `1740`, `2260`).

Impact:

- Missing behavior for remote clipboard integration and query-response sequences.
- Some TUI and shell utilities will silently lose expected functionality.

Recommendation:

- Implement at least `ColorRequest` and `TextAreaSizeRequest` responses through PTY write-back.
- Add policy-controlled clipboard support for OSC 52 (`load`/`store`).

### Low: Potential Memory Pressure Under Extreme PTY Output

Evidence:

- PTY reader uses unbounded `std::sync::mpsc::channel()` (`src/terminal/alacritty.rs:456`).
- Reader sends `Vec<u8>` chunks continuously (`src/terminal/alacritty.rs:482`) while UI polling cadence is timer-driven.

Impact:

- If producer throughput greatly exceeds consumer throughput, queue can grow.

Recommendation:

- Consider bounded channel or backpressure/coalescing strategy for very large bursts.

## Correctness Checks by Area

### Alacritty Grid / Parsing

What is correct:

- ANSI parsing path is correct: `vte::Processor::advance(&mut term, bytes)` (`src/terminal/alacritty.rs:316-319`) matches v0.15 API.
- Grid resize and display scroll APIs are used correctly (`src/terminal/alacritty.rs:627-645`, `754-795`).
- Terminal mode checks for bracketed paste and app cursor are correct (`src/terminal/alacritty.rs:901-922`).

Main correctness gap:

- Event response path (`PtyWrite` and request events) is incomplete.

### PTY Spawn / Resize / Close

What is correct:

- PTY pair creation and shell spawn are handled with context-rich errors (`src/terminal/pty.rs:147-205`).
- Resize goes through master PTY with updated state (`src/terminal/pty.rs:242-267`, `src/terminal/alacritty.rs:627-645`).
- Cleanup attempts to kill running child in `Drop` (`src/terminal/pty.rs:417-427`).

Risk areas:

- Detached background reader thread without join.
- Exit code fallback logic may mask actual process failure.

### `bitflags` Usage

`bitflags` use for `CellAttributes` mapping from Alacritty flags appears correct and maintainable (`src/terminal/alacritty.rs:187-259`).

## Prioritized Remediation Plan

1. Implement `PtyWrite` write-back pipeline (highest impact).
2. Make reader thread lifecycle deterministic (`JoinHandle`, explicit unblock/join).
3. Add support for `TextAreaSizeRequest`/`ColorRequest`; gate clipboard behavior via config/security policy.
4. Improve exit status fidelity and add tests around non-zero exits.
5. Consider bounded queue/backpressure if stress tests show memory growth.

## Suggested Tests

1. `test_terminal_writes_back_device_status_response_when_cpr_requested`
2. `test_terminal_handles_text_area_size_request_via_ptywrite`
3. `test_terminal_reader_thread_joins_on_drop`
4. `test_terminal_preserves_nonzero_exit_code_when_child_exits_without_event`
5. `test_terminal_high_output_does_not_unboundedly_grow_queue` (stress/integration)

