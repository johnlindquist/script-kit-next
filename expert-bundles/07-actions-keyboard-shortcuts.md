# Actions Window Keyboard Shortcuts Expert Bundle

## Original Goal

> Supporting the keyboard shortcuts from the actions window when the main window is open
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The actions window (Cmd+K popup) shows available actions with their shortcuts. These shortcuts should work even when the main window is open, not just when the actions popup is focused. This requires proper keyboard event routing.

### Key Problems:
1. **Shortcut scope confusion** - Main window may intercept shortcuts meant for actions
2. **Context priority** - ActionsDialog shortcuts should take precedence
3. **Event propagation** - Events may not reach the correct handler

### Required Fixes:
1. **src/actions/dialog.rs** - Ensure shortcuts are registered with correct priority
2. **src/shortcuts/context.rs** - Update context stack for proper routing
3. **src/app_actions.rs** - Route actions correctly when popup is visible
4. **src/render_script_list.rs** - Check actions popup state before handling keys

### Files Included:
- `src/actions/dialog.rs`: Actions dialog component
- `src/actions/window.rs`: Separate actions popup window
- `src/actions/builders.rs`: Action list builders
- `src/actions/types.rs`: Action type definitions
- `src/shortcuts/registry.rs`: Shortcut registration and matching
- `src/shortcuts/context.rs`: Context-based shortcut routing
- `src/app_actions.rs`: Application action handlers

---

