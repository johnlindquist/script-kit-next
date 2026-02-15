//! Lightweight theme extraction helpers
//!
//! These structs pre-compute theme values for efficient use in render closures.
//! They implement Copy to avoid heap allocations when captured by closures.

use gpui::{rgb, rgba, Rgba};
use tracing::debug;

use super::types::{ColorScheme, Theme};
pub use crate::list_item::ListItemColors;

#[allow(dead_code)]
impl ColorScheme {
    /// Extract only the colors needed for list item rendering
    ///
    /// Uses the canonical list item color struct from `crate::list_item`.
    /// A temporary `Theme` is built to preserve the existing `ColorScheme` API.
    pub fn list_item_colors(&self) -> ListItemColors {
        ListItemColors::from_theme(&Theme {
            colors: self.clone(),
            ..Theme::default()
        })
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

    /// Create InputFieldColors from a Theme (preferred method)
    pub fn from_theme(theme: &Theme) -> Self {
        Self::from_color_scheme(&theme.colors)
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
    /// Whether dark mode is active (for syntax highlighting)
    pub is_dark: bool,
}

#[allow(dead_code)]
impl PromptColors {
    /// Create PromptColors from a ColorScheme
    ///
    /// This extracts only the colors needed for rendering HTML prompts.
    /// `is_dark` is derived from the scheme's background luminance.
    pub fn from_color_scheme(colors: &ColorScheme) -> Self {
        #[cfg(debug_assertions)]
        debug!("Extracting prompt colors");

        // Keep dark/light detection aligned with Theme::has_dark_colors() semantics.
        let bg = colors.background.main;
        let r = ((bg >> 16) & 0xFF) as f32 / 255.0;
        let g = ((bg >> 8) & 0xFF) as f32 / 255.0;
        let b = (bg & 0xFF) as f32 / 255.0;
        let is_dark = (0.2126 * r) + (0.7152 * g) + (0.0722 * b) < 0.5;

        PromptColors {
            text_primary: colors.text.primary,
            text_secondary: colors.text.secondary,
            text_tertiary: colors.text.tertiary,
            accent_color: colors.accent.selected,
            code_bg: colors.background.search_box,
            quote_border: colors.ui.border,
            hr_color: colors.ui.border,
            is_dark,
        }
    }

    /// Create PromptColors from a Theme (preferred method)
    pub fn from_theme(theme: &Theme) -> Self {
        let mut colors = Self::from_color_scheme(&theme.colors);
        colors.is_dark = theme.is_dark_mode();
        colors
    }
}

// =============================================================================
// Theme-aware overlay utilities
// =============================================================================

/// Get a modal overlay background color that works in both light and dark modes
///
/// For dark mode: black overlay (darkens content behind)
/// For light mode: white overlay (keeps content readable on light backgrounds)
///
/// # Arguments
/// * `theme` - The theme to use for dark/light detection
/// * `opacity` - Alpha value (0-255), e.g., 0x80 for 50%
///
/// # Returns
/// A Rgba color suitable for use with `.bg()`
pub fn modal_overlay_bg(theme: &Theme, opacity: u8) -> Rgba {
    let base_color = if theme.has_dark_colors() {
        0x000000u32 // black for dark mode
    } else {
        0xffffffu32 // white for light mode
    };
    rgba((base_color << 8) | (opacity as u32))
}

/// Get a hover overlay color that works in both light and dark modes
///
/// For dark mode: white overlay (brightens/lifts the element)
/// For light mode: black overlay (darkens/highlights the element)
///
/// # Arguments
/// * `theme` - The theme to use for dark/light detection
/// * `opacity` - Alpha value (0-255), e.g., 0x26 for ~15%
///
/// # Returns
/// A Rgba color suitable for use with `.bg()` on hover
#[allow(dead_code)]
pub fn hover_overlay_bg(theme: &Theme, opacity: u8) -> Rgba {
    let base_color = if theme.has_dark_colors() {
        0xffffffu32 // white for dark mode (brightens)
    } else {
        0x000000u32 // black for light mode (darkens)
    };
    rgba((base_color << 8) | (opacity as u32))
}

#[cfg(test)]
mod tests {
    use super::{ColorScheme, PromptColors};

    #[test]
    fn test_list_item_colors_text_on_accent_uses_text_on_accent_from_scheme() {
        let mut colors = ColorScheme::dark_default();
        colors.text.primary = 0x010203;
        colors.text.on_accent = 0xa1b2c3;

        let list_item_colors = colors.list_item_colors();

        assert_eq!(list_item_colors.text_on_accent, 0xa1b2c3);
        assert_ne!(list_item_colors.text_on_accent, list_item_colors.text_primary);
    }

    #[test]
    fn test_prompt_colors_from_color_scheme_sets_is_dark_for_dark_scheme() {
        let colors = PromptColors::from_color_scheme(&ColorScheme::dark_default());
        assert!(colors.is_dark);
    }

    #[test]
    fn test_prompt_colors_from_color_scheme_sets_is_dark_for_light_scheme() {
        let colors = PromptColors::from_color_scheme(&ColorScheme::light_default());
        assert!(!colors.is_dark);
    }
}
