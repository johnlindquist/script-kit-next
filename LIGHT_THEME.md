# Light Theme Implementation Plan

## Executive Summary

This document provides a comprehensive analysis and implementation plan for adding proper light theme support to Script Kit GPUI. The investigation was conducted by 10 specialized agents examining every aspect of the theming system.

**Bottom Line**: The theme architecture is well-designed and already contains light mode color definitions. However, there are **12 critical blockers** that must be fixed for light themes to work correctly.

---

## Table of Contents

1. [Critical Blockers](#1-critical-blockers-must-fix)
2. [Architecture Overview](#2-architecture-overview)
3. [Color System Analysis](#3-color-system-analysis)
4. [Vibrancy & Transparency](#4-vibrancy--transparency)
5. [Component-by-Component Guide](#5-component-by-component-guide)
6. [Hardcoded Colors Inventory](#6-hardcoded-colors-inventory)
7. [Implementation Checklist](#7-implementation-checklist)
8. [Theme JSON Reference](#8-theme-json-reference)
9. [Testing Strategy](#9-testing-strategy)

---

## 1. Critical Blockers (MUST FIX)

### Blocker 1: ThemeMode Hardcoded to Dark
**File**: `src/theme/gpui_integration.rs:184`
```rust
theme.mode = ThemeMode::Dark; // Script Kit uses dark mode by default
```
**Impact**: gpui-component always renders with dark mode styling regardless of actual theme colors.
**Fix**: Detect theme luminance or add explicit `mode` field to theme.json:
```rust
theme.mode = if is_light_theme(&sk_theme) { ThemeMode::Light } else { ThemeMode::Dark };
```

### Blocker 2: Window Appearance Forced to VibrantDark
**File**: `src/platform.rs:1038-1044`
```rust
let vibrant_dark: id = msg_send![
    class!(NSAppearance),
    appearanceNamed: NSAppearanceNameVibrantDark
];
```
**Impact**: NSVisualEffectView always uses dark rendering path, causing incorrect vibrancy on light themes.
**Fix**: Dynamically select appearance:
```rust
let appearance_name = if is_light_theme {
    NSAppearanceNameVibrantLight // or NSAppearanceNameAqua
} else {
    NSAppearanceNameVibrantDark
};
```

### Blocker 3: White Selection Color (`selected_subtle`)
**File**: `src/theme/types.rs:310-312`
```rust
fn default_selected_subtle() -> HexColor {
    0xffffff // White - rendered at very low opacity for subtle brightening
}
```
**Impact**: White at low opacity is INVISIBLE on white/light backgrounds.
**Fix**: For light themes, `selected_subtle` must be dark (e.g., `0x000000` black):
- Dark theme: `0xffffff` (white brightening)
- Light theme: `0x000000` (black darkening)

### Blocker 4: Hardcoded Tint Alpha (0.37)
**File**: `src/theme/gpui_integration.rs:62`
```rust
let tint_alpha = 0.37; // Tuned for dark mode
```
**Impact**: Light theme backgrounds become too transparent and washed out.
**Fix**: Use higher alpha for light themes:
```rust
let tint_alpha = if is_light_theme { 0.65 } else { 0.37 };
```

### Blocker 5: Opacity Values Designed for Dark Backgrounds
**File**: `src/theme/types.rs:124-143` (BackgroundOpacity defaults)
- `selected: 0.12` - Nearly invisible on light backgrounds
- `hover: 0.07` - Invisible on light backgrounds
- `main: 0.30` - Too transparent for light mode

**Fix**: Light themes need higher opacity values:
| Field | Dark Theme | Light Theme |
|-------|------------|-------------|
| main | 0.30 | 0.65 |
| selected | 0.12 | 0.08 (with black base) |
| hover | 0.07 | 0.05 (with black base) |

### Blocker 6: White Hover Overlays
**File**: `src/components/button.rs:200`
```rust
rgba(0xffffff26) // 15% white overlay - invisible on white
```
**Impact**: Button hover states invisible on light backgrounds.
**Fix**: Use theme-aware overlay:
```rust
let overlay = if is_light_theme {
    rgba(0x00000015) // Black overlay
} else {
    rgba(0xffffff26) // White overlay
};
```

### Blocker 7: Black Overlay Backgrounds
**Files**:
- `src/app_shell/shell.rs:265` - `rgba(0x00000080)` footer
- `src/notes/window.rs:1579,1623` - overlays
- `src/notes/browse_panel.rs:426` - panel overlay

**Impact**: Creates dark bands on light theme.
**Fix**: Add semantic overlay colors to theme.

### Blocker 8: to_unfocused() Blends Toward Gray
**File**: `src/theme/types.rs:741-754`
```rust
fn darken_hex(color: HexColor) -> HexColor {
    let gray = 0x80u32; // Mid-gray
    // Blends 30% toward gray
}
```
**Impact**: For light themes, blending white toward gray darkens it incorrectly.
**Fix**: Light themes should blend toward white or use separate unfocused colors.

### Blocker 9: Terminal Background/Foreground Defaults
**File**: `src/terminal/theme_adapter.rs:264-268`
```rust
let foreground = hex_to_rgb(0xd4d4d4); // Light gray text
let background = hex_to_rgb(0x1e1e1e); // Dark background
```
**Impact**: Terminal always renders with dark defaults.
**Fix**: Use theme colors: `colors.text.primary` and `colors.background.main`.

### Blocker 10: HudColors Only Has dark_default()
**File**: `src/hud_manager.rs:71`
```rust
fn dark_default() -> Self { ... }
```
**Impact**: HUD notifications always use dark styling.
**Fix**: Add `light_default()` method and use based on theme.

### Blocker 11: Chat Prompt White Overlays
**File**: `src/prompts/chat.rs:1514-1515`
```rust
rgba((0xFFFFFF << 8) | 0x15) // White container background
rgba((0xFFFFFF << 8) | 0x28) // White copy hover
```
**Impact**: Invisible on light backgrounds.
**Fix**: Use theme-derived colors.

### Blocker 12: AI Window Hardcoded Colors
**File**: `src/ai/window.rs:3124-3125`
```rust
hsla(45.0 / 360.0, 0.9, 0.55, 1.0) // Gold button
hsla(0.0, 0.0, 0.1, 1.0)           // Dark text
```
**Impact**: Button text may be invisible on certain backgrounds.
**Fix**: Use `colors.text.on_accent` for text on colored backgrounds.

---

## 2. Architecture Overview

### Theme Data Flow
```
theme.json (~/.scriptkit/kit/theme.json)
    ↓
load_theme() [src/theme/types.rs:1072]
    ↓
Theme struct
    ├── colors: ColorScheme
    │   ├── background: BackgroundColors
    │   ├── text: TextColors
    │   ├── accent: AccentColors
    │   ├── ui: UIColors
    │   └── terminal: TerminalColors
    ├── opacity: BackgroundOpacity
    ├── vibrancy: VibrancySettings
    └── fonts: FontConfig
    ↓
sync_gpui_component_theme() [src/theme/gpui_integration.rs:171]
    ↓
gpui_component::Theme (global)
```

### Key Files
| File | Purpose |
|------|---------|
| `src/theme/types.rs` | All struct definitions, dark/light defaults, load_theme() |
| `src/theme/gpui_integration.rs` | Maps to gpui-component ThemeColor |
| `src/theme/semantic.rs` | SemanticColors with dark/light variants |
| `src/theme/helpers.rs` | ListItemColors, InputFieldColors extraction |
| `src/platform.rs` | macOS vibrancy, NSVisualEffectView config |

### Existing Light Mode Support
The codebase already has light mode color definitions:

**ColorScheme::light_default()** (`src/theme/types.rs:707-736`):
- Background: `main: 0xffffff`, `title_bar: 0xf3f3f3`
- Text: `primary: 0x000000`, `secondary: 0x333333`
- Accent: `selected: 0x0078d4` (blue instead of gold)

**SemanticColors::light()** (`src/theme/semantic.rs:270-306`):
- Complete light mode semantic colors
- Blue accent instead of gold

**TerminalColors::light_default()** (`src/theme/types.rs:470-489`):
- Light-appropriate ANSI colors

---

## 3. Color System Analysis

### Background Colors

| Role | Dark Default | Light Default | Notes |
|------|-------------|---------------|-------|
| main | `#1E1E1E` | `#FFFFFF` | Primary window background |
| title_bar | `#2D2D30` | `#F3F3F3` | Header areas |
| search_box | `#3C3C3C` | `#ECECEC` | Input field backgrounds |
| log_panel | `#0D0D0D` | `#FAFAFA` | Terminal/log areas |

### Text Colors

| Role | Dark Default | Light Default | Usage |
|------|-------------|---------------|-------|
| primary | `#FFFFFF` | `#000000` | Main text, headings |
| secondary | `#CCCCCC` | `#333333` | List items, descriptions |
| tertiary | `#999999` | `#666666` | Italic, bullets |
| muted | `#808080` | `#999999` | Placeholders, hints |
| dimmed | `#666666` | `#CCCCCC` | Shortcuts, metadata |
| on_accent | `#FFFFFF` | `#FFFFFF` | Text on colored backgrounds |

### Accent Colors

| Role | Dark Default | Light Default | Usage |
|------|-------------|---------------|-------|
| selected | `#FBBF24` (gold) | `#0078D4` (blue) | Primary accent |
| selected_subtle | `#FFFFFF` | `#000000` | Selection/hover backgrounds |

**CRITICAL**: `selected_subtle` must be inverted for light mode!

### UI Colors

| Role | Dark Default | Light Default | Usage |
|------|-------------|---------------|-------|
| border | `#464647` | `#D0D0D0` | Borders, dividers |
| success | `#00FF00` | `#22C55E` | Success states |
| error | `#EF4444` | `#DC2626` | Error states |
| warning | `#F59E0B` | `#D97706` | Warning states |
| info | `#3B82F6` | `#2563EB` | Info states |

---

## 4. Vibrancy & Transparency

### How Vibrancy Works

1. **GPUI creates NSVisualEffectView** with `WindowBackgroundAppearance::Blurred`
2. **Script Kit swizzles BlurredView** to preserve CAChameleonLayer (tint)
3. **Semi-transparent backgrounds** let blur show through
4. **NSVisualEffectMaterial** controls blur intensity (POPOVER=6 is default)

### Current Vibrancy Settings (Dark Theme Optimized)

```rust
// gpui_integration.rs:62
let tint_alpha = 0.37; // Works well on dark backgrounds

// BackgroundOpacity defaults
main: 0.30
selected: 0.12
hover: 0.07
```

### Light Theme Vibrancy Requirements

| Setting | Dark Value | Light Value | Reason |
|---------|-----------|-------------|--------|
| tint_alpha | 0.37 | 0.60-0.80 | White needs more opacity to be visible |
| main opacity | 0.30 | 0.45-0.65 | Prevent washed-out appearance |
| selected_subtle | 0xFFFFFF | 0x000000 | Opposite for visibility |
| selected opacity | 0.12 | 0.06-0.10 | Black needs less opacity |
| hover opacity | 0.07 | 0.04-0.06 | Black needs less opacity |
| Window appearance | VibrantDark | VibrantLight | Proper macOS rendering |

### Vibrancy Material Options

| Material | Value | Best For |
|----------|-------|----------|
| POPOVER | 6 | Dark themes (current default) |
| TITLEBAR | 3 | Light themes (used in theme-light.json) |
| SIDEBAR | 7 | Alternative for light |
| HUD_WINDOW | 13 | High contrast dark |

---

## 5. Component-by-Component Guide

### List Items (`src/list_item.rs`)

**Current Implementation**:
- Normal: Transparent background, `text_secondary`
- Hover: `selected_subtle` at 7% opacity
- Selected: `selected_subtle` at 12% opacity, `text_primary`
- 3px accent bar on left when selected

**Light Theme Changes**:
- Use black `selected_subtle` (0x000000)
- Reduce opacity values (black is more visible)

### Input Fields (`src/components/prompt_input.rs`)

**Current Implementation**:
- Background: `search_box` color at opacity
- Text: `text_primary`
- Placeholder: `text_muted`
- Cursor: `text_primary`

**Light Theme**: Works correctly if theme colors are set.

### Buttons (`src/components/button.rs`)

**Current Implementation**:
- Primary: `accent.selected` background, `background.main` text
- Ghost: Transparent, `text_primary`
- Hover: White overlay at 15%

**Light Theme Changes**:
- Line 200: Change `rgba(0xffffff26)` to theme-aware overlay

### Scrollbar (`src/components/scrollbar.rs`)

**Current Implementation**:
- Track: `background.main`
- Thumb: `text_dimmed` at 40%
- Hover: `text_muted` at 60%

**Light Theme**: Works correctly if theme colors are set.

### Toast (`src/components/toast.rs`)

**Current Implementation**:
- Background at 94% opacity
- Color-coded left border (success/error/warning/info)

**Light Theme**: Uses theme colors, should work.

### Terminal (`src/term_prompt.rs`)

**Current Implementation**:
- Uses ThemeAdapter with ANSI colors
- Default foreground/background hardcoded

**Light Theme Changes**:
- Use `TerminalColors::light_default()` for light themes
- Fix hardcoded defaults in `theme_adapter.rs:264-268`

### Notes Window (`src/notes/window.rs`)

**Current Implementation**:
- Uses same theme system
- Has hardcoded black overlays

**Light Theme Changes**:
- Fix lines 1579, 1623: Replace `rgba(0x00000080)` with semantic color

### AI Window (`src/ai/window.rs`)

**Current Implementation**:
- Uses theme colors mostly
- Some hardcoded HSLA for buttons

**Light Theme Changes**:
- Fix lines 3124-3125: Use theme colors

---

## 6. Hardcoded Colors Inventory

### Critical (MUST FIX for light theme)

| File | Line | Value | Fix |
|------|------|-------|-----|
| `src/app_shell/shell.rs` | 265 | `rgba(0x00000080)` | Semantic overlay |
| `src/components/button.rs` | 200 | `rgba(0xffffff26)` | Theme-aware overlay |
| `src/notes/window.rs` | 1579 | `gpui::rgba(0x00000080)` | Semantic overlay |
| `src/notes/window.rs` | 1623 | `gpui::rgba(0x00000080)` | Semantic overlay |
| `src/notes/browse_panel.rs` | 426 | `gpui::rgba(0x00000080)` | Semantic overlay |
| `src/prompts/chat.rs` | 1514 | `rgba((0xFFFFFF << 8) \| 0x15)` | Theme color |
| `src/prompts/chat.rs` | 1515 | `rgba((0xFFFFFF << 8) \| 0x28)` | Theme color |
| `src/ai/window.rs` | 3124 | `hsla(45.0/360.0, 0.9, 0.55, 1.0)` | `accent.selected` |
| `src/ai/window.rs` | 3125 | `hsla(0.0, 0.0, 0.1, 1.0)` | `text.on_accent` |

### Medium (Suboptimal on light theme)

| File | Line | Value | Notes |
|------|------|-------|-------|
| `src/terminal/theme_adapter.rs` | 264-268 | Hardcoded fg/bg | Use theme colors |
| `src/ai/window.rs` | 829 | `hsla(0.0, 0.7, 0.5, 0.2)` | Red hover |

### Acceptable (Test/Demo only)

- All files in `src/stories/` - Storybook demos
- All files in `src/storybook/` - Storybook browser
- All files in `src/designs/` - Experimental designs
- `src/icons/render.rs:217-234` - Test theme provider

---

## 7. Implementation Checklist

### Phase 1: Core Theme System
- [ ] Add `is_light_mode: bool` field to Theme or detect from colors
- [ ] Fix `sync_gpui_component_theme()` to set correct ThemeMode
- [ ] Add `light_default()` opacity preset to BackgroundOpacity
- [ ] Make `selected_subtle` default depend on theme mode

### Phase 2: Platform/Vibrancy
- [ ] Fix `configure_window_vibrancy_material()` to use VibrantLight for light themes
- [ ] Make tint_alpha dynamic in gpui_integration.rs
- [ ] Add light mode material option (TITLEBAR=3 works well)

### Phase 3: Fix Hardcoded Colors
- [ ] `src/components/button.rs:200` - Theme-aware hover overlay
- [ ] `src/app_shell/shell.rs:265` - Theme footer overlay
- [ ] `src/notes/window.rs:1579,1623` - Theme overlays
- [ ] `src/notes/browse_panel.rs:426` - Theme overlay
- [ ] `src/prompts/chat.rs:1514-1515` - Theme backgrounds
- [ ] `src/ai/window.rs:3124-3125` - Theme button colors

### Phase 4: Secondary Systems
- [ ] Add `HudColors::light_default()` to hud_manager.rs
- [ ] Fix terminal defaults in theme_adapter.rs
- [ ] Fix `to_unfocused()` to handle light themes correctly

### Phase 5: Add Semantic Overlay System
- [ ] Add `overlay_light` and `overlay_dark` to UIColors
- [ ] Create `theme_overlay(opacity)` helper function
- [ ] Replace all hardcoded overlays with semantic colors

### Phase 6: Testing
- [ ] Test with theme-light.json
- [ ] Test vibrancy on light desktop backgrounds
- [ ] Test all prompt types in light mode
- [ ] Test Notes and AI windows in light mode
- [ ] Test HUD notifications in light mode
- [ ] Visual regression screenshots

---

## 8. Theme JSON Reference

### Complete Light Theme Example

```json
{
  "colors": {
    "background": {
      "main": "#FFFFFF",
      "title_bar": "#F3F3F3",
      "search_box": "#ECECEC",
      "log_panel": "#F5F5F5"
    },
    "text": {
      "primary": "#000000",
      "secondary": "#333333",
      "tertiary": "#666666",
      "muted": "#808080",
      "dimmed": "#999999",
      "on_accent": "#FFFFFF"
    },
    "accent": {
      "selected": "#0078D4",
      "selected_subtle": "#000000"
    },
    "ui": {
      "border": "#D0D0D0",
      "success": "#22C55E",
      "error": "#DC2626",
      "warning": "#D97706",
      "info": "#2563EB"
    },
    "terminal": {
      "black": "#000000",
      "red": "#CD3131",
      "green": "#00BC00",
      "yellow": "#949800",
      "blue": "#0451A5",
      "magenta": "#BC05BC",
      "cyan": "#0598BC",
      "white": "#555555",
      "bright_black": "#666666",
      "bright_red": "#CD3131",
      "bright_green": "#14CE14",
      "bright_yellow": "#B5BA00",
      "bright_blue": "#0451A5",
      "bright_magenta": "#BC05BC",
      "bright_cyan": "#0598BC",
      "bright_white": "#A5A5A5"
    }
  },
  "opacity": {
    "main": 0.65,
    "title_bar": 0.70,
    "search_box": 0.75,
    "log_panel": 0.70,
    "selected": 0.06,
    "hover": 0.04,
    "input": 0.60,
    "panel": 0.50,
    "dialog": 0.40
  },
  "drop_shadow": {
    "enabled": true,
    "blur_radius": 20.0,
    "spread_radius": 0.0,
    "offset_x": 0.0,
    "offset_y": 8.0,
    "color": "#000000",
    "opacity": 0.12
  },
  "vibrancy": {
    "enabled": true,
    "material": "titlebar"
  }
}
```

### Key Differences from Dark Theme

| Setting | Dark | Light | Reason |
|---------|------|-------|--------|
| `text.primary` | #FFFFFF | #000000 | Inverted for contrast |
| `accent.selected` | #FBBF24 | #0078D4 | Gold→Blue for visibility |
| `accent.selected_subtle` | #FFFFFF | #000000 | Inverted for selection visibility |
| `opacity.main` | 0.30 | 0.65 | Higher for light visibility |
| `opacity.selected` | 0.12 | 0.06 | Lower (black more visible) |
| `vibrancy.material` | popover | titlebar | Better for light backgrounds |
| `drop_shadow.opacity` | 0.25 | 0.12 | Lighter shadow for light theme |

---

## 9. Testing Strategy

### Manual Testing Checklist

1. **Basic Appearance**
   - [ ] Main window background is light
   - [ ] Text is readable (dark on light)
   - [ ] Borders are visible

2. **Selection & Hover**
   - [ ] List item selection visible (subtle darkening)
   - [ ] List item hover visible
   - [ ] Button hover visible

3. **Vibrancy**
   - [ ] Blur effect works on light desktop
   - [ ] Window doesn't look washed out
   - [ ] Content behind window is blurred

4. **Components**
   - [ ] Input fields readable
   - [ ] Buttons styled correctly
   - [ ] Scrollbars visible
   - [ ] Toasts readable

5. **Secondary Windows**
   - [ ] Notes window light themed
   - [ ] AI window light themed
   - [ ] Actions dialog light themed

### Automated Testing

```bash
# Build and test
cargo check && cargo clippy --all-targets -- -D warnings && cargo test

# Visual test with light theme
cp ~/.scriptkit/kit/theme-light.json ~/.scriptkit/kit/theme.json
echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Capture screenshot for comparison
# (Use captureScreenshot() SDK function in test script)
```

### Test Scripts

Create `tests/smoke/light-theme-test.ts`:
```typescript
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

// Test basic prompt rendering
await div(`<div class="p-4">
  <h1>Light Theme Test</h1>
  <p>Text should be dark on light background</p>
</div>`);

await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const shot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(
  join(dir, `light-theme-${Date.now()}.png`),
  Buffer.from(shot.data, 'base64')
);

process.exit(0);
```

---

## Summary

The theme system architecture is solid and already contains light mode color definitions. The main work is:

1. **Fix 12 critical blockers** (ThemeMode, vibrancy, hardcoded colors)
2. **Adjust opacity values** for light backgrounds
3. **Invert selection technique** (black on light instead of white on dark)
4. **Test thoroughly** with visual verification

Estimated scope: ~15-20 files need modification, with most changes being small fixes to use theme colors instead of hardcoded values.