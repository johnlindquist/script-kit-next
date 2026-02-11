// --- merged from part_01.rs ---
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

// --- merged from part_02.rs ---
#[allow(dead_code)] // Public API - many methods for future integrations
impl CommandBar {
    #[inline]
    fn reset_open_state(&mut self) {
        self.is_open = false;
        self.dialog = None;
    }

    /// Create a new CommandBar with actions and configuration
    pub fn new(actions: Vec<Action>, config: CommandBarConfig, theme: Arc<theme::Theme>) -> Self {
        Self {
            dialog: None,
            actions,
            config,
            theme,
            is_open: false,
            on_action: None,
        }
    }

    /// Set the action callback
    pub fn with_on_action(mut self, callback: CommandBarActionCallback) -> Self {
        self.on_action = Some(callback);
        self
    }

    /// Set the action callback (mutable version)
    pub fn set_on_action(&mut self, callback: CommandBarActionCallback) {
        self.on_action = Some(callback);
    }

    /// Update the actions list
    pub fn set_actions(&mut self, actions: Vec<Action>, cx: &mut App) {
        self.actions = actions.clone();

        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                d.actions = actions;
                d.filtered_actions = (0..d.actions.len()).collect();
                d.search_text.clear();
                d.grouped_items = rebuild_grouped_items_for_command_bar(
                    &d.actions,
                    &d.filtered_actions,
                    d.config.section_style,
                );

                let old_count = d.list_state.item_count();
                d.list_state.splice(0..old_count, d.grouped_items.len());
                d.selected_index = first_selectable_index(&d.grouped_items).unwrap_or(0);
                if !d.grouped_items.is_empty() {
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                }
                cx.notify();
            });

            if self.is_open {
                resize_actions_window(cx, dialog);
            }
        }
    }

    /// Update the theme
    pub fn set_theme(&mut self, theme: Arc<theme::Theme>, cx: &mut App) {
        self.theme = theme.clone();

        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                d.update_theme(theme);
                cx.notify();
            });

            if self.is_open {
                notify_actions_window(cx);
            }
        }
    }

    /// Toggle open/close state (for Cmd+K binding)
    pub fn toggle<V: 'static>(&mut self, window: &mut Window, cx: &mut Context<V>) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }

    /// Open the command bar at the default position (bottom-right)
    pub fn open<V: 'static>(&mut self, window: &mut Window, cx: &mut Context<V>) {
        self.open_at_position(window, cx, super::window::WindowPosition::BottomRight);
    }

    /// Open the command bar at top-center (Raycast-style, for Notes window)
    pub fn open_centered<V: 'static>(&mut self, window: &mut Window, cx: &mut Context<V>) {
        self.open_at_position(window, cx, super::window::WindowPosition::TopCenter);
    }

    /// Open the command bar at a specific position
    pub fn open_at_position<V: 'static>(
        &mut self,
        window: &mut Window,
        cx: &mut Context<V>,
        position: super::window::WindowPosition,
    ) {
        if self.is_open {
            return;
        }

        // Create callback for dialog
        let on_select: Arc<dyn Fn(String) + Send + Sync> = Arc::new(|_| {
            // Action handling is done via execute_selected_action()
        });

        // Create the dialog entity
        let theme = self.theme.clone();
        let actions = self.actions.clone();
        let config = self.config.dialog_config.clone();

        // Log what actions we're creating the dialog with
        logging::log(
            "COMMAND_BAR",
            &format!(
                "Creating dialog with {} actions: [{}]",
                actions.len(),
                actions
                    .iter()
                    .take(5)
                    .map(|a| a.title.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );

        let dialog = cx.new(|cx| {
            let mut d =
                ActionsDialog::with_config(cx.focus_handle(), on_select, actions, theme, config);
            // Tell dialog to skip track_focus - ActionsWindow handles focus instead
            // This ensures keyboard events go to ActionsWindow's on_key_down handler
            d.set_skip_track_focus(true);
            d
        });

        // Get window bounds and display for positioning
        let bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());

        // Store dialog and mark as open
        self.dialog = Some(dialog.clone());
        self.is_open = true;

        // Open the vibrancy window at the specified position
        match open_actions_window(cx, bounds, display_id, dialog, position) {
            Ok(_) => {
                logging::log(
                    "COMMAND_BAR",
                    &format!("Command bar opened at {:?}", position),
                );
            }
            Err(e) => {
                logging::log("COMMAND_BAR", &format!("Failed to open command bar: {}", e));
                self.reset_open_state();
            }
        }

        cx.notify();
    }

    /// Close the command bar
    pub fn close<V: 'static>(&mut self, cx: &mut Context<V>) {
        if !self.is_open {
            return;
        }

        close_actions_window(cx);
        self.reset_open_state();
        logging::log("COMMAND_BAR", "Command bar closed");
        cx.notify();
    }

    /// Close the command bar (App context version)
    pub fn close_app(&mut self, cx: &mut App) {
        if !self.is_open {
            return;
        }

        close_actions_window(cx);
        self.reset_open_state();
        logging::log("COMMAND_BAR", "Command bar closed");
    }

    /// Check if the command bar is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the currently selected action ID
    pub fn get_selected_action_id(&self, cx: &App) -> Option<String> {
        self.dialog
            .as_ref()
            .and_then(|d| d.read(cx).get_selected_action_id())
    }

    /// Get the currently selected action
    pub fn get_selected_action<'a>(&'a self, cx: &'a App) -> Option<&'a Action> {
        self.dialog
            .as_ref()
            .and_then(|d| d.read(cx).get_selected_action())
    }

    /// Execute the selected action and optionally close the command bar
    ///
    /// Returns the action ID if an action was executed, None otherwise.
    pub fn execute_selected_action<V: 'static>(&mut self, cx: &mut Context<V>) -> Option<String> {
        let action_id = match self.get_selected_action_id(cx) {
            Some(action_id) => action_id,
            None => {
                logging::log(
                    "COMMAND_BAR",
                    "No selected action available during execute_selected_action",
                );
                return None;
            }
        };

        logging::log(
            "COMMAND_BAR",
            &format!("Executing selected action '{}'", action_id),
        );

        // Call the callback if set
        if let Some(callback) = &self.on_action {
            callback(&action_id);
        }

        // Close if configured to do so
        if self.config.close_on_select {
            self.close(cx);
        }

        Some(action_id)
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
            resize_actions_window(cx, dialog);
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.handle_backspace(cx));
            resize_actions_window(cx, dialog);
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.move_up(cx));
            notify_actions_window(cx);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.move_down(cx));
            notify_actions_window(cx);
        }
    }

    /// Jump to first action in the list.
    pub fn select_first(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if let Some(first) = first_selectable_index(&d.grouped_items) {
                    d.selected_index = first;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Jump to last action in the list.
    pub fn select_last(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if let Some(last) = last_selectable_index(&d.grouped_items) {
                    d.selected_index = last;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Move one page up in the list.
    pub fn select_page_up(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if d.grouped_items.is_empty() {
                    return;
                }

                let target = d.selected_index.saturating_sub(COMMAND_BAR_PAGE_JUMP);
                if let Some(next_index) = selectable_index_at_or_before(&d.grouped_items, target)
                    .or_else(|| first_selectable_index(&d.grouped_items))
                {
                    d.selected_index = next_index;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Move one page down in the list.
    pub fn select_page_down(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if d.grouped_items.is_empty() {
                    return;
                }

                let last_index = d.grouped_items.len() - 1;
                let target = (d.selected_index + COMMAND_BAR_PAGE_JUMP).min(last_index);
                if let Some(next_index) = selectable_index_at_or_after(&d.grouped_items, target)
                    .or_else(|| last_selectable_index(&d.grouped_items))
                {
                    d.selected_index = next_index;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, _cx| {
                d.set_cursor_visible(visible);
            });
            notify_actions_window(cx);
        }
    }

    /// Get the dialog entity (for advanced use cases)
    pub fn dialog(&self) -> Option<&Entity<ActionsDialog>> {
        self.dialog.as_ref()
    }

    /// Get access to the focus handle of the underlying dialog
    pub fn focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        self.dialog
            .as_ref()
            .map(|d| d.read(cx).focus_handle.clone())
    }
}

fn rebuild_grouped_items_for_command_bar(
    actions: &[Action],
    filtered_actions: &[usize],
    section_style: SectionStyle,
) -> Vec<GroupedActionItem> {
    let mut grouped = Vec::new();
    if filtered_actions.is_empty() {
        return grouped;
    }

    let mut previous_section: Option<String> = None;
    for (filter_idx, &action_idx) in filtered_actions.iter().enumerate() {
        if let Some(action) = actions.get(action_idx) {
            if section_style == SectionStyle::Headers {
                if let Some(section) = &action.section {
                    if previous_section.as_ref() != Some(section) {
                        grouped.push(GroupedActionItem::SectionHeader(section.clone()));
                        previous_section = Some(section.clone());
                    }
                }
            }

            grouped.push(GroupedActionItem::Item(filter_idx));
        }
    }

    grouped
}

#[cfg(test)]
mod command_bar_set_actions_tests {
    use super::*;
    use crate::actions::types::ActionCategory;

    fn test_action(id: &str, section: Option<&str>) -> Action {
        Action {
            id: id.to_string(),
            title: id.to_string(),
            description: None,
            category: ActionCategory::ScriptContext,
            shortcut: None,
            has_action: true,
            value: None,
            icon: None,
            section: section.map(str::to_string),
            title_lower: id.to_lowercase(),
            description_lower: None,
            shortcut_lower: None,
        }
    }

    #[test]
    fn test_rebuild_grouped_items_for_command_bar_adds_headers_for_new_sections() {
        let actions = vec![
            test_action("a", Some("Alpha")),
            test_action("b", Some("Alpha")),
            test_action("c", Some("Beta")),
        ];
        let filtered = vec![0, 1, 2];

        let grouped =
            rebuild_grouped_items_for_command_bar(&actions, &filtered, SectionStyle::Headers);

        assert_eq!(grouped.len(), 5);
        assert!(matches!(
            grouped.first(),
            Some(GroupedActionItem::SectionHeader(section)) if section == "Alpha"
        ));
        assert!(matches!(grouped.get(1), Some(GroupedActionItem::Item(0))));
        assert!(matches!(grouped.get(2), Some(GroupedActionItem::Item(1))));
        assert!(matches!(
            grouped.get(3),
            Some(GroupedActionItem::SectionHeader(section)) if section == "Beta"
        ));
        assert!(matches!(grouped.get(4), Some(GroupedActionItem::Item(2))));
    }

    #[test]
    fn test_rebuild_grouped_items_for_command_bar_keeps_compact_rows_for_separators() {
        let actions = vec![
            test_action("a", Some("Alpha")),
            test_action("b", Some("Beta")),
        ];
        let filtered = vec![0, 1];

        let grouped =
            rebuild_grouped_items_for_command_bar(&actions, &filtered, SectionStyle::Separators);

        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped.first(), Some(GroupedActionItem::Item(0))));
        assert!(matches!(grouped.get(1), Some(GroupedActionItem::Item(1))));
    }
}

// --- merged from part_03.rs ---
/// Trait for views that can host a command bar
///
/// Implement this trait to enable Cmd+K command bar functionality in your view.
#[allow(dead_code)] // Public API - trait for future integrations
pub trait CommandBarHost {
    /// Get a reference to the command bar
    fn command_bar(&self) -> &CommandBar;

    /// Get a mutable reference to the command bar
    fn command_bar_mut(&mut self) -> &mut CommandBar;

    /// Get actions for the current context
    ///
    /// Override this to provide context-aware actions.
    fn get_context_actions(&self) -> Vec<Action> {
        vec![]
    }

    /// Handle action execution
    ///
    /// Called when an action is selected from the command bar.
    /// Override this to implement action handling.
    fn execute_action(&mut self, action_id: &str, window: &mut Window, cx: &mut Context<Self>)
    where
        Self: Sized;

    /// Toggle the command bar (Cmd+K)
    fn toggle_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>)
    where
        Self: Sized + 'static,
    {
        self.command_bar_mut().toggle(window, cx);
    }

    /// Handle keyboard input when command bar is open
    ///
    /// Returns true if the key was handled, false otherwise.
    fn handle_command_bar_key(
        &mut self,
        key: &str,
        modifiers: &gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool
    where
        Self: Sized + 'static,
    {
        if !self.command_bar().is_open() {
            return false;
        }

        match command_bar_key_intent(key, modifiers) {
            Some(CommandBarKeyIntent::MoveUp) => {
                self.command_bar_mut().select_prev(cx);
                true
            }
            Some(CommandBarKeyIntent::MoveDown) => {
                self.command_bar_mut().select_next(cx);
                true
            }
            Some(CommandBarKeyIntent::MoveHome) => {
                self.command_bar_mut().select_first(cx);
                true
            }
            Some(CommandBarKeyIntent::MoveEnd) => {
                self.command_bar_mut().select_last(cx);
                true
            }
            Some(CommandBarKeyIntent::MovePageUp) => {
                self.command_bar_mut().select_page_up(cx);
                true
            }
            Some(CommandBarKeyIntent::MovePageDown) => {
                self.command_bar_mut().select_page_down(cx);
                true
            }
            Some(CommandBarKeyIntent::ExecuteSelected) => {
                if let Some(action_id) = self.command_bar_mut().execute_selected_action(cx) {
                    self.execute_action(&action_id, window, cx);
                }
                true
            }
            Some(CommandBarKeyIntent::Close) => {
                if self.command_bar().config.close_on_escape {
                    self.command_bar_mut().close(cx);
                }
                true
            }
            Some(CommandBarKeyIntent::Backspace) => {
                self.command_bar_mut().handle_backspace(cx);
                true
            }
            Some(CommandBarKeyIntent::TypeChar(ch)) => {
                self.command_bar_mut().handle_char(ch, cx);
                true
            }
            None => false,
        }
    }
}

/// Check if any command bar window is currently open (global check)
#[allow(dead_code)] // Public API - global check function for future integrations
pub fn is_command_bar_open() -> bool {
    is_actions_window_open()
}

#[cfg(test)]
mod command_bar_config_tests {
    use super::*;

    #[test]
    fn test_command_bar_config_defaults() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    #[test]
    fn test_command_bar_config_ai_style() {
        let config = CommandBarConfig::ai_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Top
        ));
        assert!(matches!(
            config.dialog_config.section_style,
            SectionStyle::Headers
        ));
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn test_command_bar_config_main_menu_style() {
        let config = CommandBarConfig::main_menu_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Bottom
        ));
        assert!(matches!(
            config.dialog_config.section_style,
            SectionStyle::Separators
        ));
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn test_command_bar_config_no_search() {
        let config = CommandBarConfig::no_search();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Hidden
        ));
    }

    #[test]
    fn test_command_bar_key_intent_supports_aliases_and_jump_keys() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            command_bar_key_intent("return", &no_mods),
            Some(CommandBarKeyIntent::ExecuteSelected)
        );
        assert_eq!(
            command_bar_key_intent("esc", &no_mods),
            Some(CommandBarKeyIntent::Close)
        );
        assert_eq!(
            command_bar_key_intent("home", &no_mods),
            Some(CommandBarKeyIntent::MoveHome)
        );
        assert_eq!(
            command_bar_key_intent("end", &no_mods),
            Some(CommandBarKeyIntent::MoveEnd)
        );
        assert_eq!(
            command_bar_key_intent("pageup", &no_mods),
            Some(CommandBarKeyIntent::MovePageUp)
        );
        assert_eq!(
            command_bar_key_intent("pagedown", &no_mods),
            Some(CommandBarKeyIntent::MovePageDown)
        );
    }

    #[test]
    fn test_selectable_index_helpers_skip_section_headers() {
        let rows = vec![
            GroupedActionItem::SectionHeader("System".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("Script".to_string()),
            GroupedActionItem::Item(1),
        ];

        assert_eq!(first_selectable_index(&rows), Some(1));
        assert_eq!(last_selectable_index(&rows), Some(3));
        assert_eq!(selectable_index_at_or_before(&rows, 2), Some(1));
        assert_eq!(selectable_index_at_or_after(&rows, 2), Some(3));
    }
}
