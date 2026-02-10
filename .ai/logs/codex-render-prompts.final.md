Implemented the `src/render_prompts/*` split for oversized files.

Changed files:
- `src/render_prompts/arg.rs` (now a thin include wrapper)
- `src/render_prompts/form.rs` (now a thin include wrapper)
- `src/render_prompts/arg/helpers.rs`
- `src/render_prompts/arg/render.rs`
- `src/render_prompts/arg/render_input.rs`
- `src/render_prompts/arg/render_prompt.rs`
- `src/render_prompts/arg/tests.rs`
- `src/render_prompts/form/helpers.rs`
- `src/render_prompts/form/render.rs`
- `src/render_prompts/form/tests.rs`

Result:
- All files in `src/render_prompts/` are now under 500 lines (`max = 454`, with `arg/render_prompt.rs = 419`).

How to test:
1. `rg --files src/render_prompts | sort | xargs wc -l`
2. `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Verification status:
- I ran the required cargo gate command.
- It is currently blocked by unrelated compile failures from other active agent work (examples: `src/terminal/alacritty.rs` missing module files, multiple `src/designs/*` split syntax/doc-comment errors, and an unclosed delimiter in `src/app_launcher/part_01.rs`).

Risks / known gaps:
- Full repo verification could not complete because of those external errors, so end-to-end validation is pending once the tree is buildable again.

Commits:
- None (I did not commit unverified work).