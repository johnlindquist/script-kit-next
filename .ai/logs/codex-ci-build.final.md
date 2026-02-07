Created the requested report at `plans/build-ci-improvements.md` with prioritized, line-referenced recommendations for build speed, CI reliability, binary size, and dev iteration.

Changed files:
- `plans/build-ci-improvements.md` (new)

How to test:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Review `plans/build-ci-improvements.md` for the proposed rollout and commands.

Verification run results:
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due pre-existing workspace issues (not introduced by this report), including:
  - duplicate `mod tests` (`src/render_prompts/arg.rs:620` vs `src/main.rs:3846`)
  - missing `AiApp::message_body_content` (`src/ai/window.rs:8549`)
  - many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`
  - `clippy::needless_return` in `src/render_prompts/other.rs:313`
- `cargo test`: failed on the same pre-existing `AiApp::message_body_content` error and warnings.

Risks / known gaps:
- This task produced analysis/reporting only; recommendations are not yet implemented.
- Full lint/test baseline is currently red in the repository, which will block verification of future build/CI changes until fixed.

Commits made:
- None (I did not commit because the required verification gate is currently failing in the existing workspace).