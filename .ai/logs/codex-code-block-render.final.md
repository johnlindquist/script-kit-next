Implemented the code-block rendering fix on the actual chat turn renderer (`render_turns.rs`; `render_messages.rs` does not exist in this checkout).

Changed files:
- `src/prompts/chat/types.rs:147`
  - Added `assistant_response_markdown_source(...)` to wrap raw script-generation responses in a fenced `typescript` block when no ``` fence exists.
- `src/prompts/chat/render_turns.rs:111`
  - Assistant responses now pass through `assistant_response_markdown_source(...)` before `render_markdown(...)` (streaming and non-streaming paths).
- `src/prompts/chat/tests.rs:217`
  - Added tests for:
    - wrapping plain script text in script-generation mode
    - preserving already fenced responses
    - leaving plain text unchanged outside script-generation mode

Verification run:
- `cargo check -p script-kit-gpui --lib` passed.
- `cargo check -p script-kit-gpui` currently fails due concurrent/unrelated work in `src/prompts/chat/render_core.rs` (argument mismatch in `render_script_generation_footer_button` call).
- `cargo test -p script-kit-gpui ...` is currently blocked by unrelated existing test-module compile issues in other areas of the repo.

How to test:
1. Run `cargo check -p script-kit-gpui --lib`.
2. Open script-generation chat mode and generate a script response without markdown fences.
3. Confirm the assistant response renders as a code block (monospace, code-block background/padding) instead of regular chat text.

Risks / known gaps:
- In script-generation mode, any non-empty unfenced assistant response is wrapped as code; if a model returns prose + code, prose will also be fenced.
- Full workspace `cargo check`/`cargo test` could not be used as a clean signal due unrelated in-progress failures outside this change set.