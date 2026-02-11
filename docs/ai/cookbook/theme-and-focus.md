# Theme And Focus Patterns (GPUI)

## When to use
- You are styling UI surfaces and text in render code.
- You need focused vs unfocused visuals (input border/background, cursor, accent emphasis).
- You need color values that respect user theme settings and focus-aware overrides.

## Do not do
- Do not hardcode RGB/hex values for UI colors when a theme token exists.
- Do not branch on focus with ad-hoc constants; use theme-provided focused/unfocused colors.
- Do not bypass `Theme::get_colors(is_focused)` for focus-aware color selection.

## Canonical files
- `src/theme/types/part_04.rs:19` is the canonical `Theme::get_colors(is_focused)` implementation (focus-aware + fallback behavior).
- `src/components/prompt_input.rs:290` shows focus-aware color extraction via `theme.get_colors(is_focused)`.
- `src/components/form_fields/colors.rs:35` maps form field colors from `theme.colors` (no hardcoded palette values).
- `src/components/form_fields/text_field/render.rs:15` reads runtime focus state via `self.focus_handle.is_focused(window)`.
- `src/components/form_fields/text_field/render.rs:35` applies focus-aware color selection for border/background.
- `src/ui_foundation/part_000.rs:165` shows theme-token to UI-token mapping (`UIDesignColors::from_theme`).

## Minimal snippet
```rust
// Prefer focus-aware scheme selection from Theme.
let colors = theme.get_colors(is_focused);

let prompt_colors = PromptInputColors {
    text_primary: colors.text.primary,
    text_muted: colors.text.muted,
    text_dimmed: colors.text.dimmed,
    accent: colors.accent.selected,
    background: colors.background.main,
    border: colors.ui.border,
};

let border_color = if is_focused {
    rgb(prompt_colors.border)
} else {
    rgba((prompt_colors.border << 8) | 0x80)
};
```
