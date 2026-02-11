# Prompt Input Component Audit

## Scope
- Component: `src/components/prompt_input.rs`
- Focus: font size/padding resolution, cursor placement, placeholder alignment, and canonical reuse contract for prompt wrappers.

## Current Findings
1. `PromptInputConfig::font_size` and `PromptInput::with_config()` resolved font size, but render used `.text_lg()` (resolved value was ignored).
2. `PromptInputConfig::padding` and `with_config()` resolved padding, but render did not apply the padding values.
3. Placeholder alignment depended on a negative margin (`ml(-(CURSOR_WIDTH + CURSOR_GAP_X))`) to offset left-cursor space.
4. `enable_selection` / `enable_clipboard` are behavioral flags on config, but this component is currently presentational only.

## Canonical Prompt Input Model

### 1) Metrics
- `font_size`: resolved from config (`config.font_size` or `Config::get_editor_font_size()`)
- `cursor_height`: `max(font_size, CURSOR_HEIGHT_LG)`
- `line_height`: `cursor_height + 2 * CURSOR_MARGIN_Y`
- Leading cursor slot width: `PROMPT_INPUT_CURSOR_SLOT_WIDTH = CURSOR_WIDTH + CURSOR_GAP_X`

### 2) Padding
- Apply `InputPadding { top, bottom, left, right }` directly on the outer input container.
- No prompt-specific hardcoded per-screen input padding; wrappers should pass config overrides only when intentionally different.

### 3) Cursor + Placeholder Layout
- Always reserve a leading cursor slot to stabilize text origin across empty/non-empty states.
- Empty state:
  - Render placeholder text at normal text origin.
  - Render blinking cursor in the leading slot when focused and visible.
- Non-empty state:
  - Keep leading slot reserved (layout stability).
  - Render trailing cursor after input text with `CURSOR_GAP_X` spacing.
- Do not use negative margins for placeholder alignment.

### 4) Focused vs Unfocused Rendering
- Focused: cursor blinks according to `cursor_visible && is_focused`; text uses focused theme tokens.
- Unfocused: cursor hidden but slot retained (no layout shift); text/placeholder colors remain token-driven.

### 5) Selection / Clipboard Expectations
- `PromptInput` should remain the canonical visual renderer.
- Interactive behaviors (selection range, copy/paste/cut, word-nav, etc.) should be owned by wrapper/state layers.
- Canonical expectations by preset:
  - `main_menu()`: selection/clipboard disabled.
  - `search()` and `arg()`: selection/clipboard enabled.

## Replacement Contract For Ad-Hoc Prompt Layouts
Prompt wrappers that currently hand-roll cursor + placeholder rows (e.g. arg/header-like views) should use `PromptInput` as the rendering primitive and stop duplicating:
- cursor dimensions and blink rendering,
- placeholder color logic,
- path-prefix dimmed text rules,
- ad-hoc negative margin alignment hacks.

Wrappers should only supply:
- `PromptInputConfig` preset + overrides,
- `PromptInputColors::from_theme_focused(theme, is_focused)`,
- `filter_text`, optional `path_prefix`, and cursor/focus state.

## Changes Made In This Audit
- `src/components/prompt_input.rs` now:
  - applies resolved `font_size` via `.text_size(px(self.font_size))`,
  - applies resolved outer padding (`pt/pb/pl/pr`),
  - replaces negative-margin placeholder alignment with a canonical leading cursor slot,
  - defines `PROMPT_INPUT_CURSOR_SLOT_WIDTH` for shared cursor-slot math.
