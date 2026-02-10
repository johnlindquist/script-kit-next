Implemented the prompt-header hint styling change.

Changed files:
- `src/components/prompt_header/component.rs:304`  
  - Updated `render_ask_ai_hint()` so both `"Ask AI"` and `"Tab"` hint chips use transparent backgrounds (`.bg(rgba(transparent_bg))`) and explicit hover cursor (`.cursor_pointer()`).
- `src/components/prompt_header/tests.rs:15`  
  - Added `test_render_ask_ai_hint_uses_transparent_backgrounds`.
- `src/components/prompt_header/tests.rs:33`  
  - Added `test_render_ask_ai_hint_uses_pointer_cursor_for_hint_buttons`.

Verification run:
- `cargo check --lib`
- `cargo clippy --lib -- -D warnings`
- `cargo test components::prompt_header::tests:: -- --nocapture`

How to test manually:
1. Launch the app path that renders `PromptHeader` with `show_ask_ai_hint = true`.
2. Confirm `"Ask AI"` and `"Tab"` show no background fill (ghost/transparent look).
3. Hover each chip and confirm cursor changes to pointer.

Risks / known gaps:
- New tests are source-structure regression tests (string-based), not pixel-level visual assertions.
- Full workspace gate was not run; verification was scoped to this change set.

Commit made:
- `3bcd2c9` `fix(prompt-header): make Ask AI and Tab hint ghost-style`