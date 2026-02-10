Implemented the button-affordance audit fixes in scope by standardizing the clickable actions backdrops.

**Changed files**
- `src/render_prompts/arg/render.rs:484`
- `src/render_prompts/div.rs:226`
- `src/render_prompts/editor.rs:339`
- `src/render_prompts/form/render.rs:259`

In each file:
- Added `.cursor_pointer()` to the `*-actions-backdrop` clickable `div`.
- Added a regression test that asserts the backdrop click target keeps `cursor_pointer()`:
  - `src/render_prompts/arg/render.rs:505`
  - `src/render_prompts/div.rs:246`
  - `src/render_prompts/editor.rs:419`
  - `src/render_prompts/form/render.rs:279`

Audit result for hover/padding/radius in the requested render files:
- No additional direct button/action-item/link style blocks were found in these files beyond shared `PromptFooter` usage and backdrop click targets, so no extra hover/padding/radius token changes were needed here.

**Verification run**
- `cargo check -p script-kit-gpui` ✅
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` ✅
- `cargo test -p script-kit-gpui backdrop_uses_cursor_pointer_when_clickable` ✅ (0 matched in lib target)
- `cargo test -p script-kit-gpui --bin script-kit-gpui backdrop_uses_cursor_pointer_when_clickable` ❌ blocked by unrelated pre-existing test-module compile issues outside this scope (concurrent agent changes)

**Commits**
- `9325d37` `fix(render-prompts): standardize clickable action backdrops`

**Risks / known gaps**
- Bin-scoped test execution is currently blocked by unrelated concurrent test reorg/compile issues in other modules, so the new binary-target tests could not be executed end-to-end in this shared state.