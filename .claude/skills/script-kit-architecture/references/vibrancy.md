# macOS Vibrancy

## GPUI Vibrancy Gotcha (CRITICAL)

**Problem:** App looks washed out / too transparent over white backgrounds, even after configuring `NSVisualEffectView` materials correctly.

**Root Cause:** GPUI intentionally hides the macOS `CAChameleonLayer` (the native tint layer that provides dark tinting on top of blur):

```rust
// blurred_view_update_layer in GPUI hides the tint:
if class_name.isEqualToString("CAChameleonLayer") {
    let _: () = msg_send![layer, setHidden: YES];
}
```

This means:
- Changing `NSVisualEffectMaterial` (SIDEBAR, HUD_WINDOW, POPOVER, etc.) has **no visible effect**
- The material's tint color is never applied because CAChameleonLayer is hidden
- Native macOS apps like Raycast/Spotlight get automatic dark tinting; we do not

**The Fix:** Provide our own dark tint via GPUI theme colors at **70-85% opacity**:

```rust
// src/theme/gpui_integration.rs
let main_bg = if vibrancy_enabled {
    let tint_alpha = opacity.main.clamp(0.70, 0.85);
    with_vibrancy(colors.background.main, tint_alpha)
} else {
    hex_to_hsla(colors.background.main)
};
```

**What doesn't work:**
- Changing material from SIDEBAR → HUD_WINDOW → ULTRA_DARK → POPOVER
- Changing appearance from DarkAqua → VibrantDark
- Setting `window.backgroundColor = clearColor`
- Setting `window.isOpaque = false`
- Recursively configuring all NSVisualEffectViews

All of these are correct but won't produce visible tinting.

**Key files:**
- `src/theme/gpui_integration.rs:49-54` - The fix
- `src/platform.rs` - NSVisualEffectView configuration
- `expert-bundles/vibrancy-*.md` - Full documentation

## Vibrancy Selection/Hover Colors

**Problem:** Selection highlights appear as opaque bands while footer shows vibrancy through.

**Root Cause:**
1. User's `~/.scriptkit/kit/theme.json` overrides code defaults with old values like `selected_subtle: "#2A2A2A"`
2. Serde defaults vs struct defaults mismatch

**The Fix:** For vibrancy-compatible selection highlights:
- Use **white (`#FFFFFF`)** as base color for `selected_subtle`
- Use **low opacity (6-33%)** via `opacity.selected` and `opacity.hover`
- Combination `white @ low opacity` creates subtle brightening that lets blur show through

**Code pattern:**
```rust
// In types.rs - serde defaults MUST match struct Default impl
fn default_selected_opacity() -> f32 {
    0.33
}

// In list_item.rs - rgba computation
let selected_bg = rgba((colors.accent_selected_subtle << 8) | selected_alpha);
```

**User theme.json fix:**
```json
{
  "colors": { "accent": { "selected_subtle": "#FFFFFF" } },
  "opacity": { "selected": 0.33, "hover": 0.15 }
}
```

## Vibrancy Requirements

1. `WindowBackgroundAppearance::Blurred` in window options
2. Semi-transparent background colors (70-85% alpha)

Missing either = no vibrancy effect.

## Consistent Opacity Levels

- Selection highlight: ~50% alpha (`0x80` suffix)
- Hover highlight: ~25% alpha (`0x40` suffix)
- Disabled state: ~30% opacity via `.opacity(0.3)`

## SVG Icon Theming

Always use `stroke='currentColor'` (not hardcoded colors) so icons inherit theme's text color.
