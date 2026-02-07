Implemented the audit deliverable and committed it.

Changed files:
- `plans/memory-safety-audit.md`

How to test/verify:
1. Open `plans/memory-safety-audit.md` and review the severity-ranked findings/fixes.
2. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

Verification results on this run:
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due pre-existing unrelated workspace issues (examples: duplicate `tests` module in `src/render_prompts/arg.rs:620` vs `src/main.rs:3846`, missing `AiApp::message_body_content` in `src/ai/window.rs:8549`, multiple unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`).
- `cargo test`: blocked by the same compile errors.

Risks / known gaps:
- This is a report-only change (no runtime behavior changes).
- Full green verification is currently blocked by existing unrelated compile/clippy failures.
- Some findings are lifecycle/concurrency dependent and still benefit from dedicated macOS integration tests.

Commits made:
- `1b4dc04` `docs(memory-safety): add comprehensive unsafe/FFI audit report`