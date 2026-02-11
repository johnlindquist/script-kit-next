# Select Prompt Entity Layout Audit

## Scope
- File: `src/prompts/select/render.rs`
- Focus: search box layout, list insets/row rounding, background/border shell treatment, vibrancy behavior, and row highlight semantics.

## User-Visible Surprises Found
1. Shell duplication and vibrancy conflict.
- `SelectPrompt` applied an opaque base background even when vibrancy was enabled.
- The prompt shell wrapper already provides the canonical frame/background behavior, so this created a layered shell effect and reduced blur consistency relative to other prompts.

2. Row highlight semantics collapsed focused and selected states.
- Selected rows used the same accent fill as the keyboard-focused row.
- In multi-select mode, this made it harder to identify which row would activate on Enter/arrow navigation.

3. Search field style diverged from canonical prompt header model.
- The search area used a flat strip with bottom border instead of an inset rounded field treatment used elsewhere.

4. List inset and row radius drift.
- Row cards and list inset values were custom and not aligned to shared list density expectations.

## Alignment Applied
1. Vibrancy-compatible shell behavior.
- Removed the always-on opaque root background in `SelectPrompt`.
- Kept conditional `when_some(vibrancy_bg, ...)` behavior only, matching the project’s vibrancy pattern.

2. Distinct row-state highlighting.
- Introduced separate selected-row alpha (`ROW_SELECTED_BG_ALPHA`) so:
  - focused row = strongest emphasis,
  - selected (non-focused) row = subtler emphasis,
  - hovered row = hover tint.
- Updated `resolve_row_bg_hex` to prioritize `focused > selected > hovered`.

3. Canonical search-box framing.
- Converted the filter input from bottom-border strip to an inset rounded field with subtle border alpha.
- Added outer top/x/bottom spacing to mirror floating-header input spacing.

4. List spacing and row corner alignment.
- Added `min_h(0)` to the list container for stable flex sizing.
- Normalized list insets and reduced row corner radius to match shared list geometry more closely.

## Canonical Layout Model (Select)
- Outer shell ownership:
  - Wrapper (`render_prompts/other.rs` + `prompt_shell_container`) owns frame-level radius/clip/background policy.
  - Entity (`SelectPrompt`) should avoid forcing opaque shell backgrounds.
- Input region:
  - Inset rounded search field with low-alpha border and placeholder-muted text treatment.
- List region:
  - Consistent horizontal inset, canonical row geometry, and explicit state hierarchy for focus/selection/hover.
- Highlight semantics:
  - Focus and selection should remain visually distinguishable, especially for multi-select.

## Verification Notes
- Added/updated row-state unit tests in `src/prompts/select/render.rs` covering:
  - selected-row color resolution,
  - hover color resolution,
  - focus priority over selected/hovered.
