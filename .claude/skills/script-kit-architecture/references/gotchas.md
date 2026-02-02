# Common Gotchas

Real failures that have occurred. Check this list when debugging.

## UI Issues

- **UI not updating** → forgot `cx.notify()`
- **Theme not applying** → hardcoded `rgb(0x...)`
- **Laggy list scrolling** → no key coalescing
- **Window on wrong monitor** → use mouse display + `Bounds::centered(...)`
- **Focus styling wrong** → missing `Focusable` or focus-change re-render

## Script Execution Issues

- **Spawn failures silent** → match `Command::spawn()` and log errors
- **Script doesn't exit after finishing** → SDK calls `process.stdin.resume()`; add `(process.stdin as any).unref?.()` after resume
- **Script `console.error()` not visible live** → GPUI may read stderr only on exit; add stderr reader thread forwarding lines to `logging::log("SCRIPT", ...)`

## Hard-Won Learnings

### RefCell Borrow Conflicts During Resize

Window resize/move callbacks can trigger while a RefCell is already borrowed. Fix: use `Window::defer()`:
```rust
// Don't call platform APIs directly during render:
// platform::resize_first_window_to_height(height); // WRONG - can panic

// Use Window::defer to run after borrows are unwound:
window.defer(cx, |_window, _cx| {
    platform::resize_first_window_to_height(height);
});
```

### async_channel vs std::sync::mpsc

Use `async_channel` for event-driven behavior. `mpsc::try_recv()` polling wastes CPU.

### Multi-Monitor Bounds Accuracy

`cx.displays()` can return stale bounds. For accurate positioning, use native NSScreen APIs:
```rust
#[cfg(target_os = "macos")]
fn get_mouse_display_bounds() -> Option<Bounds<Pixels>> {
    // Use CGMainDisplayID, NSScreen.screens, mouseLocation
}
```

### Key Event Handling Completeness

Match both variants: `"enter"|"Enter"`, `"escape"|"Escape"`, `"tab"|"Tab"`. GPUI normalization varies by platform.

### Visual Verification Requires Reading the File

Capturing a screenshot is not enough:
```ts
const shot = await captureScreenshot();
writeFileSync(path, Buffer.from(shot.data, 'base64'));
// THEN: Use Read tool to actually examine the PNG
```

### Bead Completion Protocol

Use `swarm_complete()`, never `hive_close()`. The swarm_complete function:
- Releases file reservations
- Records metrics for implicit feedback scoring
- Properly closes the bead

`hive_close()` skips reservation cleanup → causes conflicts.
