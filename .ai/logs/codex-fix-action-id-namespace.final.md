Implemented the namespace-ID fix for your assigned scope and updated handlers so the new IDs resolve correctly.

Changed files:
- `src/actions/builders/file_path.rs:25`  
  All file/path builder `Action::new` IDs now use `file:` prefix; added namespace tests at `src/actions/builders/file_path.rs:244` and `src/actions/builders/file_path.rs:257`.
- `src/actions/builders/clipboard.rs:41`  
  All clipboard builder IDs now use `clip:` prefix; added namespace test at `src/actions/builders/clipboard.rs:244`.
- `src/actions/builders/chat.rs:33`  
  All chat builder IDs now use `chat:` prefix (including dynamic model IDs); added namespace tests at `src/actions/builders/chat.rs:220` and `src/actions/builders/chat.rs:226`.
- `src/app_actions/handle_action.rs:241`  
  Added action-ID normalization for `clip:`, `file:`, `chat:` before existing match logic.
- `src/app_impl/execution_paths.rs:19`  
  Added `file:` prefix normalization.
- `src/app_impl/chat_actions.rs:7`  
  Added `chat:` prefix normalization.
- `src/ai/window/command_bar.rs:238`  
  Added `chat:` prefix normalization for command bar actions.

Verification run:
- `cargo check` -> failed due unrelated pre-existing workspace issues (not from these files), e.g. missing `ClipboardEntry` fields in `src/clipboard_history/database.rs` and non-exhaustive `ContentType` matches in clipboard modules.
- `cargo clippy --all-targets -- -D warnings` -> failed due many unrelated pre-existing issues (duplicate test modules, unused imports, same clipboard compile errors).

How to test:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. Manual runtime spot-check:
   - Open file actions, clipboard history actions, and chat actions dialogs.
   - Confirm selected actions still execute (open/copy/paste/chat actions) with namespaced IDs.

Risks / known gaps:
- Workspace has substantial unrelated compile/lint failures, so full gate is currently blocked outside this change.
- Because handlers normalize by prefix, any custom SDK action IDs starting with `file:`, `clip:`, or `chat:` may be interpreted as built-in namespace IDs.

Commits:
- None (not committed due failing required verification gate in current workspace state).