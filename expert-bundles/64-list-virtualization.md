# List Virtualization - Expert Bundle

## Overview

Script Kit uses GPUI's `uniform_list` for high-performance list rendering with fixed-height items, scroll handling, and keyboard navigation coalescing.

## Uniform List Basics

### Core Pattern

```rust
use gpui::{uniform_list, UniformListScrollHandle, ScrollStrategy};

pub struct ScriptList {
    scripts: Vec<Arc<Script>>,
    filtered_scripts: Vec<Arc<Script>>,
    selected_index: usize,
    list_scroll_handle: UniformListScrollHandle,
}

impl ScriptList {
    pub fn new(scripts: Vec<Arc<Script>>) -> Self {
        Self {
            filtered_scripts: scripts.clone(),
            scripts,
            selected_index: 0,
            list_scroll_handle: UniformListScrollHandle::new(),
        }
    }
}

impl Render for ScriptList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let item_count = self.filtered_scripts.len();
        
        uniform_list(
            "script-list",
            item_count,
            cx.processor(|this, range, _window, _cx| {
                this.render_items(range)
            }),
        )
        .h_full()
        .track_scroll(&self.list_scroll_handle)
    }
}
```

### Fixed Item Height (CRITICAL)

```rust
const ITEM_HEIGHT: f32 = 52.0; // Must be consistent!

impl ScriptList {
    fn render_items(&self, range: Range<usize>) -> Vec<impl IntoElement> {
        range.map(|index| {
            self.render_item(index)
        }).collect()
    }

    fn render_item(&self, index: usize) -> impl IntoElement {
        let script = &self.filtered_scripts[index];
        let is_selected = index == self.selected_index;
        
        div()
            .h(px(ITEM_HEIGHT)) // Fixed height - required!
            .w_full()
            .px_3()
            .flex()
            .items_center()
            .when(is_selected, |d| d.bg(rgb(0x3B82F6).opacity(0.2)))
            .child(
                div()
                    .flex_1()
                    .child(&script.name)
            )
    }
}
```

## Scroll Handling

### Scroll to Selection

```rust
impl ScriptList {
    fn scroll_to_selected(&self) {
        self.list_scroll_handle.scroll_to_item(
            self.selected_index,
            ScrollStrategy::Nearest,
        );
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_scripts.len().saturating_sub(1) {
            self.selected_index += 1;
            self.scroll_to_selected();
            cx.notify();
        }
    }

    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_to_selected();
            cx.notify();
        }
    }
}
```

### Scroll Strategies

```rust
pub enum ScrollStrategy {
    /// Only scroll if item is not visible
    Nearest,
    /// Always center the item
    Center,
    /// Scroll to top of viewport
    Top,
}

// Usage examples
self.list_scroll_handle.scroll_to_item(index, ScrollStrategy::Nearest);
self.list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top); // Jump to top
```

## Keyboard Navigation Coalescing

### The Problem

Without coalescing, rapid arrow key presses can cause:
- UI lag
- Missed keystrokes  
- Janky scrolling

### Coalescing Implementation

```rust
use std::time::{Duration, Instant};

pub struct ScrollCoalescer {
    pending_dir: Option<ScrollDirection>,
    pending_delta: i32,
    last_scroll_time: Instant,
    coalesce_window: Duration,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
}

impl ScrollCoalescer {
    pub fn new() -> Self {
        Self {
            pending_dir: None,
            pending_delta: 0,
            last_scroll_time: Instant::now(),
            coalesce_window: Duration::from_millis(20),
        }
    }

    /// Queue a scroll event, returns true if should start batch
    pub fn queue(&mut self, dir: ScrollDirection) -> bool {
        let now = Instant::now();
        
        if now.duration_since(self.last_scroll_time) < self.coalesce_window
           && self.pending_dir == Some(dir) {
            // Coalesce with existing batch
            self.pending_delta += 1;
            false
        } else {
            // Start new batch
            self.pending_dir = Some(dir);
            self.pending_delta = 1;
            self.last_scroll_time = now;
            true
        }
    }

    /// Take pending delta, resetting state
    pub fn take(&mut self) -> Option<(ScrollDirection, i32)> {
        let result = self.pending_dir.take().map(|dir| (dir, self.pending_delta));
        self.pending_delta = 0;
        result
    }
}
```

### Integration with List

```rust
impl ScriptList {
    fn handle_arrow_key(&mut self, dir: ScrollDirection, cx: &mut Context<Self>) {
        if self.scroll_coalescer.queue(dir) {
            // First in batch - schedule flush
            cx.spawn(|this, mut cx| async move {
                Timer::after(Duration::from_millis(20)).await;
                let _ = this.update(&mut cx, |list, cx| {
                    list.flush_scroll(cx);
                });
            }).detach();
        }
    }

    fn flush_scroll(&mut self, cx: &mut Context<Self>) {
        if let Some((dir, delta)) = self.scroll_coalescer.take() {
            match dir {
                ScrollDirection::Up => {
                    self.selected_index = self.selected_index.saturating_sub(delta as usize);
                }
                ScrollDirection::Down => {
                    let max = self.filtered_scripts.len().saturating_sub(1);
                    self.selected_index = (self.selected_index + delta as usize).min(max);
                }
            }
            self.scroll_to_selected();
            cx.notify();
        }
    }
}
```

## Page Navigation

```rust
const VISIBLE_ITEMS: usize = 10; // Approximate visible items

impl ScriptList {
    fn page_down(&mut self, cx: &mut Context<Self>) {
        let max = self.filtered_scripts.len().saturating_sub(1);
        self.selected_index = (self.selected_index + VISIBLE_ITEMS).min(max);
        self.scroll_to_selected();
        cx.notify();
    }

    fn page_up(&mut self, cx: &mut Context<Self>) {
        self.selected_index = self.selected_index.saturating_sub(VISIBLE_ITEMS);
        self.scroll_to_selected();
        cx.notify();
    }

    fn jump_to_start(&mut self, cx: &mut Context<Self>) {
        self.selected_index = 0;
        self.scroll_to_selected();
        cx.notify();
    }

    fn jump_to_end(&mut self, cx: &mut Context<Self>) {
        self.selected_index = self.filtered_scripts.len().saturating_sub(1);
        self.scroll_to_selected();
        cx.notify();
    }
}
```

## Keyboard Event Handling

```rust
impl ScriptList {
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            // Handle both arrow key variants!
            "up" | "arrowup" => {
                self.handle_arrow_key(ScrollDirection::Up, cx);
            }
            "down" | "arrowdown" => {
                self.handle_arrow_key(ScrollDirection::Down, cx);
            }
            "pageup" | "PageUp" => {
                self.page_up(cx);
            }
            "pagedown" | "PageDown" => {
                self.page_down(cx);
            }
            "home" | "Home" => {
                self.jump_to_start(cx);
            }
            "end" | "End" => {
                self.jump_to_end(cx);
            }
            _ => {}
        }
    }
}
```

## Filtering

```rust
impl ScriptList {
    fn filter(&mut self, query: &str, cx: &mut Context<Self>) {
        let query_lower = query.to_lowercase();
        
        self.filtered_scripts = self.scripts
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower) ||
                s.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        
        // Reset selection to valid range
        self.selected_index = self.selected_index.min(
            self.filtered_scripts.len().saturating_sub(1)
        );
        
        self.scroll_to_selected();
        cx.notify();
    }
}
```

## Click Handling

```rust
impl ScriptList {
    fn render_item(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let script = &self.filtered_scripts[index];
        let is_selected = index == self.selected_index;
        
        div()
            .id(ElementId::from(index))
            .h(px(ITEM_HEIGHT))
            .w_full()
            .cursor_pointer()
            .when(is_selected, |d| d.bg(selection_color))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected_index = index;
                this.submit(cx);
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                this.selected_index = index;
                cx.notify();
            }))
            .child(&script.name)
    }
}
```

## Hover States

```rust
impl ScriptList {
    fn render_item(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let is_selected = index == self.selected_index;
        
        div()
            .h(px(ITEM_HEIGHT))
            .w_full()
            .when(is_selected, |d| d.bg(selection_color))
            .hover(|d| d.bg(hover_color))
            .child(/* ... */)
    }
}
```

## Performance Tips

### 1. Use Arc for Items

```rust
// Good - cheap clone for range iteration
filtered_scripts: Vec<Arc<Script>>

// Bad - clones entire script
filtered_scripts: Vec<Script>
```

### 2. Extract Copyable Colors

```rust
// Good - Copy type in closure
let colors = self.theme.list_item_colors(); // ListItemColors is Copy

uniform_list("items", count, move |_this, range, _w, _cx| {
    range.map(|i| {
        div().bg(rgb(colors.background)) // colors is Copy
    }).collect()
})

// Bad - requires Clone/move
let theme = self.theme.clone(); // Theme is not Copy
```

### 3. Avoid Re-filtering on Every Render

```rust
// Good - filter once, store result
fn set_filter(&mut self, query: &str, cx: &mut Context<Self>) {
    self.filtered_scripts = filter(&self.scripts, query);
    cx.notify();
}

// Bad - re-filter on every render
fn render(&mut self) {
    let filtered = filter(&self.scripts, &self.query); // Expensive!
}
```

## Performance Thresholds

- **Single key latency**: < 16.67ms (60fps)
- **Scroll operation**: < 8ms
- **P95 key latency**: < 50ms
- **Filter 1000 items**: < 10ms

## Summary

1. **Fixed item height** (52px) is required for virtualization
2. **Coalesce rapid key events** (20ms window)
3. **Handle both arrow key variants** (`"up"` and `"arrowup"`)
4. **Use `ScrollStrategy::Nearest`** for smooth scrolling
5. **Arc-wrap items** for cheap cloning
6. **Extract copyable colors** for closures
7. **Filter once, render many** - avoid re-filtering in render
