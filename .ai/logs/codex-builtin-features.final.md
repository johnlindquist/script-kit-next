Implemented deliverable: wrote the built-in feature analysis report at `plans/builtin-features.md`.

Changed files:
- `plans/builtin-features.md`

How to test:
1. Open `plans/builtin-features.md` and review the coverage/flow findings and prioritized recommendations.
2. Re-run verification:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

Verification results I got:
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due pre-existing issues outside this task (for example duplicate `tests` module in `src/render_prompts/arg.rs` vs `src/main.rs`, and many unused imports in `src/actions/dialog_builtin_action_validation_tests_*`).
- `cargo test`: failed due pre-existing `AiApp::message_body_content` missing in `src/ai/window.rs`.

Risks / known gaps:
- This was an audit/report task only; no runtime code fixes were applied.
- Existing branch-wide clippy/test failures remain and may block follow-up implementation work until resolved.

Commits:
- None.