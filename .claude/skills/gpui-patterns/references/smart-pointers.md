# Rust Smart Pointers

Essential patterns for GPUI apps. Most bugs come from misusing these.

## Quick Decision Tree

```
Need multiple owners?
  NO  → Use regular ownership/references
  YES → Will it cross threads?
          NO  → Rc<T>
          YES → Arc<T>

Need interior mutability?
  Single-threaded → RefCell<T> (panics if misused!)
  Multi-threaded  → Mutex<T> or RwLock<T>

Need lazy initialization?
  → OnceLock<T> or LazyLock<T>
```

## Arc<Mutex<T>> - Shared Mutable State Across Threads

```rust
let state = Arc::new(Mutex::new(AppState::new()));
let state_clone = Arc::clone(&state);
thread::spawn(move || {
    let mut guard = state_clone.lock().unwrap();
    guard.update();
});
```

## OnceLock<Mutex<T>> - Global Singletons

```rust
static WINDOW_MANAGER: OnceLock<Mutex<WindowManager>> = OnceLock::new();

fn get_manager() -> &'static Mutex<WindowManager> {
    WINDOW_MANAGER.get_or_init(|| Mutex::new(WindowManager::new()))
}
```

## Arc vs Rc

- `Rc<T>` - Single-threaded only (UI callbacks that never leave main thread)
- `Arc<T>` - Thread-safe (callbacks that might cross async/thread boundaries)

```rust
// UI callbacks (single-threaded) - Rc is fine
on_click: Option<Rc<OnClickCallback>>,

// Hotkey handlers (cross threads) - must use Arc
pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;
```

## Common Anti-Patterns

**Deadlock from nested locks:**
```rust
// DEADLOCK - locking same mutex twice
let guard1 = data.lock().unwrap();
let guard2 = data.lock().unwrap();  // Blocks forever!

// FIX - drop first guard
{ let guard = data.lock().unwrap(); /* use */ }
{ let guard = data.lock().unwrap(); /* OK */ }
```

**RefCell borrow panics:**
```rust
// PANIC - overlapping borrows
let r = data.borrow();
data.borrow_mut().push(4);  // PANIC!

// FIX - use try_borrow_mut or ensure no overlap
if let Ok(mut r) = data.try_borrow_mut() { r.push(4); }
```

**Holding locks across await:**
```rust
// BAD - lock held during await
let mut guard = data.lock().unwrap();
do_async_work().await;  // Lock held!

// GOOD - copy, unlock, await
let value = { data.lock().unwrap().clone() };
do_async_work().await;
```

## Quick Reference Table

| Type | Thread-safe | Multiple owners | Mutable | Use case |
|------|-------------|-----------------|---------|----------|
| `Box<T>` | Yes* | No | Yes | Heap allocation, trait objects |
| `Rc<T>` | No | Yes | No | Single-threaded sharing |
| `Arc<T>` | Yes | Yes | No | Multi-threaded sharing |
| `RefCell<T>` | No | N/A | Yes | Interior mut (runtime check) |
| `Mutex<T>` | Yes | N/A | Yes | Thread-safe interior mut |
| `OnceLock<T>` | Yes | N/A | No | One-time initialization |
