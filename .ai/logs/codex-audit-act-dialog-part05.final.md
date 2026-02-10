# Audit Report: `src/actions/dialog/part_05.rs`

## Scope
- Audited `src/actions/dialog/part_05.rs` for closing/cleanup/error-handling coverage.
- Traced close/dismiss behavior implemented in:
  - `src/actions/dialog/part_02/part_01.rs`
  - `src/actions/dialog/part_02/part_02.rs`
  - `src/actions/dialog/part_02/part_03.rs`
  - `src/app_impl/actions_dialog.rs`
  - `src/actions/window/part_01.rs`

## Changes Made
- Added focused audit tests in `src/actions/dialog/part_05.rs` to verify close/cleanup patterns:
  - `test_trigger_on_close_invokes_callback_and_returns_bool_when_callback_presence_changes`
  - `test_submit_cancel_emits_cancel_sentinel_when_dialog_is_cancelled`
  - `test_dismiss_on_click_outside_delegates_to_submit_cancel_when_backdrop_is_clicked`
  - `test_selected_action_should_close_uses_protocol_flag_and_builtin_true_fallback`
  - `test_clear_sdk_actions_resets_mapping_search_and_selection_when_restoring_builtins`

## Findings

### 1) High: `close_actions_popup` bypasses per-dialog `on_close` cleanup callbacks
- Evidence:
  - Escape/Enter close path always calls `close_actions_popup` in `src/app_impl/actions_dialog.rs:96`, `src/app_impl/actions_dialog.rs:106`.
  - `close_actions_popup` clears only shared popup state and focus, but does not invoke the dialog `on_close` callback in `src/app_impl/actions_dialog.rs:223`.
  - File-search-specific cleanup (`file_search_actions_path = None`) exists only in the callback set by `set_on_close` in `src/render_builtins/actions.rs:79`.
  - `on_close` callback is invoked on the separate window path in `src/actions/window/part_01.rs:302` and `src/actions/window/part_01.rs:316`.
- Impact:
  - Main-window close paths can skip host-specific cleanup that is currently encoded in `on_close` callbacks.
  - This can leave stale host state (for example `file_search_actions_path`) until another code path clears it.

### 2) Coverage gap addressed in this patch
- `part_05.rs` previously covered only destructive action styling and section separators, not close/dismiss/cleanup behavior.
- Added targeted audit tests now pin the expected close/cancel/callback/reset behavior patterns.

## Verification
- Ran scoped tests:
  - `cargo test --lib actions::dialog::tests:: -- --nocapture`
- Result:
  - Passed: 8
  - Failed: 0
