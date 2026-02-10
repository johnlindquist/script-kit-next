**Changed files**
- `src/app_actions/handle_action.rs:39`  
  Added `reveal_in_finder_with_feedback_async(...)` and `launch_editor_with_feedback_async(...)` helpers that return async completion results to UI.
- `src/app_actions/handle_action.rs:558`  
  Updated `"reveal_in_finder"` to show success HUD only after async reveal completes; now shows error HUD on failure.
- `src/app_actions/handle_action.rs:993`  
  Updated `"edit_script"` to wait for async editor launch result; now shows error HUD when launch fails.
- `src/app_actions/handle_action.rs:2060`  
  Updated `"edit_scriptlet"` with the same async editor launch feedback flow.
- `src/app_actions/handle_action.rs:2107`  
  Updated `"reveal_scriptlet_in_finder"` to show success/error HUD from async completion.
- `src/app_actions/helpers.rs:379`  
  Added regression tests:
  - `test_edit_actions_show_error_feedback_when_editor_launch_fails`
  - `test_reveal_actions_show_success_hud_after_async_completion`

**Verification run**
- `cargo test --lib app_actions_tests:: -- --nocapture`  
  Failed due unrelated workspace conflict: duplicate module files (`src/protocol/message/tests.rs` and `src/protocol/message/tests/mod.rs`).
- `cargo check && cargo clippy --all-targets -- -D warnings`  
  `cargo check` failed before clippy due unrelated existing workspace errors (clipboard type/field mismatches and non-exhaustive matches).
- `cargo clippy --all-targets -- -D warnings`  
  Failed due many unrelated pre-existing workspace issues (duplicate test modules, missing imports in other test suites, clipboard/type regressions).

**How to test**
1. Set editor to an invalid command in config and trigger `edit_script` / `edit_scriptlet`; confirm HUD shows `Failed to open in <editor>: ...`.
2. Trigger `reveal_in_finder` / `reveal_scriptlet_in_finder`; confirm success HUD appears only after reveal operation completes.
3. After workspace compile issues are resolved, rerun:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --lib app_actions_tests::`

**Risks / known gaps**
- Full compile/lint/test verification is currently blocked by unrelated parallel-agent changes in the shared tree.
- This fix targets `edit_script` and `edit_scriptlet` paths (plus reveal paths touched by m15); other `self.edit_script(...)` call sites keep existing behavior.