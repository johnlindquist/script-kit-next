Changed files:
- `plans/code-pattern-consistency.md`

What I delivered:
- Wrote a full consistency audit for `src/**/*.rs` with impact-ordered findings, concrete file/line evidence, and a prioritized remediation plan.

How to test:
1. Open `plans/code-pattern-consistency.md` and verify sections:
   - Executive summary
   - Findings with source references
   - Suggested remediation order
   - Verification hooks + known gaps
2. Verification commands I ran:
   - `cargo check` (passed)
   - `cargo clippy --all-targets -- -D warnings` (failed due pre-existing repo issues, e.g. unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`, missing `AiApp::message_body_content` in `src/ai/window.rs:8549`, duplicate `mod tests` in `src/render_prompts/arg.rs:620` vs `src/main.rs:3846`)
   - `cargo test` (failed due pre-existing repo issue: missing `AiApp::message_body_content` in `src/ai/window.rs:8549`)

Risks / known gaps:
- This task intentionally produced a report only; no Rust behavior changes were implemented.
- Findings are snapshot-based and may need quick revalidation if other agents continue editing `src/**/*.rs`.

Commits made:
- None.