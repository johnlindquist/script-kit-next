Implemented the in-scope HUD feedback polish and regression coverage.

**Changed files**
- `src/app_actions.rs:517`
- `src/app_actions.rs:1650`
- `src/clipboard_actions_tests.rs:147`

**What changed**
- Updated Finder reveal HUD text to use the requested wording:
  - `"Opened in Finder"` (replaced `"Revealed in Finder"`) in both reveal paths.
- Kept/verified existing clipboard HUD improvements from prior partial work:
  - pin/unpin success text (`"Pinned clipboard entry"`, `"Unpinned clipboard entry"`)
  - save-file feedback with full path (`"Saved to: ..."`) and Finder reveal.
- Added regression assertion:
  - `test_reveal_actions_show_opened_in_finder_hud` in `src/clipboard_actions_tests.rs`.

**How to test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Focused regression:
   - `cargo test --quiet webcam_actions_consistency_tests -- --nocapture`
5. Stdin protocol smoke launch:
   - `timeout 8 bash -c 'printf "%s\n" "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`

**Validation results**
- `cargo check`: passed.
- `cargo test`: passed (lib/tests/doc-tests).
- `webcam_actions_consistency_tests`: passed.
- Stdin JSON `show` smoke: command parsed and window show path executed in logs.
- `cargo clippy --all-targets -- -D warnings`: fails due pre-existing repo-wide lint debt outside this task scope (many `dialog_builtin_action_validation_tests_*` files and one `src/ai/window.rs:5724` clippy issue).

**Risks / known gaps**
- Clippy gate is still red due unrelated existing warnings/errors outside assigned scope.
- Webcam/path-action core implementations remain in `src/app_impl.rs` (owned by parallel workstream); this pass focused on scoped feedback wording/tests in `app_actions` + test coverage.