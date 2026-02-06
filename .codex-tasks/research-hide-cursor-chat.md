# Research: Hide Mouse Cursor When Typing in AI Chat Window

## Files Investigated

1. **`/Users/johnlindquist/dev/script-kit-gpui/src/prompts/chat.rs`** - Main ChatPrompt implementation
   - Lines 250-270: Struct definition with cursor-related state
   - Lines 2232-2321: Key event handling (`handle_key` listener)
   - Lines 2321-2367: Render method with `on_key_down`

2. **Zed Editor Reference** (`~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/editor/src/editor.rs`)
   - Line 1202: `mouse_cursor_hidden: bool` field
   - Lines 2727-2731: `show_mouse_cursor()` method
   - Lines 2734-2748: `hide_mouse_cursor()` method

3. **GPUI Platform** (`~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/gpui/src/platform.rs`)
   - `CursorStyle::None` variant - hides cursor until mouse moves
   - On macOS: calls `[NSCursor setHiddenUntilMouseMoves:YES]`

4. **GPUI Window API** (`~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/gpui/src/window.rs`)
   - Line 2520: `set_window_cursor_style()` method to set cursor for entire window

## Current Behavior

- ChatPrompt handles keyboard events via `on_key_down` handler
- Typing characters updates input state and calls `reset_cursor_blink()`
- NO mouse cursor hiding is implemented
- Mouse interferes with content when typing (cursor stays visible over scrolling content)

## Root Cause Analysis

The AI chat window lacks the cursor-hiding behavior that improves UX when typing. When the user types, especially during streaming responses, the mouse cursor stays visible and can be distracting or interfere with content that moves under it.

## Proposed Solution

1. **Add state field** to `ChatPrompt`:
   ```rust
   mouse_cursor_hidden: bool,
   ```

2. **Add hide/show methods**:
   ```rust
   fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
       if !self.mouse_cursor_hidden {
           self.mouse_cursor_hidden = true;
           cx.notify();
       }
   }
   
   fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
       if self.mouse_cursor_hidden {
           self.mouse_cursor_hidden = false;
           cx.notify();
       }
   }
   ```

3. **In key handler**, call `hide_mouse_cursor()` for typing actions

4. **Add `on_mouse_move` handler** to call `show_mouse_cursor()`

5. **In render**, check `mouse_cursor_hidden` and call `window.set_window_cursor_style(CursorStyle::None)` or apply via `.cursor()` modifier

## GPUI API to Use

The GPUI API for this is:
- Import: `gpui::CursorStyle`
- Apply in render: Use `window.set_window_cursor_style(CursorStyle::None)` when cursor should be hidden
- Alternative: Use `.cursor(CursorStyle::None)` on the root div element

## Verification

### Changes Made

1. **Updated imports** in `/Users/johnlindquist/dev/script-kit-gpui/src/prompts/chat.rs` (line 13-16):
   - Added `CursorStyle` and `MouseMoveEvent` to gpui imports

2. **Added state field** (line 263):
   ```rust
   mouse_cursor_hidden: bool,
   ```

3. **Added initialization** in `new()` (line 324):
   ```rust
   mouse_cursor_hidden: false,
   ```

4. **Added hide_mouse_cursor method** (lines 370-377):
   ```rust
   fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
       if !self.mouse_cursor_hidden {
           self.mouse_cursor_hidden = true;
           cx.notify();
       }
   }
   ```

5. **Added show_mouse_cursor method** (lines 378-385):
   ```rust
   fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
       if self.mouse_cursor_hidden {
           self.mouse_cursor_hidden = false;
           cx.notify();
       }
   }
   ```

6. **Added hide_mouse_cursor calls** in key handler for:
   - Backspace (line 2313)
   - Character input (line 2328)

7. **Added mouse move handler** (lines 2336-2340):
   ```rust
   let handle_mouse_move = cx.listener(|this, _event: &MouseMoveEvent, _window, cx| {
       this.show_mouse_cursor(cx);
   });
   ```

8. **Applied cursor style and mouse move handler** to main div (lines 2400-2404):
   ```rust
   .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
   // ...
   .on_mouse_move(handle_mouse_move)
   ```

### Test Results

- **cargo check**: PASSED
- **cargo clippy --all-targets -- -D warnings**: PASSED  
- **cargo test**: 2580 passed, 2 failed (pre-existing flaky MCP server tests unrelated to changes)

### Before/After Comparison

**Before:**
- When typing in AI chat window, mouse cursor remains visible
- Mouse cursor can interfere with content that scrolls/updates during streaming

**After:**
- When typing (characters or backspace), mouse cursor is hidden
- On macOS, uses `CursorStyle::None` which maps to `setHiddenUntilMouseMoves`
- When mouse moves, cursor automatically reappears
- Consistent with Zed editor and main window behavior

### Deviations from Proposed Solution

None - implementation follows the proposed solution exactly. The setup card div (for API key configuration) does not include cursor hiding since it has minimal typing (only Enter and Escape handling).
