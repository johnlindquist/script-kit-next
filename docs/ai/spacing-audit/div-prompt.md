# Div Prompt Spacing Audit

## Scope

- `src/render_prompts/div.rs`
- `src/prompts/div/render.rs`
- `src/components/prompt_footer.rs`

No Rust edits were made. This is an audit-only report.

## Render Call Chain

- `ScriptListApp::render_div_prompt(...)` builds the outer shell and prompt
  chrome (`src/render_prompts/div.rs:11`).
- Outer wrapper uses `prompt_shell_container(radius_lg, vibrancy_bg)`
  (`src/render_prompts/div.rs:134`).
- Wrapper injects a header row using `HEADER_PADDING_X`, `HEADER_PADDING_Y`,
  and `HEADER_GAP` (`src/render_prompts/div.rs:146`).
- Wrapper injects a divider using `mx(design_spacing.padding_lg)` and
  `h(design_visual.border_thin)` (`src/render_prompts/div.rs:172`).
- Wrapper injects content via `prompt_shell_content(entity.clone())`
  (`src/render_prompts/div.rs:177`).
- `prompt_shell_content(...)` is a thin shared wrapper:
  `flex_1 + w_full + min_h(0) + overflow_hidden` with no insets
  (`src/components/prompt_layout_shell.rs:60`,
  `src/components/prompt_layout_shell.rs:86`).
- The entity renders through `DivPrompt::render(...)`
  (`src/prompts/div/render.rs:10`).
- `DivPrompt::render(...)` adds `.p(px(container_padding))` to its root and
  puts HTML content in an `overflow_y_scroll()` container with tracked scroll
  (`src/prompts/div/render.rs:129`, `src/prompts/div/render.rs:113`).
- `ScriptListApp::render_div_prompt(...)` appends `PromptFooter` under content
  (`src/render_prompts/div.rs:180`).
- Actions dialog overlay is absolute with `.top(actions_dialog_top)` and
  `.right(actions_dialog_right)` over full prompt bounds
  (`src/render_prompts/div.rs:220`).

## Spacing Inventory

### Header, Divider, Content

- Header horizontal inset: `HEADER_PADDING_X = 16.0` (`src/panel.rs:53`), used
  in div prompt at `src/render_prompts/div.rs:148`.
- Header vertical inset: `HEADER_PADDING_Y = 8.0` (`src/panel.rs:58`), used in
  div prompt at `src/render_prompts/div.rs:149`.
- Header internal gap: `HEADER_GAP = 12.0` (`src/panel.rs:61`), used in div
  prompt at `src/render_prompts/div.rs:154`.
- Divider horizontal inset: `mx(design_spacing.padding_lg)`
  (`src/render_prompts/div.rs:173`).
- Divider thickness: `design_visual.border_thin`
  (`src/render_prompts/div.rs:174`).
- Div content container padding source:
  `container_options.get_padding(default_container_padding(...))`
  (`src/prompts/div/render.rs:80`).
- Default div content padding is `spacing.padding_md`
  (`src/prompts/div/types.rs:119`).
- Scroll behavior is applied on the padded inner content container using
  `.overflow_y_scroll().track_scroll(...)` (`src/prompts/div/render.rs:113`).

### Footer

- Footer height is `FOOTER_HEIGHT = 30.0` (`src/window_resize/mod.rs:229`),
  used in `src/components/prompt_footer.rs:493`.
- Footer horizontal inset is `PROMPT_FOOTER_PADDING_X_PX = 12.0`
  (`src/components/prompt_footer.rs:54`), used at
  `src/components/prompt_footer.rs:498`.
- Footer bottom optical padding is `PROMPT_FOOTER_PADDING_BOTTOM_PX = 2.0`
  (`src/components/prompt_footer.rs:56`), used at
  `src/components/prompt_footer.rs:500`.
- Footer section gap (left/right clusters) is `8.0`
  (`src/components/prompt_footer.rs:42`, used at
  `src/components/prompt_footer.rs:436` and
  `src/components/prompt_footer.rs:522`).
- Footer button-row gap is `4.0` (`src/components/prompt_footer.rs:44`, used at
  `src/components/prompt_footer.rs:457`).
- Per-button internal spacing is `gap 6`, `px 8`, `py 6`, and radius `4`
  (`src/components/prompt_footer.rs:366`).

### Actions Overlay

- Offset source function is `prompt_actions_dialog_offsets(padding_sm,
  border_thin)` (`src/render_prompts/div.rs:27`), defined at
  `src/render_prompts/arg/helpers.rs:2`.
- Formula used by all call sites:
  - `top = HEADER_TOTAL_HEIGHT + padding_sm - border_thin`
  - `right = padding_sm`
- `HEADER_TOTAL_HEIGHT = 45.0` (`src/panel.rs:72`).
- With default token values (`padding_sm = 8`, `border_thin = 1`), the overlay
  anchor becomes `top = 52`, `right = 8`.

## Inconsistencies vs Shared Components

- Header/content/footer horizontal rhythm is not unified.
  - Header uses `16` (`HEADER_PADDING_X`).
  - Inner div content default uses `padding_md` (default is `12`).
  - Footer uses fixed `12` (`PROMPT_FOOTER_PADDING_X_PX`).
  - Result: content and footer align to `12`, while header/divider align to
    `16` in the default design.
- Divider inset is token-driven while header inset is panel-constant.
  - Divider uses `design_spacing.padding_lg`.
  - Header uses fixed `HEADER_PADDING_X`.
  - They only match when `padding_lg == 16`; other variants can drift.
- Actions dialog top anchor is tied to canonical input-header math, not div
  header geometry.
  - Offset assumes `HEADER_TOTAL_HEIGHT` includes a 28px input row.
  - Div header row in `render_div_prompt` has no fixed 28px row height.
  - Overlay top can land lower than the visible divider/content seam.
- `prompt_shell_content(...)` is spacing-neutral, but `DivPrompt::render(...)`
  applies padding internally.
  - Div prompt spacing depends on inner implementation (`.p(container_padding)`)
    instead of wrapper-level chrome policy.
- Footer docs comment drift.
  - `PromptFooter` docs say `Height: 40px fixed`
    (`src/components/prompt_footer.rs:302`), but rendered height uses
    `FOOTER_HEIGHT = 30`.

## Suggested Unified Inset and Gap Usage (No Code Changes Applied)

- Pick one canonical horizontal inset for div prompt chrome and content.
  - Preferred: reuse `HEADER_PADDING_X` for header, divider, content, and
    footer.
  - Alternative: use a design token inset (for example `padding_lg`) across all
    four.
- Keep vertical spacing ownership split by layer.
  - Wrapper owns header and divider geometry.
  - `DivPrompt::render` owns HTML flow and scroll behavior, but consumes the
    same horizontal inset source as wrapper chrome.
- Align actions dialog anchor with actual header block geometry.
  - Either give div header an explicit height matching `HEADER_TOTAL_HEIGHT`
    assumptions.
  - Or derive overlay top from the same constants used by
    `render_div_prompt` header and divider.
- Make footer horizontal padding follow the same inset source as
  header/content in this prompt class.
  - Avoid mixed `16`, `12`, and variant tokens in one prompt layout.
- Keep gap scale tokenized and simple.
  - Header uses `HEADER_GAP`.
  - Footer clusters/buttons keep `8/4` unless moved to shared gap tokens.
