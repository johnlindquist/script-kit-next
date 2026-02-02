# Raycast Light Theme Vibrancy POC

This POC demonstrates how to achieve macOS vibrancy (frosted glass blur) effect using GPUI, matching Raycast's light theme appearance.

## Run

```bash
cargo run --bin vibrancy-poc
```

## Key Concepts

### 1. Window Background Appearance

The most critical setting is `WindowBackgroundAppearance::Blurred`:

```rust
let window_options = WindowOptions {
    window_background: WindowBackgroundAppearance::Blurred,
    // ...
};
```

This enables macOS NSVisualEffectView, which provides the native blur effect.

### 2. Semi-Transparent Background Colors

Use RGBA colors with alpha < 1.0 to let the blur show through:

```rust
// 85% opacity - light gray that lets blur through
let container_bg = rgba(0xFAFAFAD9);  // #FAFAFA at 85% (0xD9 = 217/255)

// 90% opacity - slightly more opaque for input areas
let input_area_bg = rgba(0xFFFFFFE6);  // white at 90% (0xE6 = 230/255)

// 80% opacity - for selected items
let selected_bg = rgba(0xE8E8E8CC);    // light gray at 80% (0xCC = 204/255)
```

### 3. Light Color Palette

Raycast's light theme uses these approximate colors:

| Element | Color | Hex |
|---------|-------|-----|
| Container BG | Light gray @ 85% | `#FAFAFA` / `rgba(0xFAFAFAD9)` |
| Input BG | White @ 90% | `#FFFFFF` / `rgba(0xFFFFFFE6)` |
| Selected BG | Gray @ 80% | `#E8E8E8` / `rgba(0xE8E8E8CC)` |
| Primary Text | Near black | `#1A1A1A` |
| Secondary Text | Medium gray | `#6B6B6B` |
| Hint Text | Light gray | `#9B9B9B` |
| Separator | Light gray | `#E0E0E0` |
| Border | Subtle transparent | `rgba(0xD0D0D040)` |

### 4. No Titlebar

For the clean Raycast-like appearance:

```rust
let window_options = WindowOptions {
    titlebar: None,
    // ...
};
```

## File Structure

- `main.rs` - The complete POC source (also at `src/bin/vibrancy-poc.rs`)

## Notes

- The vibrancy effect is macOS-specific (NSVisualEffectView)
- Background transparency must be balanced - too transparent loses readability, too opaque loses the blur effect
- The blur shows what's behind the window, so test with different backgrounds
