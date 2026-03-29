# Core GPUI Fundamentals

GPUI is a hybrid immediate/retained mode, GPU-accelerated UI framework for Rust from Zed.

## Key Mental Model Shift (React vs GPUI)

| Web/React Concept | GPUI Equivalent | Key Difference |
|-------------------|-----------------|----------------|
| `useState` | Entity state | State lives in `Entity<T>`, not components |
| Component | View | Views are Entities that implement `Render` |
| DOM elements | Elements | Elements are rebuilt every frame (immediate mode) |
| Virtual DOM diffing | GPU rendering | No diffing - direct GPU rendering each frame |
| `useEffect` | Subscriptions/observers | Explicit subscription to state changes |
| Props drilling | Context + Entity handles | Pass `Entity<T>` handles, use `cx.global()` |

## Application Lifecycle

### Entry Point Pattern

```rust
use gpui::{Application, App};

fn main() {
    Application::new().run(|cx: &mut App| {
        // Application is now running
        // cx is your App context - use it to:
        // - Open windows
        // - Register global state
        // - Set up observers
    });
}
```

### With GPUI-Component Library

```rust
use gpui::{Application, App};
use gpui_component::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        // MUST call init first when using gpui-component
        gpui_component::init(cx);
        
        // Then open windows...
    });
}
```

## Window Creation

```rust
use gpui::{App, Application, Context, Window, WindowOptions, WindowBounds, Bounds, size, px};

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None, // No specific display
                    size(px(800.0), px(600.0)),
                    cx,
                ))),
                ..Default::default()
            },
            |window, cx| {
                // This closure creates the root view
                cx.new(|_| MyRootView::new())
            },
        );
    });
}
```

## The Three Registers

### Register 1: Entity State Management

```rust
use gpui::{Entity, Context, App};

struct Counter { value: i32 }

// Create an entity
fn create_counter(cx: &mut App) -> Entity<Counter> {
    cx.new(|_| Counter { value: 0 })
}

// Read entity state
fn read_counter(counter: &Entity<Counter>, cx: &App) {
    let value = counter.read(cx).value;
}

// Update entity state
fn update_counter(counter: &Entity<Counter>, cx: &mut App) {
    counter.update(cx, |counter, _cx| {
        counter.value += 1;
    });
}
```

### Register 2: Declarative UI with Views

```rust
use gpui::{div, prelude::*, rgb, Context, IntoElement, Render, SharedString, Window};

struct HelloWorld { text: SharedString }

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x505050))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
    }
}
```

### Register 3: Imperative UI with Elements

```rust
use gpui::{div, prelude::*, px, rgb};

fn build_ui() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .h(px(200.0))
        .p_4()
        .bg(rgb(0x1e1e1e))
        .text_color(rgb(0xffffff))
        .border_1()
        .rounded_md()
        .child("First child")
        .child(div().child("Nested div"))
}
```

## Complete Counter Example

```rust
use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds,
    Context, IntoElement, Render, SharedString, Window,
    WindowBounds, WindowOptions,
};

struct CounterApp { count: i32, label: SharedString }

impl CounterApp {
    fn new() -> Self {
        Self { count: 0, label: "Click me!".into() }
    }

    fn increment(&mut self) {
        self.count += 1;
        self.label = format!("Count: {}", self.count).into();
    }
}

impl Render for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            .size_full()
            .justify_center()
            .items_center()
            .gap_4()
            .child(div().text_xl().text_color(rgb(0xffffff)).child(self.label.clone()))
            .child(
                div()
                    .id("increment-button")
                    .px_4().py_2()
                    .bg(rgb(0x3b82f6))
                    .rounded_md()
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x2563eb)))
                    .on_click(cx.listener(|this, _event, _window, _cx| {
                        this.increment();
                    }))
                    .child("Increment")
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None, size(px(400.0), px(300.0)), cx,
                ))),
                ..Default::default()
            },
            |_window, cx| cx.new(|_| CounterApp::new()),
        );
    });
}
```

## Cargo.toml Setup

```toml
[package]
name = "my-gpui-app"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = "0.2.2"
gpui-component = "0.6.0-preview0"  # Optional
```

## Common Patterns to Avoid

- **Don't store UI in state** - Rebuild UI each render
- **Don't expect component-local state** - State lives in struct fields
- **Don't forget the context** - Always need `cx` for read/update operations
