# GPUI Anti-Patterns

Common mistakes that cause bugs. Check this list when debugging.

## Forgetting `cx.notify()`

```rust
// WRONG - UI won't update
fn increment(&mut self) {
    self.count += 1;
}

// CORRECT
fn increment(&mut self, cx: &mut Context<Self>) {
    self.count += 1;
    cx.notify();  // Required for re-render
}
```

## Skipping Root Wrapper

```rust
// WRONG - theming breaks
cx.open_window(opts, |_window, cx| {
    cx.new(|cx| MyView::new())
})

// CORRECT
cx.open_window(opts, |window, cx| {
    let view = cx.new(|cx| MyView::new(window, cx));
    cx.new(|cx| Root::new(view, window, cx))  // Required
})
```

## Missing `gpui_component::init()`

```rust
Application::new().run(|cx: &mut App| {
    gpui_component::init(cx);  // MUST call before opening windows
    cx.open_window(...);
});
```

## Creating Entities in render()

```rust
// WRONG - creates new entity every frame, loses state
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = cx.new(|cx| TextInput::new(window, cx));  // BAD
        div().child(input)
    }
}

// CORRECT - create once in new(), store in struct
struct MyView { input: Entity<TextInput> }

impl MyView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self { input: cx.new(|cx| TextInput::new(window, cx)) }
    }
}
```

## Blocking the Main Thread

```rust
// WRONG - freezes UI
let data = reqwest::blocking::get(url).unwrap();

// CORRECT - use async
cx.spawn(|this, mut cx| async move {
    let data = reqwest::get(url).await.unwrap();
    this.update(&mut cx, |this, cx| {
        this.data = data;
        cx.notify();
    }).ok();
}).detach();
```

## Not Detaching Spawned Tasks

```rust
// WRONG - task may never run (dropped immediately)
cx.spawn(|this, mut cx| async move { ... });

// CORRECT
cx.spawn(|this, mut cx| async move { ... }).detach();
```

## Debugging Checklist

1. UI not updating? → Missing `cx.notify()`
2. Theming broken? → Missing `init()` or `Root`
3. State resetting? → Creating entities in `render()`
4. App freezing? → Blocking calls on main thread
5. Async not completing? → Task not detached
