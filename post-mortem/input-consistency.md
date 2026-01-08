# Post-Mortem: Input Field Consistency Between Views

**Date:** January 8, 2025  
**Issue:** Visual inconsistency between main menu and file search input fields  
**Time to Fix:** ~2 hours across multiple sessions  
**Commits:** 3 separate fixes

## The "Simple" Task

Make the input field in File Search View look identical to the main menu input field. When switching between views, the experience should feel seamless—like using the same application.

## What Actually Happened

What appeared to be a single styling issue was actually **four separate bugs** stacked on top of each other, each masking the others:

### Bug 1: Placeholder Text Not Syncing
- **Symptom:** File search showed "Script Kit" instead of "Search files..."
- **Root Cause:** `sync_filter_input_if_needed()` wasn't called for `FileSearchView`
- **Fix:** Add `AppView::FileSearchView { .. }` to the match statement in `main.rs`

### Bug 2: Header Height Mismatch  
- **Symptom:** Placeholder text shifted up when entering file search
- **Root Cause:** Main menu header was 28px tall (due to "Ask AI" button padding), file search header was 22px tall
- **Initial Wrong Fix:** Added `min_h(px(28))` to the flex row—didn't work properly
- **Correct Fix:** Add `py(px(4.))` to the "0 files" text wrapper to match "Ask AI" button structure

### Bug 3: Implicit Height via Sibling Elements
- **Symptom:** Even with same constants, heights differed
- **Root Cause:** Main menu's header height was implicitly determined by its tallest child (the "Ask AI" button with 8px vertical padding), not by any explicit height declaration
- **Lesson:** Flex layouts derive height from children; you must match *all* children's sizing, not just the primary element

### Bug 4: Container Border Adding Padding
- **Symptom:** Entire window content shifted down/inward in file search
- **Root Cause:** File search had `.border(px(1.))` and `.bg()` that main menu didn't have
- **Fix:** Remove border and background to match main menu's container structure

## Why This Was So Painful

### 1. Onion Layers of Bugs
Each fix revealed the next bug. The placeholder fix exposed the height issue. The height fix exposed the border issue. You couldn't see bug #4 until bugs #1-3 were fixed.

### 2. No Shared Component
Main menu and file search each implemented their own header structure:
```rust
// Main menu (render_script_list.rs)
div().px(...).py(...).flex().flex_row().items_center().gap(...)
    .child(Input::new(...))
    .child(/* Ask AI button with py(4px) */)

// File search (render_builtins.rs) - DUPLICATED
div().px(...).py(...).flex().flex_row().items_center().gap(...)
    .child(Input::new(...))
    .child(/* "0 files" text with NO padding */)
```

No shared `PromptHeader` component meant:
- Easy for implementations to diverge
- Changes to one didn't automatically apply to the other
- Had to manually ensure every property matched

### 3. Implicit vs Explicit Sizing
The main menu header height wasn't explicitly set anywhere. It was *implicitly* 28px because:
- The "Ask AI" button had `.py(px(4.))` = 8px vertical padding
- Button content (text + badge) was ~20px
- Total: ~28px, making the flex row 28px tall
- Input (22px) was centered within via `items_center()`

This implicit relationship wasn't documented and wasn't obvious from reading the code.

### 4. Multiple Interacting Styling Layers
```
Container
  └── border: 1px (file search only!)
  └── background (file search only!)
  └── Header Div
        └── padding: 8px vertical
        └── flex row with items_center
        └── Input Wrapper
              └── Input (22px explicit height)
        └── Right-side Element
              └── padding: 4px vertical (main menu only!)
```

A mismatch at ANY layer caused visual inconsistency.

### 5. Visual Bugs Require Visual Verification
- Code that "looks correct" can still render incorrectly
- Log output doesn't show layout issues
- Required actual screenshots to verify fixes
- Each iteration needed manual comparison

## The Fixes (In Order Applied)

1. **Placeholder sync:** Add `FileSearchView` to `sync_filter_input_if_needed()` match
2. **Header height:** Wrap "0 files" text with `py(px(4.))` padding (matching "Ask AI" button)
3. **Container structure:** Remove `.border()` and `.bg()` from file search container

## Prevention: What Would Have Caught This Earlier

### Shared Components
```rust
// A single PromptHeader component used by all views
pub struct PromptHeader {
    input: Entity<InputState>,
    right_element: AnyElement,
}

impl PromptHeader {
    pub fn render(&self) -> impl IntoElement {
        div()
            .px(px(HEADER_PADDING_X))
            .py(px(HEADER_PADDING_Y))
            .flex().flex_row().items_center()
            .gap(px(HEADER_GAP))
            .child(self.render_input())
            .child(
                // Wrapper ensures consistent height regardless of content
                div().py(px(4.)).child(&self.right_element)
            )
    }
}
```

### Visual Regression Tests
Automated screenshot comparison between views:
```typescript
test('header consistency', async () => {
    const mainMenu = await captureView('main-menu');
    const fileSearch = await captureView('file-search');
    
    // Extract header regions and compare pixel-by-pixel
    expect(mainMenu.header).toMatchPixels(fileSearch.header);
});
```

### Explicit Documentation of Implicit Relationships
```rust
// LAYOUT CONTRACT: Header height is determined by the right-side element.
// The "Ask AI" button is 28px tall (20px content + 8px padding).
// All header right-side elements MUST use py(px(4.)) wrapper for consistency.
// Input (22px) will be vertically centered within the 28px row.
```

## Key Lessons

1. **Visual consistency requires structural consistency.** Same constants aren't enough if the container/wrapper structures differ.

2. **Implicit sizing is a maintenance trap.** Document why things are sized the way they are, especially when one element's size affects siblings.

3. **Bugs stack.** What looks like one bug can be multiple bugs that only become visible sequentially.

4. **Shared components > shared constants.** Constants ensure same values; components ensure same structure.

5. **Borders are sneaky.** A 1px border adds 2px total to each dimension and shifts all content inward.

## Files Changed

- `src/main.rs` - Add FileSearchView to sync match
- `src/render_builtins.rs` - Match header structure and remove container border/bg
