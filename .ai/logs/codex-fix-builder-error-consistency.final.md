Implemented the consistency fix in the claimed builder scope:

- `src/actions/builders/chat.rs:21` added input validation helpers and early-return guards in `get_chat_context_actions` with `tracing::warn!(target = "script_kit::actions", ...)` for invalid/missing model data (`blank current model` or `missing model fields`), returning `Vec::new()`.
- `src/actions/builders/clipboard.rs:24` added early-return guard for invalid clipboard context (`blank entry id`) with consistent `warn` logging and `Vec::new()`.
- `src/actions/builders/file_path.rs:5` added shared field validator and early-return guards in both `get_file_context_actions` (`src/actions/builders/file_path.rs:16`) and `get_path_context_actions` (`src/actions/builders/file_path.rs:157`) with consistent `warn` logs and `Vec::new()`.

Added behavior tests:

- `src/actions/builders/chat.rs:270` and `src/actions/builders/chat.rs:280`
- `src/actions/builders/clipboard.rs:268`
- `src/actions/builders/file_path.rs:295` and `src/actions/builders/file_path.rs:305`

How to test:

1. `cargo test -p script-kit-gpui returns_empty_when`
2. `cargo check -p script-kit-gpui`
3. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

Verification results in current shared tree:

- `cargo check -p script-kit-gpui` fails due unrelated workspace breakages (e.g. non-exhaustive `ContentType` matches in `src/clipboard_history/clipboard.rs:53`, missing new fields in `ClipboardEntry` initializers in `src/clipboard_history/database.rs:454`).
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` also fails due unrelated pre-existing/in-flight issues (duplicate test module files and many unused-import warnings/errors outside this scope).

Risks / known gaps:

- I could not get a green compile/lint run because the workspace is currently broken by unrelated parallel changes, so new tests could not be executed to completion in this environment.
- No production `unwrap`/panic was introduced in the edited builders; the only `expect` in these files remains in an existing macOS-only test (`src/actions/builders/file_path.rs:146`).

Commits made:

- None.