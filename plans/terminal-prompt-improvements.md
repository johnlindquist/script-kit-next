# Terminal Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-terminal-prompt`  
Scope: `src/render_prompts/term.rs` (with dependent analysis in `src/term_prompt.rs` and `src/terminal/alacritty.rs`)

## Executive Summary

`render_term_prompt` is a clean wrapper, but the terminal stack has several high-impact gaps in correctness and scalability:

1. ANSI/style fidelity is incomplete (many parsed attributes are not rendered).
2. Theme/focus terminal colors are not wired to app theme updates.
3. High-volume output can monopolize the UI loop due to unbounded draining.
4. Scrollback UX is limited (find is unimplemented, copy-all only copies visible rows).
5. Escape/dismiss behavior is internally inconsistent.

## Responsibility Map (Current)

`src/render_prompts/term.rs` currently:

1. Syncs `TermPrompt.suppress_keys` with actions popup state (`src/render_prompts/term.rs:20`).
2. Handles high-level key routing for Cmd+K actions and view-level escapes (`src/render_prompts/term.rs:37`).
3. Renders terminal entity + footer + actions overlay (`src/render_prompts/term.rs:147`).

Actual terminal emulation/rendering happens in:

1. `src/term_prompt.rs` (key input mapping, timer loop, cell batching renderer).
2. `src/terminal/alacritty.rs` (PTY reader, ANSI parser, grid snapshot, scrollback state).

## Findings (Ranked)

### P1: ANSI attribute parsing is richer than what the renderer displays

Evidence:

1. `CellAttributes` tracks italic, dim, strikeout, inverse, hidden, multiple underline styles (`src/terminal/alacritty.rs:224`).
2. `render_content` only applies `BOLD` and `UNDERLINE` (`src/term_prompt.rs:795`).

Impact:

1. ANSI-heavy tools (git, ripgrep, test runners, TUIs) lose semantic emphasis.
2. Terminal output fidelity diverges from expected xterm behavior.

Recommendation:

1. Extend span styling for `ITALIC`, `DIM`, `STRIKEOUT`, and `INVERSE`.
2. Add explicit handling for hidden text and non-standard underline variants.

### P1: Terminal theme adapter is initialized to dark defaults and not propagated from app theme/focus

Evidence:

1. `TerminalHandle` always starts with `ThemeAdapter::dark_default()` (`src/terminal/alacritty.rs:453`).
2. `TerminalHandle::update_theme` and `update_focus` exist (`src/terminal/alacritty.rs:929`) but are not called from the TermPrompt render path.

Impact:

1. ANSI/default colors can mismatch current Script Kit theme and focus state.
2. Light-theme readability and focus dimming behavior are inconsistent.

Recommendation:

1. Initialize terminal adapter from `ThemeAdapter::from_theme(&theme)` when creating `TermPrompt`.
2. Propagate focus/theme changes into `TerminalHandle` on focus and theme transitions.

### P1: Large-output throughput is vulnerable to frame stalls and memory spikes

Evidence:

1. PTY output uses unbounded `mpsc::channel()` (`src/terminal/alacritty.rs:456`).
2. `process()` drains all queued chunks in a tight `while try_recv()` loop (`src/terminal/alacritty.rs:580`).
3. Refresh timer calls processing every 16ms (`src/term_prompt.rs:525`).

Impact:

1. Massive burst output can block UI updates inside one tick.
2. Backpressure is absent; queued `Vec<u8>` chunks can grow memory rapidly.

Recommendation:

1. Add bounded buffering (or ring buffer) with explicit backpressure policy.
2. Process with a per-tick byte/time budget and continue next frame.
3. Emit queue-depth/bytes-processed metrics for regressions.

### P1: Escape behavior is inconsistent between wrapper policy and terminal component

Evidence:

1. Wrapper comments indicate term prompts are not ESC-dismissable (`src/render_prompts/term.rs:72`).
2. `TermPrompt` directly closes on `escape` (`src/term_prompt.rs:864`).

Impact:

1. Behavior is surprising and difficult to reason about.
2. QuickTerminal vs SDK terminal semantics are easy to regress.

Recommendation:

1. Centralize dismiss policy in one layer (prefer app-level routing).
2. Add explicit mode flag for `QuickTerminalView` vs SDK `TermPrompt` escape semantics.

### P2: Scrollback feature set is limited and partially misleading

Evidence:

1. Scrollback default is fixed at 10,000 lines (`src/terminal/alacritty.rs:50`).
2. `CopyAll` uses only visible lines via `terminal.content().lines` (`src/term_prompt.rs:211`).
3. `Find` is declared but not implemented (`src/terminal/command_bar.rs:88`, `src/term_prompt.rs:357`).
4. Clear sends `ESC[2J` + `ESC[H` and does not explicitly clear scrollback (`src/term_prompt.rs:185`).

Impact:

1. Users cannot search historical output.
2. “Copy all” does not match user expectations for full scrollback.
3. Clearing view may leave history unexpectedly.

Recommendation:

1. Make scrollback size configurable from user config.
2. Add real terminal find (incremental search over visible+history).
3. Add an explicit “Clear Scrollback” command (`ESC[3J` semantics where appropriate).

### P2: Render path still does full-grid snapshot and heavy per-frame allocation

Evidence:

1. `content()` snapshots all visible cells every render (`src/terminal/alacritty.rs:672`).
2. `render_content` rebuilds selection `HashSet` and batch strings each frame (`src/term_prompt.rs:670`, `src/term_prompt.rs:736`).

Impact:

1. Higher CPU/alloc cost at larger terminal sizes and frequent repaints.
2. Sustained output workloads may trigger jitter and dropped frames.

Recommendation:

1. Introduce row/run caching keyed by dirty generation.
2. Avoid `HashSet<(col,row)>` when selection range can be checked arithmetically.
3. Consider dirty-rect rendering from terminal damage markers.

### P2: Terminal command UX is underutilized in prompt wrapper

Evidence:

1. `render_term_prompt` only exposes Cmd+K when SDK actions exist (`src/render_prompts/term.rs:85`).
2. Built-in terminal command palette plumbing exists but is marked dead code (`src/app_impl.rs:3901`).

Impact:

1. Native terminal actions (clear/copy/scroll/find) are not consistently discoverable.
2. Quick terminal users miss expected command palette controls.

Recommendation:

1. Always provide a terminal command set fallback when SDK actions are absent.
2. Merge SDK actions + terminal actions in one palette with section headers.

### P3: Input/write path flushes on every call

Evidence:

1. `TerminalHandle::input` does `write_all` + `flush` every invocation (`src/terminal/alacritty.rs:602`).

Impact:

1. High-frequency input (paste/keypress bursts) can create excess syscalls.

Recommendation:

1. Buffer writes and coalesce flushes (or flush only when necessary for interactive latency).

### P3: Observability is mostly string logs, not structured terminal telemetry

Evidence:

1. `render_term_prompt` uses `logging::log("KEY", ...)` strings for core routing (`src/render_prompts/term.rs:86`).
2. No queue-depth/output-rate fields are emitted during process loop.

Impact:

1. Hard to diagnose regressions in burst output, dropped frames, or key routing.

Recommendation:

1. Add structured `tracing` fields: `correlation_id`, `bytes_processed`, `queue_depth`, `render_ms`, `display_offset`.
2. Add slow-path spans around `process()` and `render_content()`.

## Improvement Plan

### Phase 1 (Correctness)

1. Resolve ESC semantics split between wrapper and `TermPrompt`.
2. Wire terminal theme/focus updates from app state.
3. Render additional ANSI attributes beyond bold/underline.

### Phase 2 (Scalability)

1. Add bounded PTY buffering + per-tick processing budget.
2. Add cached/dirty rendering to reduce full-grid rebuild cost.
3. Add telemetry for queue depth and frame cost.

### Phase 3 (UX)

1. Implement terminal find over scrollback.
2. Make copy-all/clear-scrollback semantics explicit and accurate.
3. Unify Cmd+K to always offer terminal command actions.

## Suggested Tests (TDD Names)

1. `test_term_prompt_escape_does_not_close_sdk_terminal_when_non_dismissable`
2. `test_terminal_renders_italic_dim_strike_inverse_attributes`
3. `test_terminal_theme_adapter_uses_app_theme_on_create_and_theme_reload`
4. `test_terminal_process_respects_per_tick_byte_budget_under_burst_output`
5. `test_terminal_copy_all_includes_scrollback_history`
6. `test_terminal_find_filters_results_across_visible_and_history_lines`
7. `test_terminal_cmd_k_shows_fallback_terminal_actions_without_sdk_actions`
8. `test_terminal_input_coalesces_flushes_for_large_paste`

## Risks / Known Gaps

1. True damage-based rendering requires careful integration with GPUI element diffing to avoid stale rows.
2. Scrollback search across very large histories needs bounded indexing to avoid memory spikes.
3. Escape policy changes affect user muscle memory and should be gated behind clear product decisions for SDK TermPrompt vs QuickTerminal.
