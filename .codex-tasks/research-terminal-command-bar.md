# Research: Terminal Command Bar Patterns + Proposal

Date: 2026-02-02

## 1) Existing patterns in actions system

### src/actions/types.rs
- Action struct fields:
  - id: String
  - title: String
  - description: Option<String>
  - category: ActionCategory
  - shortcut: Option<String>
  - icon: Option<IconName>
  - section: Option<String>
  - has_action: bool
  - value: Option<String>
- Cached lowercase fields for filtering (precomputed at creation):
  - title_lower: String
  - description_lower: Option<String>
  - shortcut_lower: Option<String>
- Construction helpers:
  - Action::new(...) computes title_lower, description_lower
  - Action::with_shortcut(...) sets shortcut + shortcut_lower
  - Action::with_icon(...), Action::with_section(...)
- Routing: has_action=false (built-in handled locally), has_action=true (send ActionTriggered to SDK)
- ActionsDialogConfig and enums:
  - SearchPosition: Top | Bottom | Hidden
  - SectionStyle: Headers | Separators | None
  - AnchorPosition: Top | Bottom
  - ActionsDialogConfig: search_position, section_style, anchor, show_icons, show_footer

### src/actions/dialog.rs
- ActionsDialog state:
  - actions: Vec<Action>
  - filtered_actions: Vec<usize> (indices into actions)
  - grouped_items: Vec<GroupedActionItem> (SectionHeader | Item(filter_idx))
  - selected_index: usize (index into grouped_items, NOT filtered_actions)
  - search_text, list_state, focus_handle, theme, config, etc.
- Grouping behavior:
  - build_grouped_items_static(...):
    - SectionStyle::Headers => insert SectionHeader when section changes
    - SectionStyle::Separators or None => no headers, but category change tracked for rendering
  - coerce_action_selection(...) skips headers when selecting or after refilter
- Filtering behavior:
  - refilter() scores actions using cached lowercase fields
  - Scoring:
    - title prefix: +100
    - title contains: +50
    - title fuzzy subsequence: +25
    - description contains: +15
    - shortcut contains: +10
  - Results sorted by score desc
  - Selection is preserved by action id when possible
- Selection/navigation:
  - move_up/move_down skip headers via grouped_items
  - get_selected_action() maps grouped_items -> filtered_actions -> actions
- Search input:
  - handle_char/backspace update search_text, refilter(), cx.notify()
  - search position controlled by ActionsDialogConfig
- SDK action conversion:
  - set_sdk_actions converts ProtocolAction to Action and precomputes lowercase fields
  - clear_sdk_actions restores built-in actions

### src/actions/command_bar.rs
- CommandBar wraps ActionsDialog with window management (open/close/toggle)
- Holds actions, dialog entity, theme, and callbacks
- Config presets (CommandBarConfig):
  - main_menu_style(): Bottom search, separators, anchor bottom, no icons/footer
  - ai_style(): Top search, headers, anchor top, icons+footer
  - no_search(): Hidden search, separators, anchor bottom
  - notes_style(): Top search, separators, anchor top, icons+footer
- Updates:
  - set_actions(...) resets dialog actions + filtered state and resizes window if open
  - set_theme(...) updates dialog theme and notifies window

## 2) Terminal module structure (src/terminal/)

### mod.rs
- Re-exports: TerminalHandle, TerminalContent, CellAttributes
- Defines TerminalEvent (Output, Bell, Title, Exit)
- Describes architecture: PTY manager + terminal handle + theme adapter

### alacritty.rs
- Wraps alacritty_terminal for terminal emulation
- Core pieces:
  - EventProxy: implements EventListener, collects TerminalEvent in Arc<Mutex<Vec<_>>>
  - TerminalHandle: owns terminal state + PTY manager + theme adapter
  - TerminalContent: snapshot for rendering (lines, styled_lines, cursor, selected_cells)
- Important behaviors:
  - create_internal(): sets up PTY, Term config, background reader thread
  - process(): consumes PTY output channel, parses bytes, returns (had_output, events)
  - input(): write bytes to PTY
  - resize(): resizes PTY + terminal grid
  - scroll / scroll_page_up / scroll_page_down / scroll_to_top / scroll_to_bottom
  - selection APIs: start_selection, start_semantic_selection, start_line_selection,
    update_selection, selection_to_string, clear_selection
  - is_bracketed_paste_mode / is_application_cursor_mode
  - update_theme / update_focus

### pty.rs
- Portable PTY wrapper using portable-pty
- Responsibilities:
  - spawn shell/command with env (TERM, COLORTERM, PATH, etc.)
  - size management (resize, size)
  - IO (read, write, write_all, flush)
  - lifecycle (is_running, wait, kill)
  - take_reader() for background threads

### theme_adapter.rs
- Maps Script Kit theme to Alacritty colors
- Provides 16-color ANSI palette + fg/bg/cursor/selection
- Supports focus dimming
- ThemeAdapter::from_theme(...) pulls from theme colors (text, background, accent, terminal.*)

## 3) Proposed design: TerminalAction + TerminalCommandItem

Goal: mirror actions system patterns (cached lowercase fields, simple construction helpers) while
remaining terminal-focused.

### TerminalAction enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalAction {
    Clear,
    Copy,
    Paste,
    SelectAll,
    ScrollToTop,
    ScrollToBottom,
    Find,
    Kill,
    Suspend,
    Quit,
    SendEOF,
    Reset,
    NewShell,
}
```

### TerminalCommandItem struct
```rust
#[derive(Debug, Clone)]
pub struct TerminalCommandItem {
    pub name: String,
    pub description: Option<String>,
    pub shortcut: Option<String>,
    pub action: TerminalAction,
    // Cached lowercase fields for filtering
    pub name_lower: String,
    pub description_lower: Option<String>,
    pub shortcut_lower: Option<String>,
}

impl TerminalCommandItem {
    pub fn new(
        name: impl Into<String>,
        description: Option<String>,
        action: TerminalAction,
    ) -> Self {
        let name = name.into();
        let name_lower = name.to_lowercase();
        let description_lower = description.as_ref().map(|d| d.to_lowercase());
        Self {
            name,
            description,
            shortcut: None,
            action,
            name_lower,
            description_lower,
            shortcut_lower: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        let shortcut = shortcut.into();
        self.shortcut_lower = Some(shortcut.to_lowercase());
        self.shortcut = Some(shortcut);
        self
    }
}
```

### Notes on behavior mapping (for later implementation)
- Clear: send clear-screen escape sequence (e.g., "\x1b[2J\x1b[H")
- Copy: use TerminalHandle::selection_to_string()
- Paste: write clipboard text to PTY (wrap if bracketed paste is enabled)
- SelectAll: create selection range covering visible grid
- ScrollToTop / ScrollToBottom: TerminalHandle::{scroll_to_top, scroll_to_bottom}
- Find: open/find UI (no backend change)
- Kill: PtyManager::kill()
- Suspend: send Ctrl-Z ("\x1a")
- Quit: send "exit\n" or close session
- SendEOF: send Ctrl-D ("\x04")
- Reset: send full reset ("\x1bc")
- NewShell: spawn a new TerminalHandle/PTY session
