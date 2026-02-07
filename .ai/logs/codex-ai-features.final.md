Implemented deliverable: created `plans/ai-features-improvements.md` with a severity-ranked AI/chat audit covering UX, streaming performance, error recovery, conversation management, and integration patterns, plus a phased remediation plan and required tests.

Changed files:
- `plans/ai-features-improvements.md`

Verification run:
- Command: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result:
  - `cargo check` passed.
  - `cargo clippy` failed due pre-existing repo issues outside this task, including:
    - duplicate `mod tests` definitions (`src/render_prompts/arg.rs:620`, `src/main.rs:3846`)
    - missing symbol `AiApp::message_body_content` (`src/ai/window.rs:8549`)
    - many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`
    - clippy `needless_return` (`src/render_prompts/other.rs:313`)
  - `cargo test` did not complete because compile/lint failed first.

How to test:
1. Review the report content: `plans/ai-features-improvements.md`
2. After fixing existing repository compile/lint failures, rerun:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Risks / known gaps:
- This task produced an analysis/report only; no runtime behavior was changed.
- Full verification is currently blocked by pre-existing repository errors listed above.
- The report flags high-impact issues (retry/stop semantics, streaming status API gaps, multimodal persistence) that still require implementation.

Commits made:
- None.