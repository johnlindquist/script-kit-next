Implemented the error-message UX improvements from `plans/error-messages-ux.md` in the unblocked files for this agent.

**Changed files**
- `src/prompt_handler.rs:4`
  - Added `unhandled_message_warning(...)` with recovery guidance.
  - Replaced unhandled protocol toast copy to: unsupported + “update script/update GPUI” guidance.
  - Added test: `test_unhandled_message_warning_includes_recovery_guidance` at `src/prompt_handler.rs:2099`.
- `src/app_actions.rs:60`
  - Added `selection_required_message_for_action(...)` for action-specific “selection required” guidance.
  - Replaced ambiguous “No item selected” HUD calls with contextual copy at multiple callsites (example: `src/app_actions.rs:849`).
  - Added tests:
    - `test_selection_required_message_for_action_returns_action_specific_guidance` at `src/app_actions.rs:259`
    - `test_selection_required_message_for_action_returns_safe_default` at `src/app_actions.rs:275`
- `src/app_impl.rs:1`
  - Added `calculate_fallback_error_message(...)`.
  - Updated calculator fallback HUD to: `Could not evaluate expression "...". Check the syntax and try again.` at `src/app_impl.rs:2681`.
  - Added test: `test_calculate_fallback_error_message_includes_expression_and_recovery` at `src/app_impl.rs:7431`.

**Verification run**
- `cargo check`  
  - Failed due unrelated in-flight tree issues outside this task (currently in `src/main.rs` / `src/stdin_commands.rs`, plus duplicate `mod tests` in other files).
- `CARGO_TARGET_DIR=/tmp/script-kit-gpui-impl-error-messages cargo check --lib`  
  - Passed.
- `CARGO_TARGET_DIR=/tmp/script-kit-gpui-impl-error-messages cargo test --bin script-kit-gpui test_unhandled_message_warning_includes_recovery_guidance -- --nocapture`  
  - Failed before reaching this test due unrelated existing compile errors in other modules.

**How to test**
1. Run `cargo check` after cross-agent compile blockers are resolved.
2. Run:
   - `cargo test --bin script-kit-gpui test_unhandled_message_warning_includes_recovery_guidance`
   - `cargo test --bin script-kit-gpui test_selection_required_message_for_action_returns_action_specific_guidance`
   - `cargo test --bin script-kit-gpui test_calculate_fallback_error_message_includes_expression_and_recovery`
3. Manual UX spot-check:
   - Trigger an unsupported prompt message and verify warning includes update guidance.
   - Invoke an action with no selection (e.g. copy path) and verify action-specific guidance.
   - Enter invalid calculator expression and verify recovery-oriented fallback message.

**Risks / known gaps**
- `src/main.rs:703` still has generic fallback `Error: ...` copy from the plan’s P1 list; not changed here because `src/main.rs` was actively claimed by another parallel agent during this task.
- Bin-targeted tests for these new test functions are currently blocked by unrelated compile failures in the shared working tree.

**Commits**
- None.