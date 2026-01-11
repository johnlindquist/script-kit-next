//! ScriptKitInput - Unified input component for Script Kit
//!
//! Wraps gpui_component::Input to provide:
//! - Config-driven styling (font size, padding from ~/.scriptkit/config.ts)
//! - Factory methods for different contexts (chat, search, arg, main_menu)
//! - Consistent cursor, selection, and placeholder behavior everywhere

use gpui::{prelude::*, App, Entity, IntoElement, Styled, Window};
use gpui_component::input::{Input, InputState};

use crate::config::Config;

/// Configuration for ScriptKitInput appearance
#[allow(dead_code)] // Will be used by ChatPrompt and other prompts
#[derive(Clone, Debug)]
pub struct ScriptKitInputConfig {
    pub placeholder: String,
    pub font_size: f32,
    pub appearance: bool, // Show border/background
    pub bordered: bool,
    pub focus_bordered: bool,
}

impl Default for ScriptKitInputConfig {
    fn default() -> Self {
        Self {
            placeholder: "Type here...".into(),
            font_size: 16.0,
            appearance: true,
            bordered: true,
            focus_bordered: true,
        }
    }
}

#[allow(dead_code)] // Will be used by ChatPrompt and other prompts
impl ScriptKitInputConfig {
    /// Chat input configuration (no border, blends with container)
    pub fn chat() -> Self {
        Self {
            placeholder: "Ask anything...".into(),
            font_size: 14.0,
            appearance: false,
            bordered: false,
            focus_bordered: false,
        }
    }

    /// Search input configuration
    pub fn search() -> Self {
        Self {
            placeholder: "Search...".into(),
            font_size: 16.0,
            appearance: false,
            bordered: false,
            focus_bordered: false,
        }
    }

    /// Main menu input configuration
    pub fn main_menu() -> Self {
        Self {
            placeholder: "Script Kit".into(),
            font_size: 18.0,
            appearance: false,
            bordered: false,
            focus_bordered: false,
        }
    }

    /// Arg prompt input configuration
    pub fn arg() -> Self {
        Self {
            placeholder: "Enter value...".into(),
            font_size: 16.0,
            appearance: false,
            bordered: false,
            focus_bordered: false,
        }
    }

    /// Apply config values (font size from app config)
    pub fn with_app_config(mut self, config: &Config) -> Self {
        self.font_size = config.get_editor_font_size();
        self
    }

    /// Set placeholder
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

/// Unified input component for Script Kit
///
/// Wraps gpui_component::Input with config-driven styling.
/// Use factory methods to create for different contexts.
#[allow(dead_code)] // Will be used by ChatPrompt and other prompts
pub struct ScriptKitInput {
    state: Entity<InputState>,
    config: ScriptKitInputConfig,
}

#[allow(dead_code)] // Will be used by ChatPrompt and other prompts
impl ScriptKitInput {
    /// Create a new ScriptKitInput with the given state and config
    pub fn new(state: Entity<InputState>, config: ScriptKitInputConfig) -> Self {
        Self { state, config }
    }

    /// Create input state for chat context
    pub fn create_chat_state(window: &mut Window, cx: &mut App) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx).placeholder("Ask anything..."))
    }

    /// Create input state for search context
    pub fn create_search_state(window: &mut Window, cx: &mut App) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx).placeholder("Search..."))
    }

    /// Create input state for main menu context
    pub fn create_main_menu_state(window: &mut Window, cx: &mut App) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx).placeholder("Script Kit"))
    }

    /// Create input state for arg prompt context
    pub fn create_arg_state(window: &mut Window, cx: &mut App) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx).placeholder("Enter value..."))
    }

    /// Get the underlying state entity
    pub fn state(&self) -> &Entity<InputState> {
        &self.state
    }

    /// Get the current input value
    pub fn value(&self, cx: &App) -> String {
        self.state.read(cx).value().to_string()
    }

    /// Clear the input
    pub fn clear(&self, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
    }

    /// Set the input value
    pub fn set_value(&self, value: impl Into<String>, window: &mut Window, cx: &mut App) {
        let value = value.into();
        self.state.update(cx, |state, cx| {
            state.set_value(&value, window, cx);
        });
    }

    /// Render the input component
    pub fn render(&self) -> Input {
        Input::new(&self.state)
            .appearance(self.config.appearance)
            .bordered(self.config.bordered)
            .focus_bordered(self.config.focus_bordered)
    }

    /// Render with full width
    pub fn render_full_width(&self) -> impl IntoElement {
        self.render().w_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScriptKitInputConfig::default();
        assert_eq!(config.placeholder, "Type here...");
        assert_eq!(config.font_size, 16.0);
        assert!(config.appearance);
        assert!(config.bordered);
        assert!(config.focus_bordered);
    }

    #[test]
    fn test_chat_config() {
        let config = ScriptKitInputConfig::chat();
        assert_eq!(config.placeholder, "Ask anything...");
        assert_eq!(config.font_size, 14.0);
        assert!(!config.appearance);
        assert!(!config.bordered);
        assert!(!config.focus_bordered);
    }

    #[test]
    fn test_search_config() {
        let config = ScriptKitInputConfig::search();
        assert_eq!(config.placeholder, "Search...");
        assert_eq!(config.font_size, 16.0);
        assert!(!config.appearance);
    }

    #[test]
    fn test_main_menu_config() {
        let config = ScriptKitInputConfig::main_menu();
        assert_eq!(config.placeholder, "Script Kit");
        assert_eq!(config.font_size, 18.0);
        assert!(!config.appearance);
    }

    #[test]
    fn test_arg_config() {
        let config = ScriptKitInputConfig::arg();
        assert_eq!(config.placeholder, "Enter value...");
        assert_eq!(config.font_size, 16.0);
        assert!(!config.appearance);
    }

    #[test]
    fn test_builder_methods() {
        let config = ScriptKitInputConfig::default()
            .placeholder("Custom placeholder")
            .font_size(20.0);

        assert_eq!(config.placeholder, "Custom placeholder");
        assert_eq!(config.font_size, 20.0);
    }
}
