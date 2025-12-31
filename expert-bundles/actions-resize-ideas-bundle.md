# Actions Menu Resize UX Expert Bundle

## Executive Summary

When a user has a clean `arg()` prompt with only a text input (no choices), the window uses `ArgPromptNoChoices` sizing (compact ~44px height). Pressing Cmd+K opens the ActionsDialog overlay which is 320x400px max. This creates a jarring visual experience because the actions panel is much taller than the underlying prompt, potentially forcing a window resize or causing the overlay to clip/overflow.

### Key Problems:
1. **Height mismatch**: `ArgPromptNoChoices` is ~44px (MIN_HEIGHT), ActionsDialog is up to 400px
2. **No resize coordination**: The current code treats `ActionsDialog` as "an overlay, don't resize" (line 2039-2041 in main.rs)
3. **User expectation disconnect**: Users expect smooth, professional transitions between states

### Current Behavior:
- `ViewType::ArgPromptNoChoices` = `layout::MIN_HEIGHT` (~44px)  
- `ActionsDialog` max height = 400px (POPUP_MAX_HEIGHT)
- When Cmd+K is pressed, the dialog overlays but the window size doesn't change
- This means the actions panel extends below the window bounds or clips unexpectedly

### Files Included:
- `src/actions.rs`: ActionsDialog implementation, sizing constants, overlay rendering
- `src/window_resize.rs`: Window height calculations, ViewType enum, resize functions
- `src/prompts/arg.rs`: ArgPrompt implementation (for context on the minimal input prompt)
- `src/shortcuts.rs`: Shortcut parsing (for Cmd+K handling context)

---

## Source Code

### src/window_resize.rs (Key Height Calculations)

```rust
//! Dynamic Window Resizing Module
//!
//! **Key Rules:**
//! - ScriptList (main window with preview): FIXED at 500px, never resizes
//! - ArgPrompt with choices: Dynamic height based on choice count (capped at 500px)
//! - ArgPrompt without choices (input only): Compact input-only height
//! - Editor/Div/Term: Full height 700px

pub mod layout {
    use gpui::{px, Pixels};
    use crate::panel::{CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y};

    /// Input row vertical padding (matches default design spacing padding_md)
    pub const ARG_INPUT_PADDING_Y: f32 = 12.0;
    /// Total input-only height (header only, no list)
    pub const ARG_HEADER_HEIGHT: f32 =
        (ARG_INPUT_PADDING_Y * 2.0) + ARG_INPUT_LINE_HEIGHT;

    /// Minimum window height (input only) - for input-only prompts
    pub const MIN_HEIGHT: Pixels = px(ARG_HEADER_HEIGHT);  // ~44px

    /// Standard height for views with preview panel (script list, arg with choices)
    pub const STANDARD_HEIGHT: Pixels = px(500.0);

    /// Maximum window height for full-content views (editor, div, term)
    pub const MAX_HEIGHT: Pixels = px(700.0);
}

/// View types for height calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    ScriptList,
    ArgPromptWithChoices,
    ArgPromptNoChoices,  // <-- This is the ~44px case
    DivPrompt,
    EditorPrompt,
    TermPrompt,
}

pub fn height_for_view(view_type: ViewType, item_count: usize) -> Pixels {
    match view_type {
        ViewType::ScriptList | ViewType::DivPrompt => STANDARD_HEIGHT,
        ViewType::ArgPromptWithChoices => {
            let visible_items = item_count.max(1) as f32;
            let list_height = (visible_items * LIST_ITEM_HEIGHT) + ARG_LIST_PADDING_Y + ARG_DIVIDER_HEIGHT;
            clamp_height(px(ARG_HEADER_HEIGHT + list_height))
        }
        ViewType::ArgPromptNoChoices => MIN_HEIGHT,  // ~44px - THE PROBLEM CASE
        ViewType::EditorPrompt | ViewType::TermPrompt => MAX_HEIGHT,
    }
}
```

### src/actions.rs (ActionsDialog Sizing)

```rust
/// Overlay popup dimensions and styling constants
pub const POPUP_WIDTH: f32 = 320.0;
pub const POPUP_MAX_HEIGHT: f32 = 400.0;  // <-- Much taller than MIN_HEIGHT (~44px)
pub const ACTION_ITEM_HEIGHT: f32 = 42.0;

impl Render for ActionsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Calculate dynamic height based on number of items
        let num_items = self.filtered_actions.len();
        let search_box_height = if self.hide_search { 0.0 } else { 60.0 };
        let items_height = (num_items as f32 * ACTION_ITEM_HEIGHT)
            .min(POPUP_MAX_HEIGHT - search_box_height);
        let total_height = items_height + search_box_height + border_height;

        div()
            .w(px(POPUP_WIDTH))
            .h(px(total_height))  // Dynamic height up to 400px
            // ... rest of rendering
    }
}
```

### main.rs (Current ActionsDialog Handling - excerpts)

```rust
// Line 2039-2041: ActionsDialog is treated as overlay, no resize
AppView::ActionsDialog => {
    // Actions dialog is an overlay, don't resize
    return;
}

// Lines 8371-8410: Actions dialog overlay positioning in arg prompt
.when_some(
    if self.show_actions_popup { self.actions_dialog.clone() } else { None },
    |d, dialog| {
        d.child(
            div()
                .absolute()
                .inset_0()
                .child(
                    div()
                        .absolute()
                        .top(px(52.))  // Clear the header bar
                        .right(px(8.))
                        .child(dialog),
                ),
        )
    },
)
```

---

## Implementation Ideas

### Idea 1: Pre-expand Window When Actions Are Available

**Concept**: If the SDK provides actions for an arg prompt, pre-expand the window to accommodate the actions panel before the user presses Cmd+K. This eliminates the resize transition entirely.

**Implementation**:

```rust
// In main.rs, when receiving SetActions message:
Message::SetActions { actions } => {
    self.sdk_actions = Some(actions.clone());
    
    // If current view is ArgPromptNoChoices and we have actions,
    // pre-expand to STANDARD_HEIGHT to accommodate potential actions panel
    if let AppView::ArgPrompt { choices, .. } = &self.current_view {
        if choices.is_empty() && !actions.is_empty() {
            // Calculate minimum height needed for actions
            let actions_height = (actions.len() as f32 * ACTION_ITEM_HEIGHT)
                .min(POPUP_MAX_HEIGHT);
            let target = px(ARG_HEADER_HEIGHT + actions_height + 20.0);
            
            defer_resize_to_view_height(target, cx);
        }
    }
    cx.notify();
}
```

**Pros**:
- No jarring resize when opening actions
- Simple to implement
- Actions panel always fits within window

**Cons**:
- Wastes space when actions aren't being used
- Window appears larger than necessary for simple text input
- Might confuse users who expect a compact input

---

### Idea 2: Smooth Animated Resize on Cmd+K

**Concept**: When Cmd+K is pressed, animate the window resize to accommodate the actions panel. When closed, animate back to the original size.

**Implementation**:

```rust
// In window_resize.rs, add animated resize:
#[cfg(target_os = "macos")]
pub fn animate_resize_to_height(target_height: Pixels, duration_ms: u64) {
    let height_f64: f64 = f32::from(target_height) as f64;
    let window = match window_manager::get_main_window() {
        Some(w) => w,
        None => return,
    };

    unsafe {
        let current_frame: NSRect = msg_send![window, frame];
        let height_delta = height_f64 - current_frame.size.height;
        let new_origin_y = current_frame.origin.y - height_delta;
        
        let new_frame = NSRect::new(
            NSPoint::new(current_frame.origin.x, new_origin_y),
            NSSize::new(current_frame.size.width, height_f64),
        );

        // animate:true for smooth transition
        let _: () = msg_send![window, setFrame:new_frame display:true animate:true];
    }
}

// In toggle_arg_actions():
fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
    if self.show_actions_popup {
        // Closing - animate back to original size
        self.show_actions_popup = false;
        self.actions_dialog = None;
        
        // Calculate original size
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            let view_type = if choices.is_empty() {
                ViewType::ArgPromptNoChoices
            } else {
                ViewType::ArgPromptWithChoices
            };
            animate_resize_to_height(height_for_view(view_type, choices.len()), 200);
        }
    } else {
        // Opening - animate to accommodate actions
        self.show_actions_popup = true;
        
        let actions_count = self.sdk_actions.as_ref().map(|a| a.len()).unwrap_or(0);
        let actions_height = (actions_count as f32 * ACTION_ITEM_HEIGHT)
            .min(POPUP_MAX_HEIGHT);
        let target = px(ARG_HEADER_HEIGHT + actions_height + 60.0);
        
        animate_resize_to_height(target, 200);
        // Create dialog entity...
    }
}
```

**Pros**:
- Smooth, professional feel
- Window always contains the actions panel properly
- macOS native animation support via `animate:true`

**Cons**:
- Animation may feel slow for frequent toggling
- Requires coordinating animation timing with GPUI render
- Window "bouncing" effect if repeatedly pressed

---

### Idea 3: Floating Detached Actions Panel (macOS HUD-style)

**Concept**: Instead of overlaying inside the window, spawn the actions panel as a separate floating window positioned near the main window.

**Implementation**:

```rust
// In main.rs, modify toggle_arg_actions:
fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
    if self.show_actions_popup {
        // Close floating actions window
        if let Some(actions_window) = self.actions_window.take() {
            cx.close_window(actions_window);
        }
        self.show_actions_popup = false;
    } else {
        // Get main window position
        let main_bounds = window.bounds();
        
        // Calculate position below and to the right of input
        let actions_origin = Point {
            x: main_bounds.origin.x + main_bounds.size.width - POPUP_WIDTH - 16.0,
            y: main_bounds.origin.y - POPUP_MAX_HEIGHT - 8.0, // Above on macOS coords
        };
        
        let actions_size = size(px(POPUP_WIDTH), px(POPUP_MAX_HEIGHT));
        
        // Spawn floating panel window
        let actions_window = cx.open_window(
            WindowOptions {
                bounds: WindowBounds::Windowed(Bounds::new(actions_origin, actions_size)),
                focus: true,
                kind: WindowKind::PopUp,
                decorations: WindowDecorations::None,
                level: Some(WindowLevel::PopUp),
                ..Default::default()
            },
            |cx| {
                let theme_arc = Arc::new(self.theme.clone());
                let sdk_actions = self.sdk_actions.clone();
                cx.new(|cx| {
                    let mut dialog = ActionsDialog::new(
                        cx.focus_handle(),
                        Arc::new(|_| {}),
                        theme_arc,
                    );
                    if let Some(actions) = sdk_actions {
                        dialog.set_sdk_actions(actions);
                    }
                    dialog
                })
            },
        );
        
        self.actions_window = Some(actions_window);
        self.show_actions_popup = true;
    }
}
```

**Pros**:
- Main window size is completely independent
- Actions panel can be positioned optimally
- Matches macOS HUD/popup conventions
- No clipping issues

**Cons**:
- More complex window management
- Focus handling between windows is tricky
- Keyboard events need to route to correct window
- May feel disconnected from the main prompt

---

### Idea 4: Inline Expansion with Reserved Space

**Concept**: Always reserve vertical space below the input for the actions panel, but keep it collapsed/hidden. When Cmd+K is pressed, expand that reserved area with a smooth CSS-like transition.

**Implementation**:

```rust
// Add to ArgPrompt state:
pub struct ArgPrompt {
    // ... existing fields
    actions_expanded: bool,
    actions_target_height: f32,
}

// In render, always render the actions container but control its height:
fn render_arg_prompt(&mut self, ...) -> impl IntoElement {
    let actions_height = if self.actions_expanded {
        self.actions_target_height
    } else {
        0.0
    };
    
    div()
        .flex()
        .flex_col()
        .child(/* input header */)
        .child(
            // Actions container - height animates between 0 and target
            div()
                .h(px(actions_height))
                .overflow_hidden()
                .when(self.actions_expanded, |d| {
                    d.child(self.render_actions_dialog(cx))
                })
        )
}

// Window resize coordination:
fn toggle_arg_actions(&mut self, ...) {
    self.actions_expanded = !self.actions_expanded;
    
    // Calculate new window height including actions space
    let base_height = ARG_HEADER_HEIGHT;
    let actions_height = if self.actions_expanded {
        (self.sdk_actions.len() as f32 * ACTION_ITEM_HEIGHT).min(POPUP_MAX_HEIGHT)
    } else {
        0.0
    };
    
    let target = px(base_height + actions_height);
    defer_resize_to_view_height(target, cx);
    cx.notify();
}
```

**Pros**:
- Inline feel - actions appear as part of the prompt
- Clean expand/collapse behavior
- Window height naturally follows content

**Cons**:
- Still requires resize, just coordinated better
- Actions panel position is fixed (below input)
- Loses the "overlay" aesthetic

---

### Idea 5: Overlay with Window Bounds Expansion (Hybrid)

**Concept**: Keep the overlay positioning but expand the window bounds just enough to ensure the overlay doesn't clip. The overlay still floats above the content, but the window silently grows to contain it.

**Implementation**:

```rust
// In toggle_arg_actions:
fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
    if self.show_actions_popup {
        // Closing
        self.show_actions_popup = false;
        self.actions_dialog = None;
        
        // Restore original window height
        self.restore_pre_actions_height(cx);
    } else {
        // Opening
        let actions_count = self.sdk_actions.as_ref().map(|a| a.len()).unwrap_or(0);
        let actions_height = (actions_count as f32 * ACTION_ITEM_HEIGHT)
            .min(POPUP_MAX_HEIGHT);
        
        // Save current height for restoration
        self.pre_actions_height = get_first_window_height();
        
        // Calculate minimum window height to contain overlay
        // Overlay positioned at top:52px, so window needs 52 + actions_height
        let min_height = 52.0 + actions_height + 8.0; // 8px bottom margin
        let current_height = get_first_window_height().unwrap_or(px(0.0));
        
        if f32::from(current_height) < min_height {
            // Silently expand window to contain overlay
            resize_first_window_to_height(px(min_height));
        }
        
        self.show_actions_popup = true;
        // Create dialog entity...
    }
    cx.notify();
}

// Add field to track pre-actions height:
struct ScriptListApp {
    // ... existing fields
    pre_actions_height: Option<Pixels>,
}

fn restore_pre_actions_height(&mut self, cx: &mut Context<Self>) {
    if let Some(height) = self.pre_actions_height.take() {
        defer_resize_to_view_height(height, cx);
    }
}
```

**Pros**:
- Preserves overlay aesthetic
- Minimal visual disruption - only expands if needed
- Restores to original size on close
- Works with any number of actions

**Cons**:
- Still has resize, just more subtle
- May look odd if window expands downward significantly
- Need to handle edge cases (screen bounds, etc.)

---

## Recommendation

**Best approach for a polished UX: Idea 2 (Smooth Animated Resize) combined with Idea 5 (Minimum Bounds Expansion)**

1. When Cmd+K is pressed, calculate the minimum window height needed
2. If current height is insufficient, animate the window resize with `animate:true`
3. Keep the overlay positioning (top-right inside the window)
4. When actions close, animate back to original height

This gives:
- Smooth, professional feel
- Actions always visible without clipping
- Preserves the overlay aesthetic
- Uses native macOS window animation

---

## Testing

To verify the fix works:

1. Create a test script with a minimal arg prompt (no choices) plus SDK actions:
```typescript
import '../../scripts/kit-sdk';

await arg("Enter something", [], {
  actions: [
    { name: "Action 1" },
    { name: "Action 2" },
    { name: "Action 3" },
    { name: "Action 4" },
    { name: "Action 5" },
  ]
});
```

2. Run via stdin JSON protocol:
```bash
echo '{"type":"run","path":"..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

3. Press Cmd+K and verify:
   - Window expands smoothly (if using animated approach)
   - Actions panel is fully visible
   - No clipping at bottom
   - Closing actions restores original height

4. Capture screenshot to verify visual state:
```typescript
await new Promise(r => setTimeout(r, 500));
const screenshot = await captureScreenshot();
// Save and analyze
```

---

## Instructions For The Next AI Agent

You are reading the "Actions Menu Resize UX Expert Bundle". This file is self-contained and includes all the context you should assume you have.

Your job:

* Design and describe the minimal, safe changes needed to fully resolve the issues described in the Executive Summary and Key Problems.
* Operate **only** on the files and code snippets included in this bundle. If you need additional files or context, clearly say so.

When you propose changes, follow these rules strictly:

1. Always provide **precise code snippets** that can be copy-pasted directly into the repo.
2. Always include **exact file paths** (e.g. `src/window_resize.rs`) and, when possible, line numbers or a clear description of the location (e.g. "replace the existing `toggle_arg_actions` function").
3. Never describe code changes only in prose. Show the full function or block as it should look **after** the change, or show both "before" and "after" versions.
4. Keep instructions **unmistakable and unambiguous**. A human or tool following your instructions should not need to guess what to do.
5. Assume you cannot see any files outside this bundle. If you must rely on unknown code, explicitly note assumptions and risks.

When you answer, you do not need to restate this bundle. Work directly with the code and instructions it contains and return a clear, step-by-step plan plus exact code edits.
