Implemented the assignment by analyzing `src/render_prompts/arg.rs` and writing a ranked improvement report at `plans/arg-prompt-improvements.md`.

**Changed files**
- `plans/arg-prompt-improvements.md`

**What’s in the report**
- Text input UX gaps (caret visibility, mouse editing, duplicated submit logic)
- Validation feedback gaps (silent/no-op submit, ambiguous Enter behavior)
- Placeholder handling issues (overlay/negative-margin approach)
- Auto-completion gaps (no Tab completion, weak filtering/ranking)
- Input field behavior recommendations and phased roadmap
- Suggested TDD test names and known risks

**Verification run**
- `cargo check` ✅ passed
- `cargo clippy --all-targets -- -D warnings` ❌ failed due pre-existing repo issues, including:
  - duplicate `mod tests` conflict involving `src/render_prompts/arg.rs:620` and `src/main.rs:3846`
  - missing `AiApp::message_body_content` in `src/ai/window.rs:8549`
  - many existing unused-import failures in `src/actions/dialog_builtin_action_validation_tests_*.rs`
- `cargo test` ❌ failed due pre-existing `AiApp::message_body_content` error in `src/ai/window.rs:8549`

**How to test**
1. Open and review `plans/arg-prompt-improvements.md`.
2. Re-run the verification gate:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / known gaps**
- This change is documentation-only (no runtime behavior changed).
- Full clippy/test gate is currently blocked by pre-existing repository errors above.

**Commits**
- `d286766` — `docs(plans): add arg prompt improvement audit`