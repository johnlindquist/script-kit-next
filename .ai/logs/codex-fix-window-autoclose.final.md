Implemented the window auto-close fix in `src/actions/window/part_01.rs`.

**Changed Files**
- `src/actions/window/part_01.rs`

**What Changed**
- Added a single close entrypoint `request_close(...)` so all close flows go through one lifecycle path: `on_close` callback, optional main-window re-activation, then deferred close.
  - `src/actions/window/part_01.rs:254`
- Kept deferred close as the only place that actually removes the window, and it clears `ACTIONS_WINDOW` before `remove_window()`.
  - `src/actions/window/part_01.rs:239`
- Focus-loss auto-close via activation subscription now calls `request_close(..., "focus_lost", false)` instead of directly calling `clear_actions_window_handle`/`remove_window`.
  - `src/actions/window/part_01.rs:279`
  - `src/actions/window/part_01.rs:307`
- Enter/Escape close paths now also call `request_close(...)`, so they use the same singleton-safe deferred close path.
  - `src/actions/window/part_01.rs:422`
  - `src/actions/window/part_01.rs:433`

**Verification Run**
- Ran: `cargo check -p script-kit-gpui`
- Ran: `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
- Ran: `cargo test -p script-kit-gpui --lib window_lifecycle_tests`

All three are currently blocked by unrelated in-progress repo errors outside this task scope (e.g. `src/clipboard_history/database.rs`, `src/clipboard_history/clipboard.rs`, `src/protocol/message/mod.rs`, `src/designs/core/part_04.rs`).

**How To Test (once tree compiles)**
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `cargo test -p script-kit-gpui --lib window_lifecycle_tests`
4. Run app with logs and verify focus-loss close path logs:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Risks / Known Gaps**
- Could not complete end-to-end runtime verification because unrelated compile failures in other files currently prevent a green build.
- No commits were made in this run.