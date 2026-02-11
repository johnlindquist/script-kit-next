# PromptHeader Component Audit

## Scope

- `src/components/prompt_header/component.rs`
- `src/components/prompt_header/types.rs`
- `src/components/prompt_header/tests.rs`
- Consumer sanity check: `src/prompts/path/render.rs`

## Current Layout Contract

The current header row is built as:

1. Input area (`flex_1`, floating bordered field).
1. Optional Ask-AI hint cluster.
1. Right actions slot (`min_w` reserved by density, absolute stacked layers).
1. Fixed logo (`21x21`).

Actions-mode toggle is CLS-resistant today: button and search layers stay
mounted in the same relative container and switch via opacity plus visibility.

## Findings

### 1) Main input cursor and placeholder alignment is intentionally corrected

`render_input_area()` reserves left-cursor space when empty, then applies
`ml(-(CURSOR_WIDTH + CURSOR_GAP_X))` to placeholder text so empty and typed
text share the same x-origin. This prevents first-character horizontal jump in
the main query field.

### 2) Actions search does not apply the same anti-jump alignment

`render_actions_search()` puts the cursor before empty placeholder but does not
offset placeholder back. Result: first typed character shifts left by cursor
width plus gap in actions mode.

### 3) Actions reservation is robust for toggling, weak for overflow

Density tokens reserve `min_w` (`168/200/236`), which prevents mode-switch
collapse. The slot is not fixed-width and does not clamp child intrinsic width.
Long primary labels or shortcuts can visually overflow left into adjacent
header content.

### 4) Baseline and vertical rhythm is mixed across slots

The input field uses extra wrapper top and bottom padding (`pt 8 / pb 6`) while
the actions rail is `h 1.75rem` and Ask-AI uses ghost-button min height. This
can produce subtle optical misalignment between query text, buttons, hint
badges, and logo centerline.

### 5) Narrow-width behavior lacks an explicit slot-priority policy

Current behavior relies on flex defaults and fixed `path_prefix` max width
(`320`). There is no canonical shrink and hide order for prefix, query text,
Ask-AI hint, actions slot, and logo at very small widths.

## Edge Cases To Cover

- Very long `path_prefix` plus long query plus Ask-AI on narrow windows.
- Long `primary_button_label` (or localized text) in `Compact` density.
- Large UI font scaling where `⌘K + search + |` exceeds compact reserved width.
- Runtime toggling of Ask-AI visibility causing query width jump.
- Non-empty actions search with long text clipping and no explicit policy.

## Canonical Header Slots (Proposed)

Use these slot names consistently in code and docs:

1. `header_slot_query`: main input region (`flex_1`, `min_w(0)`), owns
   prefix, query, and cursor behavior.
1. `header_slot_hint`: optional Ask-AI affordance (`flex_shrink_0`), removable
   under width pressure.
1. `header_slot_actions`: mode-toggled action and search rail
   (`flex_shrink_0`, fixed density width).
1. `header_slot_brand`: Script Kit logo (`flex_shrink_0`, fixed `21x21`).

## Canonical Spacing and Alignment Rules (Proposed)

### Root Row

- Keep `px(1rem)`, `py(0.5rem)`, and `gap(0.75rem)`.
- Add `items_center()` on root to enforce one cross-axis baseline.

### Query Slot

- Require `min_w(0)` to allow safe shrinking inside flex rows.
- Keep prefix truncation, but cap prefix to `min(320px, 40% of query slot)`.
- Apply one cursor reservation token:
  `CURSOR_RESERVE_X = CURSOR_WIDTH + CURSOR_GAP_X`.
- Use the same cursor-reservation logic for both main query and actions search
  placeholders.

### Actions Slot

- Use explicit `w(px(density.reserved_min_width_px()))` plus matching min and
  max width instead of `min_w` only.
- Keep both layers mounted and absolutely stacked; toggle only
  opacity/visibility.
- Define density widths as max intrinsic budgets for primary plus actions
  buttons and for the actions-search row.

### Hint and Brand

- Keep Ask-AI hint at ghost-button height and center-align with actions rail.
- Width-pressure priority: truncate query first, then hide hint, then preserve
  actions slot and logo.

## Suggested Follow-up Tests

- Add a source-level test asserting actions-search placeholder compensation
  mirrors main input logic.
- Add a layout-contract test for fixed actions slot width per density
  (`w=min_w=max_w`).
- Add a narrow-width visual test for overflow priority: query truncates, hint
  drops, actions and logo stay aligned.
