//! Terminal Command Bar UI
//!
//! A Cmd+K command bar component for the terminal, providing quick access to
//! terminal actions like clear, scroll, copy, paste, etc.
//!
//! This follows the patterns established by ActionsDialog in src/actions/dialog.rs.
//! Uses types from command_bar.rs (TerminalAction, TerminalCommandItem).

use std::sync::Arc;

use gpui::{Context, FocusHandle};

use crate::theme;

use super::command_bar::{get_terminal_commands, TerminalAction, TerminalCommandItem};

mod render;
#[cfg(test)]
mod tests;

/// Width of the command bar popup
const COMMAND_BAR_WIDTH: f32 = 320.0;

/// Height of each command item row
const COMMAND_ITEM_HEIGHT: f32 = 44.0;

/// Height of the search input area
const SEARCH_INPUT_HEIGHT: f32 = 40.0;

/// Maximum height of the popup (before scrolling)
const COMMAND_BAR_MAX_HEIGHT: f32 = 400.0;

/// Border radius for the popup
const POPUP_RADIUS: f32 = 12.0;

/// Horizontal padding for items
const ITEM_PADDING_X: f32 = 16.0;

/// Minimum keycap width for shortcut display
const KEYCAP_MIN_WIDTH: f32 = 24.0;

/// Keycap height
const KEYCAP_HEIGHT: f32 = 22.0;

/// Events emitted by the command bar
#[derive(Debug, Clone)]
pub enum TerminalCommandBarEvent {
    /// A command was selected
    SelectCommand(TerminalAction),
    /// The dialog was dismissed (ESC pressed)
    Close,
}

/// Callback type for command bar events
pub type CommandBarCallback = Arc<dyn Fn(TerminalCommandBarEvent) + Send + Sync>;

/// A floating command bar for terminal actions
pub struct TerminalCommandBar {
    /// Available commands
    commands: Vec<TerminalCommandItem>,
    /// Indices of commands matching current search
    filtered_indices: Vec<usize>,
    /// Current selection index (in filtered_indices)
    selected_index: usize,
    /// Current search text
    search_text: String,
    /// Focus handle for keyboard events
    focus_handle: FocusHandle,
    /// Theme for styling
    theme: Arc<theme::Theme>,
    /// Callback for events
    on_event: CommandBarCallback,
    /// Cursor blink state (controlled externally)
    cursor_visible: bool,
}

impl TerminalCommandBar {
    /// Create a new command bar with default terminal commands
    pub fn new(
        focus_handle: FocusHandle,
        theme: Arc<theme::Theme>,
        on_event: impl Fn(TerminalCommandBarEvent) + Send + Sync + 'static,
    ) -> Self {
        let commands = get_terminal_commands();
        let filtered_indices: Vec<usize> = (0..commands.len()).collect();

        Self {
            commands,
            filtered_indices,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            theme,
            on_event: Arc::new(on_event),
            cursor_visible: true,
        }
    }

    /// Create a command bar with custom commands
    pub fn with_commands(
        focus_handle: FocusHandle,
        theme: Arc<theme::Theme>,
        commands: Vec<TerminalCommandItem>,
        on_event: impl Fn(TerminalCommandBarEvent) + Send + Sync + 'static,
    ) -> Self {
        let filtered_indices: Vec<usize> = (0..commands.len()).collect();

        Self {
            commands,
            filtered_indices,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            theme,
            on_event: Arc::new(on_event),
            cursor_visible: true,
        }
    }

    /// Update the theme
    pub fn update_theme(&mut self, theme: Arc<theme::Theme>) {
        self.theme = theme;
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Get the currently selected command
    pub fn get_selected_command(&self) -> Option<&TerminalCommandItem> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.commands.get(idx))
    }

    /// Handle character input for search
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        if !Self::should_accept_search_char(ch) {
            return;
        }

        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace for search
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_indices.len().saturating_sub(1) {
            self.selected_index += 1;
            cx.notify();
        }
    }

    /// Submit the currently selected command
    pub fn submit_selected(&mut self, _cx: &mut Context<Self>) {
        if let Some(cmd) = self.get_selected_command() {
            let action = cmd.action.clone();
            (self.on_event)(TerminalCommandBarEvent::SelectCommand(action));
        }
    }

    /// Dismiss the command bar
    pub fn dismiss(&mut self, _cx: &mut Context<Self>) {
        (self.on_event)(TerminalCommandBarEvent::Close);
    }

    /// Refilter commands based on search text
    fn refilter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_indices = (0..self.commands.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_indices = self
                .commands
                .iter()
                .enumerate()
                .filter(|(_, cmd)| cmd.matches(&search_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        self.selected_index = 0;
    }

    /// Returns whether a character should be accepted as search input.
    pub(super) fn should_accept_search_char(ch: char) -> bool {
        !ch.is_control()
    }

    /// Computes list viewport height based on current filtered item count.
    pub(super) fn command_list_height(item_count: usize) -> f32 {
        let rows = item_count.max(1) as f32;
        (rows * COMMAND_ITEM_HEIGHT).min(COMMAND_BAR_MAX_HEIGHT - SEARCH_INPUT_HEIGHT)
    }
}
