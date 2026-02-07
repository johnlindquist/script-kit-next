Completed the animation/transition audit and added the report at `plans/animation-improvements.md`.

Changed files:
- `plans/animation-improvements.md`

Verification run:
- Executed: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result: failed due pre-existing unrelated issues in the working tree, including:
  - unused imports in multiple `src/actions/dialog_builtin_action_validation_tests_*.rs` files
  - missing symbol `AiApp::message_body_content` at `src/ai/window.rs:8549`
  - duplicate `mod tests` at `src/render_prompts/arg.rs:620` and `src/main.rs:3846`
  - clippy `needless_return` at `src/render_prompts/other.rs:313`

How to test:
1. Open/read `plans/animation-improvements.md`.
2. Re-run the verification gate once unrelated compile/lint issues are resolved:
   `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Risks / known gaps:
- This task delivered analysis/reporting only; no runtime animation code was changed.
- Full CI gate cannot pass until unrelated existing errors in other files are fixed.

Commits made:
- None.