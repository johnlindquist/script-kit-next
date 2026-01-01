# Search Keystroke Flow Analysis

> **Document Type:** Performance Analysis  
> **Created:** 2024-12-31  
> **Epic:** Search Debounce Optimization (cell--9bnr5-mjv12wqb2qh)  
> **Task:** cell--9bnr5-mjv12wqhu9f

## Executive Summary

This document traces the complete keystroke flow from key event to UI update in the Script Kit search system, documenting all sources of delay. The analysis reveals a **two-stage architecture** that separates immediate visual feedback from expensive search computation.

### Key Findings

| Delay Source | Duration | Purpose | Impact |
|--------------|----------|---------|--------|
| FilterCoalescer timer | 16ms | Batch rapid keystrokes for search | Delays search results |
| NavCoalescer window | 20ms | Batch arrow key navigation | Delays list scrolling |
| Window resize defer | 16ms | Avoid RefCell borrow conflicts | Delays window height changes |
| Scrollbar fade-out | 1000ms | Hide scrollbar after inactivity | Visual only, no impact |
| App loading poll | 50ms | Poll for background app scan | Startup only |

**Critical Insight:** The 16ms debounce exists in **TWO locations** - both are active but effectively redundant:
1. `app_impl.rs:746` - in `update_filter()` (legacy code path)
2. `render_script_list.rs:595` - in `on_key_down` handler (current code path)

Both use the same `FilterCoalescer` instance, so whichever runs first starts the timer.

---

## Complete Keystroke Flow

### Phase 1: Key Event Capture (Synchronous, ~0ms)

```
User presses key
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  render_script_list.rs:666 - on_key_down(handle_key)            │
│  GPUI event handler receives KeyDownEvent                        │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  render_script_list.rs:504 - match key_str.as_str()             │
│  Route key to appropriate handler:                               │
│    - "up"/"arrowup" → NavCoalescer for navigation               │
│    - "down"/"arrowdown" → NavCoalescer for navigation           │
│    - "enter" → execute_selected()                                │
│    - "escape" → clear filter or close window                     │
│    - "space" → check alias match, else insert character          │
│    - _ (default) → TextInputState.handle_key()                   │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 2: Text Input Handling (Synchronous, ~0ms)

For printable characters (the `_` default case):

```
render_script_list.rs:569-612
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. Capture old_text = filter_input.text()                       │
│  2. Call filter_input.handle_key(key_str, key_char, modifiers)  │
│  3. Check if text actually changed                               │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼ (if text changed)
┌─────────────────────────────────────────────────────────────────┐
│  render_script_list.rs:587-590                                   │
│  IMMEDIATE UI STATE UPDATE:                                      │
│    - selected_index = 0                                          │
│    - last_scrolled_index = None                                  │
│    - main_list_state.scroll_to_reveal_item(0)                   │
│    - last_scrolled_index = Some(0)                              │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  render_script_list.rs:611                                       │
│  cx.notify() - TRIGGERS IMMEDIATE REPAINT                       │
│  ⚡ Input field updates INSTANTLY (responsive typing)            │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 3: Search Coalescing (Asynchronous, +16ms delay)

```
render_script_list.rs:593-609
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  filter_coalescer.queue(new_text)                                │
│                                                                  │
│  FilterCoalescer Logic (filter_coalescer.rs:12-20):             │
│    if pending {                                                  │
│      // Timer already running - just update latest value        │
│      return false                                                │
│    } else {                                                      │
│      // No timer - start one                                     │
│      pending = true                                              │
│      return true                                                 │
│    }                                                             │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼ (if queue() returned true - first keystroke in batch)
┌─────────────────────────────────────────────────────────────────┐
│  cx.spawn(async move |this, cx| {                               │
│      ┌──────────────────────────────────────────────────────┐   │
│      │  Timer::after(Duration::from_millis(16)).await       │   │
│      │  ⏱️ 16ms DELAY - ONE FRAME AT 60FPS                   │   │
│      └──────────────────────────────────────────────────────┘   │
│                                                                  │
│      cx.update(|cx| {                                            │
│          this.update(cx, |app, cx| {                            │
│              if let Some(latest) = app.filter_coalescer.take_latest() { │
│                  if app.computed_filter_text != latest {        │
│                      app.computed_filter_text = latest;         │
│                      app.update_window_size();                  │
│                      cx.notify(); // ← Triggers search UI update│
│                  }                                               │
│              }                                                   │
│          })                                                      │
│      });                                                         │
│  }).detach();                                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 4: Cache Recomputation (On Next Render, variable duration)

```
After cx.notify() triggers a new render cycle
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  render_script_list.rs:79                                        │
│  let (grouped_items, flat_results) = self.get_grouped_results_cached(); │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  app_impl.rs:441-481 - get_grouped_results_cached()             │
│                                                                  │
│  Cache Check:                                                    │
│    if computed_filter_text == grouped_cache_key {               │
│      return cached results (HIT, ~0ms)                          │
│    }                                                             │
│                                                                  │
│  Cache Miss:                                                     │
│    1. get_filtered_results_cached() → fuzzy_search_unified_all()│
│       (Variable: depends on script count and filter complexity) │
│    2. scripts::get_grouped_results() → group by section         │
│    3. Store in Arc for cheap clone to render closures           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Delay Source Details

### 1. FilterCoalescer (16ms)

**Location:** `src/filter_coalescer.rs`

**Purpose:** Batch multiple rapid keystrokes into a single expensive search operation.

**Mechanism:**
```rust
pub fn queue(&mut self, value: impl Into<String>) -> bool {
    self.latest = Some(value.into());  // Always update latest value
    if self.pending {
        false  // Timer already running, just update value
    } else {
        self.pending = true;
        true   // Caller should start timer
    }
}
```

**Behavior:**
- First keystroke: Starts 16ms timer
- Rapid subsequent keystrokes: Update `latest` value but don't restart timer
- After 16ms: Process only the final `latest` value

**Trade-off:** 16ms delay ensures we don't compute fuzzy search on every single character, which would be wasteful during fast typing.

### 2. NavCoalescer (20ms)

**Location:** `src/navigation.rs`

**Purpose:** Batch rapid arrow key presses during keyboard repeat.

**Mechanism:**
```rust
pub const WINDOW: std::time::Duration = std::time::Duration::from_millis(20);
```

**Behavior:**
- First arrow key: Apply immediately (move by 1)
- Rapid subsequent same-direction keys: Coalesce into batch
- Direction change: Flush old delta, start new direction
- Background task: Flushes pending delta every 20ms

**Note:** This is separate from search - only affects list navigation.

### 3. Window Resize Defer (16ms)

**Location:** `src/window_resize.rs:127-150`

**Purpose:** Avoid RefCell borrow conflicts during GPUI render cycle.

**Mechanism:**
```rust
pub fn defer_resize_to_view<T: Render>(
    view_type: ViewType,
    item_count: usize,
    cx: &mut Context<T>,
) {
    cx.spawn(async move |_this, _cx| {
        Timer::after(Duration::from_millis(16)).await;  // 16ms delay
        resize_first_window_to_height(target_height);
    }).detach();
}
```

**Impact:** Window height changes (when list shrinks/grows) are delayed by 16ms. This is only visible when filtered results change significantly.

### 4. Scrollbar Fade-out (1000ms)

**Location:** `src/app_navigation.rs:124-146`

**Purpose:** Hide scrollbar after keyboard navigation completes.

**Mechanism:**
```rust
fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
    self.is_scrolling = true;
    cx.spawn(async move |this, cx| {
        Timer::after(Duration::from_millis(1000)).await;
        // Hide scrollbar if no new activity
    }).detach();
}
```

**Impact:** Visual only - does not affect search responsiveness.

### 5. Background App Loading (50ms poll)

**Location:** `src/app_impl.rs:77-100`

**Purpose:** Load application list in background without blocking startup.

**Mechanism:**
```rust
cx.spawn(async move |this, cx| {
    loop {
        Timer::after(Duration::from_millis(50)).await;
        match rx.try_recv() {
            Ok((apps, elapsed)) => { /* update app list */ }
            Err(TryRecvError::Empty) => continue,
            Err(_) => break,
        }
    }
}).detach();
```

**Impact:** Only affects startup. No impact on keystroke handling.

---

## Two-Stage Architecture Analysis

The search system uses a two-stage architecture to separate **visual responsiveness** from **computational work**:

### Stage 1: Immediate Visual Feedback (0ms)
```
filter_input.handle_key()  →  filter_input updated  →  cx.notify()
```
- User sees their keystroke reflected immediately
- Input field updates in same frame
- No perceptible delay for typing

### Stage 2: Deferred Search Computation (+16ms)
```
filter_coalescer.queue()  →  16ms timer  →  computed_filter_text updated  →  cache recompute  →  cx.notify()
```
- Search results update after 16ms coalescing window
- Multiple keystrokes within 16ms are batched
- Reduces CPU usage during fast typing

### Dual Code Path Issue

**WARNING:** There are two code paths that both use the FilterCoalescer:

1. **Legacy path in `app_impl.rs:743-762`** (`update_filter()` method)
   - Used for: Backspace, clear, space (alias check), direct character insertion
   - Triggers coalescer with 16ms delay

2. **Current path in `render_script_list.rs:593-609`** (`on_key_down` handler)
   - Used for: All character keys that go through TextInputState
   - Also triggers coalescer with 16ms delay

Both paths use the same `FilterCoalescer` instance on `ScriptListApp`, so there's no double-delay. However, this duplication is a maintenance concern.

---

## Timing Diagram

```
T=0ms    ┃ User types 'h'
         ┃ ┌──────────────────────────────────────────────────────────┐
         ┃ │ KeyDownEvent → filter_input.handle_key('h')              │
         ┃ │ filter_input: "" → "h"                                    │
         ┃ │ cx.notify() → Input field displays "h" IMMEDIATELY        │
         ┃ └──────────────────────────────────────────────────────────┘
         ┃ filter_coalescer.queue("h") → returns true → start timer
         ┃
T=5ms    ┃ User types 'e'
         ┃ ┌──────────────────────────────────────────────────────────┐
         ┃ │ filter_input: "h" → "he"                                  │
         ┃ │ cx.notify() → Input field displays "he" IMMEDIATELY       │
         ┃ └──────────────────────────────────────────────────────────┘
         ┃ filter_coalescer.queue("he") → returns false → timer pending
         ┃
T=10ms   ┃ User types 'l'
         ┃ ┌──────────────────────────────────────────────────────────┐
         ┃ │ filter_input: "he" → "hel"                                │
         ┃ │ cx.notify() → Input field displays "hel" IMMEDIATELY      │
         ┃ └──────────────────────────────────────────────────────────┘
         ┃ filter_coalescer.queue("hel") → returns false → timer pending
         ┃
T=16ms   ┃ Timer fires
         ┃ ┌──────────────────────────────────────────────────────────┐
         ┃ │ filter_coalescer.take_latest() → Some("hel")             │
         ┃ │ computed_filter_text = "hel"                              │
         ┃ │ update_window_size()                                      │
         ┃ │ cx.notify() → TRIGGER CACHE RECOMPUTE                     │
         ┃ └──────────────────────────────────────────────────────────┘
         ┃
T=16+Xms ┃ Next render frame
         ┃ ┌──────────────────────────────────────────────────────────┐
         ┃ │ get_grouped_results_cached() → cache MISS                │
         ┃ │ fuzzy_search_unified_all("hel", ...) → ~2-5ms typical    │
         ┃ │ get_grouped_results() → group by section                 │
         ┃ │ Store in cache                                            │
         ┃ │ List displays filtered results for "hel"                  │
         ┃ └──────────────────────────────────────────────────────────┘
```

---

## Performance Characteristics

### Latency Breakdown (Typical)

| Phase | Duration | Notes |
|-------|----------|-------|
| Key event to input update | <1ms | Synchronous, single frame |
| Coalescing delay | 16ms | Fixed delay for batching |
| Fuzzy search | 2-5ms | Depends on script count (100-500 scripts) |
| Result grouping | <1ms | Simple iteration |
| **Total: First keystroke** | **~18-22ms** | |
| **Total: Batched keystrokes** | **<1ms input + 18-22ms results** | Input updates instantly |

### Comparison to Competitors

| App | Perceived Latency | Architecture |
|-----|-------------------|--------------|
| Raycast | ~0-5ms | Native search, no batching |
| Alfred | ~0-5ms | Native search, no batching |
| Script Kit (current) | ~16-22ms | Batched search |
| Script Kit (optimized) | ~0-5ms? | Could reduce to 0ms debounce |

---

## Recommendations

### Option 1: Reduce Debounce to 0ms (Aggressive)

**Change:** Remove the 16ms `Timer::after` completely.

**Impact:**
- Search results update on every keystroke
- No batching = more CPU usage during fast typing
- May cause UI jank if fuzzy search takes >16ms

**Risk:** Medium - need to profile fuzzy search performance under load.

### Option 2: Reduce Debounce to 8ms (Conservative)

**Change:** Reduce timer from 16ms to 8ms.

**Impact:**
- Half the latency
- Still provides some batching benefit
- Lower CPU usage than 0ms

**Risk:** Low.

### Option 3: Adaptive Debounce (Advanced)

**Change:** Start with 0ms, increase to 16ms if CPU usage high.

**Implementation:**
```rust
let debounce_ms = if self.last_search_time.as_millis() < 5 {
    0  // Fast search, no debounce needed
} else {
    16  // Slow search, debounce to prevent jank
};
```

**Risk:** Medium - more complex implementation.

### Option 4: Remove Duplicate Code Path

**Change:** Consolidate `update_filter()` and on_key_down handler to use single code path.

**Benefit:** Cleaner code, easier to modify debounce behavior.

---

## Code Locations Reference

| File | Line(s) | Component |
|------|---------|-----------|
| `src/filter_coalescer.rs` | 1-82 | FilterCoalescer struct |
| `src/app_impl.rs` | 705-762 | `update_filter()` method |
| `src/app_impl.rs` | 743-762 | Coalescer timer spawn (legacy path) |
| `src/render_script_list.rs` | 504-615 | Key event handler |
| `src/render_script_list.rs` | 593-609 | Coalescer timer spawn (current path) |
| `src/navigation.rs` | 1-169 | NavCoalescer for arrow keys |
| `src/app_navigation.rs` | 174-213 | Nav flush task |
| `src/window_resize.rs` | 127-150 | Deferred resize |

---

## Appendix: FilterCoalescer State Machine

```
         ┌───────────────────────────────────────────────────────────┐
         │                      IDLE                                  │
         │               pending=false, latest=None                   │
         └───────────────────────────────────────────────────────────┘
                                    │
                                    │ queue("a") → returns true
                                    │ Caller starts 16ms timer
                                    ▼
         ┌───────────────────────────────────────────────────────────┐
         │                    PENDING                                 │
         │               pending=true, latest=Some("a")               │
         │                                                            │
         │  queue("ab") → latest=Some("ab"), returns false            │
         │  queue("abc") → latest=Some("abc"), returns false          │
         │  (Timer is NOT restarted)                                  │
         └───────────────────────────────────────────────────────────┘
                                    │
                                    │ Timer fires → take_latest()
                                    │ Returns Some("abc")
                                    ▼
         ┌───────────────────────────────────────────────────────────┐
         │                      IDLE                                  │
         │               pending=false, latest=None                   │
         │               (Ready for next batch)                       │
         └───────────────────────────────────────────────────────────┘
```

---

## Appendix: cx.notify() Call Sites Related to Filtering

| Location | Trigger | Purpose |
|----------|---------|---------|
| `render_script_list.rs:611` | Key handled | Immediate input update |
| `render_script_list.rs:602` | Timer fire | Search results update |
| `app_impl.rs:739` | update_filter() | Immediate input update |
| `app_impl.rs:755` | Timer fire | Search results update |
| `app_navigation.rs:46` | move_selection_up | Selection highlight |
| `app_navigation.rs:92` | move_selection_down | Selection highlight |
| `app_navigation.rs:145` | scroll activity | Scrollbar visibility |
