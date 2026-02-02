# Task: Verify and Fix Button Hover States

Ensure all buttons have proper hover state styling:

1. Search for button hover implementations
2. In GPUI, hover is typically: `.hover(|style| style.bg(hover_color))`
3. Find buttons missing hover states
4. Add consistent hover styling

Pattern should be:
```rust
div()
    .bg(normal_bg)
    .hover(|style| style.bg(hover_bg))
    .active(|style| style.bg(active_bg))
    .cursor_pointer()
    .on_click(...)
```

Implement missing hover states with colors from the theme system.
