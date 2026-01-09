# Task: Replace Expensive Theme Cloning with Lightweight Color Structs

**Priority:** HIGH  
**Estimated Effort:** 1-2 hours  
**Skill Reference:** `script-kit-theme`

---

## Problem Description

The codebase has 18 instances of `Arc::new(self.theme.clone())` which clones the entire Theme struct (including all color schemes, opacity settings, fonts, etc.) just to pass colors into closures.

This causes:
1. **Heap allocation** on every clone
2. **Memory pressure** from duplicate theme data
3. **Unnecessary copying** of large struct
4. **Slower render cycles**

The solution already exists in the codebase: `ListItemColors` and similar lightweight `Copy` structs that can be passed into closures without allocation.

---

## Affected Files

| File | Line | Count |
|------|------|-------|
| `src/prompt_handler.rs` | 134, 220, 317, 333, 344, 1078, 1175, 1241, 1294, 1362 | 10 |
| `src/app_impl.rs` | 449, 1956, 2044, 2467 | 4 |
| `src/app_execute.rs` | 875, 952, 1101 | 3 |
| `src/render_prompts/path.rs` | 17 | 1 |

---

## Current Problematic Code

```rust
// EXPENSIVE: Clones entire Theme struct, wraps in Arc
let theme_arc = Arc::new(self.theme.clone());

// Later, in closure:
some_closure(move || {
    let colors = &theme_arc.colors;
    // use colors...
});
```

---

## Solution

### Step 1: Understand Existing Lightweight Structs

The codebase already has `Copy` color structs in `src/list_item.rs` and `src/theme/helpers.rs`:

```rust
// src/list_item.rs:220-268
#[derive(Clone, Copy)]
pub struct ListItemColors {
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_tertiary: u32,
    pub text_muted: u32,
    pub background: u32,
    pub accent: u32,
    pub accent_subtle: u32,
    pub selected_opacity: f32,
    pub hover_opacity: f32,
}

impl ListItemColors {
    pub fn from_theme(theme: &Theme) -> Self { ... }
    pub fn from_design(colors: &DesignColors) -> Self { ... }
}
```

### Step 2: Create Additional Lightweight Structs (If Needed)

If `ListItemColors` doesn't have all the colors you need, create additional structs:

```rust
// src/theme/helpers.rs - Add these if not present

#[derive(Clone, Copy)]
pub struct PromptColors {
    pub background: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub accent: u32,
    pub border: u32,
    pub input_bg: u32,
}

impl PromptColors {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
            input_bg: theme.colors.background.search_box,
        }
    }
}
```

### Step 3: Replace Arc<Theme> with Lightweight Struct

#### Pattern 1: Simple Color Access

**Before:**
```rust
let theme_arc = Arc::new(self.theme.clone());
let closure = move || {
    let bg = theme_arc.colors.background.main;
    // ...
};
```

**After:**
```rust
let colors = ListItemColors::from_theme(&self.theme);  // Copy, no allocation
let closure = move || {
    let bg = colors.background;
    // ...
};
```

#### Pattern 2: Creating Entities with Theme

**Before:**
```rust
// src/prompt_handler.rs - Creating a prompt entity
let theme_arc = Arc::new(self.theme.clone());
let entity = cx.new(|cx| {
    SomePrompt::new(theme_arc.clone(), ...)
});
```

**After:**
```rust
// Option A: Pass reference if prompt stores Arc<Theme>
let entity = cx.new(|cx| {
    SomePrompt::new(self.theme.clone(), ...)  // Single clone, not Arc wrapper
});

// Option B: Pass lightweight colors if prompt only needs colors
let colors = PromptColors::from_theme(&self.theme);
let entity = cx.new(|cx| {
    SomePrompt::new(colors, ...)
});
```

### Step 4: Fix Each Location

#### `src/prompt_handler.rs` (10 locations)

Search for `Arc::new(self.theme.clone())` and replace:

```rust
// Line ~134 - Example fix
// Before:
let theme = Arc::new(self.theme.clone());
let entity = cx.new(|cx| DivPrompt::new(theme, ...));

// After:
let entity = cx.new(|cx| DivPrompt::new(self.theme.clone(), ...));
// Or if DivPrompt only needs colors:
let colors = PromptColors::from_theme(&self.theme);
let entity = cx.new(|cx| DivPrompt::new(colors, ...));
```

#### `src/app_impl.rs` (4 locations)

```rust
// Line ~449 - Example
// Before:
let theme_arc = Arc::new(self.theme.clone());

// After:
let colors = ListItemColors::from_theme(&self.theme);
```

#### `src/app_execute.rs` (3 locations)

Same pattern as above.

#### `src/render_prompts/path.rs` (1 location)

Same pattern as above.

### Step 5: Update Prompt Constructors (If Needed)

If prompts currently expect `Arc<Theme>`, update their constructors to accept either:
- `Theme` directly (single clone is fine)
- Lightweight color struct (zero allocation)

```rust
// Before:
impl DivPrompt {
    pub fn new(theme: Arc<Theme>, ...) -> Self { ... }
}

// After - Option A: Take owned Theme
impl DivPrompt {
    pub fn new(theme: Theme, ...) -> Self { ... }
}

// After - Option B: Take colors struct
impl DivPrompt {
    pub fn new(colors: PromptColors, ...) -> Self { ... }
}
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

4. **Search for Remaining Arc Clones:**
   ```bash
   grep -rn "Arc::new(self.theme.clone())" src/
   # Should return 0 results
   ```

5. **Memory Profile (Optional):**
   - Before fix: Note memory usage
   - After fix: Compare memory usage
   - Should see reduction in heap allocations

---

## Success Criteria

- [ ] No `Arc::new(self.theme.clone())` in codebase
- [ ] Lightweight color structs used in closures
- [ ] Prompt constructors updated if needed
- [ ] All prompts render correctly with new pattern
- [ ] `cargo check && cargo clippy && cargo test` passes

---

## Performance Impact

| Metric | Before | After |
|--------|--------|-------|
| Theme clone | Full struct (~1KB+) | Copy struct (~64 bytes) |
| Allocation | Heap (Arc) | Stack (Copy) |
| Per render | Multiple clones possible | Zero allocation |

---

## Related Files

- `src/theme/helpers.rs` - Lightweight color structs
- `src/list_item.rs` - `ListItemColors` definition
- `src/theme/types.rs` - Full Theme struct
