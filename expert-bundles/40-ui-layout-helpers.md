# Expert Bundle 40: UI Layout Helpers

## Goal
Create a small set of reusable layout helper functions to improve consistency and reduce boilerplate across all UI components.

## Current State

The codebase has extensive duplication of GPUI layout patterns across ~15 files:
- 15+ uses of `div().flex().flex_col()` pattern
- 12+ uses of `div().flex().flex_row().items_center()` pattern
- 81+ theme color access patterns `rgb(colors.text_*)`
- 29 occurrences of arrow key matching with both variants
- 24+ padding applications with design token spacing

No centralized layout utilities exist, leading to inconsistent patterns.

## Specific Concerns

1. **Stack Layout Patterns (27+ copies)**: `vstack()` and `hstack()` equivalents are manually typed repeatedly with slight variations.

2. **Theme Color Access (81+ copies)**: Direct `rgb(self.theme.colors.text.primary)` calls throughout; no extension trait for cleaner access.

3. **Arrow Key Matching (29 locations)**: Inconsistent matching of `"up" | "arrowup"` vs just `"up"` - platform-dependent and error-prone.

4. **Design Token Conditionals (17 locations)**: Same `if design_variant == Default { theme } else { design }` pattern everywhere.

5. **Container Patterns (15+ copies)**: Card-like containers and list items rebuilt with slight variations each time.

## Key Questions

1. Should layout helpers be free functions (`vstack()`) or builder extensions (`div().vstack()`)?

2. Is a `ThemeColorExt` trait the right abstraction for `theme.text_primary()` style access?

3. Should key matching helpers be a `keys` module with `is_up()`, `is_down()` functions, or pattern-matching macros?

4. How prescriptive should container helpers be - just layout, or full styling (rounded corners, shadows)?

5. Should these live in `src/ui/` module or `src/ui_foundation.rs`?

## Key Patterns to Extract

```rust
// Stack layouts
pub fn vstack() -> Div { div().flex().flex_col() }
pub fn hstack() -> Div { div().flex().flex_row().items_center() }
pub fn centered() -> Div { div().flex().items_center().justify_center() }
pub fn spacer() -> Div { div().flex_1() }

// Color helpers
pub trait ThemeColorExt {
    fn text_primary(&self) -> Hsla;
    fn bg_main(&self) -> Hsla;
}

// Key matching
pub mod keys {
    pub fn is_up(key: &str) -> bool { matches!(key, "up" | "arrowup") }
    pub fn is_down(key: &str) -> bool { matches!(key, "down" | "arrowdown") }
}
```

## Implementation Checklist

- [ ] Create `src/ui/mod.rs` with submodules
- [ ] Add `src/ui/layout.rs` with `vstack()`, `hstack()`, `centered()`, `spacer()`
- [ ] Add `src/ui/colors.rs` with `ThemeColorExt` trait
- [ ] Add `src/ui/keys.rs` with key matching helpers
- [ ] Add `src/ui/containers.rs` with `card()`, `list_item_container()` builders
- [ ] Document usage patterns in module docs
- [ ] Gradually adopt in new code
- [ ] Consider systematic migration of existing code
