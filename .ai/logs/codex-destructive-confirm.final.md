Implemented destructive-action confirmation gates and removed the unsafe auto-execute fallback when confirmation UI fails.

**Changed Files**
- `src/app_execute.rs:91`  
  Removed the fallback that auto-confirmed actions if `open_confirm_window` failed; now it logs and skips execution.
- `src/app_actions.rs:1404`  
  Added confirmation modal for `clipboard_delete_multiple` with a count-aware message before deletion.
- `src/app_actions.rs:1610`  
  Added confirmation modal for `clipboard_delete_all` with unpinned-entry count before clearing.
- `src/app_impl.rs:4167`  
  Added confirmation modal for chat `clear_conversation`.
- `src/app_impl.rs:5695`  
  Added confirmation modal for path `move_to_trash` with file name in the prompt.
- `src/clipboard_actions_tests.rs:124`  
  Added regression/source-audit tests covering the new confirmation requirements and the removed auto-confirm fallback.

**Validation Run**
- `cargo check` passed.
- `cargo clippy --all-targets -- -D warnings` failed due pre-existing unrelated lint debt (not from these changes), mainly in `src/actions/dialog_builtin_action_validation_tests_*.rs` and `src/ai/window.rs:5724`.
- `cargo test` passed (`9562 passed; 0 failed; 35 ignored`).
- Stdin JSON protocol launch verified:
  - `Received: {"type":"show"}`
  - `Parsed command: Show { request_id: None }`

**How To Test**
1. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
2. Launch via stdin protocol:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. In app flows, verify destructive actions now show confirmation first:
   - Clipboard: `clipboard_delete_multiple`, `clipboard_delete_all`
   - Path actions: `move_to_trash`
   - Chat actions: `clear_conversation`

**Risks / Known Gaps**
- Repo-wide clippy is currently red from unrelated existing files.
- `src/clipboard_actions_tests.rs` is a source-audit test file under main-bin wiring; with current `Cargo.toml` bin test settings, these tests are not executed by the default harness.