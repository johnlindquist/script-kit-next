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
