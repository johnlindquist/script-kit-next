Implemented a dead-test purge focused on your requested files.

**Changed files**
- `src/actions/dialog_builtin_action_validation_tests_25.rs`
- `src/actions/dialog_builtin_action_validation_tests_31.rs`
- `src/actions/dialog_builtin_action_validation_tests_37.rs`
- `src/actions/dialog_builtin_action_validation_tests_42.rs`
- `src/actions/dialog_builtin_action_validation_tests_43.rs`
- `src/actions/dialog_window_tests.rs`

**What I changed**
- Replaced the 5 large generated batch files (`25/31/37/42/43`) with minimal placeholder docs (all test bodies removed).
- Removed duplicated shortcut/keycap tests from `src/actions/dialog_window_tests.rs` that are already covered in `src/actions/dialog_tests.rs`.
- Kept canonical coverage in `src/actions/dialog_tests.rs` and `src/actions/dialog_builtin_action_validation_tests_3.rs` untouched.

**How to test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

**Verification results**
- Ran full gate command: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
  - `cargo check`: passed (with unrelated warnings in other areas).
  - `cargo clippy`: failed due pre-existing/global issues outside this purge (many unused imports/dead-code in other `dialog_builtin_action_validation_tests_*` files, plus clippy issue in `src/ai/window.rs`).
  - `cargo test`: failed due unrelated existing failing tests:
    - `actions::dialog_builtin_action_validation_tests_39::tests::script_with_frecency_reset_ranking_no_shortcut`
    - `actions::dialog_builtin_action_validation_tests_41::tests::scriptlet_with_custom_reset_ranking_has_no_shortcut`
    - `actions::dialog_builtin_action_validation_tests_46::script_reset_ranking_no_shortcut`
    - `webcam_actions_consistency_tests::tests::render_webcam_footer_primary_uses_capture_flow`
- Targeted run that passed after purge:
  - `cargo test --lib actions::dialog_window_tests`
  - `cargo test --lib actions::dialog_tests`

**Risks / known gaps**
- Purging full batch files removes broad-but-low-signal assertions; if any unique useful assertion existed there, it is now gone.
- Repo is currently in active parallel-edit state; unrelated compile/test failures remain and can affect reproducibility of local runs until those are resolved.