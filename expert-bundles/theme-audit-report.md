# Theme.json Usage Audit Report

## Executive Summary

This audit analyzes which `theme.json` fields are actually being used across the three windows (Main, Notes, AI) versus which are defined but not applied. The key findings:

1. **Colors**: All color fields are used and applied across windows via gpui-component theme mapping
2. **Opacity**: Fully applied to all windows via `theme.get_opacity()`
3. **Drop Shadow**: Applied ONLY to main window (via `create_box_shadows()`)
4. **Vibrancy**: Defined but NEVER applied at runtime - relies on hardcoded `WindowBackgroundAppearance::Blurred`
5. **Padding**: Defined in theme but OVERRIDDEN by config.ts - `theme.get_padding()` is rarely called

---

## 1. Complete Theme.json Field Reference

### Theme Structure (from `src/theme.rs`)

```rust
pub struct Theme {
    pub colors: ColorScheme,
    pub focus_aware: Option<FocusAwareColorScheme>,  // Optional
    pub opacity: Option<BackgroundOpacity>,
    pub drop_shadow: Option<DropShadow>,
    pub vibrancy: Option<VibrancySettings>,
    pub padding: Option<Padding>,
}
```

---

## 2. Field-by-Field Usage Analysis

### 2.1 Colors (ColorScheme)

| Field | Default Value | Main Window | Notes Window | AI Window |
|-------|---------------|-------------|--------------|-----------|
| **Background** | | | | |
| `background.main` | `0x1e1e1e` | ✅ Applied | ✅ Applied | ✅ Applied |
| `background.title_bar` | `0x2d2d30` | ✅ Applied | ✅ Applied | ✅ Applied |
| `background.search_box` | `0x3c3c3c` | ✅ Applied | ✅ Applied | ✅ Applied |
| `background.log_panel` | `0x0d0d0d` | ✅ Applied | ❌ Not used | ❌ Not used |
| **Text** | | | | |
| `text.primary` | `0xffffff` | ✅ Applied | ✅ Applied | ✅ Applied |
| `text.secondary` | `0xcccccc` | ✅ Applied | ✅ Applied | ✅ Applied |
| `text.tertiary` | `0x999999` | ✅ Applied | ✅ Applied | ✅ Applied |
| `text.muted` | `0x808080` | ✅ Applied | ✅ Applied | ✅ Applied |
| `text.dimmed` | `0x666666` | ✅ Applied | ✅ Applied | ✅ Applied |
| **Accent** | | | | |
| `accent.selected` | `0xfbbf24` | ✅ Applied | ✅ Applied | ✅ Applied |
| `accent.selected_subtle` | `0x2a2a2a` | ✅ Applied | ✅ Applied | ✅ Applied |
| **UI** | | | | |
| `ui.border` | `0x464647` | ✅ Applied | ✅ Applied | ✅ Applied |
| `ui.success` | `0x00ff00` | ✅ Applied | ✅ Applied | ✅ Applied |
| `ui.error` | `0xef4444` | ✅ Applied | ✅ Applied | ✅ Applied |
| `ui.warning` | `0xf59e0b` | ✅ Applied | ✅ Applied | ✅ Applied |
| `ui.info` | `0x3b82f6` | ✅ Applied | ✅ Applied | ✅ Applied |
| **Terminal** | | | | |
| `terminal.black` | `0x000000` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.red` | `0xcd3131` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.green` | `0x0dbc79` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.yellow` | `0xe5e510` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.blue` | `0x2472c8` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.magenta` | `0xbc3fbc` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.cyan` | `0x11a8cd` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.white` | `0xe5e5e5` | ✅ TermPrompt | ❌ Not used | ❌ Not used |
| `terminal.bright_*` | (8 colors) | ✅ TermPrompt | ❌ Not used | ❌ Not used |

**How colors are applied:** Each window has a `map_scriptkit_to_gpui_theme()` function that converts Script Kit's ColorScheme to gpui-component's ThemeColor. This is done:
- Main window: `theme::sync_gpui_component_theme(cx)` in `main.rs:1064`
- Notes window: `ensure_theme_initialized(cx)` in `notes/window.rs:1573`
- AI window: `ensure_theme_initialized(cx)` in `ai/window.rs:1627`

---

### 2.2 Opacity (BackgroundOpacity)

| Field | Default Value | Main Window | Notes Window | AI Window |
|-------|---------------|-------------|--------------|-----------|
| `opacity.main` | `0.60` | ✅ Applied | ✅ Applied | ✅ Applied |
| `opacity.title_bar` | `0.65` | ✅ Applied | ✅ Applied | ✅ Applied |
| `opacity.search_box` | `0.70` | ✅ Applied | ✅ Applied | ✅ Applied |
| `opacity.log_panel` | `0.55` | ✅ Applied | ❌ Not used | ❌ Not used |

**How opacity is applied:**
- Accessed via `theme.get_opacity()` or `sk_theme.get_opacity()`
- Applied to colors via `.opacity(opacity.main)` in gpui-component theme mapping
- Used in 15+ render functions across `render_script_list.rs`, `render_builtins.rs`, `render_prompts.rs`

**theme.example.json shows:**
```json
"opacity": {
    "main": 0.6,
    "title_bar": 0.65,
    "search_box": 0.7,
    "log_panel": 0.55
}
```

---

### 2.3 Drop Shadow (DropShadow)

| Field | Default Value | Main Window | Notes Window | AI Window |
|-------|---------------|-------------|--------------|-----------|
| `drop_shadow.enabled` | `true` | ✅ Applied | ❌ Not used | ❌ Not used |
| `drop_shadow.blur_radius` | `20.0` | ✅ Applied | ❌ Not used | ❌ Not used |
| `drop_shadow.spread_radius` | `0.0` | ✅ Applied | ❌ Not used | ❌ Not used |
| `drop_shadow.offset_x` | `0.0` | ✅ Applied | ❌ Not used | ❌ Not used |
| `drop_shadow.offset_y` | `8.0` | ✅ Applied | ❌ Not used | ❌ Not used |
| `drop_shadow.color` | `0x000000` | ✅ Applied | ❌ Not used | ❌ Not used |
| `drop_shadow.opacity` | `0.25` | ✅ Applied | ❌ Not used | ❌ Not used |

**How drop shadow is applied (main window only):**
- Location: `app_impl.rs:2131-2175` (`create_box_shadows()` method)
- Converts theme config to `BoxShadow` GPUI struct
- Applied to main window container

**Why not applied to Notes/AI:**
- Both windows rely on macOS native window chrome (titlebar with traffic lights)
- Box shadows are applied to container divs, not the NSWindow itself
- Notes/AI windows don't have a custom shadow wrapper

---

### 2.4 Vibrancy (VibrancySettings) ⚠️ NOT APPLIED

| Field | Default Value | Main Window | Notes Window | AI Window |
|-------|---------------|-------------|--------------|-----------|
| `vibrancy.enabled` | `true` | ⚠️ Ignored | ⚠️ Ignored | ⚠️ Ignored |
| `vibrancy.material` | `"popover"` | ⚠️ Ignored | ⚠️ Ignored | ⚠️ Ignored |

**Current behavior:**
- All three windows use `WindowBackgroundAppearance::Blurred` (hardcoded)
- The `vibrancy.enabled` and `vibrancy.material` fields are NEVER read at window creation
- The `theme.get_vibrancy()` and `theme.is_vibrancy_enabled()` methods exist but are only used for logging

**Evidence from code:**
```rust
// main.rs:1092 - hardcoded
window_background: WindowBackgroundAppearance::Blurred,

// notes/window.rs:1702 - hardcoded  
window_background: WindowBackgroundAppearance::Blurred,

// ai/window.rs:1700 - hardcoded
window_background: WindowBackgroundAppearance::Blurred,
```

**The only runtime check:**
```rust
// theme.rs:1161-1163 - just logs, doesn't apply
vibrancy_enabled = vibrancy.enabled,
material = %vibrancy.material,
"Theme vibrancy configured"
```

---

### 2.5 Padding (Padding) ⚠️ PARTIALLY USED

| Field | Default Value | Used in theme.rs | Actually Used |
|-------|---------------|------------------|---------------|
| `padding.xs` | `4.0` | ✅ Defined | ❌ Never read |
| `padding.sm` | `8.0` | ✅ Defined | ❌ Never read |
| `padding.md` | `12.0` | ✅ Defined | ❌ Never read |
| `padding.lg` | `16.0` | ✅ Defined | ❌ Never read |
| `padding.xl` | `24.0` | ✅ Defined | ❌ Never read |
| `padding.content_x` | `16.0` | ✅ Defined | ❌ Never read |
| `padding.content_y` | `12.0` | ✅ Defined | ❌ Never read |
| `padding.prompt_x` | `16.0` | ✅ Defined | ❌ Never read |
| `padding.prompt_y` | `12.0` | ✅ Defined | ❌ Never read |
| `padding.item_x` | `16.0` | ✅ Defined | ❌ Never read |
| `padding.item_y` | `8.0` | ✅ Defined | ❌ Never read |

**Why not used:**
- Padding is defined in BOTH `theme.rs` (for theme.json) AND `config.rs` (for config.ts)
- The app uses `config.get_padding()` instead of `theme.get_padding()`
- `config.get_padding()` is used in: `editor.rs`, `term_prompt.rs`, `app_impl.rs`
- `theme.get_padding()` is ONLY used in tests and logging

**The actual padding sources:**
1. **config.ts** (user config): `padding: { top: 8, left: 12, right: 12 }`
2. **theme.json**: Has a `padding` section but it's ignored

---

## 3. Focus-Aware Colors

| Feature | Status |
|---------|--------|
| `focus_aware.focused` | ✅ Supported, rarely used |
| `focus_aware.unfocused` | ✅ Supported, rarely used |

**How it works:**
- `theme.get_colors(is_focused)` returns different ColorScheme based on focus state
- If `focus_aware` is not configured, automatically generates dimmed unfocused colors via `to_unfocused()`
- Used in main window render, but Notes/AI use gpui-component's theme system instead

---

## 4. Summary: What's NOT Applied

### 4.1 Completely Ignored Fields

| Field | Why Ignored | Recommendation |
|-------|-------------|----------------|
| `vibrancy.enabled` | Hardcoded to Blurred | Wire up to `WindowOptions` |
| `vibrancy.material` | macOS NSVisualEffectView not exposed | Would need native code |
| `padding.*` (all) | Overridden by config.ts | Remove from theme.json OR deprecate in config.ts |

### 4.2 Partially Applied Fields

| Field | Applied Where | Missing Where |
|-------|---------------|---------------|
| `drop_shadow.*` | Main window only | Notes, AI windows |
| `background.log_panel` | Main window logs | Notes, AI (no log panel) |
| `terminal.*` | TermPrompt only | Notes, AI (no terminal) |

---

## 5. Recommendations for Hot-Reload Implementation

### High Priority (Easy Wins)

1. **Drop Shadow for Notes/AI**
   - Add `create_box_shadows()` to Notes and AI windows
   - Requires wrapping content in a shadow container

2. **Wire up `vibrancy.enabled`**
   ```rust
   // In window creation
   let vibrancy = theme.get_vibrancy();
   let appearance = if vibrancy.enabled {
       WindowBackgroundAppearance::Blurred
   } else {
       WindowBackgroundAppearance::Opaque
   };
   ```

### Medium Priority (Requires Design Decision)

3. **Consolidate padding: theme.json vs config.ts**
   - Option A: Remove `padding` from theme.json, keep in config.ts
   - Option B: Remove `padding` from config.ts, use theme.json
   - Option C: Make theme.json padding override config.ts defaults

4. **Add ThemeWatcher for hot-reload**
   - ThemeWatcher exists in `watcher.rs` but is NEVER instantiated
   - Wire it up similarly to `config_watcher` and `script_watcher`

### Low Priority (Nice to Have)

5. **`vibrancy.material` support**
   - Requires Objective-C/FFI to set `NSVisualEffectView.material`
   - Values: "hud", "popover", "menu", "sidebar", "content"
   - Complex: GPUI doesn't expose this directly

6. **Per-window theme overrides**
   - Allow Notes/AI to have different accent colors
   - Useful for visual differentiation

---

## 6. theme.example.json vs Actual Usage

### Fields in theme.example.json that ARE applied:
✅ `colors.background.*` (all 4)
✅ `colors.text.*` (all 5)  
✅ `colors.accent.selected`
✅ `colors.ui.border`, `colors.ui.success`
✅ `opacity.*` (all 4)
✅ `drop_shadow.*` (main window only)

### Fields in theme.example.json that are IGNORED:
❌ `vibrancy` section (not in example, but defined in struct)
❌ `padding` section (not in example, but defined in struct)

### Fields defined in struct but NOT in theme.example.json:
- `focus_aware` (optional, rarely used)
- `terminal` (16 ANSI colors, has defaults)
- `ui.error`, `ui.warning`, `ui.info` (have defaults)
- `accent.selected_subtle` (has default)

---

## 7. Files Analyzed

| File | Purpose | Theme Usage |
|------|---------|-------------|
| `src/theme.rs` | Theme struct definitions | Source of truth |
| `src/main.rs` | Main window, entry point | `sync_gpui_component_theme()` |
| `src/notes/window.rs` | Notes window | `map_scriptkit_to_gpui_theme()` |
| `src/ai/window.rs` | AI chat window | `map_scriptkit_to_gpui_theme()` |
| `src/app_impl.rs` | Main window methods | `create_box_shadows()` |
| `src/config.rs` | Config loading | `get_padding()` |
| `theme.example.json` | Example theme file | Reference |
| `src/watcher.rs` | File watchers | ThemeWatcher exists but unused |

---

*Generated by theme-auditor agent for epic cell--9bnr5-mjx71sqvtdd*
