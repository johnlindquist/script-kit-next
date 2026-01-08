# Entity and State Management in GPUI

## Entity<T> Smart Pointers

An `Entity<T>` is a smart pointer handle to state owned by the GPUI application.

### Creating Entities

```rust
use gpui::*;

struct Counter { count: i32 }

impl Counter {
    fn new() -> Self { Self { count: 0 } }
    fn increment(&mut self) { self.count += 1; }
}

fn create_counter(cx: &mut App) -> Entity<Counter> {
    cx.new(|_cx| Counter::new())
}
```

### Reading and Updating Entities

```rust
// Read entity state
entity.read(cx).count  // Returns i32

// Update entity state  
entity.update(cx, |counter, cx| {
    counter.increment();
    cx.notify(); // Trigger re-render
});
```

## Context Types

### App Context

Root context that owns all entities' data.

```rust
fn main() {
    App::new().run(|cx: &mut App| {
        let global_state = cx.new(|_| AppState::default());
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new(|cx| RootView::new(global_state.clone(), cx))
        });
    });
}
```

### Context<T> (Entity Context)

Provided when interacting with a specific entity. Dereferences to `App`.

```rust
impl Counter {
    fn increment(&mut self, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();  // Notify observers
        cx.emit(CounterEvent::Changed(self.count));  // Emit event
    }
}
```

### AsyncApp and AsyncWindowContext

Static lifetime, can be held across `.await` points. Operations become fallible.

```rust
impl MyView {
    fn start_fetch(&mut self, cx: &mut Context<Self>) {
        cx.spawn(|this, mut cx| async move {
            let data = fetch_data().await;
            this.update(&mut cx, |view, cx| {
                view.data = Some(data);
                cx.notify();
            }).ok();  // Note: async updates are fallible
        }).detach();
    }
}
```

## Ownership Model

```
App (root context)
 └── Owns all Entity<T> data
      └── Entities can hold handles to other entities
           └── Entity<T> handles are reference-counted
```

**Key Rules:**
1. App owns all entity data
2. Entity handles are cheap to clone (like `Rc`)
3. Contexts borrow from App
4. Async contexts have `'static` lifetime

## Reactive Patterns

### notify() for Observers

```rust
impl Counter {
    fn set_count(&mut self, value: i32, cx: &mut Context<Self>) {
        self.count = value;
        cx.notify(); // Triggers re-render of this view and observers
    }
}

// Observing an entity
impl ParentView {
    fn new(counter: Entity<Counter>, cx: &mut Context<Self>) -> Self {
        cx.observe(&counter, |this, counter, cx| {
            let count = counter.read(cx).count;
            this.last_count = count;
            cx.notify();
        }).detach();
        Self { counter, last_count: 0 }
    }
}
```

### emit() for Events

```rust
enum CounterEvent { Changed(i32), Reset }

impl EventEmitter<CounterEvent> for Counter {}

impl Counter {
    fn increment(&mut self, cx: &mut Context<Self>) {
        self.count += 1;
        cx.emit(CounterEvent::Changed(self.count));
        cx.notify();
    }
}
```

### subscribe() Patterns

```rust
impl Dashboard {
    fn new(counter: Entity<Counter>, cx: &mut Context<Self>) -> Self {
        cx.subscribe(&counter, |this, _counter, event, cx| {
            match event {
                CounterEvent::Changed(value) => {
                    this.log.push(format!("Counter: {}", value));
                }
                CounterEvent::Reset => {
                    this.log.push("Counter reset".to_string());
                }
            }
            cx.notify();
        }).detach();
        Self { counter, log: Vec::new() }
    }
}
```

## Stateful vs Stateless Components

### Stateless (RenderOnce)

```rust
#[derive(IntoElement)]
struct Button {
    label: SharedString,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl RenderOnce for Button {
    fn render(self, _cx: &mut Window, _app: &mut App) -> impl IntoElement {
        div().px_4().py_2().bg(rgb(0x3b82f6)).child(self.label)
    }
}
```

### Stateful (Render + Entity)

```rust
struct Dropdown {
    items: Vec<SharedString>,
    selected: Option<usize>,
    open: bool,
}

impl Render for Dropdown {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Full render implementation with internal state
    }
}

// Usage - held as Entity in parent
struct FormView {
    dropdown: Entity<Dropdown>,  // Must store the entity handle
}

impl FormView {
    fn new(cx: &mut Context<Self>) -> Self {
        let dropdown = cx.new(|_| Dropdown::new(vec!["A".into(), "B".into()]));
        Self { dropdown }
    }
}
```

### When to Choose Which

| Use Stateless (`RenderOnce`) | Use Stateful (`Render`) |
|------------------------------|-------------------------|
| Simple display components | Components with internal state |
| Buttons, labels, icons | Dropdowns, modals, forms |
| No internal state changes | Need to track open/closed, selection |
| Props-only configuration | Need event emission |
