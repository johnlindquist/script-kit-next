Implemented the assignment by producing the scoped analysis report at `plans/header-footer-improvements.md`.

Changed files:
- `plans/header-footer-improvements.md`

Verification run:
- `cargo test --test prompt_footer --test footer_button` passed (4 tests total).
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test` failed due pre-existing unrelated repo issues (not from this change), including:
  - duplicate `mod tests` in `src/render_prompts/arg.rs` vs `src/main.rs`
  - missing `AiApp::message_body_content` in `src/ai/window.rs`
  - many unused-import `-D warnings` failures in `src/actions/dialog_builtin_action_validation_tests_*`
  - clippy `needless_return` in `src/render_prompts/other.rs`

How to test:
1. Open and review the report: `plans/header-footer-improvements.md`
2. Run relevant footer tests:
   - `cargo test --test prompt_footer --test footer_button`
3. (Optional) run full gate to see current baseline state:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Risks / known gaps:
- Full repo verification gate is currently red due unrelated baseline errors, so only targeted tests for this scope were verifiable.
- The change is documentation-only; no runtime header/footer behavior was modified in this task.

Commits made:
- `7cc7f36` `docs(header-footer): add prompt header/footer improvement audit`