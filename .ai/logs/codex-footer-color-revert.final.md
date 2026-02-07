**Root Cause Identified**
`git blame` shows an **uncommitted** change in `src/render_prompts/arg.rs` introduced `prompt_footer_colors_for_prompt(...)` and set footer background to `design_colors.background_selected` (`Not Committed Yet` line). In light designs where `background_selected` is dark/black, footer turns black.

**Fix Applied**
- `src/render_prompts/arg.rs:26`
  - Changed footer surface token from `background_selected` to `background_secondary`.
- `src/render_prompts/arg.rs:773`
  - Added regression test `prompt_footer_colors_use_secondary_background_for_surface` to lock this behavior.

**Changed Files**
- `src/render_prompts/arg.rs`

**Verification Run**
- `cargo check` ✅ passes (with existing warnings outside this change)
- `cargo clippy --all-targets -- -D warnings` ❌ fails due pre-existing repo-wide lint errors (examples: `src/actions/dialog.rs:308`, multiple `src/actions/*_tests.rs`, `src/prompts/div.rs:537`)
- `cargo test` ❌ fails due pre-existing compile/test issues (examples: missing markdown helper symbols in `src/prompts/markdown.rs:1668+`)

**How To Test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. UI smoke (stdin protocol):  
   `printf '{"type":"show"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Risks / Known Gaps**
- Full green verification is blocked by unrelated pre-existing failures in the current shared tree.
- The new regression test is added but currently cannot be executed to completion until those unrelated failures are fixed.

**Commits**
- None made.