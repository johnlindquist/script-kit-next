# Task: Audit and Fix Cursor Styles on Interactive Elements

Search for all interactive elements (buttons, clickable divs, links) and ensure they use pointer cursor.

1. Find all `.on_click()` handlers in the codebase
2. Check if corresponding elements have `.cursor_pointer()` applied
3. For any missing cursor styles, add `.cursor_pointer()` to the element chain

In GPUI, the pattern should be:
```rust
div()
    .cursor_pointer()  // <-- This should be present for all clickable elements
    .on_click(cx.listener(...))
```

Fix any elements missing the cursor_pointer() call.
