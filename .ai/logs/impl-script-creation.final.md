Changed files:
- `src/script_creation.rs`

What I changed (highest-priority fixes in-scope):
- Added collision-safe creation with auto-suffixing (`untitled.ts`, `untitled-1.ts`, etc.) in `create_new_script`/`create_new_extension` via shared helpers (`src/script_creation.rs:89`, `src/script_creation.rs:292`, `src/script_creation.rs:344`).
- Switched file writes to atomic create semantics using `OpenOptions::create_new(true)` to avoid TOCTOU overwrite races (`src/script_creation.rs:83`).
- Added filename validation for empty sanitized names, Windows reserved names, and max filename length (`src/script_creation.rs:54`).
- Updated editor launching to parse editor command + args safely (supports values like `code -r "path with spaces"`) before spawn (`src/script_creation.rs:388`, `src/script_creation.rs:443`).
- Added/updated tests for:
  - unique name generation
  - reserved/length validation
  - editor command parsing
  - atomic non-overwrite behavior
  - integration tests that call real creation helpers in temp dirs
  (`src/script_creation.rs:589` onward).

How to test:
1. `cargo check`  
2. `cargo clippy --all-targets -- -D warnings`  
3. `cargo test`

Verification results I got:
- `cargo check`: passed.
- `cargo clippy --lib -- -D warnings`: passed (in-scope code clean).
- `cargo clippy --all-targets -- -D warnings`: fails due unrelated existing repo issues (unused imports in multiple `src/actions/dialog_builtin_action_validation_tests_*.rs`, plus duplicate `mod tests` definitions in `src/render_prompts/arg.rs` and `src/render_prompts/form.rs` via `src/main.rs`).
- `cargo test`: fails due unrelated existing repo issue (`AiApp::message_body_content` missing in `src/ai/window.rs:8549`).

Risks / known gaps:
- Behavior change: duplicate names now auto-suffix instead of hard-failing.
- `parse_editor_command` is a lightweight shell-like parser; it handles quotes/escapes but is not a full shell parser.
- Full project verification is currently blocked by unrelated pre-existing errors above.

Commits:
- None made.