Created the markdown audit report at `plans/markdown-rendering.md`.

Changed files:
- `plans/markdown-rendering.md`

How to test:
1. Open `plans/markdown-rendering.md` and verify it contains severity-ranked findings, evidence, repro snippets, and recommendations.
2. Run the targeted test command I used:
   - `cargo test prompts::markdown::tests -- --nocapture`
3. Current result: this command is blocked by a pre-existing compile error outside this task:
   - `src/ai/window.rs:8549` missing `AiApp::message_body_content` (`E0599`).

Risks / known gaps:
- I only added the analysis report; no renderer code fixes were implemented in this pass.
- Full Rust test verification is currently blocked by the unrelated `src/ai/window.rs` compile failure.

Commits made:
- None.