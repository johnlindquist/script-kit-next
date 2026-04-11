//! Lightweight theme extraction helpers
//!
//! These structs pre-compute theme values for efficient use in render closures.
//! They implement Copy to avoid heap allocations when captured by closures.

use gpui::{rgba, Rgba};
use tracing::debug;

use super::types::{relative_luminance_srgb, ColorScheme, Theme};
pub use crate::list_item::ListItemColors;

/// Canonical accent palette used by theme customization UIs.
pub const ACCENT_PALETTE: &[(u32, &str)] = &[
    (0xFBBF24, "Amber"),
    (0x3B82F6, "Blue"),
    (0x8B5CF6, "Violet"),
    (0xEC4899, "Pink"),
    (0xEF4444, "Red"),
    (0xF97316, "Orange"),
    (0x22C55E, "Green"),
    (0x14B8A6, "Teal"),
    (0x06B6D4, "Cyan"),
    (0x6366F1, "Indigo"),
];

/// Additional named accents recognized outside the chooser swatch palette.
const ACCENT_NAME_ALIASES: &[(u32, &str)] = &[
    (0xF59E0B, "Amber"),
    (0xA855F7, "Purple"),
    (0x0078D4, "Blue"),
    (0x0EA5E9, "Sky"),
    (0x84CC16, "Lime"),
];

/// Resolve a human-readable accent color name from canonical palette entries.
pub fn accent_color_name(color: u32) -> &'static str {
    ACCENT_PALETTE
        .iter()
        .chain(ACCENT_NAME_ALIASES.iter())
        .find(|(accent, _)| *accent == color)
        .map(|(_, name)| *name)
        .unwrap_or("Custom")
}

/// WCAG 2.1 contrast ratio between two sRGB hex colors.
pub fn contrast_ratio(foreground: u32, background: u32) -> f32 {
    let l1 = relative_luminance_srgb(foreground);
    let l2 = relative_luminance_srgb(background);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Choose the more readable text color (dark or light) for a given background.
pub fn best_readable_text_hex(background: u32) -> u32 {
    let ink = 0x111111;
    let paper = 0xFFFFFF;
    if contrast_ratio(paper, background) >= contrast_ratio(ink, background) {
        paper
    } else {
        ink
    }
}

/// Choose a pure black or pure white text color for a given background.
pub fn hard_readable_text_hex(background: u32) -> u32 {
    let ink = 0x000000;
    let paper = 0xFFFFFF;
    if contrast_ratio(paper, background) >= contrast_ratio(ink, background) {
        paper
    } else {
        ink
    }
}

impl ColorScheme {
    /// Extract only the colors needed for list item rendering
    ///
    /// Uses the canonical list item color struct from `crate::list_item`.
    /// A temporary `Theme` is built to preserve the existing `ColorScheme` API.
    #[cfg(test)]
    pub fn list_item_colors(&self) -> ListItemColors {
        ListItemColors::from_theme(&Theme {
            colors: self.clone(),
            ..Theme::default()
        })
    }
}

impl ColorScheme {
    /// Extract colors for prompt rendering (DivPrompt, etc.)
    pub fn prompt_colors(&self) -> PromptColors {
        PromptColors::from_color_scheme(self)
    }
}

/// Lightweight struct for prompt rendering (DivPrompt HTML content)
///
/// Pre-computes colors needed for rendering HTML elements in prompts.
/// Implements Copy to avoid heap allocations when captured by closures.
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

impl PromptColors {
    /// Create PromptColors from a ColorScheme
    ///
    /// This extracts only the colors needed for rendering HTML prompts.
    /// `is_dark` is derived from the scheme's background luminance.
    pub fn from_color_scheme(colors: &ColorScheme) -> Self {
        #[cfg(debug_assertions)]
        debug!("Extracting prompt colors");

        // Keep dark/light detection aligned with Theme::has_dark_colors() semantics.
        let is_dark = relative_luminance_srgb(colors.background.main) < 0.5;

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
        colors.is_dark = theme.has_dark_colors();
        colors
    }
}

// =============================================================================
// Theme-aware overlay utilities
// =============================================================================

/// Get a modal overlay color derived from theme colors.
///
/// Uses whichever of the theme's background/text colors is darker so modal dimming
/// remains consistent for both light and dark themes.
///
/// # Arguments
/// * `theme` - The theme to use for dark/light detection
/// * `opacity` - Alpha value (0-255), e.g., 0x80 for 50%
///
/// # Returns
/// A Rgba color suitable for use with `.bg()`
pub fn modal_overlay_bg(theme: &Theme, opacity: u8) -> Rgba {
    let background_color = theme.colors.background.main;
    let foreground_color = theme.colors.text.primary;
    let base_color =
        if relative_luminance_srgb(background_color) <= relative_luminance_srgb(foreground_color) {
            background_color
        } else {
            foreground_color
        };
    rgba((base_color << 8) | (opacity as u32))
}

/// Get a hover overlay color derived from theme text color.
///
/// Hover overlays track `text.primary` so hover affordances stay theme-consistent.
///
/// # Arguments
/// * `theme` - The theme supplying the base hover color
/// * `opacity` - Alpha value (0-255), e.g., 0x26 for ~15%
///
/// # Returns
/// A Rgba color suitable for use with `.bg()` on hover
pub fn hover_overlay_bg(theme: &Theme, opacity: u8) -> Rgba {
    let base_color = theme.colors.text.primary;
    rgba((base_color << 8) | (opacity as u32))
}

#[cfg(test)]
mod tests {
    use super::{
        accent_color_name, hover_overlay_bg, modal_overlay_bg, ColorScheme, PromptColors, Theme,
        ACCENT_PALETTE,
    };
    use gpui::rgba;

    #[test]
    fn test_modal_overlay_bg_uses_darker_theme_color_when_background_is_darker() {
        let mut theme = Theme::dark_default();
        theme.colors.background.main = 0x101820;
        theme.colors.text.primary = 0xf0f4f8;

        assert_eq!(modal_overlay_bg(&theme, 0x80), rgba(0x10182080));
    }

    #[test]
    fn test_modal_overlay_bg_uses_darker_theme_color_when_text_is_darker() {
        let mut theme = Theme::light_default();
        theme.colors.background.main = 0xf8fafc;
        theme.colors.text.primary = 0x1f2937;

        assert_eq!(modal_overlay_bg(&theme, 0x66), rgba(0x1f293766));
    }

    #[test]
    fn test_hover_overlay_bg_uses_theme_primary_text_as_base_color() {
        let mut theme = Theme::dark_default();
        theme.colors.text.primary = 0x335577;

        assert_eq!(hover_overlay_bg(&theme, 0x24), rgba(0x33557724));
    }

    #[test]
    fn test_list_item_colors_text_on_accent_uses_text_on_accent_from_scheme() {
        let mut colors = ColorScheme::dark_default();
        colors.text.primary = 0x010203;
        colors.text.on_accent = 0xa1b2c3;

        let list_item_colors = colors.list_item_colors();

        assert_eq!(list_item_colors.text_on_accent, 0xa1b2c3);
        assert_ne!(
            list_item_colors.text_on_accent,
            list_item_colors.text_primary
        );
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

    #[test]
    fn test_accent_color_name_uses_canonical_palette_names() {
        assert!(ACCENT_PALETTE
            .iter()
            .any(|(color, name)| { *color == 0xFBBF24 && *name == "Amber" }));
        assert_eq!(accent_color_name(0xFBBF24), "Amber");
    }

    #[test]
    fn test_accent_color_name_maps_known_aliases() {
        assert_eq!(accent_color_name(0xA855F7), "Purple");
        assert_eq!(accent_color_name(0x0078D4), "Blue");
    }

    #[test]
    fn test_hard_readable_text_hex_returns_pure_white_for_dark_backgrounds() {
        assert_eq!(super::hard_readable_text_hex(0x101820), 0xFFFFFF);
    }

    #[test]
    fn test_hard_readable_text_hex_returns_pure_black_for_light_backgrounds() {
        assert_eq!(super::hard_readable_text_hex(0xF8FAFC), 0x000000);
    }
}
