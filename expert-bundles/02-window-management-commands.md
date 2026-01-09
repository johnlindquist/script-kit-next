# Window Management Commands Expert Bundle

## Original Goal

> Exploring other command ideas for window management
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

Script Kit GPUI has extensive window management capabilities using macOS Accessibility APIs. The current system supports tiling, moving, resizing, and focusing windows. There's opportunity to add more advanced commands like window cycling, display management, and window grouping.

### Key Problems:
1. **Limited tiling options** - Only basic half/quadrant layouts available
2. **No window cycling** - Can't quickly cycle through windows of same app
3. **Missing display commands** - No move-to-next-monitor functionality

### Required Fixes:
1. **src/window_control.rs** - Add new window operations (thirds, move to display)
2. **src/system_actions.rs** - Expose new commands to the menu system
3. **src/window_control_enhanced/** - Leverage enhanced bounds/display detection

### Files Included:
- `src/window_control.rs`: Core window control using AX APIs
- `src/window_control_enhanced/`: Enhanced window management modules
- `src/window_manager.rs`: Window handle management
- `src/window_ops.rs`: Window operation queue
- `src/window_state.rs`: Window state persistence
- `src/system_actions.rs`: System action definitions
- `src/window_resize.rs`: Window resize handling

---

