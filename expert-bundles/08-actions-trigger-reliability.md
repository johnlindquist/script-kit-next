# Actions Window Trigger Reliability Expert Bundle

## Original Goal

> Guaranteeing the correct action is always triggered from the actions window
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The actions window allows users to execute actions via selection or keyboard shortcuts. There may be race conditions or state synchronization issues that cause the wrong action to execute.

### Key Problems:
1. **Selection state drift** - Selected index may not match visible selection
2. **Action ID mismatch** - Stored action ID may differ from displayed action
3. **Async timing** - Action execution may use stale state

### Required Fixes:
1. **src/actions/dialog.rs** - Ensure selection state is synchronized
2. **src/actions/builders.rs** - Verify action IDs are unique and stable
3. **src/app_actions.rs** - Use action ID directly, not index lookup
4. **src/render_script_list.rs** - Lock state during action execution

### Files Included:
- `src/actions/dialog.rs`: Actions dialog with selection state
- `src/actions/window.rs`: Actions popup window management
- `src/actions/builders.rs`: Action list construction
- `src/actions/types.rs`: Action type with ID and metadata
- `src/actions/constants.rs`: UI constants
- `src/shortcuts/registry.rs`: Shortcut-to-action mapping
- `src/app_actions.rs`: Action execution handlers

---

