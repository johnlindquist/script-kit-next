**Changed Files**
- `src/actions/dialog/part_04/body_part_01.rs`
- `src/actions/dialog/part_04/body_part_02.rs`
- `src/actions/dialog/part_04/body_part_03.rs`

**What I Changed**
- Removed per-file block scoping so the three body parts now share one render-state flow across includes.
- Unified separator handling to use the same `separator_color` path across search/header/footer/section separators.
- Standardized action-row spacing by introducing shared `action_row_vertical_padding` and applying the same inset/padding pattern to empty-state rows.
- Consolidated layout math:
  - computed `show_search`, `has_header`, `show_footer`, header/footer/search heights, grouped counts, and `total_content_height` once in `body_part_02.rs`
  - reused those values in `body_part_03.rs` for final popup height computation
- Standardized theme token usage for header/footer text/borders using shared `dimmed_text` + `separator_color`.
- Switched footer height usage to `ACTIONS_DIALOG_FOOTER_HEIGHT` for consistency.

**Verification Run**
- Ran: `cargo check`
- Ran: `cargo clippy --all-targets -- -D warnings`
- Both commands failed due unrelated workspace issues (outside these files), including:
  - `clipboard_history` missing struct fields and non-exhaustive matches
  - duplicate test module definitions
  - broad pre-existing unused-import / test compile failures

**How To Test**
1. Re-run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
2. After workspace-wide failures are fixed, open the actions dialog UI and verify:
   - separator style is consistent across section headers/items/header/footer/search borders
   - empty-state row aligns with action-row spacing
   - list/body height remains aligned with search/header/footer visibility

**Risks / Known Gaps**
- I could not get a clean check/clippy pass because of unrelated repo-wide failures.
- These `part_04/body_part_*` fragments are not currently the active compiled render path in this branch (`src/actions/dialog.rs` includes `src/actions/dialog_part_04_rewire.rs`), so runtime impact depends on integration path.