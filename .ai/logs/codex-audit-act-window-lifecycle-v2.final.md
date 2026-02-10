Implemented the window lifecycle audit fixes in scope and wrote the required findings log.

**Changed Files**
- `src/actions/window/part_01.rs:146`
  - Added lifecycle helpers for deterministic close behavior:
  - `should_auto_close_actions_window(...)`
  - `clear_window_slot(...)`
  - `clear_actions_window_handle(...)`
- `src/actions/window/part_01.rs:220`
  - Added `activation_subscription: Option<Subscription>` to keep focus-loss observer alive.
- `src/actions/window/part_01.rs:239`
  - Added `ActionsWindow::defer_close(...)` to clear singleton before `remove_window()`.
- `src/actions/window/part_01.rs:246`
  - Added `ensure_activation_subscription(...)` to auto-close on focus loss and invoke `on_close`.
- `src/actions/window/part_01.rs:404`
  - Routed Enter/Escape close paths through `defer_close(...)` (instead of direct `remove_window()`).
- `src/actions/window/part_01.rs:459`
  - Added lifecycle unit tests (`window_lifecycle_tests`).
- `.ai/logs/codex-audit-act-window-lifecycle-v2.final.md:1`
  - Wrote the full audit report for all 5 required checks.

**Verification**
- Passed: `rustfmt --check src/actions/window/part_01.rs`
- Attempted: `cargo check --lib`
  - Failed due to unrelated parallel-agent workspace errors (outside `part_01.rs`).
- Attempted: `cargo test --lib window_lifecycle_tests::test_clear_window_slot_is_idempotent_when_called_multiple_times`
  - Failed before test execution due to unrelated workspace/module conflicts and compile errors in other files.

**How To Test**
1. Re-run scoped formatting:
   - `rustfmt --check src/actions/window/part_01.rs`
2. Once workspace compiles again, run lifecycle tests:
   - `cargo test --lib window_lifecycle_tests`
3. Runtime lifecycle check with logs:
   - Open actions window, press `Escape`/`Enter`, and confirm singleton-clear logs tagged `ACTIONS_WINDOW_LIFECYCLE`.
   - Validate focus-loss auto-close by switching app focus away and confirming window closes.
   - Use:
     - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Risks / Known Gaps**
- Full compile/test gate is currently blocked by unrelated repository changes from other agents, so end-to-end verification is pending workspace stabilization.
- Positioning logic is in `src/actions/window/part_02.rs` (out of scope); no changes were made there.

Commits made: none.