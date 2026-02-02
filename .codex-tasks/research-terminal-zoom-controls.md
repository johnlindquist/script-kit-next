# Terminal Zoom Controls Research

## Files Investigated

1. `/Users/johnlindquist/dev/script-kit-gpui/src/term_prompt.rs` - TermPrompt struct, font_size() method
2. `/Users/johnlindquist/dev/script-kit-gpui/src/config/types.rs` - Config.terminal_font_size (serde: terminalFontSize)
3. `/Users/johnlindquist/dev/script-kit-gpui/src/config/defaults.rs` - DEFAULT_TERMINAL_FONT_SIZE = 14.0
4. `/Users/johnlindquist/dev/script-kit-gpui/src/terminal/command_bar.rs` - TerminalAction enum, get_terminal_commands()
5. `/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders.rs` - get_terminal_commands() for ActionsDialog

## Current Behavior

- `TermPrompt` stores `config: Arc<Config>` (line 75) - config is immutable
- `font_size()` method (line 179) returns `self.config.get_terminal_font_size()` - no override
- Cell dimensions scale from BASE_FONT_SIZE (14.0) using ratios:
  - `cell_width() = BASE_CELL_WIDTH * (font_size / BASE_FONT_SIZE)`
  - `cell_height() = font_size * LINE_HEIGHT_MULTIPLIER`
- Terminal resizing via `resize_if_needed()` recalculates dimensions

## Root Cause Analysis

- No `font_size_override` field exists in TermPrompt
- Font size is static from config - cannot change at runtime
- No zoom actions defined in TerminalAction enum
- No handlers for zoom in execute_action methods

## Proposed Solution

1. **Add field to TermPrompt struct** (line ~93):
   ```rust
   /// Font size override for zoom controls (None = use config default)
   font_size_override: Option<f32>,
   ```

2. **Initialize in constructors** (with_height around line 159):
   ```rust
   font_size_override: None,
   ```

3. **Modify font_size() method** (line 179):
   ```rust
   fn font_size(&self) -> f32 {
       self.font_size_override.unwrap_or_else(|| self.config.get_terminal_font_size())
   }
   ```

4. **Add zoom variants to TerminalAction enum** (command_bar.rs line ~118):
   ```rust
   /// Increase font size (zoom in)
   ZoomIn,
   /// Decrease font size (zoom out)
   ZoomOut,
   /// Reset font size to config default
   ResetZoom,
   ```

5. **Add id() cases** (command_bar.rs line ~168):
   ```rust
   TerminalAction::ZoomIn => "zoom_in",
   TerminalAction::ZoomOut => "zoom_out",
   TerminalAction::ResetZoom => "reset_zoom",
   ```

6. **Add default_shortcut() cases** (command_bar.rs line ~210):
   ```rust
   TerminalAction::ZoomIn => Some("⌘+"),
   TerminalAction::ZoomOut => Some("⌘-"),
   TerminalAction::ResetZoom => Some("⌘0"),
   ```

7. **Add to get_terminal_commands()** (command_bar.rs):
   ```rust
   // === Zoom Controls ===
   TerminalCommandItem::new(
       "Zoom In",
       "Increase font size",
       Some("⌘+"),
       TerminalAction::ZoomIn,
   ),
   TerminalCommandItem::new(
       "Zoom Out",
       "Decrease font size",
       Some("⌘-"),
       TerminalAction::ZoomOut,
   ),
   TerminalCommandItem::new(
       "Reset Zoom",
       "Reset font size to default",
       Some("⌘0"),
       TerminalAction::ResetZoom,
   ),
   ```

8. **Implement handlers in execute_action** (term_prompt.rs, both methods):

   For string-based execute_action:
   ```rust
   "zoom_in" => {
       let current = self.font_size();
       self.font_size_override = Some((current + 2.0).min(32.0));
       true
   }
   "zoom_out" => {
       let current = self.font_size();
       self.font_size_override = Some((current - 2.0).max(8.0));
       true
   }
   "reset_zoom" => {
       self.font_size_override = None;
       true
   }
   ```

9. **Trigger resize after zoom** - Since font size changes cell dimensions, need to call resize_if_needed() in the render path (already happens via cx.notify())

## Keyboard Shortcuts
- `Cmd+=` / `Cmd++` - Zoom In (increase by 2px, max 32px)
- `Cmd+-` - Zoom Out (decrease by 2px, min 8px)
- `Cmd+0` - Reset Zoom (back to config default)

## Verification

### Changes Made

1. **Added zoom action variants to TerminalAction enum** (`src/terminal/command_bar.rs` lines 125-136):
   - `ZoomIn` - Increase font size by 2px, max 32px
   - `ZoomOut` - Decrease font size by 2px, min 8px
   - `ResetZoom` - Reset to config default

2. **Added id() mappings** (`src/terminal/command_bar.rs` lines 178-180):
   - `ZoomIn` → "zoom_in"
   - `ZoomOut` → "zoom_out"
   - `ResetZoom` → "reset_zoom"

3. **Added default_shortcut() mappings** (`src/terminal/command_bar.rs` lines 219-221):
   - `ZoomIn` → "⌘+"
   - `ZoomOut` → "⌘-"
   - `ResetZoom` → "⌘0"

4. **Added zoom commands to get_terminal_commands()** (`src/terminal/command_bar.rs` lines 481-499):
   - "Zoom In" command with ⌘+ shortcut
   - "Zoom Out" command with ⌘- shortcut
   - "Reset Zoom" command with ⌘0 shortcut

5. **Added font_size_override field to TermPrompt** (`src/term_prompt.rs` line 100-101):
   ```rust
   font_size_override: Option<f32>,
   ```

6. **Initialized font_size_override in constructor** (`src/term_prompt.rs` line 180):
   ```rust
   font_size_override: None,
   ```

7. **Modified font_size() method** (`src/term_prompt.rs` lines 460-462):
   ```rust
   fn font_size(&self) -> f32 {
       self.font_size_override.unwrap_or_else(|| self.config.get_terminal_font_size())
   }
   ```

8. **Added zoom action handlers** (`src/term_prompt.rs` lines 448-463):
   - `ZoomIn`: Increases font size by 2px (max 32px), triggers cx.notify()
   - `ZoomOut`: Decreases font size by 2px (min 8px), triggers cx.notify()
   - `ResetZoom`: Clears font_size_override to use config default, triggers cx.notify()

9. **Added test assertions** (`src/terminal/command_bar.rs`):
   - ID tests for zoom actions
   - Shortcut tests for zoom actions
   - Non-signal tests for zoom actions

### Test Results

```
running 11 tests
test terminal::command_bar::tests::test_command_item_creation ... ok
test terminal::command_bar::tests::test_command_item_matches ... ok
test terminal::command_bar::tests::test_command_item_without_shortcut ... ok
test terminal::command_bar::tests::test_commands_have_descriptions ... ok
test terminal::command_bar::tests::test_commands_have_lowercase_cache ... ok
test terminal::command_bar::tests::test_get_terminal_commands ... ok
test terminal::command_bar::tests::test_terminal_action_display ... ok
test terminal::command_bar::tests::test_terminal_action_ids ... ok
test terminal::command_bar::tests::test_terminal_action_is_signal ... ok
test terminal::command_bar::tests::test_terminal_action_shortcuts ... ok
test terminal::command_bar_ui::tests::test_parse_shortcut_keycaps ... ok

test result: ok. 11 passed; 0 failed; 0 ignored
```

### Library Compilation

- `cargo check --lib` passes successfully
- Pre-existing errors in `app_impl.rs` (binary) are unrelated to these changes

### Behavior

- Font size changes are applied immediately via `cx.notify()` which triggers re-render
- Terminal cell dimensions automatically recalculate based on new font size (via `cell_width()` and `cell_height()` methods)
- Terminal resize happens automatically in the render path via `resize_if_needed()`
- Font size limits: 8px minimum, 32px maximum, 2px increments
