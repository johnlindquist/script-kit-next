# Arg Prompt Spacing Audit

Snapshot date: 2026-02-11.

## Scope

Primary files audited:
- `src/render_prompts/arg/render.rs`
- `src/components/prompt_input.rs`
- `src/components/prompt_footer.rs`

Supporting call-chain references:
- `src/main_sections/render_impl.rs`
- `src/render_prompts/arg.rs`
- `src/render_prompts/arg/helpers.rs`
- `src/panel.rs`
- `src/list_item/mod.rs`
- `src/window_resize/mod.rs`
- `src/designs/traits/parts.rs`

No Rust code changes were made. This is a documentation-only spacing audit.

## Render Call Chain

1. `ScriptListApp::render()` dispatches `AppView::ArgPrompt { ... }` to `self.render_arg_prompt(id, placeholder, choices, actions, cx)` (`src/main_sections/render_impl.rs:162`, `src/main_sections/render_impl.rs:168`).
2. `src/render_prompts/arg.rs` wires the implementation via `include!("arg/helpers.rs")` and `include!("arg/render.rs")` (`src/render_prompts/arg.rs:10`, `src/render_prompts/arg.rs:11`).
3. `render_arg_prompt` resolves design tokens (`spacing`, `typography`, `visual`) and derives action-dialog offsets with `prompt_actions_dialog_offsets(spacing.padding_sm, visual.border_thin)` (`src/render_prompts/arg/render.rs:99`, `src/render_prompts/arg/render.rs:105`, `src/render_prompts/arg/helpers.rs:2`).
4. Root layout is built as `relative + flex_col + w_full + h_full + rounded(radius_lg)` with header first (`src/render_prompts/arg/render.rs:320`, `src/render_prompts/arg/render.rs:322`, `src/render_prompts/arg/render.rs:327`, `src/render_prompts/arg/render.rs:334`).
5. Header child renders the input row and either placeholder/cursor overlay (empty) or `render_arg_input_text(...)` (non-empty) (`src/render_prompts/arg/render.rs:337`, `src/render_prompts/arg/render.rs:360`, `src/render_prompts/arg/render.rs:388`).
6. Choices section is conditional on `has_choices`; if present it renders divider + list lane (`src/render_prompts/arg/render.rs:394`, `src/render_prompts/arg/render.rs:397`, `src/render_prompts/arg/render.rs:408`).
7. Footer is always appended via `PromptFooter::new(...)` (`src/render_prompts/arg/render.rs:412`, `src/render_prompts/arg/render.rs:435`).
8. Actions overlay is appended last as absolute layer (`inset_0` backdrop + absolute top-right dialog) when actions popup is open (`src/render_prompts/arg/render.rs:456`, `src/render_prompts/arg/render.rs:480`, `src/render_prompts/arg/render.rs:494`).

## Spacing Inventory (Token-Derived vs Constants/Layout Metrics)

### A) Root, Header, and Input Row (`render_arg_prompt`)

| Area | Metric | Type | Value / Source |
|---|---|---|---|
| Root container | Corner radius | Token-derived | `design_visual.radius_lg` (`src/render_prompts/arg/render.rs:327`), default token value `12.0` (`src/designs/traits/parts.rs:353`) |
| Root container | Size/fill | Layout behavior | `w_full`, `h_full` (`src/render_prompts/arg/render.rs:325`, `src/render_prompts/arg/render.rs:326`) |
| Header row | Horizontal padding | Constant (shared panel metric) | `HEADER_PADDING_X = 16.0` (`src/render_prompts/arg/render.rs:337`, `src/panel.rs:53`) |
| Header row | Vertical padding | Constant (shared panel metric) | `HEADER_PADDING_Y = 8.0` (`src/render_prompts/arg/render.rs:338`, `src/panel.rs:58`) |
| Header row | Internal gap | Constant (shared panel metric) | `HEADER_GAP = 12.0` (`src/render_prompts/arg/render.rs:342`, `src/panel.rs:61`) |
| Input line container | Height | Layout metric formula | `CURSOR_HEIGHT_LG + 2 * CURSOR_MARGIN_Y` (`src/render_prompts/arg/render.rs:346`, `src/panel.rs:187`, `src/panel.rs:199`) = `22.0` |
| Input text size | Font size | Token-derived | `design_typography.font_size_lg` (`src/render_prompts/arg/render.rs:353`), default token value `16.0` (`src/designs/traits/parts.rs:226`) |

### B) Empty/Non-empty Input Internals (`render_arg_prompt` + `render_arg_input_text`)

| Area | Metric | Type | Value / Source |
|---|---|---|---|
| Empty-state cursor block | Cursor width | Constant (panel cursor metric) | `CURSOR_WIDTH = 2.0` (`src/render_prompts/arg/render.rs:373`, `src/panel.rs:169`) |
| Empty-state cursor block | Cursor height | Constant (panel cursor metric) | `CURSOR_HEIGHT_LG = 18.0` (`src/render_prompts/arg/render.rs:374`, `src/panel.rs:187`) |
| Empty-state placeholder alignment | Negative margin | Constant formula | `ml(-(CURSOR_WIDTH))` (`src/render_prompts/arg/render.rs:381`) |
| Non-empty input line (`render_arg_input_text`) | Row height | Layout metric formula | `CURSOR_HEIGHT_LG + 2 * CURSOR_MARGIN_Y` (`src/render_prompts/arg/render.rs:19`, `src/render_prompts/arg/render.rs:42`, `src/render_prompts/arg/render.rs:68`) = `22.0` |
| Non-empty input cursor | Cursor dimensions | Constant (panel cursor metric) | width `2.0`, height `18.0` (`src/render_prompts/arg/render.rs:76`, `src/render_prompts/arg/render.rs:77`, `src/panel.rs:169`, `src/panel.rs:187`) |

### C) List Divider, List Padding, and Empty State (`render_arg_prompt`)

| Area | Metric | Type | Value / Source |
|---|---|---|---|
| Divider above list | Horizontal inset | Token-derived | `mx(design_spacing.padding_lg)` (`src/render_prompts/arg/render.rs:397`), default `16.0` (`src/designs/traits/parts.rs:153`) |
| Divider above list | Thickness/height | Token-derived | `h(design_visual.border_thin)` (`src/render_prompts/arg/render.rs:398`), default `1.0` (`src/designs/traits/parts.rs:372`) |
| List lane wrapper | Vertical padding | Token-derived | `py(design_spacing.padding_xs)` (`src/render_prompts/arg/render.rs:408`), default `4.0` (`src/designs/traits/parts.rs:150`) |
| List lane wrapper | Flex behavior | Layout behavior | `flex_1 + min_h(0) + w_full` (`src/render_prompts/arg/render.rs:405`, `src/render_prompts/arg/render.rs:406`, `src/render_prompts/arg/render.rs:407`) |
| Empty-state message block (filtered list empty) | Vertical padding | Token-derived | `py(design_spacing.padding_xl)` (`src/render_prompts/arg/render.rs:269`), default `24.0` (`src/designs/traits/parts.rs:154`) |
| Uniform list fallback row | Row height | Shared list metric | `LIST_ITEM_HEIGHT = 40.0` (`src/render_prompts/arg/render.rs:297`, `src/list_item/mod.rs:62`) |

Notes:
- Effective vertical breathing around empty-state text inside a choices-enabled prompt is parent `padding_xs` plus child `padding_xl` (both top and bottom).
- The entire choices block is omitted when `has_choices == false` (`src/render_prompts/arg/render.rs:394`).

### D) Footer Metrics Used By Arg Prompt (`PromptFooter`)

Arg prompt itself only appends `PromptFooter`, but the resulting footer spacing is defined in `prompt_footer.rs`.

| Area | Metric | Type | Value / Source |
|---|---|---|---|
| Footer container | Fixed height | Shared layout metric | `FOOTER_HEIGHT = 30.0` (`src/components/prompt_footer.rs:35`, `src/components/prompt_footer.rs:493`, `src/window_resize/mod.rs:229`) |
| Footer container | Horizontal padding | Constant (`PromptFooter`) | `PROMPT_FOOTER_PADDING_X_PX = 12.0` (`src/components/prompt_footer.rs:54`, `src/components/prompt_footer.rs:498`) |
| Footer container | Bottom padding | Constant (`PromptFooter`) | `PROMPT_FOOTER_PADDING_BOTTOM_PX = 2.0` (`src/components/prompt_footer.rs:56`, `src/components/prompt_footer.rs:500`) |
| Left/right section spacing | Section gap | Constant (`PromptFooter`) | `PROMPT_FOOTER_SECTION_GAP_PX = 8.0` (`src/components/prompt_footer.rs:42`, `src/components/prompt_footer.rs:436`, `src/components/prompt_footer.rs:522`) |
| Buttons cluster spacing | Button gap | Constant (`PromptFooter`) | `PROMPT_FOOTER_BUTTON_GAP_PX = 4.0` (`src/components/prompt_footer.rs:44`, `src/components/prompt_footer.rs:457`) |
| Button interior | Inner gap/padding/radius | Constants (`PromptFooter`) | `gap 6`, `px 8`, `py 6`, `rounded 4` (`src/components/prompt_footer.rs:366`, `src/components/prompt_footer.rs:367`, `src/components/prompt_footer.rs:368`, `src/components/prompt_footer.rs:369`) |
| Button divider | Width/height/h-margins | Constants (`PromptFooter`) | `1 x 16`, `mx 4` (`src/components/prompt_footer.rs:62`, `src/components/prompt_footer.rs:64`, `src/components/prompt_footer.rs:66`, `src/components/prompt_footer.rs:417`, `src/components/prompt_footer.rs:418`, `src/components/prompt_footer.rs:419`) |

### E) Actions Overlay (`render_arg_prompt` + helper)

| Area | Metric | Type | Value / Source |
|---|---|---|---|
| Overlay layer | Coverage | Layout behavior | `absolute + inset_0` covers full arg prompt bounds (`src/render_prompts/arg/render.rs:479`, `src/render_prompts/arg/render.rs:480`) |
| Dialog anchor (top) | Y offset | Shared metric + tokens | `HEADER_TOTAL_HEIGHT + padding_sm - border_thin` (`src/render_prompts/arg/helpers.rs:4`) |
| Dialog anchor (right) | X offset | Token-derived | `padding_sm` (`src/render_prompts/arg/helpers.rs:5`) |
| Default-design anchor values | Concrete defaults | Derived runtime value | top `52.0`, right `8.0` (`src/render_prompts/arg/tests.rs:11`, `src/render_prompts/arg/tests.rs:12`, `src/render_prompts/arg/tests.rs:13`) |

## PromptInput / PromptFooter Interaction Notes

### PromptInput interaction

Arg prompt currently does **not** instantiate `PromptInput` in its render path; it draws its own header/input row inline (`src/render_prompts/arg/render.rs:334`).

Comparison of key metrics:

| Concern | Arg prompt (`render_arg_prompt`) | `PromptInput` component |
|---|---|---|
| Horizontal/vertical inset contract | Header uses `HEADER_PADDING_X/Y` = `16/8` (`src/render_prompts/arg/render.rs:337`, `src/render_prompts/arg/render.rs:338`) | `InputPadding::default()` is also `left/right 16`, `top/bottom 8` (`src/components/prompt_input.rs:53`, `src/components/prompt_input.rs:55`) |
| Empty-placeholder cursor compensation | `ml(-(CURSOR_WIDTH))` only (`src/render_prompts/arg/render.rs:381`) | Uses explicit cursor gap and compensates with `ml(-(CURSOR_WIDTH + CURSOR_GAP_X))` (`src/components/prompt_input.rs:454`, `src/components/prompt_input.rs:463`) |
| Cursor vertical centering strategy | Fixed row height `22` with cursor `h(18)` (`src/render_prompts/arg/render.rs:346`, `src/render_prompts/arg/render.rs:374`) | Cursor itself carries `my(CURSOR_MARGIN_Y)` (`src/components/prompt_input.rs:396`) |
| Text-size source | Design token `font_size_lg` (`src/render_prompts/arg/render.rs:353`) | Uses GPUI `text_lg()` utility (`src/components/prompt_input.rs:427`) |

Implication: Arg prompt is aligned to panel/header constants but is not yet fully unified with `PromptInput` cursor-gap and sizing behavior.

### PromptFooter interaction

Arg prompt composes footer state through `prompt_footer_config_with_status(...)` and always mounts `PromptFooter` (`src/render_prompts/arg/render.rs:423`, `src/render_prompts/arg/render.rs:435`), so:
- Footer height and internals are owned by `PromptFooter` constants/`FOOTER_HEIGHT`.
- Arg prompt contributes content-state labels (helper/info) and action visibility, not footer geometry.

## Summary Findings

1. Arg prompt spacing is a hybrid of token-driven and shared constants.
- Token-driven: root radius, list divider thickness/inset, list vertical padding, empty-state vertical padding, actions offset inputs.
- Constant-driven: header paddings/gap, cursor geometry, list-row fallback height, footer internals.

2. Header/input row and footer use different spacing ownership models.
- Header relies on `panel.rs` constants.
- Footer relies on `prompt_footer.rs` constants plus shared `FOOTER_HEIGHT`.

3. `PromptInput` is not on the Arg render path.
- Some metrics are numerically aligned (16/8 insets), but cursor-gap and text-size handling differ between Arg inline rendering and `PromptInput`.

4. Actions overlay anchoring is header-aware by formula.
- `top` offset tracks `HEADER_TOTAL_HEIGHT` plus tokenized adjustments; default behavior is validated at `52/8`.
