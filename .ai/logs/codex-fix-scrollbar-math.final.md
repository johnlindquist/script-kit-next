Implemented the scrollbar viewport fix by subtracting non-list chrome (search/header/footer) from the scrollbar geometry math, and added regression tests.

**Changed files**
- `src/actions/dialog/part_01.rs`
- `src/actions/dialog/part_04/body_part_02.rs`
- `src/actions/dialog_part_04_rewire.rs`
- `src/actions/dialog/part_05.rs`

**What changed**
- Added `actions_dialog_scrollbar_viewport_height(...)` in `src/actions/dialog/part_01.rs` to compute visible list viewport height as:
  - `POPUP_MAX_HEIGHT - search_height - header_height - footer_height`
  - clamped with `min(total_content_height, available_viewport)`
- Updated scrollbar math in:
  - `src/actions/dialog/part_04/body_part_02.rs`
  - `src/actions/dialog_part_04_rewire.rs`
- Scrollbar `visible_items` now clamps to `[1, grouped_items.len()]` to avoid invalid/underflow geometry.
- Added tests in `src/actions/dialog/part_05.rs`:
  - `test_scrollbar_viewport_subtracts_header_footer_and_search_height`
  - `test_scrollbar_viewport_clamps_to_content_when_content_shorter_than_viewport`

**Verification run**
- `cargo check` → failed due unrelated pre-existing workspace errors (e.g. `clipboard_history` non-exhaustive matches/missing fields).
- `cargo clippy --all-targets -- -D warnings` → failed due unrelated pre-existing workspace issues (duplicate test modules, many existing lint errors).
- `cargo test test_scrollbar_viewport_subtracts_header_footer_and_search_height -- --nocapture` → failed to compile due unrelated pre-existing test/module errors.

**How to test (once workspace baseline compiles)**
1. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
2. Run dialog tests:
   - `cargo test test_scrollbar_viewport_subtracts_header_footer_and_search_height`
   - `cargo test test_scrollbar_viewport_clamps_to_content_when_content_shorter_than_viewport`
3. Manual UI check:
   - Open Actions dialog with header and footer enabled, enough rows to scroll, and verify thumb size/position matches only the visible list area (not including header/footer/search rows).

**Risks / known gaps**
- Full verification is currently blocked by unrelated existing compile/lint failures outside this change.
- No commit was created in this run.