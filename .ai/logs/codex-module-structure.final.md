Completed the module-structure audit and wrote the report to `plans/module-structure-improvements.md`.

**Changed files**
- `plans/module-structure-improvements.md`

**How to test**
1. Read the deliverable: `plans/module-structure-improvements.md`
2. Re-run validation commands:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Verification run results**
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due existing repo issues, including:
  - duplicate `mod tests` (`src/render_prompts/arg.rs:620` and `src/main.rs:3846`)
  - missing associated item `AiApp::message_body_content` (`src/ai/window.rs:8549`)
  - multiple unused-import errors in generated action test modules (e.g. `src/actions/dialog_builtin_action_validation_tests_7.rs:51`)
- `cargo test`: failed on the same existing `AiApp::message_body_content` error in `src/ai/window.rs:8549` (plus warnings).

**Risks / known gaps**
- This task produced an analysis/report only; no architectural refactor was implemented yet.
- Cycle detection in the report is based on current `use crate::...` graph and includes some test/module-hub effects; implementation planning should validate each cycle during extraction.

**Commits**
- None.