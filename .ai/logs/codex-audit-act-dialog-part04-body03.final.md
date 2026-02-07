# Audit: `src/actions/dialog/part_04/body_part_03.rs`

## Findings

### 1. `search_position = Hidden` still reserves search-row height (logic/layout bug)
- Severity: High
- Location: `src/actions/dialog/part_04/body_part_03.rs:24`, `src/actions/dialog/part_04/body_part_03.rs:27`, `src/actions/dialog/part_04/body_part_03.rs:132`, `src/actions/dialog/part_04/body_part_03.rs:134`
- What happens:
  - Popup height is computed with `search_box_height` before `show_search` is derived.
  - `show_search` correctly disables rendering when `SearchPosition::Hidden`, but the size calculation can still include search height.
- Impact:
  - Dialog height is larger than necessary when search is hidden.
  - Visible rows are reduced vs. available space and scrollbar math can drift from actual rendered layout.

### 2. Footer can push total popup height past `POPUP_MAX_HEIGHT`
- Severity: Medium
- Location: `src/actions/dialog/part_04/body_part_03.rs:24`, `src/actions/dialog/part_04/body_part_03.rs:26`, `src/actions/dialog/part_04/body_part_03.rs:74`, `src/actions/dialog/part_04/body_part_03.rs:128`, `src/actions/dialog/part_04/body_part_03.rs:129`
- What happens:
  - `items_height` is clamped against `POPUP_MAX_HEIGHT - search_box_height - header_height`.
  - Footer height is added after this clamp, so the final container height can exceed the configured max when `show_footer` is true.
- Impact:
  - Max-height contract is violated in footer-enabled mode.
  - On constrained windows, this can reduce visual stability and increase clipping risk.

## Verification Against Requested Criteria

- Rendering logic correctness: Fails in hidden-search and footer-enabled max-height paths (findings #1 and #2).
- Edge case handling: Hidden search configuration is not fully accounted for in sizing.
- Theme usage consistency: No major inconsistencies found in this segment; color branches consistently use `theme` for `Default` and token colors otherwise.
- Visual feedback states: Cursor, separators, and footer hint styling are present and coherent; primary risk is layout/height behavior, not feedback rendering.
