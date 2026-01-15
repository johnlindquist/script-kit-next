# Alias Dialog: Cursor Blink & Focus Restore - Expert Bundle

## Original Goal

> The modal input needs a blinking cursor (is it using the gpui input component?) and when it loses focus, the main menu input needs to refocus.

## Executive Summary

The `AliasInput` component exists and IS properly wired as a GPUI entity with `Focusable` trait - keyboard events DO work. However, there are two remaining issues:

1. **No cursor blinking** - The cursor is rendered as a static div. It needs to participate in the app's global cursor blink timer (530ms interval).
2. **No focus restoration** - When `close_alias_input()` is called, it doesn't set `pending_focus = Some(FocusTarget::MainFilter)` like `close_shortcut_recorder()` does.

### Key Problems:
1. **Static cursor** - In `alias_input.rs:295-299`, the cursor is always visible: `.bg(rgb(colors.accent))` - no conditional rendering.
2. **Missing `cursor_visible` field** - The `AliasInput` struct has no `cursor_visible: bool` field.
3. **Missing `set_cursor_visible()` method** - No way for the app's blink timer to update cursor state.
4. **Blink timer doesn't update alias input** - In `app_impl.rs:158-166`, the timer only updates `ActionsDialog`, not `AliasInput`.
5. **Missing `pending_focus` on close** - `close_alias_input()` at line 3636 doesn't restore focus.

### Required Fixes:
1. **Add `cursor_visible: bool`** to `AliasInput` struct (default `true`)
2. **Add `set_cursor_visible(&mut self, visible: bool)`** method
3. **Conditionally render cursor** based on `cursor_visible` in `render_input_field()`
4. **Update blink timer** in `app_impl.rs` to also call `set_cursor_visible` on `alias_input_entity`
5. **Add focus restoration** - `self.pending_focus = Some(FocusTarget::MainFilter)` in `close_alias_input()`

### Files to Modify:
- `src/components/alias_input.rs`: Add cursor_visible field and method, conditional rendering
- `src/app_impl.rs`: Update blink timer (line ~160) and close_alias_input (line ~3640)

---

## File: src/components/alias_input.rs (Current Implementation - Lines 114-131)

```rust
/// Alias Input Modal Component
///
/// A modal dialog for entering command aliases with full keyboard support.
pub struct AliasInput {
    /// Focus handle for keyboard input - CRITICAL for keyboard events
    pub focus_handle: FocusHandle,
    /// Theme for styling
    pub theme: Arc<Theme>,
    /// Pre-computed colors
    pub colors: AliasInputColors,
    /// Name of the command being configured
    pub command_name: String,
    /// ID of the command being configured
    pub command_id: String,
    /// Text input state (handles selection, cursor, etc.)
    pub input: TextInputState,
    /// Current alias (if editing an existing one)
    pub current_alias: Option<String>,
    /// Pending action for the parent to handle (polled after render)
    pub pending_action: Option<AliasInputAction>,
    // MISSING: cursor_visible: bool,
}
```

## File: src/components/alias_input.rs (render_input_field - Lines 284-302 - THE BUG)

```rust
            } else {
                // Render with cursor indicator
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_base()
                    .text_color(rgb(colors.text_primary))
                    .child(before)
                    .child(
                        // Cursor - ALWAYS VISIBLE, NO BLINKING!
                        div()
                            .w(px(2.))
                            .h(px(18.))
                            .bg(rgb(colors.accent))  // <-- BUG: No conditional
                            .rounded(px(1.)),
                    )
                    .child(after)
            }
```

---

## File: src/app_impl.rs (Cursor Blink Timer - Lines 138-172)

```rust
        // Start cursor blink timer - updates all inputs that track cursor visibility
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(std::time::Duration::from_millis(530)).await;
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        // Skip cursor blink when:
                        // 1. Window is hidden (no visual feedback needed)
                        // 2. No window is focused (main window OR actions popup)
                        // 3. No input is focused (no cursor to blink)
                        let actions_popup_open = is_actions_window_open();
                        let any_window_focused =
                            platform::is_main_window_focused() || actions_popup_open;
                        if !script_kit_gpui::is_main_window_visible()
                            || !any_window_focused
                            || app.focused_input == FocusedInput::None
                        {
                            return;
                        }

                        app.cursor_visible = !app.cursor_visible;
                        // Also update ActionsDialog cursor if it exists
                        if let Some(ref dialog) = app.actions_dialog {
                            dialog.update(cx, |d, _cx| {
                                d.set_cursor_visible(app.cursor_visible);
                            });
                            // Notify the actions window to repaint with new cursor state
                            notify_actions_window(cx);
                        }
                        // MISSING: Update alias_input_entity cursor visibility!
                        cx.notify();
                    })
                });
            }
        })
        .detach();
```

---

## File: src/app_impl.rs (close_alias_input - Lines 3635-3642 - MISSING FOCUS RESTORE)

```rust
    /// Close the alias input and clear state.
    pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
        if self.alias_input_state.is_some() || self.alias_input_entity.is_some() {
            logging::log("ALIAS", "Closing alias input");
            self.alias_input_state = None;
            self.alias_input_entity = None; // Clear entity to reset for next open
            // MISSING: self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }
```

## Compare with: close_shortcut_recorder (Lines 3359-3372) - CORRECT PATTERN

```rust
    /// Close the shortcut recorder and clear state.
    /// Returns focus to the main filter input.
    pub fn close_shortcut_recorder(&mut self, cx: &mut Context<Self>) {
        if self.shortcut_recorder_state.is_some() || self.shortcut_recorder_entity.is_some() {
            logging::log(
                "SHORTCUT",
                "Closing shortcut recorder, returning focus to main filter",
            );
            self.shortcut_recorder_state = None;
            self.shortcut_recorder_entity = None;
            // Return focus to the main filter input
            self.pending_focus = Some(FocusTarget::MainFilter);  // <-- THIS IS NEEDED
            cx.notify();
        }
    }
```

---

## File: src/actions/dialog.rs (Reference: set_cursor_visible - Lines 353-355)

```rust
    /// Update cursor visibility (called from parent's blink timer)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }
```

---

## Implementation Guide

### Step 1: Add cursor_visible to AliasInput struct

In `src/components/alias_input.rs`, add to struct fields:

```rust
pub struct AliasInput {
    // ... existing fields ...
    
    /// Cursor visibility for blinking (controlled by parent's blink timer)
    pub cursor_visible: bool,
}
```

### Step 2: Initialize cursor_visible in new()

```rust
impl AliasInput {
    pub fn new(cx: &mut Context<Self>, theme: Arc<Theme>) -> Self {
        // ... existing code ...
        Self {
            // ... existing fields ...
            cursor_visible: true,  // Start visible
        }
    }
}
```

### Step 3: Add set_cursor_visible method

```rust
impl AliasInput {
    /// Update cursor visibility (called from parent's blink timer)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }
}
```

### Step 4: Conditionally render cursor in render_input_field

```rust
fn render_input_field(&self, _cx: &mut Context<Self>) -> impl IntoElement {
    // ... existing code ...
    
    // In the cursor rendering section:
    .child(
        // Cursor - conditionally visible for blinking
        div()
            .w(px(2.))
            .h(px(18.))
            .rounded(px(1.))
            .when(self.cursor_visible, |d| d.bg(rgb(colors.accent))),
    )
}
```

### Step 5: Update blink timer in app_impl.rs

Around line 160, after updating ActionsDialog:

```rust
app.cursor_visible = !app.cursor_visible;

// Update ActionsDialog cursor if it exists
if let Some(ref dialog) = app.actions_dialog {
    dialog.update(cx, |d, _cx| {
        d.set_cursor_visible(app.cursor_visible);
    });
    notify_actions_window(cx);
}

// Update AliasInput cursor if it exists
if let Some(ref alias_input) = app.alias_input_entity {
    alias_input.update(cx, |input, _cx| {
        input.set_cursor_visible(app.cursor_visible);
    });
}

cx.notify();
```

### Step 6: Add focus restoration to close_alias_input

In `src/app_impl.rs`, around line 3640:

```rust
pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
    if self.alias_input_state.is_some() || self.alias_input_entity.is_some() {
        logging::log("ALIAS", "Closing alias input, returning focus to main filter");
        self.alias_input_state = None;
        self.alias_input_entity = None;
        // Return focus to the main filter input (like close_shortcut_recorder does)
        self.pending_focus = Some(FocusTarget::MainFilter);
        cx.notify();
    }
}
```

---

## About gpui_component Input

**Q: Is it using the gpui_component input?**

**A: No.** The `AliasInput` uses a custom `TextInputState` (from `src/components/text_input.rs`) which handles text editing, selection, and cursor position. The rendering is manual via divs.

The app DOES have `gpui_component::input::{Input, InputState}` available (see `src/main.rs:12`), and there's even a wrapper `ScriptKitInput` in `src/components/script_kit_input.rs`. However, the alias input was built with the simpler `TextInputState` approach for consistency with other modal dialogs.

If you wanted to use `gpui_component::Input` instead, you'd need to:
1. Create an `Entity<InputState>` in `AliasInput::new()`
2. Render with `Input::new(&self.input_state)` 
3. Subscribe to `InputEvent` for value changes
4. The cursor blinking would be handled by the Input component itself

The current implementation is simpler but requires manual cursor blink handling.

---

## Testing

After implementing, test by:

1. Build: `cargo build`
2. Run: `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. Select any script/command and press `Cmd+K` to open actions
4. Select "Set Alias" action
5. Verify:
   - The cursor blinks at ~530ms intervals
   - When you close the dialog (Escape or Save), the main menu input regains focus
   - You can immediately start typing to filter scripts again

---

## Verification Gate

Before committing:
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

OUTPUT_FILE_PATH: expert-bundles/alias-cursor-blink-focus-restore.md
