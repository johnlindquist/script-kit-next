# Research: Terminal Command Bar Integration

Date: 2026-02-02

## 1. Files Investigated

- **src/render_prompts/term.rs** - has Cmd+K handler calling toggle_arg_actions()
- **src/term_prompt.rs** - TermPrompt struct with execute_action() for TerminalAction
- **src/terminal/mod.rs** - exports TerminalCommandBar, TerminalAction, TerminalCommandBarEvent
- **src/terminal/command_bar.rs** - TerminalAction enum, TerminalCommandItem, get_terminal_commands()
- **src/terminal/command_bar_ui.rs** - TerminalCommandBar GPUI component

## 2. Current Behavior

- Cmd+K in term.rs only shows SDK actions popup if `has_actions` is true
- When no SDK actions, Cmd+K does nothing
- TerminalCommandBar UI component exists but is not connected to TermPrompt

## 3. Proposed Solution

### a) Add to TermPrompt struct:
- `show_command_bar: bool` - controls visibility
- `command_bar_entity: Option<Entity<TerminalCommandBar>>` - the command bar entity

### b) Modify render_prompts/term.rs Cmd+K handler:
- When has_actions: use existing toggle_arg_actions()
- When NO actions: toggle the new terminal command bar via TermPrompt

### c) Add keyboard handling:
- When command bar is open, intercept Arrow keys, Enter, ESC
- Route to TerminalCommandBar methods (move_up, move_down, submit_selected, dismiss)

### d) Render command bar as overlay:
- Position similar to ActionsDialog overlay (top right)
- Use TerminalCommandBarEvent callback to handle command selection

### e) Execute actions:
- On SelectCommand event: call TermPrompt.execute_action()
- On Close event: hide command bar

## 4. Key Code References

### toggle_arg_actions() pattern
```rust
// From term.rs Cmd+K handler
if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
    this.toggle_arg_actions(cx, window);
    return;
}
```

### ActionsDialog overlay pattern (term.rs lines 155-175)
```rust
.when_some(
    if self.show_actions_popup {
        self.actions_dialog.clone()
    } else {
        None
    },
    |d, dialog| {
        d.child(
            div()
                .absolute()
                .inset_0()
                .child(backdrop)
                .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
        )
    },
)
```

### TermPrompt.execute_action() signature
```rust
pub fn execute_action(&mut self, action: TerminalAction, cx: &mut Context<Self>)
```

## 5. Implementation Plan

1. Add `show_command_bar` and `command_bar` fields to TermPrompt
2. Create toggle_command_bar(), open_command_bar(), close_command_bar() methods
3. Modify Cmd+K handler in term.rs to check has_actions first
4. Handle keyboard in TermPrompt when command bar is open
5. Render command bar overlay in TermPrompt's Render impl
6. Wire up TerminalCommandBarEvent callback to execute_action()

## 6. Verification Results

Date: 2026-02-02

### Changes Made

1. **src/actions/types.rs**
   - Added `Terminal` variant to `ActionCategory` enum (line 455)
   - Added `with_shortcut_opt()` method to `Action` struct (line 496)

2. **src/app_impl.rs**
   - Added `toggle_terminal_commands()` method (line 3569-3638)
   - Creates terminal commands from `get_terminal_commands()`
   - Converts them to Actions with Terminal category
   - Opens ActionsDialog with terminal-style config

3. **src/render_prompts/term.rs**
   - Modified Cmd+K handler (lines 75-84):
     - When SDK actions exist: calls `toggle_arg_actions()`
     - When NO SDK actions: calls `toggle_terminal_commands()`
   - Modified `ActionsRoute::Execute` handler (lines 97-130):
     - Maps action IDs to `TerminalAction` enum variants
     - Executes via `TermPrompt.execute_action()`
     - Closes dialog after execution

4. **src/terminal/mod.rs**
   - Removed unused exports (TerminalCommandItem, TerminalCommandBar, etc.)

### Test Results

- `cargo check` - PASS
- `cargo clippy --all-targets -- -D warnings` - PASS  
- `cargo test` - PASS (20 passed, 58 ignored)

### How It Works

1. User is in terminal view (TermPrompt or QuickTerminalView)
2. User presses Cmd+K
3. If SDK actions exist, shows SDK actions popup
4. If no SDK actions, shows terminal command bar with built-in actions
5. User selects an action (navigate with arrows, select with Enter)
6. The action is executed on the terminal:
   - Clear, Copy, Paste, Select All
   - Scroll operations
   - Signal commands (Interrupt, Kill, Suspend, etc.)
7. Dialog closes after execution

### Files Changed Summary

| File | Changes |
|------|---------|
| src/actions/types.rs | +Terminal category, +with_shortcut_opt() |
| src/app_impl.rs | +toggle_terminal_commands() method |
| src/render_prompts/term.rs | Modified Cmd+K and Execute handlers |
| src/terminal/mod.rs | Removed unused exports |
