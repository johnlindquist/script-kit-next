**Changed Files**
- `plans/editor-prompt-improvements.md`

**What I delivered**
- Wrote a full editor prompt audit report focused on `src/render_prompts/editor.rs`.
- Included ranked findings and recommendations for:
  - text editing UX
  - cursor handling
  - selection behavior
  - undo/redo
  - syntax highlighting
  - editor keybindings
- Added phased implementation plan and proposed TDD test cases.

**Verification Run**
- `cargo check` passed.
- `cargo clippy --all-targets -- -D warnings` failed due pre-existing issues outside this report change, including:
  - duplicate `mod tests` (`src/render_prompts/arg.rs:620` and `src/main.rs:3846`)
  - missing `AiApp::message_body_content` (`src/ai/window.rs:8549`)
  - multiple unused imports in existing `src/actions/dialog_builtin_action_validation_tests_*.rs` files
- `cargo test` failed on the same existing compile issue at `src/ai/window.rs:8549`.

**How To Test**
1. Inspect the report:
   - `plans/editor-prompt-improvements.md`
2. Re-run validation commands:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / Known Gaps**
- This task is analysis/reporting only; recommendations are not yet implemented in code.
- Repo-wide verification is currently blocked by unrelated pre-existing compile/clippy issues.
- Referenced line numbers may move as other agents continue editing.

**Commits**
- None.