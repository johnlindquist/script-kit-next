Added missing top-of-file `//!` docs to the files that were actually missing them, without changing runtime behavior.

**Changed Files**
- `src/main.rs`
- `src/hotkeys.rs`
- `src/panel.rs`
- `src/watcher.rs` (also removed unused `setup` import)
- `src/render_prompts/arg.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/editor.rs`
- `src/render_prompts/form.rs`
- `src/render_prompts/other.rs`
- `src/render_prompts/path.rs`
- `src/render_prompts/term.rs`

Notes:
- `src/lib.rs`, `src/windows/mod.rs`, `src/designs/mod.rs`, and most other `mod.rs` files already had `//!` docs, so they were left unchanged.
- For `src/render_prompts/*.rs` (which are `include!()` fragments), direct file-level `//!` is not valid Rust in this context, so I added compile-safe top doc blocks via tiny top-level doc modules.

**How To Test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`

Result in this repo:
- `cargo check` passes.
- `cargo clippy --all-targets -- -D warnings` fails due many pre-existing lint violations outside this docs task (across `actions`, `prompts`, `components`, tests, etc.).

**Risks / Known Gaps**
- Strict clippy remains red due existing repo-wide lint debt unrelated to these documentation edits.
- `render_prompts` docs are implemented with doc-only helper modules because of `include!()` constraints.

**Commits**
- None.