Changed file:
- `src/prompts/markdown.rs`

What I implemented (highest-priority fixes from `plans/markdown-rendering.md`):
- Nested list parsing/rendering with stack-based structure and recursive rendering.
  - Parser stack/state: `src/prompts/markdown.rs:145`
  - Recursive list renderer: `src/prompts/markdown.rs:483`
- Markdown image tag handling (`Tag::Image`/`TagEnd::Image`) with preserved alt text + URL as clickable inline image label.
  - Image state: `src/prompts/markdown.rs:65`
- Hard break preservation (`HardBreak` becomes `\n`, preserved in inline rendering).
  - Parse handling: `src/prompts/markdown.rs:374`
  - Inline rendering line-break preservation: `src/prompts/markdown.rs:1045`
- Footnote option alignment (removed `ENABLE_FOOTNOTES` where renderer has no support).
  - Parser options: `src/prompts/markdown.rs:134`
  - Test parser options: `src/prompts/markdown.rs:1168`
- Link scheme allowlist hardening before `open::that(...)`.
  - Allowlist: `src/prompts/markdown.rs:865`
  - Click gating in styled link rendering: `src/prompts/markdown.rs:963`
- Added tests for new behavior:
  - `nested_lists_preserve_parent_child_structure`: `src/prompts/markdown.rs:1461`
  - `hard_break_preserves_line_break`: `src/prompts/markdown.rs:1621`
  - `markdown_image_preserves_alt_text_and_url`: `src/prompts/markdown.rs:1631`
  - `markdown_link_url_allowlist_rejects_unsafe_schemes`: `src/prompts/markdown.rs:1651`

Verification run:
- `CARGO_BUILD_JOBS=1 cargo check` ✅ pass
- `CARGO_BUILD_JOBS=1 cargo clippy --all-targets -- -D warnings` ❌ fail due pre-existing repo errors outside scope (e.g. unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`, missing `AiApp::message_body_content` in `src/ai/window.rs`)
- `CARGO_BUILD_JOBS=1 cargo test` ❌ fail due same unrelated repo compile error (`src/ai/window.rs`)

How to test:
1. Run `cargo check`
2. Run `cargo clippy --all-targets -- -D warnings`
3. Run `cargo test`
4. After repo-wide compile blockers are fixed, verify markdown tests by running:
   - `cargo test nested_lists_preserve_parent_child_structure`
   - `cargo test hard_break_preserves_line_break`
   - `cargo test markdown_image_preserves_alt_text_and_url`
   - `cargo test markdown_link_url_allowlist_rejects_unsafe_schemes`

Risks / known gaps:
- Full `clippy` and `cargo test` verification is currently blocked by unrelated existing errors outside `src/prompts/markdown.rs`.
- Image markdown is now preserved as a clickable `[Image: ...]` label rather than rendering an actual inline bitmap element.

Commits:
- None (no commit made because required repo-wide verification is currently failing outside this scope).