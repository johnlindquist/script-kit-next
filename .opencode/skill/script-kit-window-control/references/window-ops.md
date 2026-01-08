# Window Operations (window_ops.rs)

Coalesced window resize and move operations using GPUI's `Window::defer` API.

## Why Coalescing?

During rapid UI updates (filtering, typing, state changes), multiple resize/move requests
can fire quickly. Without coalescing:
- Visual jitter/flicker
- Performance degradation from redundant system calls
- Potential RefCell borrow conflicts during GPUI's render cycle

## How It Works

1. Caller calls `queue_resize()` or `queue_move()` with desired values
2. Value stored in pending slot (overwrites previous)
3. `Window::defer()` callback scheduled (only once per effect cycle)
4. At end of effect cycle, `flush_pending_ops()` executes final values

**Result**: Only ONE window operation per effect cycle, using latest values.

## Public API

### Queue Resize
```rust
/// Queue window resize for end of effect cycle
/// Multiple calls coalesce - only final height used
pub fn queue_resize(target_height: f32, window: &mut Window, cx: &mut gpui::App);
```

### Queue Move
```rust
/// Queue window move for end of effect cycle
/// Multiple calls coalesce - only final bounds used
pub fn queue_move(bounds: Bounds<Pixels>, window: &mut Window, cx: &mut gpui::App);
```

### Utility Functions
```rust
/// Check if any operations are pending
pub fn has_pending_ops() -> bool;

/// Clear pending ops without executing (for hide/close)
pub fn clear_pending_ops();
```

## State Management

```rust
static PENDING_RESIZE: Mutex<Option<f32>>;
static PENDING_BOUNDS: Mutex<Option<Bounds<Pixels>>>;
static FLUSH_SCHEDULED: AtomicBool;
```

## Usage Example

```rust
// WRONG: Direct resize in render callback - can cause RefCell panic
platform::resize_first_window_to_height(height);

// CORRECT: Queue for deferred execution
window_ops::queue_resize(f32::from(height), window, cx);
```

### With Bounds
```rust
use gpui::{point, px, size, Bounds};

let bounds = Bounds {
    origin: point(px(100.0), px(200.0)),
    size: size(px(750.0), px(500.0)),
};
window_ops::queue_move(bounds, window, cx);
```

## Coalescing Behavior

```rust
// These all happen in same effect cycle:
queue_resize(400.0, window, cx);
queue_resize(500.0, window, cx);
queue_resize(600.0, window, cx);
// Only 600.0 is executed
```

## Internal Flow

```
queue_resize(height)
    └─> PENDING_RESIZE = Some(height)
    └─> schedule_flush() (if not already scheduled)
            └─> FLUSH_SCHEDULED = true
            └─> window.defer(|_, _| flush_pending_ops())
                    └─> (at end of effect cycle)
                    └─> FLUSH_SCHEDULED = false
                    └─> window_resize::resize_first_window_to_height()
                    └─> platform::move_first_window_to_bounds()
```
