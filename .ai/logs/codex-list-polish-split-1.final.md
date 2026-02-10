Changed files:
- `src/prompts/select/render.rs:27` added `resolve_row_bg_hex(...)` so focused OR selected rows use the accent background (`alpha 0x3A`), hovered-only rows use hover alpha.
- `src/prompts/select/render.rs:298` row background now uses that resolver; selected rows now get the same pill highlight as focused rows.
- `src/prompts/select/render.rs:345` hover override now skips both focused and selected rows so selected rows keep the stronger highlight.
- `src/prompts/select/render.rs:324` row wrapper remains `.rounded(px(8.0))` and has no per-row `.border_b_1()` calls.
- `src/prompts/select/render.rs:371` choices list container has `.px(px(8.0))`.
- `src/prompts/select/render.rs:385` replaced invalid `Rgba::opacity(...)` call with explicit RGBA alpha composition to keep this file compiling.
- `src/prompts/select/render.rs:395` added tests for focused/selected/hover background resolution.

How to test:
1. Run:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. If shared target locking is noisy in parallel runs, isolate target dir:
   - `CARGO_TARGET_DIR=target/codex-list-polish-split-1 cargo check -p script-kit-gpui`
   - `CARGO_TARGET_DIR=target/codex-list-polish-split-1 cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
   - `CARGO_TARGET_DIR=target/codex-list-polish-split-1 cargo test -p script-kit-gpui --lib prompts::select::render::tests::`

Risks / known gaps:
- Full verification is currently blocked by unrelated workspace errors outside this file (examples: `src/protocol/message/mod.rs`, `src/designs/core/part_04.rs`, `src/scripts/mod.rs`, many `src/actions/tests/*` files).
- Because of those unrelated failures, I could not produce a clean green run of the full gate in this shared tree.

Commits:
- None.