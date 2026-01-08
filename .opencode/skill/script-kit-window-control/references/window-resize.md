# Window Resize (window_resize.rs)

Dynamic window height calculation for different view types in Script Kit GPUI.

## Key Rules

| View Type | Height Behavior |
|-----------|-----------------|
| ScriptList | **FIXED** at 500px, never resizes |
| ArgPromptWithChoices | Dynamic based on item count (capped at 500px) |
| ArgPromptNoChoices | Compact input-only height |
| DivPrompt | Standard 500px |
| EditorPrompt | Full 700px |
| TermPrompt | Full 700px |

## Layout Constants

```rust
pub mod layout {
    pub const ARG_INPUT_PADDING_Y: f32 = 12.0;    // Input row vertical padding
    pub const ARG_LIST_PADDING_Y: f32 = 8.0;      // List container padding
    pub const ARG_DIVIDER_HEIGHT: f32 = 1.0;      // Divider thickness
    pub const ARG_INPUT_LINE_HEIGHT: f32 = ...;   // Cursor height + margins
    pub const FOOTER_HEIGHT: f32 = 30.0;          // PromptFooter height
    pub const ARG_HEADER_HEIGHT: f32 = ...;       // Total input-only height
    
    pub const MIN_HEIGHT: Pixels = px(ARG_HEADER_HEIGHT);  // ~92px
    pub const STANDARD_HEIGHT: Pixels = px(500.0);
    pub const MAX_HEIGHT: Pixels = px(700.0);
}
```

## Types

### ViewType
```rust
pub enum ViewType {
    ScriptList,           // Main launcher with preview - FIXED height
    ArgPromptWithChoices, // Dynamic height based on items
    ArgPromptNoChoices,   // Compact input-only
    DivPrompt,            // HTML display - standard height
    EditorPrompt,         // Code editor - full height
    TermPrompt,           // Terminal - full height
}
```

## Public Functions

### Height Calculation
```rust
/// Get target height for a view type
pub fn height_for_view(view_type: ViewType, item_count: usize) -> Pixels;

/// Initial window height (STANDARD_HEIGHT)
pub fn initial_window_height() -> Pixels;
```

### Deferred Resize (Preferred)
```rust
/// Use when you have Window access (update closures, hotkey handlers)
/// Uses window_ops::queue_resize internally for coalescing
pub fn defer_resize_to_view(
    view_type: ViewType,
    item_count: usize,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
);
```

### Synchronous Resize
```rust
/// Use when you only have ViewContext (async spawn handlers)
/// Safe outside render cycle
pub fn resize_to_view_sync(view_type: ViewType, item_count: usize);

/// Direct resize (use defer_resize_to_view or queue_resize instead)
pub fn resize_first_window_to_height(target_height: Pixels);

/// Get current main window height
pub fn get_first_window_height() -> Option<Pixels>;

/// Reset debounce timer (no-op, kept for API compatibility)
pub fn reset_resize_debounce();
```

## Dynamic Height Formula (ArgPromptWithChoices)

```rust
let visible_items = item_count.max(1) as f32;
let list_height = (visible_items * LIST_ITEM_HEIGHT) 
                + ARG_LIST_PADDING_Y 
                + ARG_DIVIDER_HEIGHT;
let total_height = ARG_HEADER_HEIGHT + list_height;
// Clamped to [MIN_HEIGHT, STANDARD_HEIGHT]
```

Where `LIST_ITEM_HEIGHT` is from `list_item.rs`.

## Top-Edge Fixed Resizing

macOS uses bottom-left origin (Y=0 at bottom, increases upward).
To keep the TOP of the window fixed during resize:

```rust
// Calculate height difference
let height_delta = new_height - current_height;

// Adjust origin.y downward to compensate
let new_origin_y = current_frame.origin.y - height_delta;
```

## Usage Patterns

### In Window Update Closure
```rust
window.update(cx, |_, window, cx| {
    defer_resize_to_view(ViewType::ArgPromptWithChoices, choices.len(), window, cx);
});
```

### In Async Handler (ViewContext only)
```rust
cx.spawn(|_, _| async move {
    // Safe: runs outside render cycle
    resize_to_view_sync(ViewType::ArgPromptWithChoices, choices.len());
});
```

### Avoid in Render Callbacks
```rust
// BAD: RefCell borrow conflict
fn render(&mut self, cx: &mut ViewContext) -> impl Element {
    resize_first_window_to_height(px(500.0)); // PANIC!
}

// GOOD: Defer to end of effect cycle
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl Element {
    window_ops::queue_resize(500.0, window, cx);
}
```
