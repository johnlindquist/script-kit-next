## Findings (Preview Panel Audit)

1. **High (fixed): Preview panel content could not scroll**
- Cause: the panel root used `overflow_y_hidden`, so long metadata/code previews were clipped.
- Fix: switched to `overflow_y_scrollbar` in `src/app_render/preview_panel.rs:245`.

2. **High (fixed): Unicode edge case could panic during scriptlet preview truncation**
- Cause: truncation used byte slicing (`&line[..MAX_LINE_LENGTH]`), which can panic on non-ASCII boundaries.
- Fix: added Unicode-safe truncation helper `truncate_preview_line_for_display` in `src/app_render/preview_panel.rs:9` and used it in scriptlet preview generation at `src/app_render/preview_panel.rs:757`.

3. **Medium (fixed): Scriptlet preview cache could return stale/wrong highlighted content**
- Cause: cache key was only `scriptlet.name`, so same-name scriptlets, theme toggles, or content changes could collide.
- Fix: added `preview_scriptlet_cache_key` in `src/app_render/preview_panel.rs:24` and used it at `src/app_render/preview_panel.rs:740` with source/command/tool/theme/code-hash components.

## Changed Files

- `src/app_render/preview_panel.rs`
- `src/app_render/tests.rs`

## How To Test

- `cargo test --bin script-kit-gpui preview_panel_metadata_tests -- --nocapture`
- `cargo check --bin script-kit-gpui`

Also attempted:
- `cargo clippy --bin script-kit-gpui --tests -- -D warnings`  
  This currently fails outside this task’s scope in `src/prompts/div/render_html.rs:54`, `src/prompts/div/render_html.rs:64`, `src/prompts/div/render_html.rs:203`.

## Risks / Known Gaps

- I did not change script preview cache behavior in `src/app_impl/filtering_cache.rs`; theme-aware invalidation for **script** previews is still a potential follow-up outside assigned scope.
- No additional `src/render_prompts/arg/**/*.rs` code changes were needed from this audit pass.

## Commits

- `5f688e95b539ba8b6919af955adbe1a4e5ed4b96`