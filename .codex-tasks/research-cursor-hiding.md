# Research: Hide Mouse Cursor on Keyboard Input

## Files Investigated

1. `/Users/johnlindquist/dev/script-kit-gpui/src/main.rs`
   - Lines 1210-1350: `ScriptListApp` struct definition
   - Lines 1516-1860: `impl Render for ScriptListApp`
   - The main render method builds the outer div at lines 1810-1860

2. `/Users/johnlindquist/dev/script-kit-gpui/src/platform.rs`
   - Platform utilities for macOS window management
   - No existing cursor visibility functions

3. `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-app-kit-*/src/generated/NSCursor.rs`
   - Lines 130-140: NSCursor APIs:
     - `hide()` - hides cursor
     - `unhide()` - shows cursor
     - `setHiddenUntilMouseMoves(flag: bool)` - hides until mouse moves (BEST API)

## Current Behavior

- No cursor hiding is implemented
- The main window renders via `impl Render for ScriptListApp`
- Key events are handled by individual views/prompts via `on_key_down` listeners
- Mouse events are tracked for hover states but not for cursor visibility

## Root Cause Analysis

The mouse cursor remains visible when typing, which can interfere with content that moves under it (like scrolling lists or changing selections).

## Proposed Solution Approach

Use `NSCursor::setHiddenUntilMouseMoves(true)` - this is the ideal macOS API because:
1. It hides the cursor immediately
2. It automatically shows the cursor when the mouse moves (no need to track mouse_move events)
3. It's a single API call, no state management needed

### Implementation Plan:

1. **Add to `platform.rs`**: New function `hide_cursor_until_mouse_moves()` that calls `NSCursor::setHiddenUntilMouseMoves(true)`

2. **Trigger on key events**: Call this function whenever a key is pressed in the main window
   - Option A: Add `on_any_key_down` to the outer div in render method
   - Option B: Call it from the various key handlers

3. **No state needed**: macOS handles showing cursor automatically when mouse moves

### Code Locations to Modify:

1. `/Users/johnlindquist/dev/script-kit-gpui/src/platform.rs` - add hide_cursor function
2. `/Users/johnlindquist/dev/script-kit-gpui/src/main.rs` - add on_any_key_down to outer div OR call from key handlers

### macOS API Reference:

```objc
// From NSCursor.h
+ (void)setHiddenUntilMouseMoves:(BOOL)flag;
```

This is the standard pattern used by text editors to hide the cursor while typing.

## Verification

### What was changed

1. **Added `hide_cursor_until_mouse_moves()` function to `/Users/johnlindquist/dev/script-kit-gpui/src/platform.rs`**
   - Located at lines 3131-3139 (macOS) and 3139-3141 (non-macOS stub)
   - Uses Objective-C message sending: `msg_send![class!(NSCursor), setHiddenUntilMouseMoves: true]`
   - Has proper cfg attributes for cross-platform support

2. **Added cursor hide calls to all keyboard event handlers:**
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_script_list.rs` - main script list
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins.rs` - clipboard, app launcher, window switcher, file search, design gallery
   - `/Users/johnlindquist/dev/script-kit-gpui/src/app_render.rs` - actions dialog
   - `/Users/johnlindquist/dev/script-kit-gpui/src/editor.rs` - editor prompt
   - `/Users/johnlindquist/dev/script-kit-gpui/src/term_prompt.rs` - terminal prompt
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/arg.rs` - arg prompt
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/div.rs` - div prompt
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/editor.rs` - editor prompt (render)
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/form.rs` - form prompt
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/other.rs` - select, env, drop, template, chat prompts
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/path.rs` - path prompt
   - `/Users/johnlindquist/dev/script-kit-gpui/src/render_prompts/term.rs` - term prompt (render)

### Test Results

- `cargo check`: PASSED
- `cargo clippy --all-targets -- -D warnings`: PASSED (no warnings)
- `cargo test --lib`: PASSED (2582 tests pass)

### Before/After Comparison

**Before:**
- Mouse cursor remained visible when typing
- Cursor could obscure content under it during keyboard navigation

**After:**
- Cursor automatically hides when any key is pressed
- Cursor automatically reappears when mouse moves
- Uses macOS native API (NSCursor.setHiddenUntilMouseMoves)
- Zero state management required - OS handles re-showing cursor

### Deviations from Proposed Solution

None - the implementation follows the proposed solution exactly:
1. Added the platform function as planned
2. Called it from all keyboard event handlers
3. Used the macOS setHiddenUntilMouseMoves API which automatically handles mouse movement detection
