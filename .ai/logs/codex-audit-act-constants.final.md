# codex-audit-act-constants

## Scope
- Audited: `src/actions/constants.rs`
- Cross-checked usages in: `src/actions/dialog/**`, `src/actions/window/**`, `src/actions/dialog_part_04_rewire.rs`

## Verification Results

1. Dimension constants visual sanity (`POPUP_WIDTH=320`, `POPUP_MAX_HEIGHT=400`, etc.)
- Status: PASS
- Notes: Current values are coherent for compact action popups: fixed width (`320`) with bounded height (`400`) keeps list density manageable and avoids oversized overlays. `ACTION_ITEM_HEIGHT=36` and `SEARCH_INPUT_HEIGHT=44` are internally consistent with current layout math.

2. Constants usage coverage (no magic-number drift)
- Status: PARTIAL
- Finding: duplicate width constant outside `constants.rs`.
  - `src/actions/window/part_01.rs:146` defines `ACTIONS_WINDOW_WIDTH: f32 = 320.0` instead of using `POPUP_WIDTH`.
  - Risk: width can drift between dialog and separate actions window if only one side is updated.

3. Constants sufficiency (missing shared constants)
- Status: PARTIAL
- Finding: repeated layout literals remain uncentralized in dialog rendering paths.
  - `src/actions/dialog/part_04/body_part_02.rs:96` and `src/actions/dialog/part_04/body_part_03.rs:49` use repeated `16.0` horizontal padding.
  - `src/actions/dialog/part_04/body_part_03.rs:50` and `src/actions/dialog_part_04_rewire.rs:634` use repeated `8.0` top padding.
- Impact: low (styling consistency/maintainability only), but centralizing would reduce multi-file edits.

4. Reasonableness across screen sizes
- Status: PASS (with caveat)
- Notes: `320x<=400` is reasonable on typical laptop/desktop sizes and keeps popups compact. Caveat: no adaptive width scaling exists for very small windows; width is hard-fixed.

## Additional Audit Finding
- Stale comments reference old row/header sizes (documentation drift only):
  - `src/actions/dialog/part_01.rs:82` says section headers are `24px` and items are `44px`.
  - `src/actions/dialog/part_04/body_part_02.rs:467` says `ACTION_ITEM_HEIGHT (44px)` and `SECTION_HEADER_HEIGHT (24px)`.
  - `src/actions/dialog_part_04_rewire.rs:573` repeats the same.
- Actual constants are `ACTION_ITEM_HEIGHT=36` and `SECTION_HEADER_HEIGHT=22` in `src/actions/constants.rs`.

## Verification Commands Run
- `cargo test -p script-kit-gpui test_popup_constants -- --nocapture`
- `cargo test -p script-kit-gpui actions::constants::tests -- --nocapture`

Both scoped runs passed.
