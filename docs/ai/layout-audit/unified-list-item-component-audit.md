# Unified List Item Component Audit

## Scope
- Reviewed files: `src/components/unified_list_item/mod.rs`, `src/components/unified_list_item/types.rs`, `src/components/unified_list_item/render.rs`, `src/components/unified_list_item/tests.rs`, `src/components/unified_list_item_tests.rs`
- Focus areas: density/height tiers, leading/trailing slot alignment, hover/selected styling, accent bar behavior
- Audit date: 2026-02-11

## Current Contract (As Implemented)

### Density and Height Tiers
- `ListItemLayout::from_density` keeps `height` fixed at `crate::list_item::LIST_ITEM_HEIGHT` (40px) for both densities (`src/components/unified_list_item/types.rs:249`).
- `Comfortable`: `padding_x=12`, `padding_y=6`, `gap=8`, `leading_size=20`, `radius=6` (`src/components/unified_list_item/types.rs:251`).
- `Compact`: `padding_x=8`, `padding_y=4`, `gap=6`, `leading_size=16`, `radius=4` (`src/components/unified_list_item/types.rs:259`).

### Leading/Trailing Slot Behavior
- Leading and trailing slots are optional and collapse entirely when content is absent (`src/components/unified_list_item/render.rs:175`, `src/components/unified_list_item/render.rs:181`).
- `LeadingContent::Custom` and `TrailingContent::Custom` are dropped (`None`) instead of rendered (`src/components/unified_list_item/render.rs:298`, `src/components/unified_list_item/render.rs:343`).
- Title/subtitle text slot uses `flex_1 + min_w(0) + overflow_hidden`, which is virtualization-safe for truncation (`src/components/unified_list_item/render.rs:148`).

### Hover/Selected Styling
- Background state matrix is `selected > hovered > transparent` (`src/components/unified_list_item/render.rs:123`).
- Hover styling is duplicated by two mechanisms: explicit `state.is_hovered` and pseudo-class `.hover(...)` (`src/components/unified_list_item/render.rs:125`, `src/components/unified_list_item/render.rs:192`).
- Selected and hover background colors both derive from `accent_subtle` with theme opacity scalars (`src/components/unified_list_item/render.rs:118`).

### Accent Bar
- Accent rail is optional via `with_accent_bar` and implemented as a `3px` left border (`src/components/unified_list_item/render.rs:205`).
- No balancing right padding or reserved gutter is applied in container geometry (`src/components/unified_list_item/render.rs:197`).

## Layout Assumptions That Can Cause Misalignment Across Prompts

1. **Mixed rows shift text start when leading content is missing**
- Assumption: optional leading slot may collapse with no layout consequence.
- Reality: when some rows have icons and others do not, title start x-position changes because no placeholder width is reserved.

2. **Trailing accessories are unconstrained in width and baseline semantics**
- Assumption: any trailing element can be dropped into a shrink-disabled wrapper and still align consistently.
- Reality: trailing variants (`Shortcut`, `Count`, `Chevron`, `Hint`) have distinct intrinsic sizes and typographic metrics, so columns do not line up across rows.

3. **Accent rail changes left geometry without a compensating rule**
- Assumption: border-left is only a visual add-on.
- Reality: enabling accent rail introduces additional left width without a canonical balancing rule, making row content appear offset relative to prompts that do not use it.

4. **Section header x-padding is not tied to row x-padding token**
- `SectionHeader` uses `px(16)` (`src/components/unified_list_item/render.rs:466`) while row body uses density-based `padding_x` of `12/8` (`src/components/unified_list_item/render.rs:164`).
- This creates header-label misalignment against row text starts.

5. **Two-line rows rely on fixed 40px row height with no adaptive vertical contract**
- Assumption: optional subtitle can be added without changing row vertical metrics.
- Reality: subtitle is always rendered when present, but row height remains fixed at 40px; this can create clipping/tight leading depending on font metrics and density.

6. **Density exposes `radius` token but render path does not consume it**
- Assumption: density uniformly controls all geometric traits.
- Reality: radius is computed but never applied to row background/containers, so density does not fully map to visible row geometry.

7. **Custom content variants are declared but not composable**
- `TextContent::Custom`, `LeadingContent::Custom`, and `TrailingContent::Custom` resolve to empty/no-op rendering (`src/components/unified_list_item/render.rs:393`, `src/components/unified_list_item/render.rs:298`, `src/components/unified_list_item/render.rs:343`).
- Callers may assume they can preserve custom alignment while extending slots; they cannot.

## Proposed Unified Vocabulary for All List-Based Prompts

Use this common language in component APIs and prompt renderers:

- **Row Shell**: fixed-height, virtualization-safe outer container (`h=40` today).
- **Row Body**: interactive background layer (selected/hover/disabled visuals).
- **Accent Rail**: optional selection indicator rail with fixed width and reserved geometry.
- **Leading Slot**: left accessory lane (icon/emoji/app icon/placeholder), fixed lane width by density.
- **Content Slot**: title/subtitle stack with truncation and highlight fragments.
- **Trailing Slot**: right accessory lane (shortcut/count/hint/status icon) with canonical baseline alignment.
- **Row Density**: the only knob that changes internal spacing (not row height unless explicitly adopting variable-height lists).
- **Row State**: `idle`, `hovered`, `selected`, `disabled` with a single state priority table.
- **Section Header Row**: non-selectable grouped-list row aligned to the same content x-grid as item rows.

## Canonical Spacing Rules

These tokens should be shared across list-based prompts.

| Token | Comfortable | Compact | Rule |
|---|---:|---:|---|
| `row.height` | 40 | 40 | Must match virtualization row height constant. |
| `row.padding.x` | 14 | 10 | Horizontal content inset token; section headers must use this same inset. |
| `row.padding.y` | 4 | 3 | Vertical inset token for row body. |
| `row.gap.leading_to_content` | 8 | 6 | Gap between leading slot and content slot. |
| `row.leading.slot` | 20 | 16 | Fixed leading lane width; reserve lane even when empty if list mixes icon/no-icon rows. |
| `row.trailing.gap` | 6 | 4 | Gap between trailing accessories inside trailing slot. |
| `row.trailing.min_width` | 20 | 16 | Reserve minimum trailing lane for stable right-edge alignment. |
| `accent.rail.width` | 3 | 3 | Fixed width; geometry reserved whenever rail feature is enabled for that list. |
| `section.header.height` | 32 | 32 | Shared grouped-list header height. |
| `section.header.padding.x` | = `row.padding.x` | = `row.padding.x` | Header labels must align with row content start. |

## Canonical State/Styling Rules

1. State priority: `disabled > selected > hovered > idle`.
2. Use exactly one hover signal source per list:
- pointer-driven hover pseudo-class, or
- externally controlled `is_hovered` state.
3. Selected and hovered backgrounds should share one semantic palette (`accent_subtle + opacity`) to avoid prompt-specific color drift.
4. Accent rail behavior:
- if list enables rails, reserve rail width on every row in that list,
- selected row paints rail color,
- non-selected row rail is transparent (layout still reserved).
5. Disabled rows should suppress hover affordances and use dimmed text/icon tokens consistently.

## Recommended Integration Contract

For every list-based prompt renderer (select/path/arg/etc.), enforce this composition order:

1. `RowShell(height)`
2. `AccentRail(width, enabled_for_list)`
3. `RowBody(padding, bg, radius)`
4. `LeadingSlot(fixed lane)`
5. `ContentSlot(title/subtitle/highlights, truncation)`
6. `TrailingSlot(fixed lane + accessory gap)`

If a prompt needs variable-height rows, it should opt out explicitly and not reuse fixed-height virtualization assumptions.

## Gaps To Resolve Before Broad Adoption

- Wire `Custom` content variants so extensions can preserve slot geometry.
- Apply density `radius` token in render path (or remove it from vocabulary).
- Decide whether subtitle rows are allowed in fixed-height lists; if yes, constrain subtitle visibility policy (for example: selected-only) to avoid vertical clipping.
- Normalize header x-padding to row x-padding token.
