# Terminal Copy Enhancement Research

## Files Investigated

1. **src/term_prompt.rs** - TermPrompt component
   - Line ~681: Cmd+C handler that copies selection if exists via arboard crate, else sends SIGINT
   - Uses `self.terminal.selection_to_string()` to get selected text
   - Uses `self.terminal.content()` to get terminal content

2. **src/terminal/alacritty.rs** - TerminalHandle and TerminalContent
   - Line 672: `content()` method returns `TerminalContent`
   - `TerminalContent.lines` is `Vec<String>` with all visible terminal lines
   - Line 807: `selection_to_string()` returns selected text

3. **src/actions/builders.rs** - Action building patterns
   - `get_ai_command_bar_actions()` at line 1137 shows pattern for creating command bar actions
   - Actions use `Action::new()` with id, label, description, category
   - Actions can have `.with_shortcut()`, `.with_icon()`, `.with_section()`

4. **src/render_prompts/term.rs** - Terminal rendering
   - Shows how terminal prompt integrates with actions system
   - Uses `sdk_actions` for SDK-provided actions
   - Has `trigger_action_by_name()` for executing actions

## Current Behavior

- **Cmd+C with selection**: Copies selected text to clipboard via arboard
- **Cmd+C without selection**: Sends SIGINT (0x03) to terminal
- **Clipboard access**: Uses `arboard::Clipboard` crate
- **Terminal content**: Available via `self.terminal.content().lines` as `Vec<String>`

## Root Cause Analysis

No built-in terminal copy actions exist for:
- Copy All (all visible content)
- Copy Last Command (parse and copy last command entered)
- Copy Last Output (copy output between last command and current prompt)

These are commonly requested terminal features for quickly grabbing output.

## Proposed Solution

### 1. Add helper methods to TermPrompt

```rust
fn copy_all_content(&self) {
    let content = self.terminal.content();
    let all_text = content.lines.join("\n").trim_end().to_string();
    // Copy to clipboard using arboard
}

fn copy_last_command(&self) -> Option<String> {
    // Parse terminal content to find last command
    // Look for lines starting with prompt characters ($ or %)
    // Copy just the command portion
}

fn copy_last_output(&self) -> Option<String> {
    // Find last command line
    // Copy everything between last command and current prompt
}
```

### 2. Add get_terminal_commands() function

Following the pattern in builders.rs, create actions:
- "Copy Selection" (existing, Cmd+C)
- "Copy All" - copies all visible terminal content
- "Copy Last Command" - copies the last command entered
- "Copy Last Output" - copies output of last command

### 3. Add execute_action() method

Handle the new action IDs:
- `copy_all` -> `copy_all_content()`
- `copy_last_command` -> `copy_last_command()`
- `copy_last_output` -> `copy_last_output()`

## Implementation Details

- Use `arboard::Clipboard` for clipboard access (already in use)
- Parse terminal lines to identify prompt patterns ($ or %)
- Handle edge cases: empty terminal, no commands yet, multiple prompts
- Actions should be available via Cmd+K command bar

## Verification Plan

1. Run `cargo check` to verify compilation
2. Run `cargo clippy --all-targets -- -D warnings` for lints
3. Run `cargo test` for unit tests
4. Manual testing: verify copy actions work in terminal UI

## Verification

### Changes Made

1. **src/terminal/command_bar.rs**:
   - Added `CopyAll`, `CopyLastCommand`, `CopyLastOutput` variants to `TerminalAction` enum
   - Added ID mappings in `id()` method
   - Added shortcuts in `default_shortcut()` method: `⇧⌘C`, `⌥⌘C`, `⌃⌘C`
   - Added new commands to `get_terminal_commands()` function

2. **src/term_prompt.rs**:
   - Added handlers for `TerminalAction::CopyAll`, `TerminalAction::CopyLastCommand`, `TerminalAction::CopyLastOutput` in `execute_terminal_action()` method

### Implementation Details

- **CopyAll**: Joins all visible terminal lines with `\n` and copies to clipboard
- **CopyLastCommand**: Searches terminal content in reverse for prompt patterns (`$ `, `% `, `> `) and extracts the command portion
- **CopyLastOutput**: Finds prompt line indices, identifies the output region between the second-to-last and last prompt, and copies that text

### Build Status

Unable to verify with `cargo check` due to filesystem issues on the development system (temp directory creation failures). The code changes are syntactically correct as verified by manual inspection.

### Files Modified

- `/Users/johnlindquist/dev/script-kit-gpui/src/terminal/command_bar.rs`
- `/Users/johnlindquist/dev/script-kit-gpui/src/term_prompt.rs`
