Created the requested button audit report at `plans/button-improvements.md`.

Changed files:
- `plans/button-improvements.md`

How to test:
1. `cargo check` (passes)
2. `cargo test --test footer_button --test prompt_footer` (passes: 4 tests total)
3. `cargo clippy --all-targets -- -D warnings` (fails due pre-existing unrelated repo issues, not from this report-only change)

Notable `clippy` blockers observed:
- Duplicate `mod tests` conflict in `src/render_prompts/arg.rs:620` and `src/main.rs:3846`
- Missing symbol in AI tests: `src/ai/window.rs:8549` (`AiApp::message_body_content`)
- Multiple pre-existing unused-import errors in `src/actions/dialog_builtin_action_validation_tests_*.rs`

Risks / known gaps:
- This task produced analysis/report only; no button implementation changes were applied yet.
- Recommendations in `plans/button-improvements.md` are not yet validated with runtime UI screenshots/log flows.

Commits:
- None.