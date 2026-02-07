Changed files:
- `plans/terminal-prompt-improvements.md` (new report with prioritized findings and recommendations)

What I delivered:
- Wrote a full audit focused on `src/render_prompts/term.rs` and its terminal stack dependencies (`src/term_prompt.rs`, `src/terminal/alacritty.rs`), covering:
  - terminal emulation
  - ANSI escape/style fidelity
  - scrollback behavior
  - large-output performance
  - terminal UX

How to test / verify:
1. Open and review `plans/terminal-prompt-improvements.md`.
2. Run the validation gate:
   - `cargo check` (passed)
   - `cargo clippy --all-targets -- -D warnings` (failed due pre-existing workspace issues)
   - `cargo test` (failed due pre-existing workspace issues)

Verification blockers observed (pre-existing, unrelated to this doc change):
- `E0428`: duplicate `mod tests` (`src/render_prompts/arg.rs:620` and `src/main.rs:3846`)
- `E0599`: missing `AiApp::message_body_content` (`src/ai/window.rs:8549`)
- Multiple unused-import/clippy warnings in `src/actions/dialog_builtin_action_validation_tests_*.rs`
- Clippy `needless_return` in `src/render_prompts/other.rs:313`

Risks / known gaps:
- This task produced analysis/report only; no implementation changes were made to terminal code.
- Because the tree is actively modified by parallel agents, some referenced issues may shift as upstream files change.

Commits made:
- None.