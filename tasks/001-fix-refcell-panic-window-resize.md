# Task: Fix RefCell Panic Risk in Window Resize Calls

**Priority:** CRITICAL  
**Estimated Effort:** 30 minutes  
**Skill Reference:** `script-kit-window-control`, AGENTS.md section 28.1

---

## Problem Description

Direct calls to `resize_first_window_to_height()` during the render cycle can cause RefCell borrow conflicts and panic. Window resize/move callbacks can trigger while a RefCell is already borrowed, leading to runtime panics.

The GPUI framework uses RefCell internally for window state management. When you call platform APIs that modify window bounds during a render callback, you risk:
1. Double-borrow panics
2. UI jitter from rapid resize operations
3. Inconsistent window state

---

## Affected Files

| File | Line | Method |
|------|------|--------|
| `src/app_impl.rs` | 1840 | `update_window_size()` |
| `src/app_impl.rs` | 3749 | `reset_to_script_list()` |

---

## Current Problematic Code

### Location 1: `src/app_impl.rs:1840`

```rust
// In update_window_size() method
let target_height = height_for_view(view_type, item_count);
resize_first_window_to_height(target_height);  // DIRECT CALL - DANGEROUS
```

### Location 2: `src/app_impl.rs:3749`

```rust
// In reset_to_script_list() method
resize_first_window_to_height(height_for_view(ViewType::ScriptList, count));  // DIRECT CALL - DANGEROUS
```

---

## Solution

Use `Window::defer()` to schedule resize operations at the end of the current effect cycle. The codebase already has a proper coalescing solution in `src/window_ops.rs`.

### Step 1: Understand the Existing Solution

Read `src/window_ops.rs` to understand the coalescing pattern:

```rust
// src/window_ops.rs provides:
pub fn queue_resize(height: f32, window: &mut Window, cx: &mut gpui::App)
pub fn queue_move(bounds: Bounds<Pixels>, window: &mut Window, cx: &mut gpui::App)
```

These functions:
1. Store the pending operation in a static Mutex
2. Use an AtomicBool to prevent multiple flush schedules
3. Call `window.defer()` to schedule execution at end of effect cycle
4. Coalesce multiple rapid calls into one operation

### Step 2: Update `update_window_size()`

In `src/app_impl.rs`, find the `update_window_size()` method around line 1840.

**Before:**
```rust
fn update_window_size(&mut self, view_type: ViewType, item_count: usize) {
    let target_height = height_for_view(view_type, item_count);
    resize_first_window_to_height(target_height);
}
```

**After:**
```rust
fn update_window_size(&mut self, view_type: ViewType, item_count: usize, window: &mut Window, cx: &mut gpui::App) {
    let target_height = height_for_view(view_type, item_count);
    crate::window_ops::queue_resize(target_height, window, cx);
}
```

Note: You'll need to update the method signature to accept `window` and `cx` parameters, and update all call sites.

### Step 3: Update `reset_to_script_list()`

In `src/app_impl.rs`, find `reset_to_script_list()` around line 3749.

**Before:**
```rust
fn reset_to_script_list(&mut self, /* ... */) {
    // ... other code ...
    resize_first_window_to_height(height_for_view(ViewType::ScriptList, count));
}
```

**After:**
```rust
fn reset_to_script_list(&mut self, window: &mut Window, cx: &mut gpui::App, /* ... */) {
    // ... other code ...
    let height = height_for_view(ViewType::ScriptList, count);
    crate::window_ops::queue_resize(height, window, cx);
}
```

### Step 4: Update Call Sites

Search for all calls to `update_window_size()` and `reset_to_script_list()` and ensure they pass `window` and `cx`:

```bash
grep -rn "update_window_size\|reset_to_script_list" src/
```

Update each call site to pass the required parameters.

### Step 5: Alternative - If Window/Context Not Available

If a call site doesn't have access to `Window` and `App` context (e.g., in an async spawn), use `resize_to_view_sync()` but document why:

```rust
// SAFE: Called from async spawn outside render cycle
// Direct resize is safe here because we're not in a GPUI callback
crate::window_resize::resize_to_view_sync(ViewType::ScriptList, count);
```

---

## Verification Steps

1. **Build Check:**
   ```bash
   cargo check
   ```

2. **Lint Check:**
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

3. **Test:**
   ```bash
   cargo test
   ```

4. **Manual Testing:**
   ```bash
   cargo build
   echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/hello-world.ts"}' | \
     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   ```

5. **Stress Test Resizing:**
   - Open Script Kit
   - Rapidly switch between views (script list, editor, terminal)
   - Filter scripts rapidly (causes resize)
   - Ensure no panics in logs

---

## Success Criteria

- [ ] No direct calls to `resize_first_window_to_height()` in `update_window_size()`
- [ ] No direct calls to `resize_first_window_to_height()` in `reset_to_script_list()`
- [ ] All call sites updated with proper window/cx parameters
- [ ] `cargo check && cargo clippy && cargo test` passes
- [ ] No panics during rapid view switching

---

## Related Files

- `src/window_ops.rs` - Coalescing implementation (reference)
- `src/window_resize.rs` - Height calculation functions
- `src/platform.rs` - Low-level window operations
