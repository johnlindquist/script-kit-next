//! Unified PromptInput component for GPUI Script Kit
//!
//! This module provides a config-driven input component that unifies styling
//! across all prompt types (main menu, arg prompt, search command, etc.).
//!
//! # Design Goals
//!
//! 1. **Single source of truth** - All prompt inputs use this component
//! 2. **Config-driven** - Font sizes, padding from config/theme, not hardcoded
//! 3. **Factory methods** - Pre-configured modes for common use cases
//!
//! # Usage
//!
//! ```ignore
//! // For search/path mode with prefix
//! let config = PromptInputConfig::search()
//!     .placeholder("Search files...")
//!     .path_prefix(Some("/Users/john/".to_string()));
//!
//! // For arg prompt with full features
//! let config = PromptInputConfig::arg()
//!     .placeholder("Enter a value");
//!
//! // For main menu (simple)
//! let config = PromptInputConfig::main_menu();
//! ```

#![allow(dead_code)]

use gpui::*;

use super::input_tokens::{
    INPUT_PLACEHOLDER_ARG, INPUT_PLACEHOLDER_DEFAULT, INPUT_PLACEHOLDER_MAIN_MENU,
    INPUT_PLACEHOLDER_SEARCH,
};
use crate::config::Config;
use crate::panel::{CURSOR_GAP_X, CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y, CURSOR_WIDTH};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

/// Padding configuration for the input
#[derive(Debug, Clone, Copy)]
pub struct InputPadding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for InputPadding {
    fn default() -> Self {
        Self {
            top: 8.0,
            bottom: 8.0,
            left: 16.0,
            right: 16.0,
        }
    }
}

impl InputPadding {
    /// Create padding from config's ContentPadding
    pub fn from_config(config: &Config) -> Self {
        let padding = config.get_padding();
        Self {
            top: padding.top,
            bottom: padding.top, // Use top for both vertical
            left: padding.left,
            right: padding.right,
        }
    }

    /// Create symmetric padding
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    /// Create uniform padding
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }
}

/// Configuration for PromptInput display and behavior
#[derive(Debug, Clone)]
pub struct PromptInputConfig {
    /// Placeholder text shown when input is empty
    pub placeholder: String,
    /// Optional path prefix displayed before the input (e.g., "/Users/john/")
    pub show_path_prefix: bool,
    /// Enable text selection support
    pub enable_selection: bool,
    /// Enable clipboard operations (copy/paste/cut)
    pub enable_clipboard: bool,
    /// Font size for input text (None = use config default)
    pub font_size: Option<f32>,
    /// Padding around the input
    pub padding: Option<InputPadding>,
    /// Whether cursor is currently visible (for blinking)
    pub cursor_visible: bool,
    /// Whether the input is focused
    pub is_focused: bool,
}

impl Default for PromptInputConfig {
    fn default() -> Self {
        Self {
            placeholder: INPUT_PLACEHOLDER_DEFAULT.to_string(),
            show_path_prefix: false,
            enable_selection: true,
            enable_clipboard: true,
            font_size: None, // Use config default
            padding: None,   // Use config default
            cursor_visible: true,
            is_focused: true,
        }
    }
}

impl PromptInputConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration for search/path mode (with path prefix support)
    ///
    /// This mode is used by Search Command and path-related prompts.
    /// Features:
    /// - Path prefix display support
    /// - Full selection and clipboard support
    pub fn search() -> Self {
        Self {
            placeholder: INPUT_PLACEHOLDER_SEARCH.to_string(),
            show_path_prefix: true,
            enable_selection: true,
            enable_clipboard: true,
            font_size: None,
            padding: None,
            cursor_visible: true,
            is_focused: true,
        }
    }

    /// Create configuration for arg prompt (full features)
    ///
    /// This mode is used by ArgPrompt for script arguments.
    /// Features:
    /// - Full selection and clipboard support
    /// - No path prefix
    pub fn arg() -> Self {
        Self {
            placeholder: INPUT_PLACEHOLDER_ARG.to_string(),
            show_path_prefix: false,
            enable_selection: true,
            enable_clipboard: true,
            font_size: None,
            padding: None,
            cursor_visible: true,
            is_focused: true,
        }
    }

    /// Create configuration for main menu (simple search)
    ///
    /// This mode is used by the main script list.
    /// Features:
    /// - Simple search styling
    /// - No path prefix
    pub fn main_menu() -> Self {
        Self {
            placeholder: INPUT_PLACEHOLDER_MAIN_MENU.to_string(),
            show_path_prefix: false,
            enable_selection: false, // Main menu typically doesn't need selection
            enable_clipboard: false, // Main menu typically doesn't need clipboard
            font_size: None,
            padding: None,
            cursor_visible: true,
            is_focused: true,
        }
    }

    // Builder pattern methods

    /// Set the placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set whether to show path prefix
    pub fn show_path_prefix(mut self, show: bool) -> Self {
        self.show_path_prefix = show;
        self
    }

    /// Set whether selection is enabled
    pub fn enable_selection(mut self, enabled: bool) -> Self {
        self.enable_selection = enabled;
        self
    }

    /// Set whether clipboard operations are enabled
    pub fn enable_clipboard(mut self, enabled: bool) -> Self {
        self.enable_clipboard = enabled;
        self
    }

    /// Set the font size (None = use config default)
    pub fn font_size(mut self, size: Option<f32>) -> Self {
        self.font_size = size;
        self
    }

    /// Set the padding
    pub fn padding(mut self, padding: Option<InputPadding>) -> Self {
        self.padding = padding;
        self
    }

    /// Set cursor visibility (for blinking)
    pub fn cursor_visible(mut self, visible: bool) -> Self {
        self.cursor_visible = visible;
        self
    }

    /// Set whether the input is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    /// Get the effective font size, using config default if not set
    pub fn get_font_size(&self, config: &Config) -> f32 {
        self.font_size
            .unwrap_or_else(|| config.get_editor_font_size())
    }

    /// Get the effective padding, using config default if not set
    pub fn get_padding(&self, config: &Config) -> InputPadding {
        self.padding
            .unwrap_or_else(|| InputPadding::from_config(config))
    }
}

/// Pre-computed colors for PromptInput rendering
///
/// This struct holds the primitive color values needed for input rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct PromptInputColors {
    /// Main text color (for typed input)
    pub text_primary: u32,
    /// Placeholder/muted text color
    pub text_muted: u32,
    /// Dimmed text color (for path prefix)
    pub text_dimmed: u32,
    /// Accent color for cursor
    pub accent: u32,
    /// Background color
    pub background: u32,
    /// Border color
    pub border: u32,
}

impl PromptInputColors {
    /// Create PromptInputColors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            text_primary: theme.colors.text.primary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            background: theme.colors.background.main,
            border: theme.colors.ui.border,
        }
    }

    /// Create PromptInputColors from theme with focus awareness
    pub fn from_theme_focused(theme: &Theme, is_focused: bool) -> Self {
        let colors = theme.get_colors(is_focused);
        Self {
            text_primary: colors.text.primary,
            text_muted: colors.text.muted,
            text_dimmed: colors.text.dimmed,
            accent: colors.accent.selected,
            background: colors.background.main,
            border: colors.ui.border,
        }
    }
}

impl Default for PromptInputColors {
    fn default() -> Self {
        Self {
            text_primary: 0xffffff,
            text_muted: 0x808080,
            text_dimmed: 0x666666,
            accent: 0xfbbf24, // Script Kit yellow/gold
            background: 0x1e1e1e,
            border: 0x464647,
        }
    }
}

/// A unified input component for prompts
///
/// Provides consistent styling across all prompt types with config-driven
/// font sizes, padding, and colors. Supports:
/// - Path prefix display (for search/path modes)
/// - Blinking cursor
/// - Placeholder text
///
/// # Example
///
/// ```ignore
/// let input = PromptInput::new(
///     PromptInputConfig::search().placeholder("Search files..."),
///     PromptInputColors::from_theme(&theme),
/// )
/// .filter_text("my-scr")
/// .path_prefix(Some("/Users/john/"));
/// ```
#[derive(IntoElement)]
pub struct PromptInput {
    config: PromptInputConfig,
    colors: PromptInputColors,
    /// Current input text
    filter_text: String,
    /// Optional path prefix (displayed before filter text in search mode)
    path_prefix: Option<String>,
    /// Effective font size (resolved from config)
    font_size: f32,
    /// Effective padding (resolved from config)
    padding: InputPadding,
}

impl PromptInput {
    /// Create a new PromptInput with the given configuration and colors
    pub fn new(config: PromptInputConfig, colors: PromptInputColors) -> Self {
        Self {
            config,
            colors,
            filter_text: String::new(),
            path_prefix: None,
            font_size: 16.0, // Will be overridden by with_config
            padding: InputPadding::default(),
        }
    }

    /// Create a new PromptInput with configuration resolved from app config
    pub fn with_config(mut self, app_config: &Config) -> Self {
        self.font_size = self.config.get_font_size(app_config);
        self.padding = self.config.get_padding(app_config);
        self
    }

    /// Set the current filter/input text
    pub fn filter_text(mut self, text: impl Into<String>) -> Self {
        self.filter_text = text.into();
        self
    }

    /// Set the path prefix (only shown if config.show_path_prefix is true)
    pub fn path_prefix(mut self, prefix: Option<String>) -> Self {
        self.path_prefix = prefix;
        self
    }

    /// Set the font size directly (overrides config)
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the padding directly (overrides config)
    pub fn padding(mut self, padding: InputPadding) -> Self {
        self.padding = padding;
        self
    }

    /// Render the cursor element
    fn render_cursor(&self, visible: bool) -> impl IntoElement {
        let cursor_bg = if visible {
            self.colors.text_primary.to_rgb()
        } else {
            0x000000u32.with_opacity(0.0)
        };

        div()
            .w(px(CURSOR_WIDTH))
            .h(px(CURSOR_HEIGHT_LG))
            .my(px(CURSOR_MARGIN_Y))
            .bg(cursor_bg)
    }
}

impl RenderOnce for PromptInput {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let filter_is_empty = self.filter_text.is_empty();
        let cursor_visible = self.config.cursor_visible && self.config.is_focused;

        // Display text: filter text or placeholder
        let display_text: SharedString = if filter_is_empty {
            self.config.placeholder.clone().into()
        } else {
            self.filter_text.clone().into()
        };

        // Text color: muted for placeholder, primary for input
        let text_color = if filter_is_empty {
            colors.text_muted.to_rgb()
        } else {
            colors.text_primary.to_rgb()
        };

        // Build input container with flex layout
        let mut input = div()
            .flex()
            .flex_row()
            .items_center()
            .flex_1()
            .text_lg()
            .text_color(text_color);

        // Path prefix (if enabled and present)
        if self.config.show_path_prefix {
            if let Some(ref prefix) = self.path_prefix {
                input = input.child(
                    div()
                        .text_color(colors.text_muted.to_rgb())
                        .child(prefix.clone()),
                );
            }
        }

        // Cursor position logic:
        // - When empty: cursor LEFT (before placeholder)
        // - When typing: cursor RIGHT (after text)
        //
        // ALIGNMENT FIX: The left cursor (when empty) takes up space.
        // We apply a negative margin to the placeholder text to pull it back,
        // so placeholder and typed text share the same starting x-position.

        // Left cursor (when empty)
        if filter_is_empty {
            input = input.child(
                div()
                    .child(self.render_cursor(cursor_visible))
                    .mr(px(CURSOR_GAP_X)),
            );
        }

        // Display text - with negative margin for placeholder alignment
        if filter_is_empty {
            // Placeholder: pull back by cursor space to align with typed text position
            input = input.child(
                div()
                    .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                    .child(display_text),
            );
        } else {
            input = input.child(display_text);
        }

        // Right cursor (when not empty)
        if !filter_is_empty {
            input = input.child(
                div()
                    .ml(px(CURSOR_GAP_X))
                    .child(self.render_cursor(cursor_visible)),
            );
        }

        input
    }
}

// Note: Tests omitted for this module due to GPUI macro recursion limit issues.
// The PromptInput component is integration-tested via the main application's
// prompt rendering. See prompt_header.rs for similar pattern.
//
// Verified traits:
// - PromptInputColors: Copy, Clone, Debug, Default
// - PromptInputConfig: Clone, Debug, Default + builder pattern
// - InputPadding: Copy, Clone, Debug, Default
// - PromptInput: builder pattern with .filter_text(), .path_prefix(), .with_config()
