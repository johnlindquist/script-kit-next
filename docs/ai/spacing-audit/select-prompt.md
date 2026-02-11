# Select Prompt Spacing Audit

Snapshot date: 2026-02-11.

## Scope

Audit target: the Select prompt path from `ScriptListApp::render_select_prompt` through `SelectPrompt::render`, including shared wrappers that affect spacing/radius.

Primary files:
- `src/render_prompts/other.rs`
- `src/prompts/select/render.rs`
- `src/components/focusable_prompt_wrapper.rs`
- `src/components/prompt_layout_shell.rs`
- `src/components/unified_list_item/render.rs`
- `src/components/unified_list_item/types.rs`

## Render Call Chain

1. `ScriptListApp::render()` dispatches `AppView::SelectPrompt` to `self.render_select_prompt(entity, cx)` (`src/main_sections/render_impl.rs:159`, `src/main_sections/render_impl.rs:182`).
2. `render_select_prompt` resolves shell radius from design tokens and composes the wrapper:
   - `shell_radius = get_tokens(self.current_design).visual().radius_lg`
   - `prompt_shell_container(shell_radius, vibrancy_bg)`
   - `.child(prompt_shell_content(entity))`
   (`src/render_prompts/other.rs:112`, `src/render_prompts/other.rs:119`, `src/render_prompts/other.rs:121`).
3. `prompt_shell_container`/`prompt_shell_content` apply outer frame rules:
   - root: `w_full`, `h_full`, `min_h(0)`, `overflow_hidden`, `relative`, `rounded(radius)`
   - content slot: `flex_1`, `w_full`, `min_h(0)`, `overflow_hidden`
   (`src/components/prompt_layout_shell.rs:40`, `src/components/prompt_layout_shell.rs:57`, `src/components/prompt_layout_shell.rs:63`, `src/components/prompt_layout_shell.rs:79`, `src/components/prompt_layout_shell.rs:86`).
4. GPUI renders `Entity<SelectPrompt>`, entering `SelectPrompt::render` (`src/prompts/select/render.rs:97`).
5. `SelectPrompt::render` builds input + list + root surface, then wraps with `FocusablePrompt::build(...)` for focus/key routing (`src/prompts/select/render.rs:150`, `src/prompts/select/render.rs:326`, `src/prompts/select/render.rs:335`, `src/prompts/select/render.rs:351`).
6. `FocusablePrompt::build` adds only `track_focus` and `on_key_down`; it introduces no spacing/radius rules (`src/components/focusable_prompt_wrapper.rs:137`, `src/components/focusable_prompt_wrapper.rs:138`).

## Spacing And Radius Inventory

### A) Shell wrapper (`render_select_prompt` + `prompt_layout_shell`)

| Layer | Rule | Type | Value / Source |
|---|---|---|---|
| Shell root | Corner radius | Token-derived | `tokens.visual().radius_lg` from `self.current_design` (`src/render_prompts/other.rs:112`) |
| Shell root | Size | Hardcoded behavior | `w_full`, `h_full`, `min_h(0)` (`src/components/prompt_layout_shell.rs:44`, `src/components/prompt_layout_shell.rs:45`, `src/components/prompt_layout_shell.rs:46`) |
| Shell root | Clip | Hardcoded behavior | `overflow_hidden` (`src/components/prompt_layout_shell.rs:49`) |
| Shell root | Position context | Hardcoded behavior | `relative` (`src/components/prompt_layout_shell.rs:53`) |
| Shell content slot | Fill behavior | Hardcoded behavior | `flex_1`, `w_full`, `min_h(0)`, `overflow_hidden` (`src/components/prompt_layout_shell.rs:65`, `src/components/prompt_layout_shell.rs:66`, `src/components/prompt_layout_shell.rs:67`, `src/components/prompt_layout_shell.rs:68`) |

### B) Select prompt root surface (`SelectPrompt::render`)

| Layer | Rule | Type | Value / Source |
|---|---|---|---|
| `window:select` container | Corner radius | Hardcoded literal | `rounded(12.0)` (`src/prompts/select/render.rs:343`) |
| `window:select` container | Padding/margin | None | No explicit margin/padding on root |
| Input row (`input:select-filter`) | Min height | Constant | `PROMPT_INPUT_FIELD_HEIGHT` = `44.0` (`src/prompts/select/render.rs:153`, `src/panel.rs:76`) |
| Input row | Horizontal inset | Token-derived | `px(spacing.item_padding_x)` (`src/prompts/select/render.rs:154`) |
| Input row | Vertical inset | Token-derived | `py(spacing.padding_md)` (`src/prompts/select/render.rs:155`) |
| Input row | Internal gap | Utility literal | `gap_2()` (`src/prompts/select/render.rs:161`) |
| Empty-state block | Horizontal inset | Token-derived | `px(spacing.item_padding_x)` (`src/prompts/select/render.rs:194`) |
| Empty-state block | Vertical inset | Token-derived | `py(spacing.padding_xl)` (`src/prompts/select/render.rs:193`) |
| Choices container (`list:select-choices`) | Horizontal inset | Hardcoded literal | `px(8.0)` (`src/prompts/select/render.rs:332`) |

### C) Row shell + row content (`uniform_list` rows)

| Layer | Rule | Type | Value / Source |
|---|---|---|---|
| Row wrapper div | Height | Constant | `LIST_ITEM_HEIGHT` = `40.0` (`src/prompts/select/render.rs:284`, `src/list_item/mod.rs:62`) |
| Row wrapper div | Corner radius | Hardcoded literal | `rounded(8.0)` (`src/prompts/select/render.rs:285`) |
| `UnifiedListItem` (Comfortable density) | Height | Constant pass-through | `height = LIST_ITEM_HEIGHT` (`src/components/unified_list_item/types.rs:249`, `src/components/unified_list_item/types.rs:252`) |
| `UnifiedListItem` (Comfortable density) | Inner horizontal padding | Hardcoded literal | `padding_x = 12.0` (`src/components/unified_list_item/types.rs:253`, applied at `src/components/unified_list_item/render.rs:164`) |
| `UnifiedListItem` (Comfortable density) | Inner vertical padding | Hardcoded literal | `padding_y = 6.0` (`src/components/unified_list_item/types.rs:254`, applied at `src/components/unified_list_item/render.rs:165`) |
| `UnifiedListItem` (Comfortable density) | Inner content gap | Hardcoded literal | `gap = 8.0` (`src/components/unified_list_item/types.rs:255`, applied at `src/components/unified_list_item/render.rs:173`) |
| Shortcut/Count chips (trailing) | Chip padding + radius | Hardcoded literals | `px(6)`, `py(2)`, `rounded(3)` (`src/components/unified_list_item/render.rs:312`, `src/components/unified_list_item/render.rs:313`, `src/components/unified_list_item/render.rs:314`, `src/components/unified_list_item/render.rs:328`, `src/components/unified_list_item/render.rs:329`, `src/components/unified_list_item/render.rs:330`) |
| App icon placeholder | Corner radius | Hardcoded literal | `rounded(4.0)` (`src/components/unified_list_item/render.rs:296`) |

### D) Margin usage across this path

Explicit margin APIs (`m*`, `mx*`, `my*`, etc.) are not used in this render path. Spacing is currently all padding/gap/height/radius driven.

## Token-Derived Fields Used By Select

Fields directly consumed by `SelectPrompt::render` and shell wrapper:
- `spacing.item_padding_x`
- `spacing.padding_md`
- `spacing.padding_xl`
- `visual.radius_lg` (shell only)

Variant values from `DesignTokens` implementations (`src/designs/traits/parts.rs`):

| DesignVariant | `item_padding_x` | `padding_md` | `padding_xl` | `radius_lg` |
|---|---:|---:|---:|---:|
| Default | 16 | 12 | 24 | 12 |
| Minimal | 80 | 24 | 48 | 0 |
| RetroTerminal | 8 | 8 | 16 | 0 |
| Glassmorphism | 20 | 16 | 28 | 24 |
| Brutalist | 16 | 16 | 32 | 0 |
| NeonCyberpunk | 16 (default spacing) | 12 | 24 | 8 |
| Paper | 18 | 14 | 28 | 6 |
| AppleHIG | 16 | 12 | 20 | 14 |
| Material3 | 16 | 12 | 24 | 16 |
| Compact | 8 | 6 | 12 | 6 |
| Playful | 20 | 14 | 28 | 24 |

## Findings

1. Radius hierarchy is mixed:
- Outer shell uses token radius (`radius_lg`) but Select root hardcodes `12.0`.
- On non-default designs this can diverge (for example shell `0`, `6`, `14`, `16`, `24` vs inner `12`).

2. Horizontal inset hierarchy is mixed:
- Input and empty states use token `item_padding_x`.
- List lane uses hardcoded `8.0` plus row-internal hardcoded `12.0` in `UnifiedListItem`.
- Net result: search row and choice rows do not share one canonical inset system.

3. `gap_2()` is not token-coupled:
- Input icon/text spacing is utility-scale based, while most other spacing is token or literal px.

4. No `SELECT_*` constants exist on this runtime path:
- `rg -n "SELECT_"` over the audited Select files returns no matches.

5. Row radius is duplicated conceptually:
- Select row wrapper hardcodes `8.0`.
- `ListItemLayout` carries a `radius` field, but current `UnifiedListItem::render` does not apply it.

## Recommended Normalized Inset/Gap Pattern

Use one 3-level spacing model for Select:

1. Surface level (prompt frame)
- `surface_radius = tokens.visual().radius_lg`
- Apply to both shell and `window:select` root.

2. Section level (header/list/empty-state lane)
- `section_inset_x = tokens.spacing().item_padding_x`
- Use this for search row, empty-state row, and list lane container (replace hardcoded `8.0`).
- Keep `PROMPT_INPUT_FIELD_HEIGHT` as the canonical row height for the search line.

3. Row level (choice items)
- Keep `LIST_ITEM_HEIGHT` as canonical row hit target.
- `row_radius = tokens.visual().radius_md` (instead of hardcoded `8.0`).
- `row_gap = tokens.spacing().gap_md` for intra-row icon/text spacing (replace utility `gap_2()` path over time).
- Keep row-internal chips (shortcut/count) on a small radius token (`radius_sm`) if/when tokenized.

Default-variant target geometry under this model:
- surface radius: `12`
- section inset x: `16`
- input y inset: `12`
- empty-state y inset: `24`
- row radius: `8`
- row height: `40`

This keeps current Default visuals close while removing cross-variant drift and hardcoded inset forks.
