//! Terminal Command Bar UI
//!
//! A Cmd+K command bar component for the terminal, providing quick access to
//! terminal actions like clear, scroll, copy, paste, etc.
//!
//! This follows the patterns established by ActionsDialog in src/actions/dialog.rs.
//! Uses types from command_bar.rs (TerminalAction, TerminalCommandItem).

use crate::theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, BoxShadow, Context, ElementId, FocusHandle, Focusable,
    Hsla, Render, SharedString, Window,
};
use std::sync::Arc;

use super::command_bar::{get_terminal_commands, TerminalAction, TerminalCommandItem};

// =============================================================================
// Constants (matching ActionsDialog for visual consistency)
// =============================================================================

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

// =============================================================================
// Events and Callback Types
// =============================================================================

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

// =============================================================================
// TerminalCommandBar Component
// =============================================================================

/// A floating command bar for terminal actions
///
/// Provides a searchable list of terminal commands with keyboard navigation.
/// Triggered by Cmd+K in the terminal.
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

        // Reset selection to first item
        self.selected_index = 0;
    }

    /// Create box shadow for the popup
    fn create_popup_shadow(&self) -> Vec<BoxShadow> {
        let is_dark = self.theme.has_dark_colors();
        let shadow_color = if is_dark {
            rgba(0x00000080) // Black shadow for dark mode
        } else {
            rgba(0x00000040) // Lighter shadow for light mode
        };

        vec![
            BoxShadow {
                color: Hsla::from(shadow_color),
                offset: gpui::point(px(0.), px(4.)),
                blur_radius: px(16.),
                spread_radius: px(0.),
            },
            BoxShadow {
                color: Hsla::from(rgba(0x00000020)),
                offset: gpui::point(px(0.), px(2.)),
                blur_radius: px(8.),
                spread_radius: px(0.),
            },
        ]
    }

    /// Parse shortcut string into individual keycaps
    fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
        shortcut.chars().map(|c| c.to_string()).collect()
    }

    /// Render a single keycap
    fn render_keycap(&self, key: &str, is_dark: bool) -> impl IntoElement {
        let keycap_bg = if is_dark {
            rgba(0xffffff18) // White at low opacity for dark mode
        } else {
            rgba(0x00000010) // Black at low opacity for light mode
        };
        let keycap_text = if is_dark {
            rgb(self.theme.colors.text.dimmed)
        } else {
            rgba(0x666666FF)
        };
        let keycap_border = if is_dark {
            rgba(0xffffff20)
        } else {
            rgba(0x00000020)
        };

        div()
            .h(px(KEYCAP_HEIGHT))
            .min_w(px(KEYCAP_MIN_WIDTH))
            .px(px(6.))
            .flex()
            .items_center()
            .justify_center()
            .bg(keycap_bg)
            .border_1()
            .border_color(keycap_border)
            .rounded(px(4.))
            .text_xs()
            .text_color(keycap_text)
            .child(key.to_string())
    }

    /// Render a command item
    fn render_command_item(
        &self,
        idx: usize,
        cmd: &TerminalCommandItem,
        is_selected: bool,
    ) -> impl IntoElement {
        let is_dark = self.theme.has_dark_colors();

        // Selection background - theme-aware
        let selected_bg = if is_dark {
            let opacity = self.theme.get_opacity();
            let alpha = (opacity.selected * 255.0) as u32;
            rgba((self.theme.colors.accent.selected_subtle << 8) | alpha)
        } else {
            rgba(0xE8E8E8CC) // Light gray for light mode
        };

        let hover_bg = if is_dark {
            let opacity = self.theme.get_opacity();
            let alpha = (opacity.hover * 255.0) as u32;
            rgba((self.theme.colors.accent.selected_subtle << 8) | alpha)
        } else {
            rgba(0xE8E8E866)
        };

        // Text colors
        let primary_text = rgb(self.theme.colors.text.primary);
        let secondary_text = rgb(self.theme.colors.text.secondary);

        // Build shortcut keycaps if present
        let shortcut_element = cmd.shortcut.as_ref().map(|shortcut| {
            let keycaps = Self::parse_shortcut_keycaps(shortcut);
            div()
                .flex()
                .flex_row()
                .gap(px(2.))
                .children(keycaps.into_iter().map(|k| self.render_keycap(&k, is_dark)))
        });

        div()
            .id(ElementId::NamedInteger("cmd-item".into(), idx as u64))
            .h(px(COMMAND_ITEM_HEIGHT))
            .w_full()
            .px(px(ITEM_PADDING_X))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .rounded(px(8.))
            .mx(px(8.))
            .when(is_selected, |d| d.bg(selected_bg))
            .when(!is_selected, |d| d.hover(|d| d.bg(hover_bg)))
            .child(
                // Left side: name and description
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(primary_text)
                            .child(cmd.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(secondary_text)
                            .child(cmd.description.clone()),
                    ),
            )
            .when_some(shortcut_element, |d, shortcut| d.child(shortcut))
    }
}

impl Focusable for TerminalCommandBar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalCommandBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = self.theme.has_dark_colors();

        // Background color - use theme modal/main background
        let dialog_bg = if is_dark {
            let opacity = if self.theme.is_vibrancy_enabled() {
                0.50
            } else {
                0.95
            };
            let alpha = (opacity * 255.0) as u32;
            rgba((self.theme.colors.background.main << 8) | alpha)
        } else {
            rgba(0xFFFFFFE8) // Near-white for light mode
        };

        // Border color
        let border_color = if is_dark {
            rgba((self.theme.colors.ui.border << 8) | 0x80)
        } else {
            rgba(0xE0E0E0FF)
        };

        // Search input colors
        let hint_text_color = if is_dark {
            rgb(self.theme.colors.text.dimmed)
        } else {
            rgba(0x9B9B9BFF)
        };

        let input_text_color = if is_dark {
            rgb(self.theme.colors.text.primary)
        } else {
            rgba(0x1A1A1AFF)
        };

        let accent_color = rgb(self.theme.colors.accent.selected);

        // Search display text
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search commands...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Calculate content height
        let item_count = self.filtered_indices.len();
        let content_height = (item_count as f32 * COMMAND_ITEM_HEIGHT)
            .min(COMMAND_BAR_MAX_HEIGHT - SEARCH_INPUT_HEIGHT);

        // Render command items
        let items: Vec<_> = self
            .filtered_indices
            .iter()
            .enumerate()
            .filter_map(|(visual_idx, &cmd_idx)| {
                self.commands.get(cmd_idx).map(|cmd| {
                    let is_selected = visual_idx == self.selected_index;
                    self.render_command_item(visual_idx, cmd, is_selected)
                })
            })
            .collect();

        // Search input separator color
        let separator_color = if is_dark {
            border_color
        } else {
            rgba(0xE0E0E0FF)
        };

        // Build the popup
        div()
            .track_focus(&self.focus_handle)
            .w(px(COMMAND_BAR_WIDTH))
            .max_h(px(COMMAND_BAR_MAX_HEIGHT))
            .bg(dialog_bg)
            .border_1()
            .border_color(border_color)
            .rounded(px(POPUP_RADIUS))
            .shadow(self.create_popup_shadow())
            .flex()
            .flex_col()
            .overflow_hidden()
            // Command list
            .child(
                div()
                    .id("command-bar-list")
                    .h(px(content_height))
                    .overflow_y_scroll()
                    .py(px(8.))
                    .children(items),
            )
            // Search input at bottom (Raycast style)
            .child(
                div()
                    .h(px(SEARCH_INPUT_HEIGHT))
                    .w_full()
                    .px(px(ITEM_PADDING_X))
                    .border_t_1()
                    .border_color(separator_color)
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_sm()
                            .text_color(if self.search_text.is_empty() {
                                hint_text_color
                            } else {
                                input_text_color
                            })
                            // Cursor at start when empty
                            .when(self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .mr(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            })
                            .child(search_display)
                            // Cursor at end when has text
                            .when(!self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .ml(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            }),
                    ),
            )
            // Empty state
            .when(self.filtered_indices.is_empty(), |d| {
                d.child(
                    div()
                        .h(px(COMMAND_ITEM_HEIGHT))
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(hint_text_color)
                        .child("No commands match"),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shortcut_keycaps() {
        let keycaps = TerminalCommandBar::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);

        let keycaps = TerminalCommandBar::parse_shortcut_keycaps("⌃⇧T");
        assert_eq!(keycaps, vec!["⌃", "⇧", "T"]);
    }
}
