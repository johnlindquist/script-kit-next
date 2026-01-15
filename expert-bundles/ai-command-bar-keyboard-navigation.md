# AI Command Bar Keyboard Navigation - Expert Debug Bundle

## Original Goal

> I think you just have a fundamental misunderstanding of how this works or something. Can you please create an expert bundle of all of our struggles and how we cannot get the up and down navigation to work so that I can take the bundle and hand it over to an expert to look into it.

This bundle documents the complete investigation of a keyboard navigation bug in the AI window's command bar (Cmd+K menu). Despite multiple fix attempts, **real arrow key presses do not move the selection** in the command bar list.

## Executive Summary

The AI window has a command bar (Cmd+K) that opens as a separate vibrancy popup window (`ActionsWindow`). The command bar displays 9 actions, but pressing up/down arrow keys does nothing. **The critical evidence from logs shows that arrow keys are never received by the `on_key_down` handler**, while other keys (k, escape, q, w) are received normally. This suggests the Input component (gpui_component's text field) is intercepting arrow keys before they reach the window's key handler.

### Key Problems:
1. **Arrow keys not reaching on_key_down handler**: Logs show `AI on_key_down: key='k'`, `key='escape'`, `key='q'`, but NEVER `key='down'` or `key='up'`
2. **Simulated keys work fine**: The `SimulateAiKey` stdin command successfully navigates (`select_next: index 0 -> 1`), proving the navigation code itself works
3. **Focus architecture is complex**: AI window has main `focus_handle` + Input component (gpui_component). When command bar opens, we focus `focus_handle`, but Input may still intercept arrow keys

### What We Tried (All Failed):
1. Reordering focus checks in render() - `needs_command_bar_focus` checked BEFORE `focus_input()`
2. Adding guard to skip `focus_input()` when command bar is open
3. Setting `focus: false` on ActionsWindow so it doesn't steal OS focus
4. Adding `skip_track_focus` flag to ActionsDialog
5. Focusing `focus_handle` in `show_command_bar()` before and after opening

### Required Fix Direction:
The Input component from gpui_component is likely using its own key handler that intercepts arrow keys for text cursor navigation. Possible approaches:
1. **Blur the Input** when command bar opens (remove it from the focus hierarchy)
2. **Custom key interception** at a higher level before Input sees events
3. **Modify gpui_component Input** to expose a way to disable arrow key handling
4. **Use GPUI's action system** instead of on_key_down for arrow key routing

### Files Included:
- `src/ai/window.rs`: AI window with keyboard handling, command bar integration, focus management
- `src/actions/window.rs`: ActionsWindow - the separate vibrancy popup for command bar
- `src/actions/dialog.rs`: ActionsDialog - the list UI component with selection state
- `src/actions/command_bar.rs`: CommandBar wrapper that manages the ActionsWindow
- `src/actions/types.rs`: Action types and configuration

## Log Evidence (CRITICAL)

### Keys That ARE Received by AI on_key_down:
```json
{"message":"AI on_key_down: key='k' command_bar_open=false"}
{"message":"AI on_key_down: key='k' command_bar_open=true"}
{"message":"AI on_key_down: key='escape' command_bar_open=true"}
{"message":"AI on_key_down: key='q' command_bar_open=false"}
{"message":"AI on_key_down: key='w' command_bar_open=false"}
```

### Keys That Are NEVER Received (the problem):
- `key='down'` - NEVER seen from real keyboard
- `key='up'` - NEVER seen from real keyboard
- `key='arrowdown'` - NEVER seen from real keyboard
- `key='arrowup'` - NEVER seen from real keyboard

### Simulated Keys That DO Work:
```json
{"message":"Queued SimulateKey: key='down' - will process on next render"}
{"message":"SimulateKey: key='down' modifiers=[] command_bar_open=true"}
{"message":"SimulateKey: Down in command bar"}
{"message":"select_next called, dialog exists: true"}
{"message":"select_next: index 0 -> 1"}
{"message":"select_next: index 1 -> 2"}
```

This proves:
1. The navigation code (`command_bar_select_next`) works correctly
2. The ActionsDialog selection state updates properly
3. The problem is specifically that **real arrow key events are intercepted before reaching our handler**

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI Window (main)                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  focus_handle (main) ← We focus this when command bar     │  │
│  │                        opens, but it may not receive      │  │
│  │                        arrow keys from Input              │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Input (gpui_component) ← Text field for chat messages    │  │
│  │  - Has its own focus handling                             │  │
│  │  - Likely intercepts arrow keys for cursor movement       │  │
│  │  - Even when not focused, may capture keyboard events     │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  on_key_down handler:                                           │
│    if command_bar.is_open() {                                   │
│      match key { "down" => select_next(), ... }  ← NEVER CALLED │
│    }                                                            │
└─────────────────────────────────────────────────────────────────┘
           │
           │ Opens popup window
           ▼
┌─────────────────────────────────────────────────────────────────┐
│              ActionsWindow (separate vibrancy window)           │
│  - WindowOptions { focus: false } ← Does NOT take OS focus      │
│  - Renders ActionsDialog entity shared with AI window           │
│  - Has its own on_key_down, but may not receive events either   │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  ActionsDialog (shared Entity)                            │  │
│  │  - selected_index: usize                                  │  │
│  │  - move_up() / move_down() ← These work when called       │  │
│  │  - filtered_actions: Vec<usize>                           │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Keyboard Event Flow (What Should Happen vs What Actually Happens)

### Expected Flow:
```
User presses ↓
    → GPUI captures KeyDownEvent
    → AI window's on_key_down fires
    → Logs "AI on_key_down: key='down' command_bar_open=true"
    → Matches "down" | "arrowdown"
    → Calls command_bar_select_next()
    → Selection moves from 0 to 1
```

### Actual Flow:
```
User presses ↓
    → GPUI captures KeyDownEvent
    → Input component intercepts it (for text cursor movement?)
    → AI window's on_key_down NEVER FIRES
    → No log entry for arrow key
    → Selection stays at 0
```

---

## Code Bundle

The following files contain all relevant code for debugging this issue.

### File: src/ai/window.rs (keyboard handling sections)

```rust
// Line 370-375: State flags for focus management
    /// Flag to request input focus on next render.
    needs_focus_input: bool,

    /// Flag to request main focus_handle focus on next render (for command bar keyboard routing).
    needs_command_bar_focus: bool,

// Line 400-401: Command bar component
    command_bar: CommandBar,

// Line 557-558: Initialization
            needs_focus_input: false,
            needs_command_bar_focus: false,

// Line 909-933: show_command_bar - Opens command bar and sets focus
    fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Open the command bar (CommandBar handles window creation internally)
        self.command_bar.open(window, cx);

        // CRITICAL: Focus main focus_handle so keyboard events route to us
        // The ActionsWindow is a visual-only popup - it does NOT take keyboard focus.
        // macOS popup windows often don't receive keyboard events properly.
        // This also unfocuses the Input component which would otherwise consume arrow keys.
        self.focus_handle.focus(window, cx);

        // Request command bar focus on next render for keyboard routing
        // This ensures the focus persists even if something else tries to steal it
        self.needs_command_bar_focus = true;

        // Log focus state for debugging
        let main_focused = self.focus_handle.is_focused(window);
        crate::logging::log(
            "AI",
            &format!(
                "show_command_bar: AI window focus_handle focused={} (AI window routes keys to command bar)",
                main_focused
            ),
        );

        cx.notify();
    }

// Line 952-958: Navigation methods
    fn command_bar_select_prev(&mut self, cx: &mut Context<Self>) {
        self.command_bar.select_prev(cx);
    }

    fn command_bar_select_next(&mut self, cx: &mut Context<Self>) {
        self.command_bar.select_next(cx);
    }

// Line 994-1066: SimulateKey handler (THIS WORKS - proves navigation code is correct)
    fn handle_simulated_key(
        &mut self,
        key: &str,
        modifiers: &[String],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // ... logging ...

        // Handle command bar navigation when it's open
        if self.command_bar.is_open() {
            match key_lower.as_str() {
                "up" | "arrowup" => {
                    crate::logging::log("AI", "SimulateKey: Up in command bar");
                    self.command_bar_select_prev(cx);
                }
                "down" | "arrowdown" => {
                    crate::logging::log("AI", "SimulateKey: Down in command bar");
                    self.command_bar_select_next(cx);
                }
                // ...
            }
            return;
        }
    }

// Line 3347-3364: render() focus handling - CRITICAL SECTION
        // Process command bar focus request FIRST (set after vibrancy window opens)
        // This ensures keyboard events route to the window's key handler for command bar navigation
        // CRITICAL: Must check this BEFORE focus_input to prevent input from stealing focus
        if self.needs_command_bar_focus {
            self.needs_command_bar_focus = false;
            self.focus_handle.focus(window, cx);
            crate::logging::log("AI", "Applied command bar focus in render");
        }
        // Process focus request flag (set by open_ai_window when bringing existing window to front)
        // Check both the instance flag and the global atomic flag
        // SKIP if command bar is open - the main focus_handle should have focus for arrow key routing
        else if !self.command_bar.is_open()
            && (self.needs_focus_input
                || AI_FOCUS_REQUESTED.swap(false, std::sync::atomic::Ordering::SeqCst))
        {
            self.needs_focus_input = false;
            self.focus_input(window, cx);
        }

// Line 3441-3521: on_key_down handler - WHERE ARROW KEYS SHOULD BE HANDLED
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;

                // Debug: Log ALL key events to verify handler is firing
                crate::logging::log(
                    "AI",
                    &format!(
                        "AI on_key_down: key='{}' command_bar_open={}",
                        key,
                        this.command_bar.is_open()
                    ),
                );

                // ... enter key handling for setup mode ...

                // Handle command bar navigation when it's open
                // This routes all relevant keys to the CommandBar
                if this.command_bar.is_open() {
                    crate::logging::log(
                        "AI",
                        &format!("AI window on_key_down (command_bar open): key='{}'", key),
                    );
                    match key {
                        "up" | "arrowup" => {
                            crate::logging::log(
                                "AI",
                                "AI window: routing UP to command_bar_select_prev",
                            );
                            this.command_bar_select_prev(cx);
                            return;
                        }
                        "down" | "arrowdown" => {
                            crate::logging::log(
                                "AI",
                                "AI window: routing DOWN to command_bar_select_next",
                            );
                            this.command_bar_select_next(cx);
                            return;
                        }
                        "enter" | "return" => {
                            this.execute_command_bar_action(window, cx);
                            return;
                        }
                        "escape" => {
                            this.hide_command_bar(cx);
                            return;
                        }
                        "backspace" | "delete" => {
                            this.command_bar_handle_backspace(cx);
                            return;
                        }
                        _ => {
                            // Handle printable characters for search
                            // ...
                        }
                    }
                }
                // ... rest of key handling ...
            }))
```

### File: src/actions/window.rs (ActionsWindow - vibrancy popup)

```rust
//! Actions Window - Separate vibrancy window for actions panel
//!
//! The window is:
//! - Non-draggable (fixed position relative to main window)
//! - Positioned below the header, at the right edge of main window
//! - Auto-closes when app loses focus
//! - Shares the ActionsDialog entity with the main app for keyboard routing

pub struct ActionsWindow {
    /// The shared dialog entity (created by main app, rendered here)
    pub dialog: Entity<ActionsDialog>,
    /// Focus handle for this window (not actively used - main window keeps focus)
    pub focus_handle: FocusHandle,
}

impl Render for ActionsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Log focus state AND window focus state
        let is_focused = self.focus_handle.is_focused(window);
        let window_is_active = window.is_window_active();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "ActionsWindow render: focus_handle.is_focused={}, window_is_active={}",
                is_focused, window_is_active
            ),
        );

        // Ensure we have focus on each render
        if !is_focused {
            self.focus_handle.focus(window, cx);
        }

        // Key handler for the actions window
        // Since this is a separate window, it needs its own key handling
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();

            crate::logging::log(
                "ACTIONS",
                &format!("ActionsWindow on_key_down received: key='{}'", key),
            );

            match key {
                "up" | "arrowup" => {
                    crate::logging::log("ACTIONS", "ActionsWindow: handling UP arrow");
                    this.dialog.update(cx, |d, cx| d.move_up(cx));
                    cx.notify();
                }
                "down" | "arrowdown" => {
                    crate::logging::log("ACTIONS", "ActionsWindow: handling DOWN arrow");
                    this.dialog.update(cx, |d, cx| d.move_down(cx));
                    cx.notify();
                }
                // ... enter, escape, backspace ...
            }
        });

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(self.dialog.clone())
    }
}

pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
) -> anyhow::Result<WindowHandle<Root>> {
    // ...
    
    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        // DON'T take focus - let the parent AI window keep focus and route keys to us
        // macOS popup windows often don't receive keyboard events properly
        focus: false,  // <-- CRITICAL: We tried this to keep focus on AI window
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    // ...
}
```

### File: src/actions/dialog.rs (ActionsDialog - selection state)

```rust
pub struct ActionsDialog {
    pub actions: Vec<Action>,
    pub filtered_actions: Vec<usize>, // Indices into actions
    pub selected_index: usize,        // Index within filtered_actions
    pub search_text: String,
    pub focus_handle: FocusHandle,
    pub on_select: ActionCallback,
    // ...
    /// When true, skip track_focus in render (parent handles focus, e.g., ActionsWindow)
    pub skip_track_focus: bool,
}

impl ActionsDialog {
    // Line 1569-1579: move_up - Navigation method
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_actions.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    // Line 1581-1591: move_down - Navigation method
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_actions.is_empty()
            && self.selected_index < self.filtered_actions.len() - 1
        {
            self.selected_index += 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }
}
```

### File: src/actions/command_bar.rs (CommandBar wrapper)

```rust
pub struct CommandBar {
    /// The shared dialog entity
    dialog: Option<Entity<ActionsDialog>>,
    /// Actions to display
    actions: Vec<Action>,
    /// Configuration
    config: CommandBarConfig,
    /// Theme for styling
    theme: Arc<Theme>,
}

impl CommandBar {
    pub fn open(&mut self, window: &mut Window, cx: &mut App) {
        // Create the dialog entity if it doesn't exist
        let dialog = cx.new(|cx| {
            let mut dialog = ActionsDialog::with_config(
                cx.focus_handle(),
                Arc::new(|_| {}),
                self.actions.clone(),
                self.theme.clone(),
                self.config.to_dialog_config(),
            );
            // Skip track_focus - ActionsWindow handles focus
            dialog.set_skip_track_focus(true);
            dialog
        });
        self.dialog = Some(dialog.clone());

        // Get main window bounds for positioning
        let bounds = window.bounds();
        let display_id = window.display().map(|d| d.id());

        // Open the actions window
        if let Err(e) = open_actions_window(cx, bounds, display_id, dialog) {
            crate::logging::log("COMMAND_BAR", &format!("Failed to open actions window: {}", e));
        } else {
            crate::logging::log("COMMAND_BAR", "Command bar opened");
        }
    }

    pub fn select_next(&mut self, cx: &mut App) {
        crate::logging::log(
            "COMMAND_BAR",
            &format!("select_next called, dialog exists: {}", self.dialog.is_some()),
        );
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                let old_idx = d.selected_index;
                d.move_down(cx);
                crate::logging::log(
                    "COMMAND_BAR",
                    &format!("select_next: index {} -> {}", old_idx, d.selected_index),
                );
            });
            notify_actions_window(cx);
        }
    }

    pub fn select_prev(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.move_up(cx));
            notify_actions_window(cx);
        }
    }
}
```

---

## Implementation Guide

### Hypothesis: Input Component Intercepting Arrow Keys

The gpui_component Input is likely designed to handle arrow keys for cursor navigation within text. Even when we focus the main `focus_handle`, the Input may still be in the GPUI focus chain and intercepting events.

### Potential Fix Approaches

#### Approach 1: Blur the Input When Command Bar Opens

```rust
// In src/ai/window.rs, show_command_bar()
fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    // Blur the input to prevent it from capturing arrow keys
    self.input_state.update(cx, |state, cx| {
        state.blur(window, cx);  // If InputState has this method
    });
    
    self.command_bar.open(window, cx);
    self.focus_handle.focus(window, cx);
    self.needs_command_bar_focus = true;
    cx.notify();
}
```

#### Approach 2: Use GPUI Actions Instead of on_key_down

GPUI has an action system that might bypass the Input's key handling:

```rust
// Define actions
actions!(ai, [CommandBarUp, CommandBarDown]);

// Register bindings
cx.bind_keys([
    KeyBinding::new("up", CommandBarUp, Some("AiApp")),
    KeyBinding::new("down", CommandBarDown, Some("AiApp")),
]);

// Handle actions
impl AiApp {
    fn command_bar_up(&mut self, _: &CommandBarUp, cx: &mut Context<Self>) {
        if self.command_bar.is_open() {
            self.command_bar_select_prev(cx);
        }
    }
}
```

#### Approach 3: Override Input's Key Handling

If gpui_component Input exposes a way to disable internal key handling:

```rust
// Hypothetical API - needs verification
self.input_state.update(cx, |state, cx| {
    state.set_arrow_keys_enabled(false);  // Disable arrow key handling
});
```

#### Approach 4: Capture Keys at Window Level Before Input

Use GPUI's window-level key capture if available:

```rust
window.set_key_capture(Some(cx.listener(|this, event: &KeyDownEvent, cx| {
    if this.command_bar.is_open() {
        match event.keystroke.key.as_str() {
            "up" | "arrowup" | "down" | "arrowdown" => {
                // Handle here, prevent propagation
                return true;
            }
            _ => {}
        }
    }
    false  // Let event propagate
})));
```

### Debugging Steps

1. **Verify Input focus state**: Add logging to check if Input is focused when command bar is open
   ```rust
   let input_focused = self.input_state.read(cx).is_focused(window);
   crate::logging::log("AI", &format!("Input focused: {}", input_focused));
   ```

2. **Check GPUI focus hierarchy**: Log all focused elements in the window

3. **Test with Input hidden**: Temporarily hide/disable the Input when command bar opens to confirm it's the culprit

4. **Inspect gpui_component source**: Look at how Input handles KeyDownEvent

---

## Instructions for the Next AI Agent

### Your Task
Fix the keyboard navigation in the AI command bar so that pressing the physical up/down arrow keys moves the selection in the actions list.

### Key Insight
The navigation code is CORRECT. The problem is that **arrow key events never reach the on_key_down handler**. Simulated keys work perfectly, proving the issue is event routing, not navigation logic.

### What to Investigate First
1. Look at gpui_component's Input implementation - how does it handle keyboard events?
2. Check if GPUI has a focus/event priority system that Input is using
3. Determine if there's a way to blur/disable the Input's key handling temporarily

### What NOT to Do
- Don't modify the navigation logic (move_up/move_down) - it works
- Don't change the ActionsWindow focus settings again - we've tried that
- Don't add more logging to on_key_down for arrow keys - they never fire

### Test Command
```bash
# Build and run with logging
cargo build && echo '{"type":"openAi"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Then press Cmd+K to open command bar
# Then press down arrow - check logs for "AI on_key_down: key='down'"
# (Currently this log NEVER appears for arrow keys)

# Simulated key test (this WORKS):
echo '{"type":"simulateAiKey","key":"down"}' 
# Check logs for "select_next: index 0 -> 1"
```

### Success Criteria
1. Pressing physical down arrow key should log: `AI on_key_down: key='down' command_bar_open=true`
2. Selection in command bar should move from index 0 to 1
3. Pressing physical up arrow should move selection back up

### Files to Focus On
1. `src/ai/window.rs` - The AI window's event handling
2. gpui_component's Input source - Look for key event handling
3. GPUI's focus/event documentation

OUTPUT_FILE_PATH: expert-bundles/ai-command-bar-keyboard-navigation.md
