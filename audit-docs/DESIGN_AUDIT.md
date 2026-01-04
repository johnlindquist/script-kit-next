# Design Audit Report

**Date:** January 3, 2026  
**Scope:** Cross-window design consistency across Main Window, Notes Window, and AI Window  
**Status:** Initial audit complete, fixes pending

---

## Executive Summary

This audit identified **17 design inconsistencies** across the Script Kit GPUI codebase. The main issues fall into three categories:

1. **Duplicated code** - Theme mapping functions are copy-pasted across 3 files
2. **Dimension mismatches** - Heights, widths, and spacing values differ between similar components
3. **Inconsistent theming approaches** - Main window uses design tokens, secondary windows use gpui-component theme directly

**Priority recommendation:** Consolidate theme mapping into `src/theme.rs` (single source of truth) before fixing individual dimension inconsistencies.

---

## Table of Contents

1. [Critical Issues](#1-critical-issues)
2. [Dimension Inconsistencies](#2-dimension-inconsistencies)
3. [Theme System Inconsistencies](#3-theme-system-inconsistencies)
4. [Spacing Inconsistencies](#4-spacing-inconsistencies)
5. [Component-Level Findings](#5-component-level-findings)
6. [Recommendations](#6-recommendations)
7. [Fix Priority Matrix](#7-fix-priority-matrix)

---

## 1. Critical Issues

### 1.1 Duplicated Theme Mapping Functions

The `map_scriptkit_to_gpui_theme()` function is defined in **three separate files**:

| File | Lines | Notes |
|------|-------|-------|
| `src/theme.rs` | 1287-1387 | **Canonical source** - well documented |
| `src/notes/window.rs` | (duplicated) | Copy of theme.rs version |
| `src/ai/window.rs` | (duplicated) | Copy with slightly different caret color |

**Problem:** Changes to theme mapping must be made in 3 places. The AI window has a different caret color mapping (cyan) that may be intentional or accidental.

**Fix:** Remove duplicates from `notes/window.rs` and `ai/window.rs`, import from `src/theme.rs`.

### 1.2 Design Token Usage Inconsistency

| Window | Theming Approach |
|--------|------------------|
| Main Window | Uses `designs::DesignTokens` system (`get_tokens(self.current_design)`) |
| Notes Window | Uses `gpui_component::theme::ActiveTheme` directly (`cx.theme()`) |
| AI Window | Uses `gpui_component::theme::ActiveTheme` directly (`cx.theme()`) |

**Problem:** Main window supports 11 design variants (Default, Minimal, RetroTerminal, etc.) but Notes/AI windows don't inherit these. Switching designs in main window doesn't affect secondary windows.

---

## 2. Dimension Inconsistencies

### 2.1 Action Item Heights

| Component | Height | File | Line |
|-----------|--------|------|------|
| Main ActionsDialog | **42.0px** | `src/actions.rs` | ~constant |
| Notes ActionsPanel | **44.0px** | `src/notes/actions_panel.rs` | 251 |

**Difference:** 2px - Notes actions are taller than main window actions.

### 2.2 Titlebar Heights

| Window | Height | File | Line |
|--------|--------|------|------|
| Notes Window | **32px** | `src/notes/window.rs` | `h(px(32.))` |
| AI Window | **36px** | `src/ai/window.rs` | `h(px(36.))` |

**Difference:** 4px - AI window has taller titlebar.

### 2.3 List Item Heights

| Component | Height | File | Line |
|-----------|--------|------|------|
| Main LIST_ITEM_HEIGHT | **48.0px** | `src/list_item.rs` | 30 |
| Browse Panel note row | **36.0px** | `src/notes/browse_panel.rs` | 278 |
| Design tokens (Default) | **40.0px** | `src/designs/traits.rs` | 441 |

**Problem:** Main window uses 48px for script list items, but design tokens define 40px as default. Notes browse panel uses 36px.

### 2.4 Panel Dimensions

| Panel | Width | Max Height | File |
|-------|-------|------------|------|
| Main ActionsDialog | **320.0px** | **400.0px** | `src/actions.rs` |
| Notes ActionsPanel | **320.0px** | **580.0px** | `src/notes/actions_panel.rs` |
| Notes BrowsePanel | **500.0px** | **400.0px** | `src/notes/browse_panel.rs` |

**Findings:**
- Width matches for actions panels (320px) ✅
- Max height differs: Main=400px, Notes=580px ❌
- Browse panel is much wider (500px vs 320px) - may be intentional for note titles

### 2.5 Search Input Heights

| Component | Height | File |
|-----------|--------|------|
| Notes ActionsPanel search | **44.0px** | `src/notes/actions_panel.rs:252` |
| AI Window search | **36.0px** | `src/ai/window.rs` |

**Difference:** 8px - Notes search is taller than AI search.

---

## 3. Theme System Inconsistencies

### 3.1 Color Definitions

Default dark theme colors from `src/theme.rs`:

| Color | Hex | Usage |
|-------|-----|-------|
| `background.main` | `#1E1E1E` | Main window background |
| `background.title_bar` | `#2D2D30` | Title bar, sidebar |
| `background.search_box` | `#3C3C3C` | Input backgrounds |
| `accent.selected` | `#FBBF24` | Yellow/gold accent |
| `accent.selected_subtle` | `#2A2A2A` | Subtle selection bg |
| `text.primary` | `#FFFFFF` | Primary text |
| `text.muted` | `#808080` | Muted text |
| `ui.border` | `#464647` | Borders |

### 3.2 gpui-component Mapping (from theme.rs)

The `map_scriptkit_to_gpui_theme()` function maps Script Kit colors to gpui-component's `ThemeColor`:

| ThemeColor field | Maps to |
|------------------|---------|
| `background` | `colors.background.main` |
| `foreground` | `colors.text.primary` |
| `accent` | `colors.accent.selected` |
| `border` | `colors.ui.border` |
| `sidebar` | `colors.background.title_bar` |
| `list_active` | `colors.accent.selected_subtle` |
| `muted_foreground` | `colors.text.muted` |
| `caret` | `colors.text.primary` |

**Potential Issue:** AI window may have different caret color mapping. Need to verify if intentional.

### 3.3 Design Tokens System

The `src/designs/` module provides comprehensive design tokens with 11 variants:

| Variant | Item Height | Key Characteristics |
|---------|-------------|---------------------|
| Default | 40.0px | Standard dark theme |
| Minimal | 64.0px | Generous spacing, thin fonts |
| RetroTerminal | 28.0px | Green-on-black, dense |
| Compact | 24.0px | Smallest, power users |
| AppleHIG | 44.0px | iOS-style, 44px touch targets |
| Material3 | 56.0px | Material You, larger items |
| Glassmorphism | 56.0px | Transparency effects |
| Brutalist | 40.0px | Bold, raw typography |
| NeonCyberpunk | 34.0px | Neon glow effects |
| Paper | 34.0px | Warm, paper-like tones |
| Playful | 56.0px | Rounded, vibrant colors |

---

## 4. Spacing Inconsistencies

### 4.1 Padding Values in Use

Observed padding values across the codebase:

| Value | Usage |
|-------|-------|
| `px(3.)` | Small gaps |
| `px(4.)` | XS padding (default token) |
| `px(6.)` | Action row inset (notes) |
| `px(8.)` | SM padding (default token) |
| `px(10.)` | Various |
| `px(12.)` | MD padding (default token), common |
| `px(16.)` | LG padding (default token) |
| `px(24.)` | XL padding (default token) |

### 4.2 Design Token Defaults (from `src/designs/traits.rs`)

```rust
DesignSpacing {
    padding_xs: 4.0,
    padding_sm: 8.0,
    padding_md: 12.0,
    padding_lg: 16.0,
    padding_xl: 24.0,
    gap_sm: 4.0,
    gap_md: 8.0,
    gap_lg: 16.0,
    item_padding_x: 16.0,
    item_padding_y: 8.0,
    icon_text_gap: 8.0,
}
```

---

## 5. Component-Level Findings

### 5.1 Main Window (`src/app_render.rs`)

- Uses design tokens via `get_tokens(self.current_design)`
- Supports all 11 design variants
- Preview panel uses dynamic styling from tokens
- Consistent with `designs::DesignColors`, `DesignSpacing`, etc.

### 5.2 Notes Window (`src/notes/`)

**window.rs:**
- Titlebar: 32px height
- Footer: 24px height
- Uses `gpui_component::theme` directly (not design tokens)
- Has duplicated `map_scriptkit_to_gpui_theme()` function

**actions_panel.rs:**
- Panel width: 320px
- Panel max height: 580px
- Action item height: 44px
- Selection radius: 8px (rounded pill style)
- Row inset: 6px

**browse_panel.rs:**
- Panel width: 500px
- Panel max height: 400px
- Note row height: 36px
- Uses gpui-component Button, Input components

### 5.3 AI Window (`src/ai/`)

**window.rs:**
- Titlebar: 36px height (4px taller than Notes)
- Sidebar: 240px expanded, 48px collapsed
- Uses `gpui_component::theme` directly
- Has duplicated theme mapping

### 5.4 Main Actions Dialog (`src/actions.rs`)

- Panel width: 320px
- Panel max height: 400px
- Action item height: 42px (2px shorter than Notes)
- Accent bar width: 3px
- Uses design tokens system

---

## 6. Recommendations

### 6.1 Immediate (High Priority)

1. **Consolidate theme mapping** - Remove `map_scriptkit_to_gpui_theme()` from `notes/window.rs` and `ai/window.rs`. Export from `src/theme.rs` and import where needed.

2. **Standardize action item height** - Pick either 42px or 44px for all action panels. Recommend 44px (matches iOS touch target guidelines).

3. **Standardize titlebar height** - Pick either 32px or 36px. Recommend 36px for better touch targets.

### 6.2 Short-term (Medium Priority)

4. **Align panel max heights** - Decide if Notes actions panel should have 580px max or match main's 400px.

5. **Standardize search input heights** - Use consistent 44px or 36px across all panels.

6. **Document LIST_ITEM_HEIGHT vs design tokens** - Clarify why `list_item.rs` uses 48px but design tokens default to 40px.

### 6.3 Long-term (Low Priority)

7. **Consider design token adoption for Notes/AI** - Decide if secondary windows should support the 11 design variants.

8. **Create shared constants module** - Extract all dimension constants to a single file.

9. **Add visual regression tests** - Capture screenshots of each window at various sizes.

---

## 7. Fix Priority Matrix

| Issue | Severity | Effort | Priority |
|-------|----------|--------|----------|
| Duplicated theme mapping | High | Low | **P0** |
| Titlebar height mismatch | Medium | Low | **P1** |
| Action item height mismatch | Medium | Low | **P1** |
| Search input height mismatch | Low | Low | **P2** |
| Panel max height mismatch | Low | Low | **P2** |
| Design token adoption for secondary windows | Low | High | **P3** |
| LIST_ITEM_HEIGHT documentation | Low | Low | **P2** |

---

## Appendix A: File Reference

| File | Purpose |
|------|---------|
| `src/theme.rs` | Central theme definitions, color schemes, gpui-component mapping |
| `src/designs/mod.rs` | Design variant enum, dispatcher |
| `src/designs/traits.rs` | DesignTokens trait, token structs (Colors, Spacing, Typography, Visual) |
| `src/list_item.rs` | Shared ListItem component, LIST_ITEM_HEIGHT constant |
| `src/actions.rs` | Main window ActionsDialog |
| `src/app_render.rs` | Main window rendering, preview panel |
| `src/notes/window.rs` | Notes window, titlebar, theme sync |
| `src/notes/actions_panel.rs` | Notes Cmd+K panel |
| `src/notes/browse_panel.rs` | Notes Cmd+P panel |
| `src/ai/window.rs` | AI chat window |

---

## Appendix B: Constants Quick Reference

### Heights

| Constant | Value | Location |
|----------|-------|----------|
| LIST_ITEM_HEIGHT | 48.0 | `src/list_item.rs:30` |
| SECTION_HEADER_HEIGHT | 24.0 | `src/list_item.rs:40` |
| PANEL_SEARCH_HEIGHT (notes) | 44.0 | `src/notes/actions_panel.rs:252` |
| ACTION_ITEM_HEIGHT (notes) | 44.0 | `src/notes/actions_panel.rs:251` |
| Notes titlebar | 32.0 | `src/notes/window.rs` |
| Notes footer | 24.0 | `src/notes/window.rs` |
| AI titlebar | 36.0 | `src/ai/window.rs` |
| Browse note row | 36.0 | `src/notes/browse_panel.rs:278` |

### Widths

| Constant | Value | Location |
|----------|-------|----------|
| PANEL_WIDTH (notes actions) | 320.0 | `src/notes/actions_panel.rs:248` |
| POPUP_WIDTH (main actions) | 320.0 | `src/actions.rs` |
| Browse panel width | 500.0 | `src/notes/browse_panel.rs:410` |
| AI sidebar expanded | 240.0 | `src/ai/window.rs` |
| AI sidebar collapsed | 48.0 | `src/ai/window.rs` |

### Corner Radii

| Constant | Value | Location |
|----------|-------|----------|
| PANEL_CORNER_RADIUS | 12.0 | `src/notes/actions_panel.rs:250` |
| SELECTION_RADIUS | 8.0 | `src/notes/actions_panel.rs:257` |
| DesignVisual.radius_md (default) | 8.0 | `src/designs/traits.rs:360` |
| DesignVisual.radius_lg (default) | 12.0 | `src/designs/traits.rs:361` |

---

*End of Design Audit Report*
