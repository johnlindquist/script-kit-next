# GPUI Quick Reference: Top 10 Gotchas for AI Agents

This is a condensed reference extracted from the full GPUI_GOTCHAS_AND_PATTERNS.md document.

---

## 1. Subscriptions Must Be Stored

**Gotcha**: Subscriptions are cancelled when dropped.

```rust
// WRONG: Never fires
cx.observe(&entity, |_this, _cx| { println!("Never!"); });

// RIGHT: Store in struct
pub struct MyView {
    subscriptions: Vec<Subscription>,
}

self.subscriptions.push(cx.observe(&entity, |_this, _cx| {
    println!("Fires correctly");
}));
```

**Why**: GPUI uses lifetime-based cleanup. When `Subscription` is dropped, the callback is unregistered.

---

## 2. cx.notify() Required for Reactivity

**Gotcha**: GPUI has NO automatic change detection.

```rust
// WRONG: UI doesn't update
div().on_click(cx.listener(|this, _ev, _window, cx| {
    this.count += 1;
    // Forgotten: cx.notify();
}))

// RIGHT: Explicitly notify
div().on_click(cx.listener(|this, _ev, _window, cx| {
    this.count += 1;
    cx.notify(); // Tells GPUI to re-render
}))
```

---

## 3. Async Tasks Must Detach or Store

**Gotcha**: Tasks are cancelled if dropped.

```rust
// WRONG: Task cancelled immediately
cx.spawn(async { /* never runs */ });

// RIGHT: Detach to let it run
cx.spawn(async { /* runs */ }).detach();

// OR: Store the task
self.tasks.push(cx.spawn(async { /* runs */ }));
```

---

## 4. Render() Cannot Spawn Async

**Gotcha**: Render is synchronous only.

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // COMPILE ERROR
        // cx.spawn(async { ... });

        // Move async work to event handlers instead
        div().on_click(cx.listener(|_this, _ev, _window, cx| {
            cx.spawn(async { /* ... */ }).detach();
        }))
    }
}
```

---

## 5. Multiple Borrow Conflicts with Context

**Gotcha**: Using outer `cx` in listener closure when inner `cx` is available.

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div().on_click(cx.listener(|_this, _ev, _window, cx_listener| {
            // Use cx_listener here, not outer cx
            cx_listener.some_call(); // ✓ Correct
            // cx.some_call(); // ✗ Borrow conflict
        }))
    }
}
```

---

## 6. Complete Focus Chain Required for Keyboard

**Gotcha**: Missing any link breaks keyboard input.

```rust
pub struct MyInput {
    focus_handle: FocusHandle,
}

impl MyInput {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(), // Step 1: Create
        }
    }
}

impl Focusable for MyInput {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone() // Step 2: Implement trait
    }
}

impl Render for MyInput {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle) // Step 3: Connect UI
            .on_action(cx.listener(|this, _: &MyAction, _window, cx| {
                // Step 4: Register handler
            }))
    }
}
```

All four steps required. Missing one = no keyboard input.

---

## 7. RenderOnce vs Render: Choose the Right Trait

**Gotcha**: Using wrong trait for your use case.

| Trait | Use When | Signature | State |
|-------|----------|-----------|-------|
| `RenderOnce` | Reusable, stateless components | `fn render(self)` | None |
| `Render` | Interactive, state-holding views | `fn render(&mut self, cx)` | Persistent |

```rust
// RenderOnce: Simple button (no state)
#[derive(IntoElement)]
struct Button { label: String }
impl RenderOnce for Button {
    fn render(self) -> impl IntoElement { div().child(self.label) }
}

// Render: Text input (has state)
struct TextInput { value: String }
impl Render for TextInput {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(&self.value)
    }
}
```

---

## 8. Weak References Must Be Upgraded Safely

**Gotcha**: Forgetting to handle `None` case.

```rust
// WRONG: Assumes weak ref is still valid
let entity = weak_entity.upgrade(cx).unwrap(); // Panic if dropped!

// RIGHT: Handle the None case
if let Some(entity) = weak_entity.upgrade(cx) {
    entity.update(cx, |view, _cx| { /* ... */ });
} else {
    log::warn!("Entity was dropped");
}
```

`WeakEntity::upgrade()` returns `Option`. The referenced entity may have been dropped.

---

## 9. Async Contexts Are Fallible

**Gotcha**: Async contexts can outlive the app.

```rust
cx.spawn(async move |cx: &mut AsyncApp| {
    // All operations are fallible (may return None)
    let result = cx.update(|cx| {
        // Work here
    });

    if result.is_none() {
        log::warn!("App was closed before update");
        return;
    }
});
```

Synchronous contexts assume app/window are alive. Async cannot make this assumption.

---

## 10. Nested Entity.update() Panics

**Gotcha**: Trying to update the same entity twice.

```rust
// PANIC: entity is already borrowed
entity.update(cx, |entity, cx| {
    entity.update(cx, |_inner, _cx| { /* PANIC! */ });
});

// FIX: Do all mutations in one closure
entity.update(cx, |entity, cx| {
    entity.do_first_thing();
    entity.do_second_thing();
});
```

GPUI prevents reentrancy. Only one `update()` per entity at a time.

---

## Bonus: Action Binding Must Be Explicit

**Gotcha**: Registering handler without keymap binding = handler never fires.

```rust
// Step 1: Define action
#[derive(Clone)]
pub struct MoveUp;

// Step 2: Register handler
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .on_action(cx.listener(|this, _: &MoveUp, _window, cx| {
                this.move_selection(-1, cx);
            }))
    }
}

// Step 3: Bind in keymap.json
// "up": "my_view::MoveUp"
```

Without step 3, the handler never fires. Actions require explicit keybindings.

---

## See Also

- Full documentation: `/Users/johnlindquist/dev/script-kit-gpui/.claude/GPUI_GOTCHAS_AND_PATTERNS.md`
- GPUI on GitHub: https://github.com/zed-industries/zed/tree/main/crates/gpui
- Zed's Blog: https://zed.dev/blog/gpui-ownership
