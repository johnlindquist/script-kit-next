# Focus Management - Expert Bundle

## Overview

Script Kit uses GPUI's focus system for keyboard navigation, focus-aware styling, and proper tab order management.

## Focus Handle Basics

### Creating Focus Handles

```rust
pub struct ArgPrompt {
    input_focus: FocusHandle,
    list_focus: FocusHandle,
    active_focus: FocusTarget,
}

#[derive(Clone, Copy, PartialEq)]
enum FocusTarget {
    Input,
    List,
}

impl ArgPrompt {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            input_focus: cx.focus_handle(),
            list_focus: cx.focus_handle(),
            active_focus: FocusTarget::Input,
        }
    }
}
```

### Implementing Focusable

```rust
impl Focusable for ArgPrompt {
    fn focus_handle(&self, _cx: &Context<Self>) -> FocusHandle {
        match self.active_focus {
            FocusTarget::Input => self.input_focus.clone(),
            FocusTarget::List => self.list_focus.clone(),
        }
    }
}
```

## Focus Tracking

### Track Focus in Elements

```rust
impl Render for ArgPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            // Input section
            .child(
                div()
                    .track_focus(&self.input_focus)
                    .child(self.render_input(cx))
            )
            // List section  
            .child(
                div()
                    .track_focus(&self.list_focus)
                    .child(self.render_list(cx))
            )
    }
}
```

### Checking Focus State

```rust
impl ArgPrompt {
    fn is_input_focused(&self, window: &Window) -> bool {
        self.input_focus.is_focused(window)
    }

    fn is_list_focused(&self, window: &Window) -> bool {
        self.list_focus.is_focused(window)
    }

    fn is_any_focused(&self, window: &Window) -> bool {
        self.input_focus.is_focused(window) || 
        self.list_focus.is_focused(window)
    }
}
```

## Focus Navigation

### Tab Navigation

```rust
impl ArgPrompt {
    fn handle_tab(&mut self, shift: bool, window: &mut Window, cx: &mut Context<Self>) {
        if shift {
            self.focus_previous(window, cx);
        } else {
            self.focus_next(window, cx);
        }
    }

    fn focus_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.active_focus {
            FocusTarget::Input => {
                self.active_focus = FocusTarget::List;
                self.list_focus.focus(window);
            }
            FocusTarget::List => {
                // Wrap around or stay
                self.active_focus = FocusTarget::Input;
                self.input_focus.focus(window);
            }
        }
        cx.notify();
    }

    fn focus_previous(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.active_focus {
            FocusTarget::Input => {
                self.active_focus = FocusTarget::List;
                self.list_focus.focus(window);
            }
            FocusTarget::List => {
                self.active_focus = FocusTarget::Input;
                self.input_focus.focus(window);
            }
        }
        cx.notify();
    }
}
```

### Direct Focus

```rust
impl ArgPrompt {
    fn focus_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_focus = FocusTarget::Input;
        self.input_focus.focus(window);
        cx.notify();
    }

    fn focus_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_focus = FocusTarget::List;
        self.list_focus.focus(window);
        cx.notify();
    }
}
```

## Focus-Aware Styling

### Detecting Focus Changes

```rust
impl App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check and update focus state
        let is_focused = self.focus_handle.is_focused(window);
        
        if self.was_focused != is_focused {
            self.was_focused = is_focused;
            cx.notify(); // Re-render with new focus state
        }
        
        let colors = self.theme.get_colors(is_focused);
        
        div()
            .bg(rgb(colors.background))
            .border_color(rgb(if is_focused { 
                colors.accent 
            } else { 
                colors.border 
            }))
    }
}
```

### Focus Ring

```rust
fn render_focusable_item(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
    let is_focused = self.selected_index == index;
    
    div()
        .when(is_focused, |d| {
            d.outline_2()
             .outline_color(rgb(0x3B82F6))
             .outline_offset(px(-2.0))
        })
        .child(/* content */)
}
```

## Keyboard Event Routing

### Focused Element Receives Events

```rust
impl Render for ArgPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            // ...
    }
}

impl ArgPrompt {
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Only receives events when focused
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            "tab" | "Tab" => {
                self.handle_tab(event.modifiers.shift, window, cx);
            }
            "escape" | "Escape" => {
                self.blur(window, cx);
            }
            _ => {}
        }
    }
}
```

## Auto-Focus

### Focus on Mount

```rust
impl ArgPrompt {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let prompt = Self {
            focus_handle: cx.focus_handle(),
            // ...
        };
        
        // Schedule focus after initial render
        cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_millis(50)).await;
            let _ = this.update(&mut cx, |prompt, cx| {
                if let Some(window) = cx.window() {
                    prompt.focus_handle.focus(&window);
                    cx.notify();
                }
            });
        }).detach();
        
        prompt
    }
}
```

### Focus on Show

```rust
impl App {
    fn show_window(&mut self, cx: &mut Context<Self>) {
        cx.activate(true);
        
        // Focus input after activation
        cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_millis(16)).await;
            let _ = this.update(&mut cx, |app, cx| {
                app.focus_input(cx);
            });
        }).detach();
    }
}
```

## Focus Events

### On Focus/Blur

```rust
impl Render for InputField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_focus(cx.listener(|this, _, _, cx| {
                this.is_focused = true;
                cx.notify();
            }))
            .on_blur(cx.listener(|this, _, _, cx| {
                this.is_focused = false;
                cx.notify();
            }))
    }
}
```

## Window Focus

### App Activation

```rust
impl App {
    fn activate_window(&mut self, cx: &mut Context<Self>) {
        cx.activate(true); // Bring window to front and focus
    }

    fn deactivate_window(&mut self, cx: &mut Context<Self>) {
        cx.activate(false);
    }
}
```

### Focus When Visible

```rust
impl App {
    fn toggle_visibility(&mut self, cx: &mut Context<Self>) {
        if self.visible {
            self.hide(cx);
        } else {
            self.show(cx);
            // Must focus after show
            self.focus_handle.focus(cx.window());
        }
    }
}
```

## Form Focus Management

### Multi-Field Forms

```rust
pub struct FormPrompt {
    fields: Vec<FormField>,
    focused_field: usize,
    field_handles: Vec<FocusHandle>,
}

impl FormPrompt {
    pub fn new(fields: Vec<FormField>, cx: &mut Context<Self>) -> Self {
        let field_handles = fields.iter()
            .map(|_| cx.focus_handle())
            .collect();
        
        Self {
            fields,
            focused_field: 0,
            field_handles,
        }
    }

    fn focus_field(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.field_handles.len() {
            self.focused_field = index;
            self.field_handles[index].focus(window);
            cx.notify();
        }
    }

    fn handle_enter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focused_field < self.fields.len() - 1 {
            // Move to next field
            self.focus_field(self.focused_field + 1, window, cx);
        } else {
            // Submit form
            self.submit(cx);
        }
    }
}
```

## Best Practices

### 1. Always Track Focus State

```rust
// Good - track focus for styling
let is_focused = self.focus_handle.is_focused(window);
if self.cached_focus != is_focused {
    self.cached_focus = is_focused;
    cx.notify();
}

// Bad - checking focus without caching causes unnecessary re-renders
```

### 2. Focus After Async Operations

```rust
// Good - focus after delay
cx.spawn(|this, mut cx| async move {
    Timer::after(Duration::from_millis(50)).await;
    this.update(&mut cx, |app, cx| {
        app.focus_input(cx);
    })
}).detach();

// Bad - focus immediately (may not work if element not rendered)
self.focus_handle.focus(window);
```

### 3. Handle Window Activation

```rust
// Good - focus after activation
cx.activate(true);
cx.spawn(|this, mut cx| async move {
    Timer::after(Duration::from_millis(16)).await;
    this.update(&mut cx, |app, cx| {
        app.focus_handle.focus(cx.window());
    })
}).detach();
```

### 4. Clean Up Focus Handles

Focus handles are automatically cleaned up when the view is dropped. No manual cleanup needed.

## Summary

| Operation | Method |
|-----------|--------|
| Create handle | `cx.focus_handle()` |
| Set focus | `handle.focus(window)` |
| Check focus | `handle.is_focused(window)` |
| Track in element | `.track_focus(&handle)` |
| Listen to focus | `.on_focus(...)` |
| Listen to blur | `.on_blur(...)` |
| Receive keys | `.on_key_down(...)` |
