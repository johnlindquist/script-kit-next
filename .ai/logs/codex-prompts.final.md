**Status**
`src/prompts/*` oversized files were split into submodules under 500 lines.  
All prompt `.rs` files are now <= 500 lines.

**Changed Files**
Deleted:
- `src/prompts/chat.rs`
- `src/prompts/div.rs`
- `src/prompts/env.rs`
- `src/prompts/markdown.rs`
- `src/prompts/path.rs`
- `src/prompts/select.rs`
- `src/prompts/template.rs`

Added:
- `src/prompts/chat/mod.rs`
- `src/prompts/chat/actions.rs`
- `src/prompts/chat/prompt.rs`
- `src/prompts/chat/render_core.rs`
- `src/prompts/chat/render_input.rs`
- `src/prompts/chat/render_setup.rs`
- `src/prompts/chat/render_turns.rs`
- `src/prompts/chat/state.rs`
- `src/prompts/chat/streaming.rs`
- `src/prompts/chat/tests.rs`
- `src/prompts/chat/types.rs`
- `src/prompts/div/mod.rs`
- `src/prompts/div/inline.rs`
- `src/prompts/div/prompt.rs`
- `src/prompts/div/render.rs`
- `src/prompts/div/render_html.rs`
- `src/prompts/div/tailwind.rs`
- `src/prompts/div/tests.rs`
- `src/prompts/div/types.rs`
- `src/prompts/env/mod.rs`
- `src/prompts/env/helpers.rs`
- `src/prompts/env/prompt.rs`
- `src/prompts/env/render.rs`
- `src/prompts/env/tests.rs`
- `src/prompts/markdown/mod.rs`
- `src/prompts/markdown/api.rs`
- `src/prompts/markdown/code_table.rs`
- `src/prompts/markdown/helpers.rs`
- `src/prompts/markdown/inline_render.rs`
- `src/prompts/markdown/parse.rs`
- `src/prompts/markdown/render_blocks.rs`
- `src/prompts/markdown/scope.rs`
- `src/prompts/markdown/test_support.rs`
- `src/prompts/markdown/tests.rs`
- `src/prompts/markdown/types.rs`
- `src/prompts/path/mod.rs`
- `src/prompts/path/prompt.rs`
- `src/prompts/path/render.rs`
- `src/prompts/path/types.rs`
- `src/prompts/select/mod.rs`
- `src/prompts/select/prompt.rs`
- `src/prompts/select/render.rs`
- `src/prompts/select/search.rs`
- `src/prompts/select/tests.rs`
- `src/prompts/select/types.rs`
- `src/prompts/template/mod.rs`
- `src/prompts/template/prompt.rs`
- `src/prompts/template/render.rs`
- `src/prompts/template/tests.rs`
- `src/prompts/template/types.rs`

Note:
- `src/prompts/mod.rs` did not need a semantic change because module names stayed the same; each split module now has its own `mod.rs`.

**Verification Run**
Ran:
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Result:
- Blocked before clippy/tests by unrelated concurrent-agent errors outside `src/prompts/*`, including:
  - `src/protocol/io/parsing.rs:1` (`E0753` inner doc comments)
  - multiple `src/stories/*/split/part_*.rs` doc-comment/item errors
  - `src/stories/path_prompt_stories/split/part_01.rs:418` (unclosed delimiter)

Also ran:
- `find src/prompts -name '*.rs' -exec wc -l {} + | grep -v ' total$' | awk '$1 > 500 {print}'`
- Output was empty (no prompt file above 500 lines).

**How To Test**
1. Validate size constraint:
   - `find src/prompts -name '*.rs' -exec wc -l {} + | grep -v ' total$' | awk '$1 > 500 {print}'`
2. Run full gate (after unrelated tree errors are fixed):
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Risks / Known Gaps**
- Full repository verification is currently blocked by external files being actively refactored by other agents.
- Because `cargo check` fails globally first, prompt tests/clippy could not be completed in this shared tree state.

**Commits**
- No commits were made (verification gate is not green yet).