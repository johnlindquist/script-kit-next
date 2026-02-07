Implemented the state-sync refactor in your scoped files and verified with `cargo check`.

**Changed files**
1. `src/app_impl.rs`
- Added shared built-in query helpers to reduce duplication and keep shared/input state synchronized:
  - `current_view_uses_shared_filter_input` at `src/app_impl.rs:2702`
  - `sync_builtin_query_state` at `src/app_impl.rs:2714`
  - `clear_builtin_query_state` at `src/app_impl.rs:2728`
- `handle_filter_input_change` now syncs shared `filter_text` for built-in views and reuses shared query helper (e.g. `src/app_impl.rs:2748`, `src/app_impl.rs:2760`).
- Unified actions close branches to the canonical close path:
  - `toggle_actions` -> `close_actions_popup(...)` at `src/app_impl.rs:3616`
  - `toggle_arg_actions` at `src/app_impl.rs:3742`
  - `toggle_terminal_commands` at `src/app_impl.rs:3907`
  - `toggle_chat_actions` at `src/app_impl.rs:3972`
- Refactored built-in ESC-clear flow to reuse query-clear helper and keep shared filter state aligned at `src/app_impl.rs:6484`.
- Extracted and reused script-list reset helpers to reduce duplicated reset logic:
  - `reset_script_list_filter_and_selection_state` at `src/app_impl.rs:6566`
  - `request_script_list_main_filter_focus` at `src/app_impl.rs:6571`
  - Reused in `go_back_or_close` and `reset_to_script_list` (e.g. `src/app_impl.rs:6594`, `src/app_impl.rs:6974`).
- Added unit tests for new state helper behavior in `src/app_impl.rs:7379`.

2. `src/app_actions.rs`
- Kept action exit-to-script-list transition centralized via:
  - `transition_to_script_list_after_action` at `src/app_actions.rs:402`
  - used in action flow at `src/app_actions.rs:713`.

**How to test**
1. `cargo check`
2. Optional (targeted helper tests are present): `cargo test app_impl_state_sync_tests --lib`

**Verification run**
1. `cargo fmt -- src/app_impl.rs src/app_actions.rs`
2. `cargo check` passed.
3. `cargo test app_impl_state_sync_tests --lib` failed due unrelated pre-existing test-compile issues outside this scope (e.g. `src/ai/window.rs` missing `AiApp::message_body_content` in tests).

**Risks / known gaps**
1. Full `cargo test` is currently blocked by unrelated upstream test compilation errors, so only compile verification (`cargo check`) is clean.
2. I did not run UI stdin protocol runtime checks for this change set since the task explicitly required `cargo check`.

Commits made: none.