All 3 swarm tasks completed successfully. Here's the summary:

### Swarm Results

| Worker | Status | Changes |
|--------|--------|---------|
| `extract-has-sdk-actions-helper` | **done** | Added `has_nonempty_sdk_actions()` helper in `other.rs`, replaced 4 copy-pasted `unwrap()` patterns in `div.rs`, `editor.rs`, `term.rs`, `form/render.rs` |
| `unify-simple-prompt-wrappers` | **done** (reported `needs_split` but work is complete) | Added `render_simple_prompt_shell()` helper in `other.rs`, collapsed 6 identical render methods into one-liners, updated source-string-matching tests in `prompt_layout_shell.rs` |
| `rm-dead-box-shadow-calls` | **done** (reported `needs_split` but work is complete) | Removed 7 dead `create_box_shadows()` calls and 2 dead `_bg_with_alpha` computations across `div.rs`, `editor.rs`, `term.rs`, `path.rs`, `form/render.rs`, `arg/render.rs`, `arg/render_prompt.rs` |

### Verification
- `cargo check` — passed
- `cargo clippy` (non-test) — passed  
- `cargo clippy --all-targets` / `cargo test` — pre-existing compilation errors in test targets (unrelated to our changes: clipboard_history type mismatches, etc.)

### Files Changed (12 files, +481 -113)
- `src/render_prompts/other.rs` — unified prompt wrappers + added helpers
- `src/render_prompts/div.rs`, `editor.rs`, `term.rs`, `form/render.rs` — sdk_actions helper + box shadow removal
- `src/render_prompts/path.rs` — dead code removal
- `src/render_prompts/arg/render.rs`, `arg/render_prompt.rs` — dead code removal
- `src/components/prompt_layout_shell.rs` — test updates

Changes are uncommitted and ready for your review.
