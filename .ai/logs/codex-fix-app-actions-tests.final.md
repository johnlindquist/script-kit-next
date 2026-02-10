Implemented the module-wiring fix for `app_actions` tests and resolved the duplicate test-module naming conflict.

**Changed files**
- `src/app_actions/tests.rs` (new): switched to explicit paths for moved test files:
  - `#[path = "tests/actions_button_visibility.rs"]`
  - `#[path = "tests/clipboard_actions.rs"]`
  - `#[path = "tests/clipboard_actions_menu_coverage.rs"]`
- `src/app_actions.rs:8`: renamed included test module to avoid duplicate `tests` module collisions in `main` include context:
  - `mod app_actions_file_tests;`

**What this fixes**
- Prevents Rust from looking for the moved files in the wrong place (`src/app_actions/*.rs`).
- Avoids duplicate test module naming conflicts from `include!("app_actions.rs")` composition.

**Verification run**
- Ran `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`.
- In the captured diagnostics, the original missing-module/duplicate-tests wiring errors for this scope were no longer present; remaining failures were unrelated concurrent issues (mainly doc-comment style errors in other files).
- Attempted isolated-target verification too, but it hit environment disk limits (`ENOSPC`).

**How to test**
1. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
2. Optional focused check:
   - `cargo check -p script-kit-gpui --all-targets 2>&1 | rg "E0583|E0428|app_actions/tests.rs|app_actions.rs"`

**Risks / known gaps**
- Full clippy green is currently blocked by unrelated errors being handled by other agents (not in this scope).
- Workspace had build-lock contention and low-disk episodes during verification.
- No commits were made.