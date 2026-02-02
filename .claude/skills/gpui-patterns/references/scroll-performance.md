# Scroll Performance

## Rapid-Key Coalescing

Coalesce rapid key events (20ms window) to avoid freezes:

```rust
enum ScrollDirection { Up, Down }

fn process_arrow(&mut self, dir: ScrollDirection, cx: &mut Context<Self>) {
  let now = Instant::now();
  if now.duration_since(self.last_scroll_time) < Duration::from_millis(20)
     && self.pending_dir == Some(dir) {
    self.pending_delta += 1;
    return;
  }
  self.flush_pending(cx);
  self.pending_dir = Some(dir);
  self.pending_delta = 1;
  self.last_scroll_time = now;
}
```

## Performance Thresholds

- P95 key latency <50ms
- Single key <16.67ms
- Scroll op <8ms

## Uniform List Pattern

```rust
uniform_list("script-list", filtered.len(), cx.processor(|this, range, _w, _cx| {
  this.render_list_items(range)
}))
.h_full()
.track_scroll(&self.list_scroll_handle);
```

Fixed-height rows (~52px) required for uniform_list virtualization.

Scroll to item:
```rust
self.list_scroll_handle.scroll_to_item(selected_index, ScrollStrategy::Nearest);
```
