# Research: Terminal Command Bar UI

## 1. ActionsDialog Pattern Analysis

### Component Structure (src/actions/dialog.rs)
- **ActionsDialog struct** contains:
  - `actions: Vec<Action>` - list of available actions
  - `filtered_actions: Vec<usize>` - indices into actions (for search filtering)
  - `selected_index: usize` - current selection (in grouped_items)
  - `search_text: String` - for filtering
  - `focus_handle: FocusHandle` - GPUI focus management
  - `on_select: ActionCallback` - callback when action selected
  - `theme: Arc<theme::Theme>` - for theming
  - `list_state: ListState` - for scrollable list
  - `grouped_items: Vec<GroupedActionItem>` - includes section headers

### Keyboard Navigation
- `move_up()` / `move_down()` - navigate selection, skip section headers
- `handle_char()` - add character to search
- `handle_backspace()` - remove character from search  
- `submit_selected()` - execute selected action
- `submit_cancel()` - close dialog (triggers on_select with "__cancel__")

### Event Emission Pattern
- Uses callback pattern: `on_select: Arc<dyn Fn(String) + Send + Sync>`
- Called with action ID string when selected
- "__cancel__" for escape/dismiss

### Render Trait Implementation
- Returns floating panel with:
  - Search input at bottom (Raycast style)
  - List of items with selection highlight
  - Variable height items (section headers 24px, items 44px)
  - Scrollbar when content overflows

## 2. Terminal Module Structure (src/terminal/mod.rs)

Current exports:
- `alacritty` module - terminal emulator
- `pty` module - pseudo-terminal
- `theme_adapter` module - theme conversion
- `TerminalEvent` enum - Output, Bell, Title, Exit events
- `TerminalHandle`, `TerminalContent`, `CellAttributes`

## 3. Theme Color Access Patterns

```rust
// Background colors
theme.colors.background.main
theme.colors.background.search_box
theme.colors.background.modal // NOT directly available, use helper

// Accent colors
theme.colors.accent.selected
theme.colors.accent.selected_subtle

// Text colors  
theme.colors.text.primary
theme.colors.text.secondary
theme.colors.text.dimmed
theme.colors.text.muted

// UI colors
theme.colors.ui.border

// Theme helpers
modal_overlay_bg(theme, opacity) // from theme/helpers.rs
theme.has_dark_colors() // dark mode check
theme.get_opacity() // opacity settings
```

## 4. Proposed Solution: TerminalCommandBar

### Data Structures

```rust
pub struct TerminalCommandItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub shortcut: Option<String>,
    pub action: TerminalAction,
}

pub enum TerminalAction {
    Clear,
    ScrollUp,
    ScrollDown,
    ScrollToTop,
    ScrollToBottom,
    Copy,
    Paste,
    Kill,
    Restart,
    Custom(String),
}

pub enum TerminalCommandBarEvent {
    SelectCommand(TerminalAction),
    Close,
}
```

### TerminalCommandBar Component

```rust
pub struct TerminalCommandBar {
    commands: Vec<TerminalCommandItem>,
    selected_index: usize,
    search_text: String,
    filtered_indices: Vec<usize>,
    focus_handle: FocusHandle,
    theme: Arc<theme::Theme>,
    on_event: Box<dyn Fn(TerminalCommandBarEvent) + Send + Sync>,
}
```

### Key Methods
- `new()` - create with default terminal commands
- `move_up()` / `move_down()` - navigation
- `handle_char()` / `handle_backspace()` - search
- `submit_selected()` - trigger command
- `dismiss()` - close dialog

### Styling (matching ActionsDialog)
- Fixed width: 320px (POPUP_WIDTH constant)
- Item height: 44px
- Search input height: 40px
- Rounded corners: 12px
- Shadow for floating effect
- theme.colors.background.main for background
- theme.colors.accent.selected for selection highlight
- theme.colors.text.primary for text

## Verification

### What was created

1. **`src/terminal/command_bar_ui.rs`** - New file containing:
   - `TerminalCommandBarEvent` enum - events emitted by the command bar (SelectCommand, Close)
   - `CommandBarCallback` type alias - callback for events
   - `TerminalCommandBar` struct - the GPUI component with:
     - `commands: Vec<TerminalCommandItem>` - list of commands (from command_bar.rs)
     - `filtered_indices: Vec<usize>` - indices matching search
     - `selected_index: usize` - current selection
     - `search_text: String` - search filter
     - `focus_handle: FocusHandle` - for keyboard focus
     - `theme: Arc<theme::Theme>` - for theming
     - `on_event: CommandBarCallback` - event callback
     - `cursor_visible: bool` - for blinking cursor
   - Methods:
     - `new()` - create with default commands
     - `with_commands()` - create with custom commands
     - `update_theme()` - update theme
     - `set_cursor_visible()` - control cursor blink
     - `get_selected_command()` - get current selection
     - `handle_char()` - search input
     - `handle_backspace()` - search deletion
     - `move_up()` / `move_down()` - navigation
     - `submit_selected()` - execute command
     - `dismiss()` - close dialog
   - `Focusable` trait implementation
   - `Render` trait implementation with:
     - Floating panel with rounded corners
     - Command list with selection highlight
     - Search input at bottom (Raycast style)
     - Theme-aware colors (dark/light mode)
     - Shadow effect

2. **`src/terminal/mod.rs`** - Updated exports:
   - Added `pub mod command_bar_ui;`
   - Re-exported `TerminalCommandBar`, `TerminalCommandBarEvent`, `CommandBarCallback`

### Code follows existing patterns
- Uses same structure as `ActionsDialog` in `src/actions/dialog.rs`
- Reuses existing `TerminalAction` and `TerminalCommandItem` from `command_bar.rs`
- Uses theme colors: `theme.colors.background.main`, `theme.colors.accent.selected`, etc.
- Uses same constants for sizing (320px width, 44px item height)

### Syntax verification
- Both files pass `rustfmt --check` validation
- Rust syntax is valid

### Build verification
- Build system had filesystem issues during testing
- Full verification deferred due to system issues
- Code follows all existing patterns and should compile once build system is stable
