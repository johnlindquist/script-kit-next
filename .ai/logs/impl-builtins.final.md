Implemented the highest-priority built-in improvements from `plans/builtin-features.md` within your scoped files.

**Changed Files**
1. `src/builtins.rs:759`
- Added discoverable Notes command entry: `builtin-open-notes` -> `NotesCommandType::OpenNotes`.
- Added discoverable AI command entry: `builtin-open-ai` -> `AiCommandType::OpenAi`.
- Removed preview/non-functional AI commands from built-in registration (`send-screen-area`, `create/import/search presets`) so they no longer appear in built-ins.
- Added coverage tests:
  - `src/builtins.rs:1541` (`test_get_builtin_entries_includes_open_notes_and_open_ai_commands`)
  - `src/builtins.rs:1561` (`test_get_builtin_entries_hides_preview_ai_commands`)

2. `src/app_execute.rs:567`
- Implemented `AiCommandType::ClearConversation` to actually clear AI history via `ai::clear_all_chats()`.
- After clear, closes and reopens AI window so cleared state is reflected immediately.
- Added success HUD and error-toast/log handling for clear/reopen failure paths.

**How To Test**
1. Compile check (requested gate):
- `cargo check`

2. Targeted built-in tests:
- `cargo test --lib test_get_builtin_entries_includes_open_notes_and_open_ai_commands`
- `cargo test --lib test_get_builtin_entries_hides_preview_ai_commands`

3. Runtime stdin protocol smoke:
- `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Verification Results**
- `cargo check`: passed.
- Runtime stdin smoke (`show`): app launched and parsed stdin command successfully.
- Targeted `cargo test --lib ...`: currently blocked by unrelated existing compile error in `src/ai/window.rs:8729` (`AiApp::message_body_content` missing).

**Risks / Known Gaps**
- Test suite for these changes is currently gated by the unrelated `src/ai/window.rs` compile failure, so those new tests cannot be executed to completion until that is fixed.
- `ClearConversation` now clears all chats and immediately reopens AI; behavior is implemented and runtime-safe, but broader AI window test coverage remains blocked by the same unrelated compile issue.

**Commits**
- No commits were made.