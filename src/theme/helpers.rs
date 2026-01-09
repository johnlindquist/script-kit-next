//! Lightweight theme extraction helpers
//!
//! These structs pre-compute theme values for efficient use in render closures.
//! They implement Copy to avoid heap allocations when captured by closures.

use gpui::{rgb, rgba, Hsla, Rgba};
use tracing::debug;

use super::hex_color::TRANSPARENT;
use super::types::ColorScheme;

/// Lightweight struct for list item rendering - Copy to avoid clone in closures
///
/// This struct pre-computes the exact colors needed for rendering list items,
/// avoiding the need to clone the full ThemeColors struct into render closures.
///
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct ListItemColors {
    /// Normal item background (usually transparent)
    pub background: Rgba,
    /// Background when hovering over an item
    pub background_hover: Rgba,
    /// Background when item is selected
    pub background_selected: Rgba,
    /// Primary text color (for item names)
    pub text: Rgba,
    /// Secondary/muted text color (for descriptions when not selected)
    pub text_secondary: Rgba,
    /// Dimmed text color (for shortcuts, metadata)
    pub text_dimmed: Rgba,
    /// Accent text color (for descriptions when selected)
    pub text_accent: Rgba,
    /// Border color for separators
    pub border: Rgba,
    /// Warning background color (for confirmation overlays)
    pub warning_bg: Rgba,
    /// Text color for content displayed on accent/colored backgrounds
    pub text_on_accent: Rgba,
}

#[allow(dead_code)]
impl ListItemColors {
    /// Create ListItemColors from a ColorScheme
    ///
    /// This extracts only the colors needed for list item rendering.
    pub fn from_color_scheme(colors: &ColorScheme) -> Self {
        // Pre-compute rgba colors with appropriate alpha values
        // Background is transparent, hover/selected use subtle colors
        let selected_subtle = colors.accent.selected_subtle;

        #[cfg(debug_assertions)]
        debug!(
            selected_subtle = format!("#{:06x}", selected_subtle),
            "Extracting list item colors"
        );

        ListItemColors {
            background: rgba(TRANSPARENT), // Fully transparent
            background_hover: rgba((selected_subtle << 8) | 0x40), // 25% opacity (0.25 * 255 ≈ 64)
            background_selected: rgba((selected_subtle << 8) | 0x59), // 35% opacity (0.35 * 255 ≈ 89)
            text: rgb(colors.text.primary),
            text_secondary: rgb(colors.text.secondary),
            text_dimmed: rgb(colors.text.dimmed),
            text_accent: rgb(colors.accent.selected),
            border: rgb(colors.ui.border),
            warning_bg: rgb(colors.ui.warning),
            // For dark themes, white text on warning/accent backgrounds provides good contrast
            // Theme default is 0xffffff (white) which works well on yellow/orange backgrounds
            text_on_accent: rgb(colors.text.primary),
        }
    }

    /// Convert a specific color to Hsla for advanced styling
    pub fn text_as_hsla(&self) -> Hsla {
        self.text.into()
    }

    /// Get description color based on selection state
    pub fn description_color(&self, is_selected: bool) -> Rgba {
        if is_selected {
            self.text_accent
        } else {
            self.text_secondary
        }
    }

    /// Get item text color based on selection state
    pub fn item_text_color(&self, is_selected: bool) -> Rgba {
        if is_selected {
            self.text
        } else {
            self.text_secondary
        }
    }
}

#[allow(dead_code)]
impl ColorScheme {
    /// Extract only the colors needed for list item rendering
    ///
    /// This creates a lightweight, Copy struct that can be efficiently
    /// passed into closures without cloning the full ColorScheme.
    pub fn list_item_colors(&self) -> ListItemColors {
        ListItemColors::from_color_scheme(self)
    }
}

/// Lightweight struct for input field rendering
///
/// Pre-computes colors for search boxes, text inputs, etc.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct InputFieldColors {
    /// Background color of the input
    pub background: Rgba,
    /// Text color when typing
    pub text: Rgba,
    /// Placeholder text color
    pub placeholder: Rgba,
    /// Border color
    pub border: Rgba,
    /// Cursor color
    pub cursor: Rgba,
}

#[allow(dead_code)]
impl InputFieldColors {
    /// Create InputFieldColors from a ColorScheme
    pub fn from_color_scheme(colors: &ColorScheme) -> Self {
        #[cfg(debug_assertions)]
        debug!("Extracting input field colors");

        InputFieldColors {
            background: rgba((colors.background.search_box << 8) | 0x80),
            text: rgb(colors.text.primary),
            placeholder: rgb(colors.text.muted),
            border: rgba((colors.ui.border << 8) | 0x60),
            // Use accent color for cursor - provides visual consistency with selection
            cursor: rgb(colors.accent.selected),
        }
    }
}

#[allow(dead_code)]
impl ColorScheme {
    /// Extract colors for input field rendering
    pub fn input_field_colors(&self) -> InputFieldColors {
        InputFieldColors::from_color_scheme(self)
    }

    /// Extract colors for prompt rendering (DivPrompt, etc.)
    pub fn prompt_colors(&self) -> PromptColors {
        PromptColors::from_color_scheme(self)
    }
}

/// Lightweight struct for prompt rendering (DivPrompt HTML content)
///
/// Pre-computes colors needed for rendering HTML elements in prompts.
/// Implements Copy to avoid heap allocations when captured by closures.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct PromptColors {
    /// Primary text color (for headings, strong text)
    pub text_primary: u32,
    /// Secondary text color (default paragraph text)
    pub text_secondary: u32,
    /// Tertiary text color (italic text, list bullets)
    pub text_tertiary: u32,
    /// Accent color (links, inline code text)
    pub accent_color: u32,
    /// Code background color (code blocks, inline code)
    pub code_bg: u32,
    /// Quote border color (blockquote left border)
    pub quote_border: u32,
    /// Horizontal rule color
    pub hr_color: u32,
}

#[allow(dead_code)]
impl PromptColors {
    /// Create PromptColors from a ColorScheme
    ///
    /// This extracts only the colors needed for rendering HTML prompts.
    pub fn from_color_scheme(colors: &ColorScheme) -> Self {
        #[cfg(debug_assertions)]
        debug!("Extracting prompt colors");

        PromptColors {
            text_primary: colors.text.primary,
            text_secondary: colors.text.secondary,
            text_tertiary: colors.text.tertiary,
            accent_color: colors.accent.selected,
            code_bg: colors.background.search_box,
            quote_border: colors.ui.border,
            hr_color: colors.ui.border,
        }
    }
}
