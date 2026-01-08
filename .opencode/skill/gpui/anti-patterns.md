# Common Anti-Patterns in GPUI

## Anti-Pattern 1: Forgetting `gpui_component::init()`

### Wrong
```rust
fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(opts, |window, cx| { ... });  // Missing init!
    });
}
```

### Correct
```rust
fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);  // Initialize first!
        cx.open_window(opts, |window, cx| { ... });
    });
}
```

## Anti-Pattern 2: Skipping the Root View

### Wrong
```rust
cx.open_window(opts, |_window, cx| {
    cx.new(|cx| MyAppView::new())  // No Root wrapper!
})
```

### Correct
```rust
cx.open_window(opts, |window, cx| {
    let view = cx.new(|cx| MyAppView::new(window, cx));
    cx.new(|cx| Root::new(view, window, cx))  // Wrap in Root
})
```

## Anti-Pattern 3: Using React useState Mental Model

### Wrong (Doesn't Exist)
```rust
impl Render for MyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let [count, set_count] = use_state(0);  // No such API!
    }
}
```

### Correct
```rust
struct MyView { count: i32 }  // State in struct

impl MyView {
    fn increment(&mut self, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();  // Required for re-render!
    }
}
```

## Anti-Pattern 4: Forgetting `cx.notify()`

### Wrong
```rust
impl Counter {
    fn increment(&mut self) {
        self.count += 1;  // UI won't update!
    }
}
```

### Correct
```rust
impl Counter {
    fn increment(&mut self, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();  // Marks entity for re-render
    }
}
```

**When you need notify():**
- After ANY state change that should update UI
- In event handlers that modify state
- After async operations complete

## Anti-Pattern 5: Prop Drilling Instead of Entity Handles

### Wrong (React-Style)
```rust
struct Parent { count: i32 }
struct Child { on_increment: Box<dyn Fn()> }  // Callback drilling
```

### Correct (Entity Handles)
```rust
struct AppState { count: i32 }
struct Parent { state: Entity<AppState> }
struct Child { state: Entity<AppState> }  // Same handle

impl Child {
    fn handle_click(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.count += 1;
            cx.notify();
        });
    }
}
```

## Anti-Pattern 6: Blocking the Main Thread

### Wrong
```rust
fn load_data(&mut self, _cx: &mut Context<Self>) {
    let data = reqwest::blocking::get(url).unwrap();  // Freezes app!
    self.data = data;
}
```

### Correct
```rust
fn load_data(&mut self, cx: &mut Context<Self>) {
    cx.spawn(|this, mut cx| async move {
        let data = reqwest::get(url).await.unwrap();
        this.update(&mut cx, |this, cx| {
            this.data = data;
            cx.notify();
        }).ok();
    }).detach();
}
```

**Blocking operations to avoid:** Network requests, file I/O, database queries, heavy computation, `thread::sleep`

## Anti-Pattern 7: Incorrect Async Patterns

### Wrong: Not Detaching
```rust
cx.spawn(|this, mut cx| async move {
    // May never run - task dropped immediately!
});
```

### Correct
```rust
cx.spawn(|this, mut cx| async move { ... }).detach();
// Or store: self.task = Some(cx.spawn(...));
```

### Wrong: Forgetting State Update
```rust
cx.spawn(|this, mut cx| async move {
    let result = fetch_data().await;
    // Data fetched but state never updated!
}).detach();
```

### Correct
```rust
cx.spawn(|this, mut cx| async move {
    let result = fetch_data().await;
    this.update(&mut cx, |this, cx| {
        this.data = result;
        cx.notify();
    }).ok();
}).detach();
```

## Anti-Pattern 8: Creating Entities in Render

### Wrong
```rust
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = cx.new(|cx| TextInput::new(window, cx));  // NEW every frame!
        div().child(input)
    }
}
```

### Correct
```rust
struct MyView { input: Entity<TextInput> }

impl MyView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self { input: cx.new(|cx| TextInput::new(window, cx)) }  // Create once
    }
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.input.clone())  // Reuse existing
    }
}
```

**Signs of this problem:** Input loses focus, state resets, memory grows, performance degrades.

## Anti-Pattern 9: Virtual DOM Assumptions

### Wrong: Caching UI
```rust
struct MyView { cached_list: Vec<Div> }  // Don't cache elements!
```

### Correct: Rebuild Every Frame
```rust
struct MyView { items: Vec<String> }

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().children(self.items.iter().map(|item| div().child(item.clone())))
    }
}
```

GPUI is GPU-accelerated - element creation is cheap, no diffing overhead.

## Quick Reference

| Don't | Do |
|-------|-----|
| Forget `gpui_component::init(cx)` | Initialize before opening windows |
| Skip `Root` wrapper | Always wrap top-level view in `Root` |
| Look for `useState` | Put state in struct fields |
| Forget `cx.notify()` | Call after every state mutation |
| Drill props through layers | Pass `Entity<T>` handles |
| Block main thread | Use `cx.spawn()` for async work |
| Create entities in `render()` | Create in `new()`, store in struct |

## Debugging Checklist

1. **UI not updating?** Missing `cx.notify()`
2. **Theming broken?** Missing `init()` or `Root`
3. **State resetting?** Creating entities in `render()`
4. **App freezing?** Blocking calls on main thread
5. **Async not completing?** Task not detached
