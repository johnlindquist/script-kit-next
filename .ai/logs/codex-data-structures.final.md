Implemented deliverable: created the data-structure audit report at `plans/data-structure-improvements.md`.

Changed files:
- `plans/data-structure-improvements.md`

How to test:
1. Open the report and verify it contains prioritized findings with file/line references:
   - `cat plans/data-structure-improvements.md`
2. Verification gate attempted:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Verification results:
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due pre-existing repo issues unrelated to this report (examples: duplicate `mod tests` in `src/render_prompts/arg.rs:620` vs `src/main.rs:3846`, missing `AiApp::message_body_content` in `src/ai/window.rs:8549`, multiple unused-import warnings in `src/actions/dialog_builtin_action_validation_tests_*.rs`).
- `cargo test`: not reached successfully because clippy/build failed first in the combined gate command.

Risks / known gaps:
- This change is analysis-only; no runtime/code-path behavior was modified.
- Recommendations in the report are not yet implemented, so no performance gains are realized yet.

Commits made:
- None.