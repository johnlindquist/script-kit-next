**Audit Findings**
1. High: PTY reader thread could outlive terminal teardown, causing cleanup lag/resource leakage under repeated terminal creation/destruction.
`src/terminal/alacritty.rs:364`, `src/terminal/alacritty/handle_creation.rs:88`
Fixed by storing a reader thread handle and joining it during `TerminalHandle::drop` after signaling stop + best-effort PTY kill.

2. High: `process()` drained PTY output unbounded per tick, which can stall UI frames during high output bursts.
`src/terminal/alacritty/handle_runtime.rs:25`
Fixed by adding a per-tick processing budget (`MAX_PROCESS_BYTES_PER_TICK`) and batching parse work within that cap.

3. Medium: Resize path accepted `0x0` and always resized even when unchanged, causing error-prone/no-op churn.
`src/terminal/alacritty/handle_runtime.rs:101`
Fixed by clamping to at least `1x1` and skipping unchanged dimensions.

4. Medium: Command bar search accepted control characters; empty-state message rendered outside the list region.
`src/terminal/command_bar_ui.rs:138`, `src/terminal/command_bar_ui/render.rs:165`
Fixed by filtering control chars and rendering the empty state inside the command list viewport with minimum list height.

**Changed Files**
- `src/terminal/alacritty.rs`
- `src/terminal/alacritty/handle_creation.rs`
- `src/terminal/alacritty/handle_runtime.rs`
- `src/terminal/alacritty/tests/performance.rs`
- `src/terminal/command_bar_ui.rs`
- `src/terminal/command_bar_ui/render.rs`
- `src/terminal/command_bar_ui/tests.rs`

**How To Test**
1. `cargo test terminal::alacritty::tests::performance -- --nocapture`
2. `cargo test terminal::command_bar_ui::tests -- --nocapture`

Both passed.

**Commits**
- `52a9783` contains these terminal changes (included during a shared-tree commit race while another agent’s commit finalized).

**Risks / Known Gaps**
- I ran scoped verification only (parallel-agent safe), not full-workspace `cargo check/clippy/test`.
- No screenshot-based UI verification was run for command bar rendering in this pass.
