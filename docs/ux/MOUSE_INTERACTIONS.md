# Mouse Interactions Patterns

This document covers mouse interaction patterns in Script Kit GPUI, including click-outside-dismiss for dialogs, focus management, click handler requirements, and visual testing infrastructure.

---

## Table of Contents

1. [Click-Outside-Dismiss Pattern](#1-click-outside-dismiss-pattern)
2. [Focus Management](#2-focus-management)
3. [Click Handler Requirements](#3-click-handler-requirements)
4. [Visual Testing Infrastructure](#4-visual-testing-infrastructure)
5. [Best Practices](#5-best-practices)

---

## 1. Click-Outside-Dismiss Pattern

The click-outside-dismiss pattern allows users to close dialogs by clicking anywhere outside the dialog bounds. This is implemented using a **backdrop layer** that covers the parent container and captures clicks.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Preview Panel (relative positioned container)           │
│  ┌─────────────────────────────────────────────────────┐│
│  │  Backdrop Layer (absolute, inset_0)                 ││
│  │  - Transparent background                           ││
│  │  - Captures clicks via on_click()                   ││
│  │  ┌───────────────────┐                              ││
│  │  │  Dialog Container │  ← Positioned above backdrop ││
│  │  │  (actions/menu)   │                              ││
│  │  └───────────────────┘                              ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

### Implementation in main.rs (Lines 7695-7738)

```rust
// Wrapper with relative positioning enables absolute children
div()
    .relative() // Enable absolute positioning for overlay
    .w_1_2()    // 50% width
    .h_full()   // Take full height
    // Base content ALWAYS renders (visible behind overlay)
    .child(self.render_preview_panel(cx))
    // Dialog overlays on top using absolute positioning
    .when_some(
        if self.show_actions_popup {
            self.actions_dialog.clone()
        } else {
            None
        },
        |d, dialog| {
            // Create click handler for backdrop dismissal
            let backdrop_click = cx.listener(
                |this: &mut Self, _event: &gpui::ClickEvent, window: &mut Window, cx: &mut Context<Self>| {
                    logging::log("FOCUS", "Actions backdrop clicked - dismissing dialog");
                    // 1. Hide the popup
                    this.show_actions_popup = false;
                    this.actions_dialog = None;
                    // 2. Restore focus to main input
                    this.focused_input = FocusedInput::MainFilter;
                    window.focus(&this.focus_handle, cx);
                    // 3. Trigger re-render
                    cx.notify();
                }
            );

            d.child(
                div()
                    .absolute()
                    .inset_0() // Cover entire parent area
                    // Backdrop layer - captures clicks outside the dialog
                    .child(
                        div()
                            .id("actions-backdrop")  // REQUIRED: ID for click handling
                            .absolute()
                            .inset_0()
                            .on_click(backdrop_click)
                    )
                    // Dialog container - positioned within the overlay
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .justify_end()
                            .pr(px(8.))
                            .pt(px(8.))
                            .child(dialog),
                    ),
            )
        },
    )
```

### ActionsDialog dismiss_on_click_outside() Method (src/actions.rs:537-544)

The `ActionsDialog` component provides a public method for external dismiss handling:

```rust
/// Dismiss the dialog when user clicks outside its bounds.
/// This is a public method called from the parent container's click-outside handler.
/// Logs the event and triggers the cancel callback.
pub fn dismiss_on_click_outside(&mut self) {
    tracing::info!(
        target: "script_kit::actions",
        "ActionsDialog dismiss-on-click-outside triggered"
    );
    logging::log("ACTIONS", "Actions dialog dismissed (click outside)");
    self.submit_cancel();
}
```

### Key Points

| Aspect | Implementation |
|--------|----------------|
| **Parent Container** | Must have `.relative()` positioning |
| **Backdrop Layer** | Uses `.absolute().inset_0()` to cover entire parent |
| **ID Requirement** | Backdrop needs `.id("...")` for click events to work |
| **Click Handler** | Use `cx.listener()` to create callback |
| **State Reset** | Must reset visibility flag, dialog entity, and focus |
| **Notification** | Always call `cx.notify()` after state changes |

---

## 2. Focus Management

Focus management ensures keyboard events are routed to the correct component after mouse interactions.

### Focus Tracking Enum

```rust
/// Tracks which input field currently has focus for cursor display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedInput {
    /// Main script list filter input
    MainFilter,
    /// Actions dialog search input
    ActionsSearch,
    /// Arg prompt input (when running a script)
    ArgPrompt,
    /// No input focused (e.g., terminal prompt)
    None,
}
```

### Focus Restoration Pattern

When dismissing a dialog, always restore focus to the appropriate element:

```rust
// In backdrop click handler:
fn dismiss_and_restore_focus(
    this: &mut Self,
    window: &mut Window,
    cx: &mut Context<Self>
) {
    // 1. Update state
    this.show_actions_popup = false;
    this.actions_dialog = None;
    
    // 2. Update focus tracking
    this.focused_input = FocusedInput::MainFilter;
    
    // 3. Actually move keyboard focus
    window.focus(&this.focus_handle, cx);
    
    // 4. Trigger re-render
    cx.notify();
}
```

### Focusable Trait Implementation

Components that receive keyboard input must implement `Focusable`:

```rust
impl Focusable for ActionsDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
```

### Delegated Focus Pattern (FormPromptState)

For containers with multiple focusable children, delegate focus to the active child:

```rust
impl Focusable for FormPromptState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        // Return the focused field's handle, not our own
        // This prevents the container from "stealing" focus during re-renders
        if let Some((_, entity)) = self.fields.get(self.focused_index) {
            match entity {
                FormFieldEntity::TextField(e) => e.read(cx).get_focus_handle(),
                FormFieldEntity::TextArea(e) => e.read(cx).get_focus_handle(),
                FormFieldEntity::Checkbox(e) => e.read(cx).focus_handle(cx),
            }
        } else {
            // Fallback to our own handle if no fields exist
            self.focus_handle.clone()
        }
    }
}
```

---

## 3. Click Handler Requirements

All click handlers must follow these requirements for proper functionality.

### Required Pattern

```rust
div()
    .id(element_id)          // REQUIRED: Unique ID for event routing
    .cursor_pointer()        // Visual feedback: cursor changes on hover
    .on_click(cx.listener(|this, event, window, cx| {
        // 1. Log the interaction
        logging::log("UI", "Element clicked");
        
        // 2. Update state
        this.handle_click_logic();
        
        // 3. Update focus if needed
        window.focus(&this.focus_handle, cx);
        
        // 4. ALWAYS notify to trigger re-render
        cx.notify();
    }))
```

### ID Requirement

**CRITICAL**: Click handlers require an ID. Without it, events won't be dispatched:

```rust
// WRONG - click handler won't work
div()
    .on_click(cx.listener(|...| { ... }))  // No ID!

// CORRECT - ID enables event routing
div()
    .id("my-button")  // Required for on_click to work
    .on_click(cx.listener(|...| { ... }))
```

### cx.notify() Requirement

After any state mutation that affects rendering, call `cx.notify()`:

```rust
fn handle_selection(&mut self, index: usize, cx: &mut Context<Self>) {
    self.selected_index = index;  // State changed
    cx.notify();                   // REQUIRED: triggers re-render
}
```

### Logging Best Practices

Include context in click logs for debugging:

```rust
logging::log("UI", &format!(
    "List item clicked: index={}, item={}",
    index, item.name
));

tracing::info!(
    target: "script_kit::actions",
    item_id = %id,
    "Action selected"
);
```

---

## 4. Visual Testing Infrastructure

The visual testing infrastructure enables automated testing of mouse interactions without manual intervention.

### Test Utilities Location

`tests/sdk/test-click-utils.ts`

### Core Functions

#### simulateClick(x, y, button?)

Simulates a mouse click at window-relative coordinates:

```typescript
import { simulateClick, waitForRender, captureAndSave } from './test-click-utils';

// Left click (default)
await simulateClick(100, 200);

// Right click
await simulateClick(150, 300, 'right');

// Middle click
await simulateClick(200, 250, 'middle');
```

#### waitForRender(ms?)

Waits for UI to update after interactions:

```typescript
await simulateClick(100, 200);
await waitForRender(300);  // Wait 300ms for animation
```

#### captureAndSave(name)

Captures screenshot and saves to `./test-screenshots/`:

```typescript
const filepath = await captureAndSave('after-click');
// Saves to ./test-screenshots/after-click-{timestamp}.png
```

#### clickAndCapture(x, y, name, waitMs?)

Convenience function combining click + wait + capture:

```typescript
const filepath = await clickAndCapture(100, 200, 'button-clicked');
```

#### getWindowBoundsForClick()

Gets window dimensions for coordinate calculation:

```typescript
const bounds = await getWindowBoundsForClick();
// Click center of window
await simulateClick(bounds.width / 2, bounds.height / 2);
```

### Complete Test Example

```typescript
// tests/smoke/test-click-dismiss.ts
import '../../scripts/kit-sdk';
import {
  simulateClick,
  waitForRender,
  captureAndSave,
  runVisualTest,
  getWindowBoundsForClick
} from '../sdk/test-click-utils';

await runVisualTest('dialog-dismiss-on-click-outside', async () => {
  // 1. Setup: Show dialog
  await arg('Pick', ['Apple', 'Banana', 'Cherry']);
  await waitForRender(500);
  
  // 2. Get window bounds
  const bounds = await getWindowBoundsForClick();
  
  // 3. Click outside dialog area (far corner)
  await simulateClick(bounds.width - 10, bounds.height - 10);
  await waitForRender(300);
  
  // 4. Capture result
  return await captureAndSave('dialog-dismissed');
});
```

### Running Visual Tests

```bash
# Build and run via stdin JSON protocol
cargo build && \
  echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-click-dismiss.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Screenshot saved to ./test-screenshots/
```

### Test Result Format (JSONL)

Tests output structured results:

```json
{"test": "dialog-dismiss", "status": "running", "timestamp": "2024-..."}
{"test": "dialog-dismiss", "status": "pass", "duration_ms": 450, "screenshot": "./test-screenshots/dialog-dismissed-1234.png"}
```

---

## 5. Best Practices

### DO

| Pattern | Example |
|---------|---------|
| Use `.id()` on clickable elements | `.id("submit-btn")` |
| Call `cx.notify()` after state changes | `self.selected = true; cx.notify();` |
| Log click events | `logging::log("UI", "Button clicked")` |
| Restore focus after dialog dismiss | `window.focus(&handle, cx)` |
| Use backdrop for click-outside | Absolute positioned overlay with `on_click` |
| Use `cx.listener()` for callbacks | Type-safe event handlers |
| Add `.cursor_pointer()` | Visual feedback for clickable elements |

### DON'T

| Anti-Pattern | Why It's Wrong |
|--------------|----------------|
| Skip `.id()` on clickable divs | Click events won't fire |
| Forget `cx.notify()` | UI won't update |
| Hardcode click coordinates in tests | Breaks on layout changes |
| Skip focus restoration | Keyboard navigation breaks |
| Use anonymous closures without `cx.listener()` | Can't access `this` |

### Click Handler Checklist

```rust
// Before every click handler implementation:
// [ ] Element has .id() with unique ID
// [ ] Handler uses cx.listener() pattern
// [ ] Handler logs the interaction
// [ ] Handler calls cx.notify() after state changes
// [ ] Focus is updated if needed
// [ ] Visual feedback provided (.cursor_pointer(), hover states)
```

### Testing Checklist

```bash
# Before committing mouse interaction changes:
# [ ] Write visual test using test-click-utils
# [ ] Run test via stdin JSON protocol
# [ ] Capture and examine screenshot
# [ ] Verify click coordinates make sense for layout
# [ ] Check logs for expected click/focus events
```

---

## References

- **Source Files**:
  - `src/actions.rs` - `dismiss_on_click_outside()` implementation
  - `src/main.rs` - Backdrop pattern for actions dialog
  - `tests/sdk/test-click-utils.ts` - Testing helpers

- **Related Documentation**:
  - `docs/ux/ACCESSIBILITY.md` - Keyboard navigation patterns
  - `AGENTS.md` - Testing protocol and verification gate
