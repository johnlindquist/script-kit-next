# Window Management

## Multi-Monitor Positioning

Pick display containing the mouse; use visible bounds for usable area; often center at an "eye-line" (upper third).

Useful display APIs:
- `cx.displays()`, `cx.primary_display()`, `cx.find_display(id)`
- `display.bounds()`, `display.visible_bounds()`, `bounds.contains(&point)`

**Multi-monitor bounds accuracy:**
`cx.displays()` can return stale or incorrect bounds. For accurate positioning, use native NSScreen APIs:
```rust
#[cfg(target_os = "macos")]
fn get_mouse_display_bounds() -> Option<Bounds<Pixels>> {
    // Use CGMainDisplayID, NSScreen.screens, mouseLocation
    // Don't rely solely on cx.displays()
}
```

## macOS Floating Panel

Call after `cx.activate(true)`:
```rust
#[cfg(target_os="macos")]
unsafe {
  let app: id = NSApp();
  let window: id = msg_send![app, keyWindow];
  if window != nil {
    let _: () = msg_send![window, setLevel:3i32]; // NSFloatingWindowLevel
    let _: () = msg_send![window, setCollectionBehavior:1u64]; // join all spaces
  }
}
```

## RefCell Borrow Conflicts During Resize

Window resize/move callbacks can trigger while a RefCell is already borrowed. Fix: use `Window::defer()`:
```rust
// Don't call platform APIs directly during render:
// platform::resize_first_window_to_height(height); // WRONG - can panic

// Use Window::defer to run after borrows are unwound:
window.defer(cx, |_window, _cx| {
    platform::resize_first_window_to_height(height);
});
```

See `src/window_ops.rs` for a coalescing queue pattern that also prevents jitter.

## async_channel vs std::sync::mpsc

Use `async_channel` for event-driven behavior. `mpsc::try_recv()` polling wastes CPU; `async_channel` integrates with async runtimes and wakes only when messages arrive.

## Secondary Windows

- Notes: `src/notes/` (`{"type":"openNotes"}`)
- AI: `src/ai/` (`{"type":"openAi"}`)

Single-instance: global `OnceLock<Mutex<Option<WindowHandle<Root>>>>`.

Open methods: hotkey (configurable), tray menu, stdin.
