# Vibrancy Transparency Technique

This document describes the technique discovered for achieving beautiful vibrancy blur effects in GPUI on macOS.

## The Problem

When using macOS vibrancy (blur effect), solid background colors block the blur entirely. Even "dark" colors like `#2a2a2a` appear as opaque bands that obscure the beautiful desktop blur beneath.

**Before:** Solid gray backgrounds created harsh contrast against the vibrancy blur.

## The Solution

Use **white (`#ffffff`) at very low opacity (3-8%)** instead of dark colors at higher opacity.

### Why This Works

1. **Vibrancy requires transparency** - The blur effect only shows through transparent areas
2. **White brightens without blocking** - Low-opacity white creates a subtle "lightening" effect
3. **Consistency with Raycast** - This is the same technique Raycast uses for their selection highlights

### The Key Code Pattern

```rust
// Use HexColorExt trait for rgba8 helper
use crate::ui_foundation::HexColorExt;

// White color as base
let highlight_color: u32 = 0xffffff;

// Apply with very low alpha (0x0f = 15 decimal = ~6% opacity)
div().bg(highlight_color.rgba8(0x0f))

// For hover states, use even lower opacity
div().bg(highlight_color.rgba8(0x08))  // ~3% opacity
```

### Opacity Reference Table

| Alpha Hex | Decimal | Percentage | Use Case |
|-----------|---------|------------|----------|
| `0x05` | 5 | ~2% | Barely perceptible hover |
| `0x08` | 8 | ~3% | Subtle hover state |
| `0x0a` | 10 | ~4% | Light hover |
| `0x0f` | 15 | ~6% | Selection highlight |
| `0x14` | 20 | ~8% | Stronger selection |
| `0x1a` | 26 | ~10% | Maximum before looking solid |

### Where Applied

1. **Footer background** (`src/components/prompt_footer.rs`)
   ```rust
   .bg(colors.background.rgba8(0x0f))  // White at 6%
   ```

2. **List item selection** (`src/list_item.rs`)
   - Uses `selected_opacity` from theme (0.06 = 6%)
   - Base color: `accent_selected_subtle` = `0xffffff`

3. **List item hover**
   - Uses `hover_opacity` from theme (0.03 = 3%)

### Theme Configuration

In `src/theme/types.rs`:

```rust
// Default opacity values for vibrancy-aware rendering
impl Default for BackgroundOpacity {
    fn default() -> Self {
        BackgroundOpacity {
            selected: 0.06,  // 6% - subtle brightening
            hover: 0.03,     // 3% - barely visible
            // ...
        }
    }
}

// Default selection color is white for vibrancy support
fn default_selected_subtle() -> HexColor {
    0xffffff  // White - rendered at low opacity
}
```

## Visual Comparison

| Approach | Result |
|----------|--------|
| Gray `#2a2a2a` at 35% | Opaque dark band, blocks blur |
| Gold `#fbbf24` at 8% | Visible warm tint, partially blocks |
| White `#ffffff` at 6% | Subtle brightening, blur shows through |

## GPUI Vibrancy Gotcha (from AGENTS.md)

GPUI intentionally hides the macOS `CAChameleonLayer` (the native tint layer). This means:
- Changing `NSVisualEffectMaterial` has no visible effect on tinting
- We must provide our own tint via semi-transparent backgrounds
- The technique in this document compensates for this GPUI behavior

## Best Practices

1. **Always use white (`0xffffff`) as the base** for selection/hover highlights
2. **Keep opacity between 3-10%** for best vibrancy visibility
3. **Test against different desktop backgrounds** - the blur adapts
4. **Use consistent opacity** for related elements (e.g., footer and selection)
5. **Avoid colored tints** unless brand identity requires it (they block more blur)

## User Theme Override (IMPORTANT)

If your `~/.scriptkit/kit/theme.json` has old values, they will **override** the code defaults!

To enable vibrancy in your theme, update these values:

```json
{
  "colors": {
    "accent": {
      "selected_subtle": "#FFFFFF"  // WHITE, not gray!
    }
  },
  "opacity": {
    "selected": 0.06,  // 6% opacity
    "hover": 0.03      // 3% opacity
  }
}
```

Or delete `~/.scriptkit/kit/theme.json` entirely to use the new defaults.
