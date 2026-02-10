Changed files: none.

Audit result for scoped builders:
- `src/actions/builders/chat.rs`
- `src/actions/builders/clipboard.rs`
- `src/actions/builders/file_path.rs`
- `src/actions/builders/notes.rs`
- `src/actions/builders/script_context.rs`
- `src/actions/builders/scriptlet.rs`

All `open/edit`, `copy`, and `delete/remove` actions are using `ActionCategory::ScriptContext` consistently, and there are no non-`ScriptContext` categories in these files.

How to test:
1. Category audit:
   - `rg -nP "ActionCategory::(?!ScriptContext)" src/actions/builders/chat.rs src/actions/builders/clipboard.rs src/actions/builders/file_path.rs src/actions/builders/notes.rs src/actions/builders/script_context.rs src/actions/builders/scriptlet.rs`
   - Expected: no output
2. Build/lint run attempted:
   - `cargo check -p script-kit-gpui`
   - `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

Risks / known gaps:
- `cargo check` and `cargo clippy` currently fail due unrelated in-progress workspace issues outside this task scope (not category-related), including:
  - missing `ClipboardEntry` fields in `src/clipboard_history/*`
  - non-exhaustive `ContentType` matches in `src/clipboard_history/*`
  - duplicate module files and broad unrelated test/import errors.
- No category code change was required, so no commit was created.