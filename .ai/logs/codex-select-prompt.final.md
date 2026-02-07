Implemented the assignment by adding the analysis report at `plans/select-prompt-improvements.md`.

Changed files:
1. `plans/select-prompt-improvements.md`

What I ran:
1. `cargo check` (passed)
2. `cargo test select::tests --lib` (failed due pre-existing unrelated error in `src/ai/window.rs:8549`: `AiApp::message_body_content` not found)

How to test:
1. Open `plans/select-prompt-improvements.md` and review the prioritized findings/roadmap.
2. Re-run:
```bash
cargo check
cargo test select::tests --lib
```
3. For the test command, first resolve the existing compile error in `src/ai/window.rs:8549`.

Risks / known gaps:
1. The report recommends grouping support that likely requires protocol/type changes outside `src/prompts/select.rs`.
2. Selection ordering semantics (toggle order vs source order) still need a product decision before implementation.
3. `cargo test` verification is currently blocked by unrelated existing test-compile failures in `src/ai/window.rs`.

Commits made:
1. None.