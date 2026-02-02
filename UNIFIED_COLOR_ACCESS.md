# Unified Color Access Pattern

## Summary

This document describes the unified color access system that eliminates the dual-path pattern for accessing theme colors and design tokens.

## Problem

Previously, code had to check `is_default_design` and route between two different color sources:
- **Default design**: `theme.colors.text.muted`
- **Other designs**: `design_colors.text_muted`

This dual-path pattern appeared throughout the codebase, creating:
- **Cognitive load**: Every color access required a conditional check
- **Maintenance burden**: Changes had to be made in two places
- **Inconsistency risk**: Easy to forget one path when updating colors

### Example of Old Pattern

```rust
let is_default_design = self.current_design == DesignVariant::Default;
let tokens = get_tokens(self.current_design);
let design_colors = tokens.colors();
let design_typography = tokens.typography();

let empty_text_color = if is_default_design {
    theme.colors.text.muted  // Direct theme access
} else {
    design_colors.text_muted // Design token access
};

let empty_font_family = if is_default_design {
    ".AppleSystemUIFont"
} else {
    design_typography.font_family
};
```

## Solution: ColorResolver, TypographyResolver, SpacingResolver

Created three unified resolver types in `src/theme/color_resolver.rs`:

### 1. ColorResolver

Provides unified access to all colors:

```rust
let color_resolver = ColorResolver::new(&theme, current_design);
let empty_text_color = color_resolver.empty_text_color();
let accent = color_resolver.accent;
let border = color_resolver.border;
```

**Available colors:**
- Background: `background`, `background_secondary`, `background_tertiary`, `background_selected`, `background_hover`
- Text: `text_primary`, `text_secondary`, `text_muted`, `text_dimmed`, `text_on_accent`
- Accent: `accent`, `accent_secondary`, `success`, `warning`, `error`
- Border: `border`, `border_subtle`, `border_focus`
- Shadow: `shadow`

**Convenience methods:**
- `empty_text_color()` → `text_muted`
- `primary_text_color()` → `text_primary`
- `main_background()` → `background`
- `selection_background()` → `background_selected`
- `primary_accent()` → `accent`
- `border_color()` → `border`

### 2. TypographyResolver

Provides unified access to typography settings:

```rust
let typography_resolver = TypographyResolver::new(&theme, current_design);
let font = typography_resolver.primary_font();
let size = typography_resolver.font_size_xl;
```

**Available properties:**
- `font_family`, `font_family_mono`
- `font_size_xs`, `font_size_sm`, `font_size_md`, `font_size_lg`, `font_size_xl`

**Convenience methods:**
- `primary_font()` → `font_family`
- `mono_font()` → `font_family_mono`

### 3. SpacingResolver

Provides unified access to spacing values:

```rust
let spacing_resolver = SpacingResolver::new(current_design);
let padding = spacing_resolver.padding_md;
let gap = spacing_resolver.gap_md;
let margin = spacing_resolver.margin_lg;
```

**Available properties:**
- Padding: `padding_xs`, `padding_sm`, `padding_md`, `padding_lg`, `padding_xl`
- Gap: `gap_sm`, `gap_md`, `gap_lg`
- Margin: `margin_sm`, `margin_md`, `margin_lg`

## Usage Pattern

### Before (Dual Path)

```rust
// Get design tokens
let tokens = get_tokens(self.current_design);
let design_colors = tokens.colors();
let design_typography = tokens.typography();
let design_spacing = tokens.spacing();

// Check if default design
let is_default_design = self.current_design == DesignVariant::Default;

// Conditionally access colors
let text_primary = if is_default_design {
    theme.colors.text.primary
} else {
    design_colors.text_primary
};

let font_family = if is_default_design {
    ".AppleSystemUIFont"
} else {
    design_typography.font_family
};
```

### After (Unified)

```rust
// Create resolvers
let color_resolver = ColorResolver::new(&theme, self.current_design);
let typography_resolver = TypographyResolver::new(&theme, self.current_design);
let spacing_resolver = SpacingResolver::new(self.current_design);

// Direct access - no conditionals needed
let text_primary = color_resolver.text_primary;
let font_family = typography_resolver.font_family;
let padding = spacing_resolver.padding_md;
```

## Benefits

1. **Single source of truth**: One API for color access regardless of design variant
2. **Reduced cognitive load**: No need to check `is_default_design` everywhere
3. **Simpler code**: Fewer lines, easier to read and maintain
4. **Type-safe**: All fields are strongly typed
5. **Copy-able**: Resolvers are `Copy` for use in closures
6. **Incremental adoption**: Can be adopted gradually without breaking existing code

## Implementation Details

### How It Works

1. **Default variant** (DesignVariant::Default):
   - Resolvers extract colors from `theme.colors.*`
   - Maps theme structure to unified resolver fields

2. **Other variants** (Minimal, RetroTerminal, etc.):
   - Resolvers extract colors from `get_tokens(variant).colors()`
   - Uses design token structure directly

3. **Automatic routing**:
   - `ColorResolver::new(&theme, variant)` checks variant internally
   - No conditional logic needed at call sites

### File Structure

- **Definition**: `/src/theme/color_resolver.rs`
- **Export**: `/src/theme/mod.rs` (re-exported as public API)
- **Tests**: Included in `color_resolver.rs` (6 tests, all passing)

## Migration Status

### Completed

- ✅ `src/theme/color_resolver.rs` - Created unified resolvers
- ✅ `src/theme/mod.rs` - Exported resolvers
- ✅ `src/render_script_list.rs` - Migrated main usage (empty state, header, footer, divider)
- ✅ `src/protocol/io.rs` - Fixed unrelated compilation error

### Remaining Files with Dual Path (for incremental migration)

These files still use the old pattern and can be migrated incrementally:

- `src/render_prompts/arg.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/editor.rs`
- `src/render_prompts/form.rs`
- `src/render_prompts/other.rs`
- `src/render_prompts/path.rs`
- `src/app_render.rs`
- `src/render_builtins.rs`
- `src/ui_foundation.rs`

## Testing

All tests pass:

```bash
$ cargo test --lib color_resolver
running 6 tests
test theme::color_resolver::tests::test_all_variants_have_valid_colors ... ok
test theme::color_resolver::tests::test_color_resolver_default_variant ... ok
test theme::color_resolver::tests::test_color_resolver_minimal_variant ... ok
test theme::color_resolver::tests::test_color_resolver_semantic_methods ... ok
test theme::color_resolver::tests::test_typography_resolver_default ... ok
test theme::color_resolver::tests::test_typography_resolver_retro_terminal ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured
```

## Future Improvements

1. **Complete migration**: Migrate all remaining files to use resolvers
2. **Remove dual path**: Once all files migrated, remove old conditional pattern
3. **Deprecate direct access**: Mark `design_colors`, `design_typography` as deprecated
4. **Visual resolver**: Add `VisualResolver` for border radius, shadows, animations
5. **Cache resolvers**: Consider caching resolvers in app state to avoid re-creation

## References

- Analysis: `/poc/consistency-analysis/10-gpui-patterns.md` (lines 151-185)
- Design tokens: `/src/designs/traits.rs`
- Theme types: `/src/theme/types.rs`
