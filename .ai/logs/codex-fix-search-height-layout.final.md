Changed files:
- `src/actions/dialog/part_04/body_part_03.rs`

What I changed:
- Moved search visibility derivation ahead of height math:
  - `search_at_top` and `show_search` are now computed before clamp calculations.
  - `search_box_height` is now derived from `show_search` (so `SearchPosition::Hidden` yields `0.0`).
- Included footer height in popup clamp budget:
  - `items_height` clamp now subtracts `footer_height`.
  - `total_height` is now computed once with footer included (removed the later “recalculate total height” step).

Verification run:
- `cargo check` (failed due unrelated repo errors, e.g. missing `ClipboardEntry` fields and non-exhaustive `ContentType` matches in `src/clipboard_history/*`).
- `cargo clippy --all-targets -- -D warnings` (failed due broad pre-existing repo issues, including module/file collisions, many unused imports, and test compile errors outside this file).

How to test this change:
1. Configure dialog with `search_position = Hidden` and ensure popup height no longer reserves one search-row worth of empty space.
2. Configure dialog with footer enabled and enough items to hit max height; confirm rendered popup height stays within `POPUP_MAX_HEIGHT` budget (no extra growth from footer).

Risks / known gaps:
- `src/actions/dialog.rs` currently includes `src/actions/dialog_part_04_rewire.rs` (not `src/actions/dialog/part_04.rs`), so this fix may need to be mirrored/synced into the rewire render file depending on your integration flow.
- I could not get a clean repo-wide compile/lint signal because of unrelated existing failures.