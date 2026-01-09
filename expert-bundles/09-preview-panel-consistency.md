# Preview Panel Consistency Expert Bundle

## Original Goal

> Improving the general look and presentation of the Preview panel across everywhere and every situation its used for a consistent experience
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The Preview panel appears in multiple contexts: main script list, clipboard history, file search, and arg prompts. The styling and behavior should be consistent across all uses, with proper syntax highlighting, sizing, and theming.

### Key Problems:
1. **Inconsistent styling** - Different backgrounds, padding, fonts across views
2. **Theme integration** - Some previews don't respect theme colors
3. **Content overflow** - Long content may not scroll or truncate correctly
4. **Opacity handling** - Preview opacity setting not applied uniformly

### Required Fixes:
1. **src/app_render.rs** - Unify `render_preview_panel()` implementation
2. **src/render_script_list.rs** - Use shared preview component
3. **src/render_builtins.rs** - Apply same preview styling to built-ins
4. **src/ui_foundation.rs** - Create shared preview container component

### Files Included:
- `src/app_render.rs`: Main render methods including preview panel
- `src/render_script_list.rs`: Script list view with preview
- `src/render_builtins.rs`: Built-in views (clipboard, file search) with previews
- `src/ui_foundation.rs`: Shared UI components
- `src/syntax.rs`: Syntax highlighting for code previews

---

