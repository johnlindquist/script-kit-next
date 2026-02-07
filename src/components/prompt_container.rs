//! Reusable PromptContainer component for GPUI Script Kit
//!
//! This module provides a theme-aware container component that wraps the overall
//! prompt window layout with consistent styling. It handles the header, content,
//! footer, and optional divider between sections.
//!

#![allow(dead_code)]

use gpui::*;

use crate::components::prompt_layout_shell::{
    prompt_frame_fill_content, prompt_frame_root, PromptFrameConfig,
};
use crate::designs::DesignColors;
use crate::theme::Theme;

/// Pre-computed colors for PromptContainer rendering
///
/// This struct holds the primitive color values needed for container rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct PromptContainerColors {
    /// Main background color
    pub background: u32,
    /// Primary text color
    pub text_primary: u32,
    /// Muted/hint text color
    pub text_muted: u32,
    /// Border color for dividers
    pub border: u32,
    /// Font size for footer hint/caption text
    pub hint_font_size: f32,
}

impl PromptContainerColors {
    /// Create PromptContainerColors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        let ui_font_size = theme.get_fonts().ui_size;
        Self {
            background: theme.colors.background.main,
            text_primary: theme.colors.text.primary,
            text_muted: theme.colors.text.muted,
            border: theme.colors.ui.border,
            hint_font_size: (ui_font_size - 4.0).max(9.0),
        }
    }

    /// Create PromptContainerColors from design colors for design system support
    pub fn from_design(colors: &DesignColors) -> Self {
        let typography = crate::designs::DesignTypography::default();
        Self {
            background: colors.background,
            text_primary: colors.text_primary,
            text_muted: colors.text_muted,
            border: colors.border,
            hint_font_size: typography.font_size_xs,
        }
    }
}

impl Default for PromptContainerColors {
    fn default() -> Self {
        Self::from_theme(&crate::theme::get_cached_theme())
    }
}

/// Content sizing mode for PromptContainer's content slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PromptContainerContentLayout {
    /// Wrap content in a fill container (`flex_1 + min_h(0) + overflow_hidden`).
    #[default]
    Fill,
    /// Render content as-is with no wrapper sizing policy.
    Intrinsic,
}

impl PromptContainerContentLayout {
    fn wraps_with_fill_slot(self) -> bool {
        matches!(self, Self::Fill)
    }
}

/// Configuration for PromptContainer display
#[derive(Clone, Debug)]
pub struct PromptContainerConfig {
    /// Border radius for rounded corners (default: 12.0)
    pub rounded_corners: f32,
    /// Show divider after header (default: true)
    pub show_divider: bool,
    /// Optional footer hint text
    pub hint_text: Option<String>,
    /// Background opacity (0x00-0xFF, default: 0xE8 = 232 ≈ 91%)
    pub background_opacity: u8,
    /// Divider horizontal margin (default: 16.0)
    pub divider_margin: f32,
    /// Content sizing mode (default: Fill)
    pub content_layout: PromptContainerContentLayout,
}

impl Default for PromptContainerConfig {
    fn default() -> Self {
        Self {
            rounded_corners: 12.0,
            show_divider: true,
            hint_text: None,
            background_opacity: 0xE8,
            divider_margin: 16.0,
            content_layout: PromptContainerContentLayout::Fill,
        }
    }
}

impl PromptContainerConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the border radius for rounded corners
    pub fn rounded_corners(mut self, radius: f32) -> Self {
        self.rounded_corners = radius;
        self
    }

    /// Set whether to show divider after header
    pub fn show_divider(mut self, show: bool) -> Self {
        self.show_divider = show;
        self
    }

    /// Set the footer hint text
    pub fn hint(mut self, text: impl Into<String>) -> Self {
        self.hint_text = Some(text.into());
        self
    }

    /// Set the background opacity (0-255)
    pub fn background_opacity(mut self, opacity: u8) -> Self {
        self.background_opacity = opacity;
        self
    }

    /// Set the divider horizontal margin
    pub fn divider_margin(mut self, margin: f32) -> Self {
        self.divider_margin = margin;
        self
    }

    /// Set content sizing mode
    pub fn content_layout(mut self, content_layout: PromptContainerContentLayout) -> Self {
        self.content_layout = content_layout;
        self
    }
}

/// A reusable container component for prompts
///
/// Provides consistent layout for all prompt types:
/// - Container with rounded corners and background
/// - Header slot (full width)
/// - Optional divider
/// - Content slot (scrollable, fills remaining space)
/// - Footer slot (optional, for hints)
///
#[derive(IntoElement)]
pub struct PromptContainer {
    colors: PromptContainerColors,
    config: PromptContainerConfig,
    header: Option<AnyElement>,
    content: Option<AnyElement>,
    footer: Option<AnyElement>,
}

impl PromptContainer {
    /// Create a new PromptContainer with the given colors
    pub fn new(colors: PromptContainerColors) -> Self {
        Self {
            colors,
            config: PromptContainerConfig::default(),
            header: None,
            content: None,
            footer: None,
        }
    }

    /// Set the configuration
    pub fn config(mut self, config: PromptContainerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the header element
    pub fn header(mut self, element: impl IntoElement) -> Self {
        self.header = Some(element.into_any_element());
        self
    }

    /// Set the content element
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content = Some(element.into_any_element());
        self
    }

    /// Set the footer element
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer = Some(element.into_any_element());
        self
    }

    /// Convenience: Set hint text as footer (creates a styled hint element)
    pub fn hint(mut self, text: impl Into<String>) -> Self {
        self.config.hint_text = Some(text.into());
        self
    }

    /// Render the divider between header and content
    fn render_divider(colors: PromptContainerColors, margin: f32) -> impl IntoElement {
        // Semi-transparent border (60% opacity)
        let border_with_alpha = (colors.border << 8) | 0x60;

        div().mx(px(margin)).h(px(1.)).bg(rgba(border_with_alpha))
    }

    /// Render the footer hint text
    fn render_hint(colors: PromptContainerColors, text: String) -> impl IntoElement {
        div()
            .w_full()
            .px(px(16.0))
            .py(px(8.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_size(px(colors.hint_font_size))
                    .text_color(rgb(colors.text_muted))
                    .child(text),
            )
    }
}

pub(crate) fn prompt_container_frame_config(config: &PromptContainerConfig) -> PromptFrameConfig {
    PromptFrameConfig::default().with_rounded_corners(config.rounded_corners)
}

impl RenderOnce for PromptContainer {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Self {
            colors,
            config,
            header,
            content,
            footer,
        } = self;
        let has_header = header.is_some();

        // Background with opacity
        let bg_with_opacity = (colors.background << 8) | (config.background_opacity as u32);

        // Build the container
        let mut container = prompt_frame_root(prompt_container_frame_config(&config))
            .bg(rgba(bg_with_opacity))
            .text_color(rgb(colors.text_primary))
            .font_family(crate::list_item::FONT_SYSTEM_UI);

        // Add header if present
        if let Some(header) = header {
            container = container.child(header);
        }

        // Add divider if configured and header was present
        if config.show_divider && has_header {
            container = container.child(Self::render_divider(colors, config.divider_margin));
        }

        // Add content if present
        if let Some(content) = content {
            container = if config.content_layout.wraps_with_fill_slot() {
                container.child(prompt_frame_fill_content(content))
            } else {
                container.child(content)
            };
        }

        // Add footer if present, or hint text if configured
        if let Some(footer) = footer {
            container = container.child(footer);
        } else if let Some(hint) = config.hint_text {
            container = container.child(Self::render_hint(colors, hint));
        }

        container
    }
}

#[cfg(test)]
mod prompt_container_tests {
    use super::{
        prompt_container_frame_config, PromptContainerColors, PromptContainerConfig,
        PromptContainerContentLayout,
    };
    use crate::components::prompt_layout_shell::prompt_shell_frame_config;

    #[test]
    fn test_prompt_container_content_fill_mode_wraps_child_with_flex_1() {
        assert!(PromptContainerContentLayout::Fill.wraps_with_fill_slot());
        let config = PromptContainerConfig::default();
        assert_eq!(config.content_layout, PromptContainerContentLayout::Fill);
    }

    #[test]
    fn test_prompt_container_intrinsic_mode_leaves_content_unwrapped() {
        assert!(!PromptContainerContentLayout::Intrinsic.wraps_with_fill_slot());
    }

    #[test]
    fn test_prompt_shell_and_prompt_container_share_same_root_contract() {
        let shell_config = prompt_shell_frame_config(12.0);
        let container_config = prompt_container_frame_config(&PromptContainerConfig::new());

        assert_eq!(shell_config.min_height_px, container_config.min_height_px);
        assert_eq!(shell_config.clip_overflow, container_config.clip_overflow);
    }

    #[test]
    fn test_prompt_container_colors_default_uses_cached_theme_tokens() {
        let resolved = PromptContainerColors::default();
        let expected = PromptContainerColors::from_theme(&crate::theme::get_cached_theme());

        assert_eq!(resolved.background, expected.background);
        assert_eq!(resolved.text_primary, expected.text_primary);
        assert_eq!(resolved.text_muted, expected.text_muted);
        assert_eq!(resolved.border, expected.border);
        assert_eq!(resolved.hint_font_size, expected.hint_font_size);
    }
}
