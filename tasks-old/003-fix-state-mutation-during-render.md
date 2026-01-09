# Task: Fix State Mutation During Render

**Priority:** CRITICAL  
**Estimated Effort:** 1 hour  
**Skill Reference:** `gpui` anti-patterns

---

## Problem Description

The `render()` method in `render_script_list.rs` modifies `selected_index` during rendering, which is an anti-pattern in immediate-mode UI frameworks like GPUI.

Mutating state during render can cause:
1. Infinite render loops (state change triggers re-render, which changes state again)
2. Inconsistent UI state
3. Hard-to-debug visual glitches
4. Performance issues from unnecessary re-renders

---

## Affected Files

| File | Lines | Issue |
|------|-------|-------|
| `src/render_script_list.rs` | 50 | `self.selected_index = valid_idx;` |
| `src/render_script_list.rs` | 53 | `self.selected_index = 0;` |
| `src/render_script_list.rs` | 59 | `self.selected_index = 0;` |

---

## Current Problematic Code

```rust
// In render_script_list() or similar render method

// Line 50 - Correcting invalid selection
if self.selected_index >= item_count {
    self.selected_index = valid_idx;  // WRONG: mutation during render
}

// Line 53 - Resetting on empty results
if results.is_empty() {
    self.selected_index = 0;  // WRONG: mutation during render
}

// Line 59 - Another reset case
self.selected_index = 0;  // WRONG: mutation during render
```

---

## Solution

Move selection validation/correction logic to event handlers where state changes belong, not in the render method.

### Step 1: Identify Why Correction is Needed

The corrections happen because:
1. Filter changes reduce item count below current `selected_index`
2. Data refresh changes available items
3. View transitions reset to default state

### Step 2: Create Selection Validation Method

Add a method to handle selection bounds checking:

```rust
impl ScriptListApp {
    /// Ensure selected_index is within valid bounds for current results
    /// Call this after any operation that changes the result count
    fn validate_selection(&mut self, cx: &mut Context<Self>) {
        let max_index = self.get_current_item_count().saturating_sub(1);
        
        if self.selected_index > max_index {
            self.selected_index = max_index.min(0);
            cx.notify();
        }
    }
    
    /// Reset selection to top (for view transitions, filter clears, etc.)
    fn reset_selection(&mut self, cx: &mut Context<Self>) {
        if self.selected_index != 0 {
            self.selected_index = 0;
            cx.notify();
        }
    }
}
```

### Step 3: Call Validation in Event Handlers

Instead of correcting in render, call validation after operations that change item count:

**After filtering:**
```rust
fn handle_filter_change(&mut self, new_filter: String, cx: &mut Context<Self>) {
    self.filter_text = new_filter;
    self.recompute_filtered_results();
    self.validate_selection(cx);  // Ensure selection is valid
    cx.notify();
}
```

**After data refresh:**
```rust
fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
    self.scripts = load_scripts();
    self.recompute_filtered_results();
    self.validate_selection(cx);  // Ensure selection is valid
    cx.notify();
}
```

**After view transition:**
```rust
fn switch_to_script_list(&mut self, cx: &mut Context<Self>) {
    self.current_view = ViewType::ScriptList;
    self.reset_selection(cx);  // Reset to top
    cx.notify();
}
```

### Step 4: Remove Mutations from Render

In `src/render_script_list.rs`, remove all `self.selected_index = ...` assignments:

**Before:**
```rust
fn render_script_list(&mut self, ...) -> impl IntoElement {
    // ... 
    if self.selected_index >= item_count {
        self.selected_index = valid_idx;  // Remove this
    }
    // ...
}
```

**After:**
```rust
fn render_script_list(&self, ...) -> impl IntoElement {
    // Note: &self instead of &mut self (if possible)
    // Selection is already validated before render
    
    let effective_index = self.selected_index.min(item_count.saturating_sub(1));
    // Use effective_index for rendering without mutating state
    // ...
}
```

### Step 5: Handle Edge Case in Render (Read-Only)

If you must handle an edge case during render, compute a local value without mutation:

```rust
fn render_script_list(&self, ...) -> impl IntoElement {
    // Compute effective index locally (no mutation)
    let effective_index = if self.selected_index >= item_count {
        item_count.saturating_sub(1)
    } else {
        self.selected_index
    };
    
    // Use effective_index for rendering
    // The actual self.selected_index will be corrected in the next event handler
}
```

### Step 6: Audit All Render Methods

Search for other mutations during render:

```bash
grep -rn "self\.\w\+ = " src/*render*.rs src/main.rs
```

Any assignment in a `render` method signature (`fn render(&mut self, ...)`) that modifies state affecting visuals should be moved to event handlers.

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

4. **Manual Testing - Filter Behavior:**
   ```bash
   cargo build
   echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   ```
   - Type a filter that matches many items
   - Navigate down to item #10
   - Change filter to match only 3 items
   - Verify selection moves to valid index (not stuck at #10)

5. **Manual Testing - Rapid Navigation:**
   - Open Script Kit
   - Hold down arrow key
   - Verify no visual glitches or lag
   - Check logs for no repeated render warnings

---

## Success Criteria

- [ ] No `self.selected_index = ` in any `render()` method
- [ ] `validate_selection()` called after filter changes
- [ ] `validate_selection()` called after data refresh
- [ ] `reset_selection()` called on view transitions
- [ ] Selection behavior unchanged (still corrects to valid values)
- [ ] `cargo check && cargo clippy && cargo test` passes
- [ ] No visual glitches during rapid filtering

---

## Why This Matters

In GPUI (and similar immediate-mode UI frameworks):

1. **Render should be pure**: Given the same state, render should produce the same output
2. **State changes trigger re-render**: Mutating during render can cause loops
3. **Event handlers own state changes**: User actions → event handler → state change → cx.notify() → re-render

This pattern ensures predictable behavior and makes the code easier to reason about.

---

## Related Files

- `src/app_impl.rs` - Event handlers that should call validation
- `src/app_navigation.rs` - Navigation methods
- `src/filter_coalescer.rs` - Filter change handling
