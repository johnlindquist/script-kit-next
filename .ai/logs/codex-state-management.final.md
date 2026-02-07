Implemented deliverable: created the state-management audit report at `plans/state-management-improvements.md`.

**Changed Files**
- `plans/state-management-improvements.md`

**Verification Run**
- Ran: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result: failed due existing repo issues unrelated to this report-only change, including:
  - duplicate `tests` module (`src/render_prompts/arg.rs:620`, `src/main.rs:3846`)
  - many unused-import warnings promoted to errors in generated validation test files (multiple `src/actions/dialog_builtin_action_validation_tests_*.rs`)
  - missing `AiApp::message_body_content` (`src/ai/window.rs:8549`)
  - clippy `needless_return` (`src/render_prompts/other.rs:313`)

**How To Test**
1. Open and review `plans/state-management-improvements.md`.
2. Re-run verification gate after baseline repo errors are resolved:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Risks / Known Gaps**
- This task produced analysis/report output only; no runtime state-management refactor was applied yet.
- Full verification is currently blocked by pre-existing workspace compile/clippy/test failures.

**Commits**
- No commits were made.