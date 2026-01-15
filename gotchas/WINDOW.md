# GPUI Window Handle Gotchas

## Stale WindowHandle in defer callbacks

### Problem
When storing a `WindowHandle<T>` globally and trying to call `handle.update()` from a `window.defer()` callback, you get:
```
handle.update FAILED: window not found
```

### Root Cause
`WindowHandle::update()` looks up the window in GPUI's internal registry. When called from a defer callback that was scheduled from the same window, the handle may be "stale" - GPUI can't find the window even though it clearly exists (we're running code inside it!).

This is likely because:
1. The defer callback runs in a different context than when the handle was stored
2. GPUI's window registry state has changed between storing and using the handle
3. There may be re-entrancy protection preventing updates to the "current" window via its handle

### Symptoms
```rust
// This pattern FAILS:
static WINDOW_HANDLE: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

// Later, in a key handler:
window.defer(cx, move |_window, cx| {
    if let Some(handle) = get_stored_handle() {
        // ERROR: "window not found"
        let _ = handle.update(cx, |root, window, cx| {
            window.resize(...);
        });
    }
});
```

### Solution
Use the `window` parameter from the defer callback directly instead of looking up the handle:

```rust
// This pattern WORKS:
window.defer(cx, move |window, cx| {
    // Use `window` directly - no handle lookup needed!
    let current_bounds = window.bounds();
    window.resize(new_size);
});
```

### Code Example
Before (broken):
```rust
let dialog = this.dialog.clone();
window.defer(cx, move |_window, cx| {
    resize_actions_window(cx, &dialog);  // Uses stored handle - FAILS
});
```

After (working):
```rust
let dialog = this.dialog.clone();
window.defer(cx, move |window, cx| {
    resize_actions_window_direct(window, cx, &dialog);  // Uses window directly - WORKS
});
```

### Key Insight
In a `window.defer()` callback with signature `|window: &mut Window, cx: &mut App|`:
- The `window` parameter IS the window you want to operate on
- Don't ignore it with `_window` if you need to resize/modify the window
- Don't try to look up the same window via a stored handle

### When Stored Handles DO Work
Stored `WindowHandle`s work fine when:
- Calling from a different window's context
- Calling from app-level code (not inside any window)
- Calling from async tasks spawned separately

They fail when:
- Calling from a defer callback scheduled from the same window
- Possibly during certain GPUI lifecycle phases

### Files Changed
- `src/actions/window.rs` - Added `resize_actions_window_direct()` that takes `&mut Window` directly
- Fixed actions popup resize after filtering

---

## Window Focus vs Activation (macOS)

### Key Concepts

On macOS, there's a critical distinction between **focusing** a window and **activating** an application:

| Action | What it does | macOS API |
|--------|--------------|-----------|
| **Focus** | Window comes to front, receives keyboard input | `[NSWindow makeKeyAndOrderFront:]` or `orderFrontRegardless` |
| **Activate** | App becomes frontmost, ALL windows come forward, steals focus from other apps | `[NSApp activateIgnoringOtherApps:YES]` |

### Script Kit Window Behavior

| Window | Focus | Activate | Why |
|--------|-------|----------|-----|
| **Main Menu** | YES | NO | Floating panel - should appear without stealing focus from other apps. Enables "copy selected text" workflows. |
| **Notes** | YES | NO | Same as main menu - utility window that shouldn't disrupt other apps. |
| **AI Chat** | YES | YES | Full application window - appears in Cmd+Tab, user expects it to behave like a normal app window. |

### Implementation Details

#### Main Menu & Notes (Focus without Activation)

```rust
// In platform.rs
pub fn show_main_window_without_activation() {
    // orderFrontRegardless brings window to front without activating app
    let _: () = msg_send![window, orderFrontRegardless];
    // makeKeyWindow lets it receive keyboard input
    let _: () = msg_send![window, makeKeyWindow];
}
```

**Why this matters:**
1. User has text selected in another app (e.g., browser)
2. User presses main hotkey (Cmd+;)
3. Main menu appears WITHOUT the browser losing OS-level focus
4. User runs a script that calls `getSelectedText()`
5. Since browser still has focus, accessibility APIs can read the selection

#### AI Window (Focus with Activation)

```rust
// In ai/window.rs
cx.activate(true);  // Makes app frontmost
window.activate_window();  // Brings this specific window forward
```

**Why AI is different:**
- AI window participates in Cmd+Tab (via `set_regular_app_mode()`)
- Users expect to switch between AI and other apps normally
- It's a "workspace" window, not a quick utility panel

### The Bug We Fixed

**Before:** `cx.activate(true)` was called when showing the main menu, which:
- Made Script Kit the frontmost application
- Caused ALL Script Kit windows to come forward
- Stole focus from the previously-focused app
- Broke `getSelectedText()` and similar features

**After:** Main menu uses `show_main_window_without_activation()`:
- Window appears and receives keyboard input
- Previous app retains OS-level focus
- `getSelectedText()` can still read from the previous app

### Related Code Locations

- `src/platform.rs`: `show_main_window_without_activation()`, `configure_as_floating_panel()`
- `src/hotkey_pollers.rs`: Main hotkey handler, script shortcut handler, AI hotkey handler
- `src/main.rs`: `show_main_window_helper()`, stdin command handlers
- `src/ai/window.rs`: `open_ai_window()` (uses activation)
- `src/notes/window.rs`: `open_notes_window()` (should use focus-only)

### Testing This Behavior

1. Select text in a browser
2. Press main hotkey (Cmd+;)
3. Main menu should appear
4. Run "Send Selected Text to AI" or similar
5. The selected text should be captured correctly

If activation is happening incorrectly, the text capture will fail because the browser lost focus.
