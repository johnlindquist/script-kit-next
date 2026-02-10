Implemented the prompt-header floating input styling in the prompt header component.

**Changed files**
- `src/components/prompt_header/component.rs:57`
  - `render_input_area()` now wraps the input with:
    - `.pt(px(8.0)).px(px(8.0)).pb(px(6.0))`
  - Input field now uses:
    - `.rounded(px(10.0))`
    - `.border_1()`
    - `.border_color(colors.border.rgba8(0x33))`
  - Kept theme-token usage (`colors.border`) and no `border_b_1` strip styling in this input section.
- `src/components/prompt_header/tests.rs:168`
  - Added tests to assert:
    - required outer spacing tokens are present
    - floating field has rounded 10px + low-alpha theme border
    - legacy `.border_b_1()` is absent in `render_input_area`

**How to test**
1. Run:
   - `cargo check -p script-kit-gpui`
   - `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
   - `cargo test -p script-kit-gpui prompt_header::tests:: -- --nocapture`
2. After workspace issues are resolved, run your requested full gate:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Verification results / known gaps**
- All three verification commands were executed, but they failed due unrelated existing workspace errors outside this task (examples: `src/prompts/chat/render_core.rs`, `src/protocol/message/mod.rs`, multiple `src/actions/tests/*` modules).
- Because of those pre-existing failures, I could not complete a clean end-to-end cargo gate pass for this branch.

Commits made: none.