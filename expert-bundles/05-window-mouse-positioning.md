# Window Mouse Display Positioning Expert Bundle

## Original Goal

> Improving the logic of how windows appear on the display where the mouse is located
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

Script Kit windows should appear on the display where the mouse cursor is located. This involves detecting the current mouse position, finding the corresponding display, and positioning the window appropriately accounting for menu bar and dock.

### Key Problems:
1. **Stale display detection** - May use cached display info instead of current mouse position
2. **Multi-monitor edge cases** - Windows may appear on wrong monitor at display boundaries
3. **Coordinate system confusion** - macOS uses bottom-left origin vs GPUI's top-left

### Required Fixes:
1. **src/platform.rs** - Improve mouse position to display mapping
2. **src/window_control_enhanced/display.rs** - Better display boundary detection
3. **src/window_manager.rs** - Use correct coordinate transforms
4. **src/panel.rs** - Apply correct window positioning on show

### Files Included:
- `src/platform.rs`: Platform-specific window configuration
- `src/window_control_enhanced/display.rs`: Display detection and bounds
- `src/window_control_enhanced/bounds.rs`: Window bounds calculations
- `src/window_control_enhanced/coords.rs`: Coordinate transformations
- `src/window_manager.rs`: Window handle management
- `src/window_ops.rs`: Window operation queue
- `src/panel.rs`: Panel window configuration

---

