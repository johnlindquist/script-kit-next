Implemented preview-panel cleanup in `src/app_render.rs` to make the right-side detail view read like a summary card instead of a labeled record dump.

**Changed files**
- `src/app_render.rs`

**What changed**
- Removed uppercase section-label rendering for description blocks in script/scriptlet preview:
  - `src/app_render.rs:419`
  - `src/app_render.rs:626`
- Built-in preview no longer renders `DESCRIPTION` / `KEYWORDS` / `FEATURE TYPE` labeled sections.
  - Description now flows directly under title: `src/app_render.rs:854`
  - Keywords + feature type now render as subtle inline chips at the bottom: `src/app_render.rs:858`
- Added helpers for metadata rendering:
  - `src/app_render.rs:1618`
  - `src/app_render.rs:1638`
- Added tests for tag normalization/dedupe/limit and feature-label mapping:
  - `src/app_render.rs:2176`
  - `src/app_render.rs:2181`
  - `src/app_render.rs:2209`

**Verification run**
- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ blocked by pre-existing unrelated errors (`src/render_prompts/arg.rs`, `src/ai/window.rs`, multiple `src/actions/dialog_builtin_action_validation_tests_*.rs`)
- `cargo test --bin script-kit-gpui preview_panel_metadata_tests -- --nocapture` ❌ blocked by same pre-existing compile issues
- Stdin protocol launch check ✅:
  - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`
  - Preview render path logged (`[PREVIEW_START]`, `[PREVIEW_PANEL_DONE]`)

**How to test**
1. `cargo build`
2. `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`
3. Select items and confirm preview hierarchy is: large title → flowing description → subtle tags (no rigid uppercase field headers).

**Risks / known gaps**
- Keyword chips are normalized to lowercase and capped at 6 tags; if exact capitalization is desired, adjust helper behavior.
- Full clippy/test gate is currently blocked by unrelated workspace errors.

**Commits**
- `c16be9e` `refactor(preview): simplify right-panel metadata layout`