Implemented the requested fixes in the two scoped files.

**Changed files**
1. `src/app_execute/builtin_execution.rs`
- Added user-visible error toasts for AI window-open failures in the previously log-only AI command paths:
  - `AiCommandType::SendScreenToAi` at `src/app_execute/builtin_execution.rs:617`
  - `AiCommandType::SendFocusedWindowToAi` at `src/app_execute/builtin_execution.rs:667`
  - `AiCommandType::SendSelectedTextToAi` at `src/app_execute/builtin_execution.rs:717`
  - `AiCommandType::SendBrowserTabToAi` at `src/app_execute/builtin_execution.rs:771`
  - `AiCommandType::CreateAiPreset | ImportAiPresets | SearchAiPresets` at `src/app_execute/builtin_execution.rs:821`
- Added helper `ai_open_failure_message(...)` at `src/app_execute/builtin_execution.rs:1`.
- Added unit test module `builtin_execution_ai_feedback_tests` at `src/app_execute/builtin_execution.rs:1218`.

2. `src/app_impl/execution_scripts.rs`
- Added `builtin-open-ai` and `builtin-open-notes` to no-main-window handling via:
  - `NO_MAIN_WINDOW_BUILTINS` at `src/app_impl/execution_scripts.rs:3`
  - `builtin_needs_main_window_for_command_id(...)` at `src/app_impl/execution_scripts.rs:14`
  - call site update in `execute_by_command_id_or_path` at `src/app_impl/execution_scripts.rs:380`
- Added unit test module `builtin_command_window_visibility_tests` at `src/app_impl/execution_scripts.rs:19`.

**How to test**
1. Requested verification commands (run, currently failing due unrelated parallel changes):
- `cargo check -p script-kit-gpui`
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

2. Targeted unit test filters for this change:
- `cargo test -p script-kit-gpui --bin script-kit-gpui builtin_execution_ai_feedback_tests`
- `cargo test -p script-kit-gpui --bin script-kit-gpui builtin_command_window_visibility_tests`

**Verification status**
- I ran:
  - `cargo check -p script-kit-gpui`
  - `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
  - targeted `cargo test` filters
- All were blocked by unrelated workspace errors outside this task scope (examples):
  - duplicate `mod tests` resolution conflicts in `src/protocol/message/mod.rs` and `src/designs/core/part_04.rs`
  - clipboard model/match errors in `src/clipboard_history/*` (missing `source_app_*` fields, non-exhaustive `ContentType` matches)
  - unrelated actions test/import failures in `src/actions/*`

**Risks / known gaps**
1. Runtime validation is limited because workspace compilation is currently broken outside these files.
2. The added unit tests are in place but cannot execute until the unrelated compile errors are resolved.

Commits made: none.