---
name: gpui-patterns
description: GPUI framework patterns for Script Kit. Use when writing UI code, handling keyboard events, managing state, or working with layouts. Covers layout chains, lists, themes, events, focus, and window management.
---

# GPUI Patterns

Essential patterns for building UI with GPUI in Script Kit.

## Quick Reference (Things That Break Most Often)

- **Layout chain order:** Layout (`flex*`) → Sizing (`w/h`) → Spacing (`px/gap`) → Visual (`bg/border`)
- **Lists:** `uniform_list` (fixed height **52px**) + `UniformListScrollHandle`
- **Theme colors:** use `theme.colors.*` (**never** `rgb(0x...)`)
- **Focus colors:** use `theme.get_colors(is_focused)`; re-render on focus change
- **State updates:** after render-affecting changes, **must** `cx.notify()`
- **Keyboard:** primary pattern is `.on_key_down(handler)` + `crate::ui_foundation::is_key_*` helpers
- **Printable chars:** `printable_char(event.keystroke.key_char.as_deref())`
- **Focus events:** keyboard handlers need the focus trio: `.track_focus(...) + .on_key_down(...) + .child(...)`

## Keyboard Handling (CRITICAL)

Import and use key helpers from `crate::ui_foundation`:

```rust
use crate::ui_foundation::{
  is_key_backspace, is_key_delete, is_key_down, is_key_enter, is_key_escape, is_key_left,
  is_key_right, is_key_space, is_key_tab, is_key_up, printable_char,
};
```

Use a dedicated `on_key_down` handler as the primary keyboard pattern:

```rust
fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut ViewContext<Self>) {
  if is_key_up(event) {
    self.move_up(cx);
    return;
  }
  if is_key_down(event) {
    self.move_down(cx);
    return;
  }
  if is_key_left(event) {
    self.move_left(cx);
    return;
  }
  if is_key_right(event) {
    self.move_right(cx);
    return;
  }
  if is_key_enter(event) {
    self.confirm(cx);
    return;
  }
  if is_key_escape(event) {
    self.cancel(cx);
    return;
  }
  if is_key_tab(event) || is_key_space(event) {
    self.toggle(cx);
    return;
  }
  if is_key_backspace(event) || is_key_delete(event) {
    self.delete(cx);
    return;
  }
  if let Some(ch) = printable_char(event.keystroke.key_char.as_deref()) {
    self.insert_char(ch, cx);
  }
}
```

## Layout System

Chain in order: layout → sizing → spacing → visual → children.

```rust
div().flex().flex_row().items_center().gap_2();
div().flex().flex_col().w_full();
div().flex().items_center().justify_center();
div().flex_1(); // fill remaining space
```

Conditional rendering:

```rust
div().when(is_selected, |d| d.bg(selected)).when_some(desc, |d, s| d.child(s));
```

## List Virtualization

Use `uniform_list` with fixed-height rows (~52px):

```rust
uniform_list("script-list", filtered.len(), cx.processor(|this, range, _w, _cx| {
  this.render_list_items(range)
}))
.h_full()
.track_scroll(&self.list_scroll_handle);
```

Scroll to item:

```rust
self.list_scroll_handle.scroll_to_item(selected_index, ScrollStrategy::Nearest);
```

## Theme System

```rust
let colors = &self.theme.colors;
div().bg(rgb(colors.background.main)).border_color(rgb(colors.ui.border));
```

Focus-aware:

- compute `is_focused = self.focus_handle.is_focused(window)`
- if changed: update state + `cx.notify()`
- use `let colors = self.theme.get_colors(is_focused);`

For closures: extract copyable structs like `colors.list_item_colors()`.

## Focus + Events

```rust
let focus_handle = cx.focus_handle();
focus_handle.focus(window);

div()
  .track_focus(&self.focus_handle)
  .on_key_down(Self::on_key_down)
  .child(self.render_content(cx));
```

Without `.track_focus(&self.focus_handle)`, key events never arrive at `.on_key_down(...)`.

## State Management

After any state mutation affecting rendering: `cx.notify()`

Shared state: `Arc<Mutex<T>>` or channels; for async, use `mpsc` sender → UI receiver.

## Entity Lifecycle + Async Work

Store subscriptions on the view struct (`Vec<Subscription>` is a common pattern), otherwise they are dropped and stop receiving events.

```rust
pub struct PromptView {
  subscriptions: Vec<Subscription>,
  poll_task: Option<Task<()>>,
  load_generation: u64,
}

fn wire_model(&mut self, cx: &mut ViewContext<Self>) {
  let sub = cx.subscribe(&self.model, |this, _model, event, cx| this.on_model_event(event, cx));
  self.subscriptions.push(sub);
}
```

Use `.detach()` for fire-and-forget background work:

```rust
cx.spawn(|_this, _cx| async move {
  send_telemetry().await;
}).detach();
```

For UI-updating async work, `cx.spawn()` gives `this: WeakEntity<_>` and `cx: AsyncApp`. Re-enter UI state with `this.update(cx, |this, cx| { ... }).ok()`:

```rust
fn reload(&mut self, cx: &mut ViewContext<Self>) {
  self.load_generation += 1;
  let generation = self.load_generation;

  self.poll_task = Some(cx.spawn(|this, cx| async move {
    let items = fetch_items().await;
    this.update(cx, |this, cx| {
      if generation != this.load_generation {
        return; // stale async result
      }
      this.items = items;
      cx.notify();
    }).ok();
  }));
}
```

Dropping a stored `Task` cancels it. Store tasks intentionally (`Option<Task<_>>` / `Vec<Task<_>>`) when they must stay alive.

## References

- [Anti-Patterns](references/anti-patterns.md) - Common mistakes that cause bugs
- [Smart Pointers](references/smart-pointers.md) - Arc, Rc, Mutex patterns
- [Window Management](references/window-management.md) - Multi-monitor, floating panels
- [Scroll Performance](references/scroll-performance.md) - Rapid-key coalescing
- [Testing Patterns](references/testing-patterns.md) - GPUI test organization
