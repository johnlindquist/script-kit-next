Implemented group-header text truncation and added a regression test.

**Changed Files**
- `src/app_render/group_header_item.rs:60`
  - Added `group_header_text_line_base()` that applies GPUI single-line truncation (`overflow_hidden()`, `text_ellipsis()`, `whitespace_nowrap()`).
  - Switched both group-header text lines (name + description) to use this helper in `render_group_header_item`.
- `src/app_render/tests.rs:101`
  - Added `test_group_header_text_line_base_applies_single_line_truncation` to assert the helper’s style matches `div().truncate()` semantics.

**Verification**
- Ran `cargo check`:
  - Passed.
  - Reported unrelated existing warnings in other modules.
- Ran `cargo clippy --all-targets -- -D warnings`:
  - Failed due unrelated pre-existing/in-progress issues outside this task scope (e.g. `src/prompts/*`, `src/watcher/*`).
- Ran scoped test:
  - `cargo test --bin script-kit-gpui preview_panel_metadata_tests::test_group_header_text_line_base_applies_single_line_truncation`
  - Passed (`1 passed`).

**How To Test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings` (currently expected to fail for unrelated files)
3. `cargo test --bin script-kit-gpui preview_panel_metadata_tests::test_group_header_text_line_base_applies_single_line_truncation`

**Risks / Known Gaps**
- Full clippy gate is currently blocked by unrelated concurrent changes in other files.
- No full UI screenshot validation was run in this task; behavior is validated via style-level unit test.

Commits made: none.