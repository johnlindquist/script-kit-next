# Div Prompt Wrapper Audit

## Scope
- `src/render_prompts/div.rs`
- Comparison baselines:
  - `src/render_prompts/arg/render.rs`
  - `src/render_prompts/editor.rs`
  - `src/render_prompts/term.rs`
  - `src/components/prompt_layout_shell.rs`

## Wrapper Contract in `render_div_prompt`
`render_div_prompt` already follows the shared shell structure:
1. `prompt_shell_container(...)` root with explicit prompt height and focus/key handling.
2. Header strip + divider + `prompt_shell_content(...)` body.
3. `PromptFooter` action bar.
4. Absolute actions overlay rendered last.

## Inconsistencies Found
1. Header row height drift (fixed): the custom header strip did not explicitly use canonical header row height, so its visual height could drift from the prompt/header contract used elsewhere.
2. Header typography drift (fixed): the wrapper used fixed `.text_sm()` / `.text_xs()` styles rather than design-token font sizes, which reduced consistency across design variants.
3. Divider thickness consistency (already correct): divider uses `design_visual.border_thin`, matching wrapper conventions in other prompt renderers.
4. Overlay anchor consistency (confirmed): actions overlay uses shared `prompt_actions_dialog_offsets(...)` and top-right absolute placement like other wrappers.

## Implemented Normalization
In `src/render_prompts/div.rs`:
- Header row now pins to canonical row height with `.h(px(crate::panel::HEADER_BUTTON_HEIGHT))`.
- Header label typography now uses design tokens:
  - Title: `.text_size(px(design_typography.font_size_md))`
  - Hint: `.text_size(px(design_typography.font_size_sm))`
- Added source-based regression tests for:
  - canonical header row height usage
  - token-based header typography
  - shared overlay offset usage

## Recommended Unified Wrapper Pattern for Output/Content Prompts
For wrappers like `DivPrompt`, `TermPrompt`, and future output/content surfaces:
1. **Surface zone**: use `prompt_shell_container(radius, vibrancy_bg)` or equivalent root contract (`relative + flex_col + overflow_hidden + rounded`).
2. **Header zone (optional)**:
   - `px(HEADER_PADDING_X)` + `py(HEADER_PADDING_Y)`
   - explicit row height `HEADER_BUTTON_HEIGHT`
   - typography from `design_typography` tokens, not fixed text presets
3. **Divider zone**:
   - horizontal inset from spacing tokens
   - thickness from `design_visual.border_thin`
4. **Content zone**: `prompt_shell_content(entity)` with scroll ownership in the prompt entity/component.
5. **Footer zone (optional)**: `PromptFooter` with shared footer config helper.
6. **Overlay zone (optional)**: render last as `absolute + inset_0`, backdrop click-dismiss, position dialog via `prompt_actions_dialog_offsets(...)`.

## Follow-up Recommendation (not implemented)
Promote the “header strip + divider + content + footer + overlay” structure into a dedicated shared helper for output/content prompt wrappers so this layout contract is enforced by API rather than repeated inline.
