# GPUI Event Dispatch Architecture

A practical guide to how GPUI dispatches keyboard events, actions, and how propagation control works. Written because the dual-dispatch behavior is undocumented and requires source-reading to understand.

## The Core Insight: Dual Dispatch

When you press a key like Enter, GPUI runs **two independent dispatch systems** in sequence:

1. **Action dispatch** — keystroke matched against keybindings → action handlers fire
2. **Raw key event dispatch** — `KeyDownEvent` handlers fire on elements

These are separate event systems. `cx.stop_propagation()` in one does **not** affect the other. An action handler stopping propagation prevents other action handlers from firing, but has no effect on `on_key_down` listeners (they simply won't fire because action dispatch consumed the event first — see dispatch order below).

## Full Dispatch Order

When a `KeyDownEvent` arrives from the OS, `dispatch_key_event()` (`vendor/gpui/src/window.rs`) processes it in this order:

```
1. Keystroke interceptors (can consume the event immediately)
2. Multi-stroke chord resolution (e.g., cmd-k cmd-l — may buffer)
3. Action dispatch (keybinding → action handlers)
4. Raw key event dispatch (on_key_down / on_key_up listeners)
5. Text input fallback (if keystroke has a key_char and an input handler exists)
```

**Critical**: Steps 4-5 only run if step 3 did NOT consume the event. If an action handler fires and doesn't call `cx.propagate()`, the raw `KeyDownEvent` listeners **never fire**.

## Propagation Control

Two methods on `cx` (`vendor/gpui/src/app.rs`):

```rust
cx.stop_propagation()  // prevents further handlers in this dispatch cycle
cx.propagate()         // re-enables propagation (cancels a previous stop)
```

### Default Behavior Differs By System

| System | Default | Meaning |
|--------|---------|---------|
| **Raw key events** | `propagate = true` | Events bubble unless a handler explicitly calls `cx.stop_propagation()` |
| **Action handlers** | `propagate = false` (bubble phase) | Actions are consumed by the first bubble-phase handler unless it calls `cx.propagate()` |

This asymmetry is the source of most confusion. Action handlers must opt-in to forwarding; key handlers must opt-in to consuming.

## Two-Phase Model: Capture + Bubble

Both systems use capture-then-bubble, but they're independent cycles.

### For raw key events (`dispatch_key_down_up_event`):

```
Capture: root → parent → focused (top-down)
Bubble:  focused → parent → root (bottom-up)
```

Handlers registered with `on_key_down()` fire during **bubble** phase. Use `capture_key_down()` for capture phase.

### For actions (`dispatch_action_on_node`):

```
1. Global capture listeners
2. Window capture: root → parent → focused
3. Window bubble:  focused → parent → root  (propagate_event = false before each listener)
4. Global bubble listeners                   (propagate_event = false before each listener)
```

## Registering Handlers on Elements

### Raw key events

```rust
div()
    .on_key_down(|event: &KeyDownEvent, window, cx| {
        // Fires during bubble phase
        // Must call cx.stop_propagation() to consume
        let key = event.keystroke.key.as_str();
        if key == "enter" || key == "Enter" {
            // handle it
            cx.stop_propagation();
        } else {
            cx.propagate(); // let unhandled keys bubble
        }
    })
    .capture_key_down(|event: &KeyDownEvent, window, cx| {
        // Fires during capture phase (before children)
    })
```

### Actions

```rust
div()
    .on_action(|action: &PressEnter, window, cx| {
        // Fires during bubble phase
        // Propagation already stopped by default
        // Call cx.propagate() if you want parent handlers to also fire
    })
```

## Action Dispatch via `window.dispatch_action()`

```rust
window.dispatch_action(Box::new(MyAction), cx);
```

Actions dispatched this way are **deferred** — they're queued via `cx.defer()` and dispatched at effect-flush time, not immediately. The action dispatches from the currently focused node.

Use `window.dispatch_action()` (not `cx.dispatch_action()`) from key handlers within a window context.

## Key Bindings and Key Contexts

Elements declare contexts that scope which keybindings are active:

```rust
div().key_context("my-prompt")
```

Keybindings in the keymap specify context predicates. A binding only matches if its context predicate matches somewhere in the context stack from root to the focused element.

Binding priority: deeper in tree > shallower; later registration > earlier.

## Multi-Stroke Chords

GPUI supports multi-key sequences (e.g., `cmd-k cmd-l`):

1. First keystroke partially matches → enters pending state
2. 1-second timeout starts
3. Next keystroke completes the chord → action fires
4. Timeout expires → pending keystrokes are replayed through the full dispatch pipeline

## Common Pitfalls

### 1. "My on_key_down handler doesn't fire for Enter"
An action binding for Enter (e.g., `PressEnter`) is consuming the event before raw key handlers run. Either:
- Use `.on_action::<PressEnter>(...)` instead
- Or remove/rebind the conflicting keybinding

### 2. "cx.stop_propagation() in my action handler doesn't prevent on_key_down"
These are separate systems. If the action consumed the event, `on_key_down` won't fire at all. If `on_key_down` IS firing, it means no action matched the keystroke.

### 3. "My action handler fires but parent handlers also fire"
Actions stop propagation by default in bubble phase. If parent handlers are firing, they may be in the **capture** phase (which runs first). Check if a parent is using action listeners in capture phase.

### 4. "Unhandled keys don't reach parent components"
You must call `cx.propagate()` in your fallthrough arm:
```rust
.on_key_down(|event, window, cx| {
    match event.keystroke.key.as_str() {
        "enter" | "Enter" => { /* handle */ cx.stop_propagation(); }
        _ => cx.propagate(),
    }
})
```

### 5. "prefer_character_input skips my keybinding"
When `KeyDownEvent.prefer_character_input` is true and an input handler is active, keybinding resolution is skipped entirely. The keystroke goes directly to raw key listeners and text input. This supports international keyboard input (e.g., AltGr characters).

## Quick Reference

| Want to... | Use |
|-----------|-----|
| Handle a semantic action (Enter, Escape, Tab) | `.on_action::<ActionType>(...)` |
| Handle raw keystrokes | `.on_key_down(...)` |
| Consume an event in key handler | `cx.stop_propagation()` |
| Forward unhandled keys | `cx.propagate()` |
| Dispatch an action programmatically | `window.dispatch_action(Box::new(action), cx)` |
| Scope keybindings to a component | `.key_context("name")` on the div |
| Intercept keys before binding resolution | Keystroke interceptors |

## Mouse Event Dispatch

### Mouse Event Types

| Event | Key Fields | Notes |
|-------|-----------|-------|
| `MouseDownEvent` | `button`, `position`, `modifiers`, `click_count`, `first_mouse` | `is_focusing()` true only for Left button |
| `MouseUpEvent` | `button`, `position`, `modifiers`, `click_count` | |
| `MouseMoveEvent` | `position`, `pressed_button`, `modifiers` | `dragging()` true when Left held |
| `ScrollWheelEvent` | `position`, `delta`, `modifiers`, `touch_phase` | `delta` is `Pixels` or `Lines` |
| `MousePressureEvent` | `pressure` (0.0-1.0), `stage`, `position` | macOS Force Touch |
| `MouseExitEvent` | `position`, `pressed_button` | Mouse left window |
| `ClickEvent` | `Mouse(down, up)` or `Keyboard` | Synthesized on press+release |
| `FileDropEvent` | `Entered`, `Pending`, `Submit`, `Exited` | Drag & drop files |

### Hit Testing & Z-Order

Mouse events use **hit testing** to determine which elements receive events. The hit test walks hitboxes in **reverse paint order** (front to back):

1. Check if mouse position is within bounds AND content mask
2. Add to hit list
3. Stop at first `BlockMouse` hitbox (it occludes everything behind)

**HitboxBehavior**:
- `Normal` — no special behavior
- `BlockMouse` — occludes all hitboxes behind; blocks all mouse events including scroll
- `BlockMouseExceptScroll` — blocks normal interaction but allows scroll events through

**Hover detection**: `hitbox.is_hovered(window)` returns true only for elements in the top slice (above any blocking hitbox).

### Mouse Dispatch Flow

`dispatch_mouse_event()` in `window.rs`:

1. Hit test at current mouse position
2. **Capture phase**: Iterate mouse listeners front-to-back
3. **Bubble phase**: Iterate mouse listeners back-to-front (frontmost element first)
4. Handle active drag state (move/drop)

**Key difference from keyboard**: Mouse events use z-order (paint order), not tree hierarchy. Front-to-back for bubble means the **topmost visible element** gets the event first.

### Mouse Handlers on Elements

```rust
div()
    // Standard handlers (bubble phase, require hitbox hover)
    .on_mouse_down(MouseButton::Left, |event, window, cx| { ... })
    .on_mouse_up(MouseButton::Left, |event, window, cx| { ... })
    .on_mouse_move(|event, window, cx| { ... })
    .on_scroll_wheel(|event, window, cx| { ... })
    .on_click(|event: &ClickEvent, window, cx| { ... })

    // Click-outside detection (capture phase, fires when NOT hovered)
    .on_mouse_down_out(|event, window, cx| { ... })
    .on_mouse_up_out(|event, window, cx| { ... })

    // Capture phase variants
    .capture_any_mouse_down(|event, window, cx| { ... })
    .capture_any_mouse_up(|event, window, cx| { ... })
```

### Drag & Drop

```rust
// Source element
div()
    .on_drag(MyDragValue { ... }, |value, window, cx| {
        // Return a view to render while dragging
        div().child("Dragging...")
    })

// Target element
div()
    .on_drop(|value: &MyDragValue, window, cx| {
        // Handle the drop
    })
```

Drag lifecycle:
1. `MouseDown` on element with `on_drag` → creates `AnyDrag` in `cx.active_drag`
2. `MouseMove` → window refreshes, drag view follows cursor
3. `MouseUp` → `on_drop` handlers fire on hovered elements, drag cleared

Utilities: `cx.has_active_drag()`, `cx.stop_active_drag(window)`, `cx.set_active_drag_cursor_style(style, window)`

## Focus Management

### FocusHandle

The primary API for managing focus. Create one in your component and associate it with an element:

```rust
struct MyComponent {
    focus_handle: FocusHandle,
}

impl MyComponent {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

// In render:
div()
    .track_focus(&self.focus_handle)  // Associates handle with element
    .child("I'm focusable")
```

### FocusHandle Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `focus(window, cx)` | `()` | Request focus for this element |
| `is_focused(window)` | `bool` | Is this exact element focused? |
| `contains_focused(window, cx)` | `bool` | Does this element contain the focused element? |
| `within_focused(window, cx)` | `bool` | Is this element within the focused element? |
| `downgrade()` | `WeakFocusHandle` | Non-owning reference (for closures) |
| `tab_index(isize)` | `Self` | Set tab order |
| `tab_stop(bool)` | `Self` | Include/exclude from tab cycle |
| `dispatch_action(action, window, cx)` | `()` | Dispatch action as if this element is focused |

### Window Focus Methods

```rust
window.focused(cx)        // -> Option<FocusHandle>  (currently focused)
window.focus(&handle, cx) // Move focus to handle
window.blur()             // Remove focus from all elements
window.disable_focus()    // Blur and prevent future focus changes
window.focus_next(cx)     // Focus next tab stop
window.focus_prev(cx)     // Focus previous tab stop
```

### Focus-Based Styling

```rust
div()
    .track_focus(&self.focus_handle)
    .focus(|style| style.border_color(gpui::blue()))           // When this element is focused
    .in_focus(|style| style.border_color(gpui::blue()))        // When a child is focused
    .focus_visible(|style| style.outline_color(gpui::blue()))  // Keyboard focus only (like CSS :focus-visible)
```

`focus_visible` uses `last_input_modality` tracking — styles only apply when focus was achieved via keyboard, not mouse click.

### Focus and Key Dispatch

Focus determines the dispatch path for keyboard events:
- The **dispatch tree** builds a path from root to the focused element
- **Capture phase**: root → focused (top-down)
- **Bubble phase**: focused → root (bottom-up)
- Only elements on this path receive keyboard events
- Elements not in the focus path don't see key events at all

### Focus Events

Focus changes emit `WindowFocusEvent` with:
- `previous_focus_path` — ancestor chain of previously focused element
- `current_focus_path` — ancestor chain of newly focused element
- `is_focus_in(id)` / `is_focus_out(id)` — detect specific element focus transitions

Subscribe with `cx.on_focus_out()` for cleanup when focus leaves.

### Auto-Focus on Click

`MouseDownEvent.first_mouse` indicates a "focusing click" — the first click that brings a window/element into focus. Use `ClickEvent::first_focus()` to check this and potentially skip the click's normal action (matching macOS convention where the first click focuses but doesn't activate).

## Source Files

| File | Contains |
|------|----------|
| `vendor/gpui/src/window.rs` | Event dispatch, focus management, hit testing |
| `vendor/gpui/src/key_dispatch.rs` | `DispatchTree::dispatch_key`, chord/binding resolution |
| `vendor/gpui/src/app.rs` | `stop_propagation()`, `propagate()`, drag state |
| `vendor/gpui/src/input.rs` | Event types (`KeyDownEvent`, `MouseDownEvent`, `FocusHandle`, etc.) |
| `vendor/gpui/src/elements/div.rs` | Element-level event/focus registration |
