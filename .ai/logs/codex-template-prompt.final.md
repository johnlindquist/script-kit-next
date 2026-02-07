Created the template prompt audit report at `plans/template-prompt-improvements.md`.

Changed files:
- `plans/template-prompt-improvements.md`

How to test / verify:
1. Open `plans/template-prompt-improvements.md` and confirm it covers rendering, variable substitution, layout flexibility, and reusability with ranked findings and file/line evidence.
2. Verification commands I ran:
   - `cargo test template::tests -- --nocapture` (failed due pre-existing unrelated compile error in `src/ai/window.rs:8549`: missing `AiApp::message_body_content`)
   - `cargo check --lib` (passed)

Risks / known gaps:
- I only added the analysis report; no runtime code changes were applied.
- Full test verification is currently blocked by the existing `src/ai/window.rs` test-compile error above.

Commits made:
- None.