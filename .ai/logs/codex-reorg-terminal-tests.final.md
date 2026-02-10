**Changed Files**
- None. No edits were needed after verification.
- Verified structure is clean in:
  - `src/terminal/alacritty/tests/mod.rs`
  - `src/terminal/alacritty.rs`
  - `src/terminal/mod.rs`
  - `src/terminal/pty/tests.rs`
  - `src/terminal/theme_adapter/tests.rs`
  - `src/terminal/command_bar/tests.rs`
  - `src/terminal/command_bar_ui/tests.rs`

**What I Verified**
- `src/terminal/alacritty/tests/mod.rs` declarations exactly match files in `src/terminal/alacritty/tests/`.
- `cargo check -p script-kit-gpui` passed.

**How To Test**
1. Run `cargo check -p script-kit-gpui`
2. Optional declaration sanity check:
   - Compare `mod ...;` entries in `src/terminal/alacritty/tests/mod.rs` to `*.rs` files in `src/terminal/alacritty/tests/`

**Risks / Known Gaps**
- Targeted `cargo test` was blocked by unrelated concurrent-agent compile errors outside this scope:
  - `src/actions/tests/dialog_builtin_validation/dialog_builtin_action_validation_tests_11/part_01.rs:1`
- So test execution for terminal modules could not be completed in this shared state.

**Commits**
- None.