// CommandBar - Reusable command palette component
//
// A high-level wrapper around ActionsDialog that provides:
// - Consistent window management (open/close/toggle)
// - Focus handling
// - Keyboard routing
// - Configuration presets for different contexts (main menu, AI chat, etc.)
//
// # Usage
//
// ```rust,ignore
// // Create a CommandBar with actions and config
// let command_bar = CommandBar::new(
//     actions,
//     CommandBarConfig::ai_style(),
//     theme,
//     cx,
// );
//
// // Toggle with Cmd+K
// command_bar.toggle(window, cx);
//
// // Handle selected action
// if let Some(action_id) = command_bar.get_selected_action_id(cx) {
//     execute_action(&action_id);
// }
// ```

use super::dialog::GroupedActionItem;
use super::types::{Action, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle};
use super::window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window,
};
use super::ActionsDialog;
use crate::logging;
use crate::theme;
use crate::ui_foundation::{is_key_backspace, is_key_down, is_key_enter, is_key_escape, is_key_up};
use gpui::{App, AppContext, Context, Entity, FocusHandle, Window};
use std::sync::Arc;

const COMMAND_BAR_PAGE_JUMP: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandBarKeyIntent {
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
    MovePageUp,
    MovePageDown,
    ExecuteSelected,
    Close,
    Backspace,
    TypeChar(char),
}

#[inline]
fn is_non_text_named_key(key: &str) -> bool {
    key.eq_ignore_ascii_case("tab")
        || key.eq_ignore_ascii_case("left")
        || key.eq_ignore_ascii_case("arrowleft")
        || key.eq_ignore_ascii_case("right")
        || key.eq_ignore_ascii_case("arrowright")
        || key.eq_ignore_ascii_case("shift")
        || key.eq_ignore_ascii_case("control")
        || key.eq_ignore_ascii_case("alt")
        || key.eq_ignore_ascii_case("meta")
        || key.eq_ignore_ascii_case("cmd")
        || key.eq_ignore_ascii_case("command")
        || key.eq_ignore_ascii_case("capslock")
        || key.eq_ignore_ascii_case("numlock")
        || key.eq_ignore_ascii_case("scrolllock")
}

#[inline]
fn command_bar_key_intent(key: &str, modifiers: &gpui::Modifiers) -> Option<CommandBarKeyIntent> {
    if is_key_up(key) {
        return Some(CommandBarKeyIntent::MoveUp);
    }
    if is_key_down(key) {
        return Some(CommandBarKeyIntent::MoveDown);
    }
    if key.eq_ignore_ascii_case("home") {
        return Some(CommandBarKeyIntent::MoveHome);
    }
    if key.eq_ignore_ascii_case("end") {
        return Some(CommandBarKeyIntent::MoveEnd);
    }
    if key.eq_ignore_ascii_case("pageup") {
        return Some(CommandBarKeyIntent::MovePageUp);
    }
    if key.eq_ignore_ascii_case("pagedown") {
        return Some(CommandBarKeyIntent::MovePageDown);
    }
    if is_key_enter(key) {
        return Some(CommandBarKeyIntent::ExecuteSelected);
    }
    if is_key_escape(key) {
        return Some(CommandBarKeyIntent::Close);
    }
    if is_key_backspace(key) || key.eq_ignore_ascii_case("delete") {
        return Some(CommandBarKeyIntent::Backspace);
    }
    if key.eq_ignore_ascii_case("space") {
        return Some(CommandBarKeyIntent::TypeChar(' '));
    }
    if is_non_text_named_key(key) {
        return None;
    }

    if !modifiers.platform && !modifiers.control && !modifiers.alt {
        if let Some(ch) = key.chars().next() {
            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                return Some(CommandBarKeyIntent::TypeChar(ch));
            }
        }
    }

    None
}

#[inline]
fn is_selectable_row(row: &GroupedActionItem) -> bool {
    matches!(row, GroupedActionItem::Item(_))
}

fn first_selectable_index(rows: &[GroupedActionItem]) -> Option<usize> {
    rows.iter().position(is_selectable_row)
}

fn last_selectable_index(rows: &[GroupedActionItem]) -> Option<usize> {
    rows.iter().rposition(is_selectable_row)
}

fn selectable_index_at_or_before(rows: &[GroupedActionItem], start: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let clamped = start.min(rows.len() - 1);
    (0..=clamped).rev().find(|&ix| is_selectable_row(&rows[ix]))
}

fn selectable_index_at_or_after(rows: &[GroupedActionItem], start: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let clamped = start.min(rows.len() - 1);
    (clamped..rows.len()).find(|&ix| is_selectable_row(&rows[ix]))
}

/// Configuration presets for common CommandBar use cases
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields used by consumers
pub struct CommandBarConfig {
    /// ActionsDialog configuration
    pub dialog_config: ActionsDialogConfig,
    /// Whether to close when an action is selected (default: true)
    pub close_on_select: bool,
    /// Whether to close when clicking outside (default: true)
    pub close_on_click_outside: bool,
    /// Whether to close on Escape key (default: true)
    pub close_on_escape: bool,
}

impl Default for CommandBarConfig {
    fn default() -> Self {
        Self {
            dialog_config: ActionsDialogConfig::default(),
            close_on_select: true,
            close_on_click_outside: true,
            close_on_escape: true,
        }
    }
}

#[allow(dead_code)] // Public API - methods for future main menu and other integrations
impl CommandBarConfig {
    /// Create config for main menu style (search at bottom, separators)
    pub fn main_menu_style() -> Self {
        Self {
            dialog_config: ActionsDialogConfig {
                search_position: SearchPosition::Bottom,
                section_style: SectionStyle::Separators,
                anchor: AnchorPosition::Bottom,
                show_icons: false,
                show_footer: false,
            },
            ..Default::default()
        }
    }

    /// Create config for AI chat style (search at top, headers, icons)
    pub fn ai_style() -> Self {
        Self {
            dialog_config: ActionsDialogConfig {
                search_position: SearchPosition::Top,
                section_style: SectionStyle::Headers,
                anchor: AnchorPosition::Top,
                show_icons: true,
                show_footer: true,
            },
            ..Default::default()
        }
    }

    /// Create config with search hidden (external search handling)
    pub fn no_search() -> Self {
        Self {
            dialog_config: ActionsDialogConfig {
                search_position: SearchPosition::Hidden,
                section_style: SectionStyle::Separators,
                anchor: AnchorPosition::Bottom,
                show_icons: false,
                show_footer: false,
            },
            ..Default::default()
        }
    }

    /// Create config for Notes window style (search at top, separators, icons)
    /// Uses SectionStyle::Separators to match main menu's denser item spacing
    pub fn notes_style() -> Self {
        Self {
            dialog_config: ActionsDialogConfig {
                search_position: SearchPosition::Top,
                section_style: SectionStyle::Separators,
                anchor: AnchorPosition::Top,
                show_icons: true,
                show_footer: true,
            },
            ..Default::default()
        }
    }
}

/// Callback type for action execution
#[allow(dead_code)] // Public API type
pub type CommandBarActionCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// CommandBar - A reusable command palette component
///
/// Provides a high-level API for creating Raycast-style command menus.
/// Wraps ActionsDialog with window management and focus handling.
#[allow(dead_code)] // Public API - many methods for future integrations
pub struct CommandBar {
    /// The underlying dialog entity
    dialog: Option<Entity<ActionsDialog>>,
    /// Actions for the command bar
    actions: Vec<Action>,
    /// Configuration
    pub config: CommandBarConfig,
    /// Theme for styling
    theme: Arc<theme::Theme>,
    /// Whether the command bar is currently visible
    is_open: bool,
    /// Callback when an action is selected
    on_action: Option<CommandBarActionCallback>,
}

#[cfg(test)]
mod command_bar_key_intent_tests {
    use super::*;

    #[test]
    fn test_command_bar_key_intent_does_not_treat_named_keys_as_typed_chars() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            command_bar_key_intent("space", &no_mods),
            Some(CommandBarKeyIntent::TypeChar(' '))
        );
        assert_eq!(command_bar_key_intent("tab", &no_mods), None);
        assert_eq!(command_bar_key_intent("arrowleft", &no_mods), None);
        assert_eq!(
            command_bar_key_intent("backspace", &no_mods),
            Some(CommandBarKeyIntent::Backspace)
        );
    }

    #[test]
    fn test_command_bar_key_intent_maps_required_key_variants_to_same_intents() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            command_bar_key_intent("up", &no_mods),
            Some(CommandBarKeyIntent::MoveUp)
        );
        assert_eq!(
            command_bar_key_intent("arrowup", &no_mods),
            Some(CommandBarKeyIntent::MoveUp)
        );

        assert_eq!(
            command_bar_key_intent("down", &no_mods),
            Some(CommandBarKeyIntent::MoveDown)
        );
        assert_eq!(
            command_bar_key_intent("arrowdown", &no_mods),
            Some(CommandBarKeyIntent::MoveDown)
        );

        assert_eq!(
            command_bar_key_intent("enter", &no_mods),
            Some(CommandBarKeyIntent::ExecuteSelected)
        );
        assert_eq!(
            command_bar_key_intent("Enter", &no_mods),
            Some(CommandBarKeyIntent::ExecuteSelected)
        );

        assert_eq!(
            command_bar_key_intent("escape", &no_mods),
            Some(CommandBarKeyIntent::Close)
        );
        assert_eq!(
            command_bar_key_intent("Escape", &no_mods),
            Some(CommandBarKeyIntent::Close)
        );
    }
}
