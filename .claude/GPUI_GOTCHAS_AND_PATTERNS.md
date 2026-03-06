# GPUI Framework: Gotchas, Best Practices, and Patterns for AI Agents

**Target GPUI Revision:** `03416097` (Zed editor)

This document is a comprehensive guide to GPUI patterns and gotchas that AI agents writing code should understand. It covers entity lifecycle, render traits, context types, async patterns, focus management, subscriptions, and common pitfalls.

---

## Table of Contents

1. [Entity Lifecycle & Memory Management](#entity-lifecycle--memory-management)
2. [Render Trait Contract](#render-trait-contract)
3. [Context Types](#context-types)
4. [Asynchronous Execution (cx.spawn)](#asynchronous-execution-cxspawn)
5. [Focus System & Keyboard Events](#focus-system--keyboard-events)
6. [Subscriptions & Observers](#subscriptions--observers)
7. [Element Traits: RenderOnce vs Render](#element-traits-renderonce-vs-render)
8. [The Critical cx.notify() Requirement](#the-critical-cxnotify-requirement)
9. [Overlays & Positioning](#overlays--positioning)
10. [Scroll Management: ListState vs UniformListScrollHandle](#scroll-management-liststate-vs-uniformlistscrollhandle)
11. [Common Gotchas & Traps](#common-gotchas--traps)
12. [Event Dispatch System](#event-dispatch-system)

---

## Entity Lifecycle & Memory Management

### Core Model: App Owns All Entities

Unlike traditional Rust code where you own data directly, GPUI inverts this model:

```rust
// WRONG: Direct ownership doesn't work
struct Counter { count: i32 }
let counter = Counter { count: 0 }; // Won't integrate with GPUI

// RIGHT: App owns all entities
let counter_entity: Entity<Counter> = cx.new_entity(|_cx| Counter { count: 0 });
```

**Key principle**: "By itself, `Entity<Counter>` handle doesn't provide access to the entity's state...it maintains a reference count to the underlying `Counter` object that is owned by the app."

### Entity Handles vs Direct References

- **`Entity<T>`**: A strong handle that prevents deallocation. Implement `Render` on `T` to make it renderable.
- **`WeakEntity<T>`**: A weak handle that doesn't prevent deallocation. Use for circular references to avoid memory leaks.

```rust
// Strong reference (keeps entity alive)
struct Parent {
    child: Entity<Child>,
}

// Weak reference (doesn't keep entity alive - prevents cycles)
struct Child {
    parent: WeakEntity<Parent>, // Use downgrade() to create
}

// Safe upgrade pattern
let child = Child {
    parent: parent.downgrade(),
};
```

### The Subscription Trap: Must Be Stored

**Critical gotcha**: Subscriptions are cancelled when dropped.

```rust
// WRONG: Subscription is dropped immediately
fn register_observer(cx: &mut Context<Self>) {
    cx.observe(&some_entity, |_this, _cx| {
        // This callback NEVER fires!
        println!("Observer fired");
    });
}

// RIGHT: Store subscription as a field
pub struct MyView {
    subscriptions: Vec<Subscription>,
}

fn register_observer(&mut self, cx: &mut Context<Self>) {
    let subscription = cx.observe(&some_entity, |_this, _cx| {
        println!("Observer fires correctly");
    });
    self.subscriptions.push(subscription);
}
```

**Why**: GPUI uses lifetime-based resource management. When a `Subscription` handle is dropped, GPUI automatically unregisters the callback. This prevents dangling callbacks but requires explicit storage.

### Safe Observation: Three Mechanisms

1. **`cx.observe()`**: Watch any entity for changes
   - Calls provided closure when observed entity calls `cx.notify()`
   - Returns a `Subscription` that must be stored

2. **`cx.subscribe()` and `emit()`**: Typed events
   - Entity implements `EventEmitter<T>`
   - Others call `entity.emit(event, cx)`
   - Returns a `Subscription` that must be stored

3. **`cx.listen()` and `cx.listener()`**: Local listeners
   - Called directly in event handlers
   - Automatically tied to entity lifecycle

### Effect Queuing Prevents Reentrancy

GPUI queues effects rather than invoking them immediately:

```rust
// Safe pattern: listeners can emit events back to the same emitter
entity.update(cx, |entity, cx| {
    entity.do_something(cx);
    // If do_something calls cx.emit(), the emission is queued
    // and flushed at the end of this update() call.
    // No reentrancy panic!
});
```

"At the end of each update we flush these effects, popping from the front of the queue until it becomes empty."

---

## Render Trait Contract

### What `render()` Can and Cannot Do

The `Render` trait is called every frame (potentially 60+ times/second). Understand the constraints:

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // ✓ Can: Read state, compute UI from state
        // ✓ Can: Access cx to dispatch actions, check focus
        // ✗ Cannot: Spawn async tasks with cx.spawn()
        // ✗ Cannot: Mutate other entities (only self)
        // ✗ Cannot: Make blocking calls
        // ✗ Cannot: Do expensive computations (should cache instead)

        div()
            .child(format!("Count: {}", self.count))
    }
}
```

### The Three-Phase Render Cycle

GPUI separates rendering into phases:

1. **Prepaint phase**: Request layouts (measure)
2. **Paint phase**: Build scene tree (geometry is known)
3. **GPU phase**: Upload and render

Elements proceed through these phases. You cannot request layout in the paint phase because positions are already finalized.

### You Cannot Spawn Async in Render

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // COMPILE ERROR: can't call cx.spawn() here
        // cx.spawn(async { ... });

        // Instead, defer to event handlers:
        div()
            .on_click(cx.listener(|this, _ev, _window, cx| {
                // NOW you can spawn
                cx.spawn(async { ... });
            }))
    }
}
```

### State Mutations in Render

Only `self` can be mutated during render. Mutating other entities causes a panic:

```rust
impl Render for Parent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // ✓ Can mutate self
        self.render_count += 1;

        // ✗ PANIC: Cannot mutate other entities during render
        // self.child.update(cx, |child, _cx| {
        //     child.value = 42; // Panic!
        // });
    }
}
```

Use pre-render setup or event handlers to coordinate multiple entities.

---

## Context Types

GPUI provides multiple context types for different scenarios. Understanding when to use each prevents borrow conflicts and lifetime issues.

### Context Type Hierarchy

| Context | Lifetime | Mutable? | Entity Access | Usage |
|---------|----------|----------|---------------|-------|
| `App` | Synchronous reference | Yes | All entities | Startup, app-wide state |
| `Context<T>` | Synchronous reference | Yes | T + observation | View code |
| `Window` | Synchronous reference | Yes | Window-specific | Platform operations |
| `AsyncApp` | `'static` | No (deferred) | Via `update()` | Background tasks |
| `AsyncWindowContext` | `'static` | No (deferred) | Via `update()` | Async code that needs window |

### Key Pattern: Context Dereferences to App

"Any function which can take an `App` reference can also take a `Context<T>` reference" because `Context<T>` dereferences into `App`.

```rust
fn use_app_functions(cx: &mut App) {
    // Works with both App and Context<T>
}

fn use_context<T>(cx: &mut Context<T>) {
    use_app_functions(cx); // OK: Context<T> dereferences to App
}
```

### Async Fallibility: Contexts May Outlive App

Critical gotcha: Async contexts make operations **fallible**:

```rust
cx.spawn(async move |cx: &mut AsyncApp| {
    // Window or app may have been closed before this fires

    // Fallible: returns Result
    let result = cx.update(|cx| {
        // OK: inside a closure
    });

    // COMPILE ERROR: Can't directly mutate async context
    // cx.notify(); // Not allowed
});
```

This is safer than synchronous contexts: "the context may outlive the window or even the app itself."

### Observation Coupling Creates Dependencies

Observing an entity creates an implicit dependency:

```rust
// In entity B:
cx.observe(&entity_a, |this, _cx| {
    // Now B depends on A being alive for this callback to work
});
```

Document these dependencies explicitly to avoid confusion during refactoring.

---

## Asynchronous Execution (cx.spawn)

### Modern Spawn Signature

**Current (03416097)**: `cx.spawn(async move |this, cx| ...)`

The closure receives:
- `this`: `WeakEntity<T>` (handle to the entity spawning)
- `cx`: `AsyncWindowContext` or `AsyncApp`

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div().on_click(cx.listener(|this, _ev, _window, cx| {
            // Spawn a task that can read/write self
            cx.spawn(async move |this, cx| {
                // Can await here
                cx.background_executor().timer(Duration::from_secs(1)).await;

                // Update self (must be inside closure)
                this.update(&mut cx, |view, _cx| {
                    view.count += 1;
                });
            })
            .detach(); // Must detach or store task
        }))
    }
}
```

### Spawn vs Background Spawn

```rust
// Foreground executor (integrates with UI event loop)
cx.spawn(async { ... })

// Background executor (separate thread pool)
cx.background_executor().spawn(async { ... })
```

For heavy computation, use background executor. For UI updates, use foreground.

### Critical: Must Detach or Store Task Handles

```rust
// WRONG: Task is dropped immediately, cancelling the async work
cx.spawn(async { ... }); // Implicit drop!

// RIGHT: Explicitly detach to let it run
cx.spawn(async { ... }).detach();

// OR: Store the task handle
pub struct MyView {
    pending_tasks: Vec<Task<()>>,
}

impl MyView {
    fn fetch(&mut self, cx: &mut Context<Self>) {
        let task = cx.spawn(async { ... });
        self.pending_tasks.push(task);
    }
}
```

Dropped tasks are **cancelled immediately**—they never run.

### Multiple Borrow Pattern: Inner vs Outer cx

When using closures with cx, always use the **inner** cx provided by the closure:

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div().on_click(cx.listener(|this, _ev, _window, cx| {
            // 'cx' here is the listener's context
            // Use 'cx', not the outer cx from render()

            this.update(cx, |view, cx| {
                view.count += 1;
                cx.notify(); // Use inner cx
            });
        }))
    }
}
```

Using the outer cx causes borrow conflicts:

```rust
// COMPILE ERROR: outer cx already borrowed by listener closure
div()
    .on_click(cx.listener(|this, _ev, _window, cx_inner| {
        // Trying to use outer cx here fails
        let x = cx.some_call(); // Borrow conflict!
    }))
```

### Async Error Handling

Async contexts return `Result` for all operations:

```rust
cx.spawn(async move |cx: &mut AsyncApp| {
    // Fallible: result is Option<()>
    let result = cx.update(|cx| {
        // Work here
    });

    if result.is_none() {
        // App was dropped before update finished
        log::warn!("App was dropped");
        return;
    }
});
```

---

## Focus System & Keyboard Events

### Complete Focus Chain Required

Keyboard shortcuts require a complete chain:

```rust
pub struct MyInput {
    pub focus_handle: FocusHandle,
}

impl MyInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for MyInput {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MyInput {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle) // Connect UI to focus system
            .on_action(cx.listener(|this, action: &MyAction, _window, cx| {
                // Handles MyAction when this view has focus
            }))
    }
}
```

### Focus Handle Lifecycle

- Create in `new()` or `init()`: `cx.focus_handle()`
- Implement `Focusable` trait: return the handle
- Connect UI element: `.track_focus(&handle)`
- Register handlers: `.on_action(cx.listener(...))`

Missing any step breaks keyboard input for that component.

### Event Dispatch Phases

Keyboard events bubble **upward** from focused element:

```
Focused Element
    ↓ (doesn't handle)
Parent Element
    ↓ (doesn't handle)
Window
```

Mouse events use **capture phase** (topmost first), then bubble:

```
Capture phase:
Window → Parent → Child (stops if handler captures)

Bubble phase:
Child → Parent → Window
```

Handlers must understand which phase they operate in:

```rust
div()
    .on_mouse_down(cx.listener(|_this, ev, _window, _cx| {
        // Fires in capture phase (earliest)
        ev.stop_propagation();
    }))
    .on_click(cx.listener(|_this, _ev, _window, _cx| {
        // Fires in bubble phase (after mouse_down)
    }))
```

### Action Binding Pattern

Actions are semantic intents, not raw keyboard events:

```rust
#[derive(Clone, Debug)]
pub struct MoveUp;

#[derive(Clone, Debug)]
pub struct Move { pub direction: Direction }

// In keymap JSON:
// "up" -> "move_up::MoveUp"
// "shift+up" -> ["move_up::Move", {"direction": "up"}]
```

Actions require explicit binding in keymaps. Raw key events don't exist in GPUI—everything goes through actions.

### Focus Gotchas

1. **Implicit loss of focus**: Clicking another element steals focus
2. **Modal focus chains**: Some views capture all keyboard input
3. **WeakEntity focus**: Checking `is_focused()` on a weak entity that's been dropped returns false silently

---

## Subscriptions & Observers

### Two Subscription Patterns

**Pattern 1: Direct Observation**

```rust
pub struct MyView {
    observed_entity: Entity<OtherView>,
    subscriptions: Vec<Subscription>,
}

impl MyView {
    fn observe_changes(&mut self, cx: &mut Context<Self>) {
        let sub = cx.observe(&self.observed_entity, |this, _cx| {
            log::info!("Observed entity changed");
        });
        self.subscriptions.push(sub);
    }
}
```

**Pattern 2: Typed Events (EventEmitter)**

```rust
pub struct MyEvent {
    pub data: String,
}

impl EventEmitter<MyEvent> for OtherView {}

// Later, emit:
other_entity.emit(MyEvent { data: "hello".into() }, cx);

// Subscribe:
let sub = cx.subscribe(
    &other_entity,
    |this, event: &MyEvent, _cx| {
        log::info!("Event: {}", event.data);
    },
);
```

### Subscription Storage Checklist

- [ ] Declared as struct field (e.g., `subscriptions: Vec<Subscription>`)
- [ ] Added during initialization or in event handler
- [ ] Stored for the entity's entire lifetime
- [ ] Removed only when the subscription should stop

Missing storage = callback never fires.

### Reentrancy Safety

GPUI automatically prevents reentrancy by queuing effects:

```rust
// Safe: listener emitting to the same entity
entity.update(cx, |entity, cx| {
    cx.emit(Event1, cx);
    // Event1 handler won't fire until after this update() completes

    // Effects are queued and flushed after the update
});
```

This prevents "entity already being updated" panics that plague naive observers.

---

## Element Traits: RenderOnce vs Render

### RenderOnce: Stateless Components

Use `RenderOnce` for immutable, reusable components:

```rust
#[derive(IntoElement)]
pub struct Button {
    label: SharedString,
    on_click: Option<Box<dyn Fn() + 'static>>,
}

impl RenderOnce for Button {
    type Element = impl IntoElement;

    fn render(self) -> Self::Element {
        // 'self' is consumed, not borrowed
        // No mutable state
        div().child(self.label)
    }
}
```

**Advantages**:
- Lightweight (no entity)
- Functional programming style
- Reusable without GPUI entities

**Disadvantages**:
- No internal state
- Callbacks must be passed in
- Limited interactivity without parent coordination

### Render: Stateful Views

Use `Render` for interactive, state-holding views:

```rust
pub struct TextInput {
    value: String,
    focus_handle: FocusHandle,
}

impl Render for TextInput {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // '&mut self' persists state across frames
        div()
            .child(format!("Value: {}", self.value))
            .on_action(cx.listener(|this, _action, _window, cx| {
                this.value.push('x');
                cx.notify(); // Signal re-render
            }))
    }
}
```

**Advantages**:
- Full GPUI entity integration
- Can spawn async tasks
- Observation and subscriptions
- Complete lifecycle control

**Disadvantages**:
- Heavier (allocates entity)
- More setup required

### Ownership Difference

```rust
// RenderOnce takes ownership
impl RenderOnce for Button {
    fn render(self) { ... } // Takes ownership, consumes Button
}

// Render borrows mutably
impl Render for TextInput {
    fn render(&mut self, cx: ...) { ... } // Borrows, can be called multiple times
```

This ownership difference is fundamental: `RenderOnce` components are one-shot rendering recipes, while `Render` views are persistent state holders.

---

## The Critical cx.notify() Requirement

### No Automatic Change Detection

**GPUI does not have automatic reactivity**. Mutating state without calling `cx.notify()` leaves the UI stale:

```rust
impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(format!("Count: {}", self.count))
            .on_click(cx.listener(|this, _ev, _window, cx| {
                this.count += 1;
                // CRITICAL: Without this, UI doesn't update
                cx.notify();
            }))
    }
}
```

### Why No Auto-Detection?

Performance. Auto-detection via:
- Value comparison (expensive, requires Clone)
- Change tracking (memory overhead)
- Instrumentation (runtime cost)

GPUI trades ergonomics for efficiency. You explicitly say "render this view again."

### When to Call cx.notify()

- After mutating self in an event handler
- Inside `update()` closures when state affects rendering
- After async operations that change state
- **NOT** during render (you'd be re-entering render)

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Render can't call cx.notify() (reentrancy)
        div()
            .on_click(cx.listener(|this, _ev, _window, cx| {
                this.do_work();
                cx.notify(); // ✓ Correct: in event handler
            }))
    }
}

pub struct MyView { /* ... */ }

impl MyView {
    pub fn external_update(&mut self, cx: &mut Context<Self>) {
        self.do_work();
        cx.notify(); // ✓ Correct: after external update
    }
}
```

### Forgetting cx.notify() Symptoms

- State changes but UI doesn't update
- Works on first render, then stalls
- "Why didn't my button update when I clicked it?"
- Manual window resize forces refresh (repaint clears the stale state)

---

## Overlays & Positioning

### Anchored Overlays for Dialogs

Use absolute positioning anchored to parent elements:

```rust
fn render_actions_backdrop(
    show_popup: bool,
    actions_dialog: Option<Entity<ActionsDialog>>,
    cx: &mut Context<ScriptListApp>,
) -> Option<impl IntoElement> {
    if !show_popup {
        return None;
    }

    let dialog = actions_dialog?;

    Some(
        div()
            .absolute()
            .top(px(HEADER_HEIGHT + PADDING_SM))
            .right(px(PADDING_SM))
            .child(dialog) // Render the dialog entity
    )
}
```

**Key patterns**:
- `.absolute()` + `.top()` / `.right()` / `.bottom()` / `.left()`
- Calculate offsets from known constants (header height, etc.)
- Render as children of a container div
- Close via click handlers on backdrop

### No Native "Popup" Primitive

GPUI doesn't have a built-in popup or modal. Build them:

```rust
// Deferred show pattern (safe way to open windows)
pub fn show_ai_window_deferred(cx: &mut Context<ScriptListApp>) {
    cx.spawn(async move |this, cx| {
        // Deferred: runs after current event handler completes
        this.update(&mut cx, |app, cx| {
            app.show_ai_window(cx);
        })
        .ok();
    })
    .detach();
}
```

Why defer? Opening a window in the middle of event handling can cause reentrancy issues. Deferring via `cx.spawn()` queues the operation safely.

---

## Scroll Management: ListState vs UniformListScrollHandle

### When Scroll State Matters

Large lists need scroll management to:
- Jump to specific items
- Auto-scroll on selection change
- Track scroll position during mutations

### ListState: Variable-Height Lists

Use `ListState` for lists with items of different heights:

```rust
pub struct MyView {
    list_state: ListState, // Manages scroll for gpui list()
}

impl MyView {
    fn scroll_to_item(&mut self, index: usize, cx: &mut Context<Self>) {
        self.list_state.scroll_to_reveal_item(index);
        cx.notify();
    }
}

impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        list(self.list_state.clone())
            .items(self.items.iter().enumerate().map(|(ix, item)| {
                // Item rendering
            }))
    }
}
```

**Capabilities**:
- `scroll_to_reveal_item(ix)`: Scroll item into view
- `scroll_to_item(ix, alignment)`: Custom alignment
- Track scroll on mutations

### UniformListScrollHandle: Fixed-Height Lists

Use for performance with uniform-height items:

```rust
let handle = UniformListScrollHandle::new();

uniform_list(
    self.items.len(),
    |ix, _cx| {
        // Item rendering
    },
    handle,
)
```

### Scroll Activity Pattern

Show scrollbar on scroll, fade after inactivity:

```rust
fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
    self.last_scroll_time = Some(Instant::now());
    self.scrollbar_visibility = Opacity::VISIBLE;
    let fade_gen = self.scrollbar_fade_gen;

    cx.spawn(async move |this, cx| {
        // Wait for idle timeout
        cx.background_executor().timer(IDLE_DURATION).await;

        // Check if still idle
        let should_fade = cx.update(|cx| {
            this.update(cx, |app, _cx| {
                app.scrollbar_fade_gen == fade_gen
                    && app.last_scroll_time.elapsed() >= IDLE_DURATION
            })
        }).unwrap_or(false);

        if should_fade {
            // Animate fade
        }
    })
    .detach();
}
```

---

## Common Gotchas & Traps

### 1. Entity Panic: "Already Being Updated"

```rust
// PANIC: Nested update() on same entity
entity.update(cx, |entity, cx| {
    entity.update(cx, |inner, _cx| { // PANIC!
        // entity is already borrowed
    });
});

// FIX: Perform mutations in one closure
entity.update(cx, |entity, cx| {
    entity.do_first_thing();
    entity.do_second_thing();
});
```

### 2. Weak Reference Gotcha

```rust
// WRONG: Weak ref might be dropped
pub fn some_handler(weak: WeakEntity<MyView>, cx: &mut Context<Self>) {
    // weak might point to dropped entity
    if let Some(entity) = weak.upgrade(cx) {
        entity.update(cx, |view, _cx| { /* ... */ });
    }
}
```

`WeakEntity::upgrade()` returns `Option`. Always handle the `None` case.

### 3. Subscription Callback Never Fires

Common mistake: forgetting to store subscription.

```rust
// WRONG: Subscription dropped immediately
fn init_observers(&mut self, cx: &mut Context<Self>) {
    cx.observe(&self.model, |_this, _cx| {
        println!("This never prints!");
    });
    // ^ Subscription is dropped here
}

// RIGHT: Store subscription
fn init_observers(&mut self, cx: &mut Context<Self>) {
    let sub = cx.observe(&self.model, |_this, _cx| {
        println!("This prints when model changes");
    });
    self.subscriptions.push(sub);
}
```

### 4. Spawn Task Not Running

```rust
// WRONG: Task is dropped, never runs
cx.spawn(async {
    // This code never executes
    println!("Never runs");
});

// RIGHT: Detach to let it run
cx.spawn(async {
    println!("Runs correctly");
})
.detach();
```

### 5. Render Calling cx.spawn()

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // COMPILE ERROR: can't call cx.spawn() during render
        // cx.spawn(async { ... });

        // Move async work to event handlers
        div()
            .on_click(cx.listener(|_this, _ev, _window, cx| {
                cx.spawn(async { /* ... */ }).detach();
            }))
    }
}
```

### 6. Multiple Borrow Conflicts

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Using listener's cx, not outer cx
        div()
            .on_click(cx.listener(|_this, _ev, _window, cx_listener| {
                // Use cx_listener, not cx
                // cx.some_call(); // Would fail: cx already borrowed
                cx_listener.some_call(); // ✓ Correct
            }))
    }
}
```

### 7. State Mutation During Render

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_count += 1; // ✓ OK: mutating self

        // PANIC: Mutating other entity during render
        // self.child.update(cx, |child, _cx| {
        //     child.value = 42;
        // });
    }
}
```

### 8. Observing Dropped Entities

```rust
// If observed_entity is dropped, observer still fires (GPUI detects and skips)
let sub = cx.observe(&observed_entity, |this, _cx| {
    // Safe: GPUI handles dropped entities gracefully
});
```

GPUI checks entity validity before calling observers. Safe but adds small overhead.

---

## Event Dispatch System

### Keyboard-First Design

GPUI is keyboard-centric. There are **no raw key events**—everything goes through Actions:

```rust
// Define action
#[derive(Clone, Debug)]
pub struct MoveUp;

#[derive(Clone, Debug)]
pub struct Move {
    pub direction: Direction,
    pub select: bool,
}

// Handle in component
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("my_view")
            .on_action(cx.listener(|this, _: &MoveUp, _window, cx| {
                this.move_selection(Direction::Up, cx);
            }))
            .on_action(cx.listener(|this, action: &Move, _window, cx| {
                this.move_selection(action.direction, cx);
            }))
    }
}

// Bind in keymap.json
// "up": "my_view::MoveUp"
// "shift+up": ["my_view::Move", {"direction": "up", "select": true}]
```

### Action Binding Requirement

**Critical**: "To expose functionality to the keyboard, you bind an _action_ in a _key context_. Nothing happens without explicit configuration."

Without keymap binding, action handlers never fire, even if registered.

### Context Scoping

Actions only fire within their declared context:

```rust
div()
    .key_context("outer")
    .on_action(cx.listener(|_this, _: &MyAction, _window, _cx| {
        // Fires when outer context is active
    }))
    .child(
        div()
            .key_context("inner") // Nested context
            .on_action(cx.listener(|_this, _: &MyAction, _window, _cx| {
                // Fires when inner context is active
                // Overrides outer if bound to same key
            }))
    )
```

### Complex Action Serialization

Parameterized actions need serialized JSON in keymaps:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct JumpTo {
    pub line: u32,
    pub column: u32,
}

// In keymap.json (requires full serialized form):
// "ctrl+g": ["editor::JumpTo", {"line": 0, "column": 0}]
```

GPUI deserializes the JSON into `JumpTo` before dispatching.

---

## Summary: AI Agent Checklist

When writing GPUI code, verify:

- [ ] **Entities**: Does the entity need async, observation, or identity? Use `Entity<T>`.
- [ ] **Subscriptions**: All subscriptions stored as struct fields?
- [ ] **cx.notify()**: Explicitly called after state mutations?
- [ ] **Weak references**: Only used for cycles, properly upgraded?
- [ ] **Render trait**: Only sync code, no async spawning, only self mutations?
- [ ] **Focus system**: Complete chain (handle → Focusable → track_focus → on_action)?
- [ ] **Async tasks**: All spawns either detached or stored?
- [ ] **Context borrowing**: Using listener's `cx`, not outer `cx`?
- [ ] **Event handlers**: Properly handling all dispatch phases?
- [ ] **RenderOnce vs Render**: Stateless components use RenderOnce, interactive views use Render?

---

## References

- [GPUI Framework | Zed | DeepWiki](https://deepwiki.com/zed-industries/zed/2.2-ui-framework-(gpui))
- [Ownership and data flow in GPUI | Zed's Blog](https://zed.dev/blog/gpui-ownership)
- [GPUI Contexts Documentation | Zed Repository](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md)
- [Key Dispatch | Zed Repository](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md)
- [GPUI Interactivity | 0xshadow's Blog](https://blog.0xshadow.dev/posts/learning-gpui/gpui-interactivity/)
- [GPUI Agents Guide | gpui-ce](https://github.com/gpui-ce/gpui-ce/blob/main/AGENTS.md)
- [GPUI Docs | Rust API](https://docs.rs/gpui/latest/gpui/)
- [Zed Editor Source](https://github.com/zed-industries/zed/tree/main/crates/gpui)

