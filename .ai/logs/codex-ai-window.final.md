Split is implemented: `src/ai/window.rs` is now a thin module root, and the old 8.7k-line body is extracted into `src/ai/window/*.rs` modules (all under 500 lines).

Changed files:
- `src/ai/window.rs`
- `src/ai/window/chat.rs`
- `src/ai/window/command_bar.rs`
- `src/ai/window/dropdowns.rs`
- `src/ai/window/images.rs`
- `src/ai/window/init.rs`
- `src/ai/window/interactions.rs`
- `src/ai/window/platform.rs`
- `src/ai/window/render_input.rs`
- `src/ai/window/render_keydown.rs`
- `src/ai/window/render_main_panel.rs`
- `src/ai/window/render_message.rs`
- `src/ai/window/render_message_actions.rs`
- `src/ai/window/render_messages.rs`
- `src/ai/window/render_overlays_attachments.rs`
- `src/ai/window/render_overlays_dropdowns.rs`
- `src/ai/window/render_overlays_shortcuts.rs`
- `src/ai/window/render_root.rs`
- `src/ai/window/render_setup.rs`
- `src/ai/window/render_sidebar.rs`
- `src/ai/window/render_sidebar_items.rs`
- `src/ai/window/render_streaming.rs`
- `src/ai/window/render_welcome.rs`
- `src/ai/window/search.rs`
- `src/ai/window/setup.rs`
- `src/ai/window/state.rs`
- `src/ai/window/streaming_control.rs`
- `src/ai/window/streaming_submit.rs`
- `src/ai/window/tests.rs`
- `src/ai/window/theme_helpers.rs`
- `src/ai/window/traits.rs`
- `src/ai/window/types.rs`
- `src/ai/window/window_api.rs`

How to test:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

Verification results:
- Ran all three commands; full workspace gate is currently blocked by unrelated in-progress files outside `src/ai/window` (notes/prompts/other refactors).
- I also ran scope checks and got no `src/ai/window` diagnostics:
  - `cargo check --message-format=short | rg "src/ai/window"`
  - `cargo clippy --all-targets -- -D warnings | rg "src/ai/window"`
  - `cargo test | rg "src/ai/window"`

Risks / known gaps:
- Full end-to-end gate is not green yet because concurrent refactors in other modules are failing build/lint/test.
- Commit `ddd673f` also includes pre-staged concurrent `src/main*` split files from another agent in this shared tree.

Commits made:
- `ddd673f`