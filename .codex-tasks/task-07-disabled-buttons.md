# Task: Audit Disabled Button States

Check that disabled buttons are properly styled and non-interactive:

1. Find all buttons that can be disabled
2. Ensure disabled buttons:
   - Have reduced opacity or grayed-out appearance
   - Do NOT have cursor_pointer (use cursor_default or cursor_not_allowed)
   - Have click handlers that check disabled state
   - Don't show hover effects when disabled

Pattern:
```rust
div()
    .when(disabled, |d| d.opacity(0.5).cursor_not_allowed())
    .when(!disabled, |d| d.cursor_pointer().hover(...))
```

Fix any disabled buttons that still appear interactive.
