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
