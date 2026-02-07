Completed the audit report at `plans/theme-consistency.md`.

**Changed Files**
- `plans/theme-consistency.md`

**How To Test**
1. Open `plans/theme-consistency.md` and review the prioritized findings, evidence, prompt-type consistency matrix, and remediation order.
2. Re-run the same verification gate I used:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Verification Results**
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due pre-existing unrelated repo issues (outside this task), including:
  - many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`
  - missing `AiApp::message_body_content` in `src/ai/window.rs`
  - duplicate `mod tests` in `src/render_prompts/arg.rs` and `src/main.rs`
  - `needless_return` in `src/render_prompts/other.rs`
- `cargo test`: not executed because the chained gate stopped at clippy failure.

**Risks / Known Gaps**
- This task was report-only; no runtime/theme fixes were applied yet.
- Line references in the report may shift as other parallel agents continue editing.
- Full lint/test gate is currently blocked by unrelated pre-existing errors.

**Commits**
- None.