# codex-audit-act-theme-styling

## Scope audited
- `src/actions/dialog/part_02/*.rs`
- `src/actions/dialog/part_04/*.rs`
- `src/actions/constants.rs`

## Checklist findings

1. **No hardcoded `rgb()` colors - all use `theme.colors.*`**
- `part_02` and `part_04` use themed values via `rgb(...)` / `rgba(hex_with_alpha(...))` from `theme.colors.*` or design token colors.
- Fixed one literal transparent color in `src/actions/dialog/part_04/body_part_02.rs` by replacing `rgba(0x00000000)` with themed transparent backgrounds derived from selected-state colors.
- Verification grep in scope found no `rgb(0x...)`/`rgba(0x...)` literals.

2. **Consistent hover/selected/focused states**
- Action row state flow in `src/actions/dialog/part_04/body_part_02.rs` remains consistent:
  - selected uses selected background,
  - non-selected defaults to transparent,
  - hover uses hover background with destructive override,
  - focused keycap tones are derived from selected state colors.
- Change made only to ensure transparent base state is theme-derived for both default/non-default variants.

3. **Proper use of constants for dimensions**
- Added/centralized repeated UI dimensions in `src/actions/constants.rs`:
  - search field/cursor beam metrics,
  - action row spacing/icon sizing,
  - keycap sizing/padding/radius,
  - context header/footer sizing and gaps.
- Replaced repeated literals in:
  - `src/actions/dialog/part_04/body_part_01.rs`
  - `src/actions/dialog/part_04/body_part_02.rs`
  - `src/actions/dialog/part_04/body_part_03.rs`

4. **Border radius consistency**
- Keycap radius now uses `KEYCAP_RADIUS` in `src/actions/dialog/part_04/body_part_02.rs`.
- Cursor beam radius now uses `SEARCH_CURSOR_BEAM_RADIUS` in `src/actions/dialog/part_04/body_part_01.rs` and `src/actions/dialog/part_04/body_part_03.rs`.
- Existing row radius behavior (`ACTION_ROW_RADIUS`) preserved.

5. **Font sizes use theme tokens**
- Scoped files already use semantic text sizing (`text_sm`, `text_xs`) and theme-aware color tokens.
- No hardcoded `px` font sizing introduced in this change.

6. **Spacing follows design system**
- Replaced hardcoded horizontal padding with token spacing (`spacing.item_padding_x`) in:
  - section headers (`src/actions/dialog/part_04/body_part_02.rs`),
  - context header/footer (`src/actions/dialog/part_04/body_part_03.rs`).
- Replaced repeated literal gaps/padding with constants from `src/actions/constants.rs`.

## Code changes made
- `src/actions/constants.rs`
- `src/actions/dialog/part_04/body_part_01.rs`
- `src/actions/dialog/part_04/body_part_02.rs`
- `src/actions/dialog/part_04/body_part_03.rs`

## Verification run
- `rg -n "rgb\\(|rgba\\(|hex\\(" src/actions/dialog/part_02 src/actions/dialog/part_04 src/actions/constants.rs`
- `rg -n "rgba\\(0x|rgb\\(0x|rgb\\(\\s*0x|hex_with_alpha\\(\\s*0x" src/actions/dialog/part_02 src/actions/dialog/part_04 src/actions/constants.rs`
- `cargo check --lib`

### Verification result
- Scoped grep checks passed for hardcoded color literal audit within scope.
- `cargo check --lib` failed due out-of-scope parallel-agent changes (not introduced here), including:
  - `BuiltInFeature::QuicklinkCommand` missing variant references in `src/scripts/search/unified.rs`
  - missing `ClipboardEntryMeta` fields in `src/clipboard_history/*`

## Risks / known gaps
- `src/actions/dialog.rs` currently wires rendering through `src/actions/dialog_part_04_rewire.rs`; audited/fixed files are in assigned scope (`src/actions/dialog/part_04/*`) and may not be the active runtime path at this moment.
