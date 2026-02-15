//! Unified color resolution for theme and design tokens
//!
//! This module provides a unified interface for accessing colors that works
//! across both the default theme system and custom design variants.
//!
//! Instead of checking `is_default_design` everywhere and routing between
//! `theme.colors.*` and `design_tokens.colors().*`, use the `ColorResolver`
//! to provide a single consistent API.
//!
//! # Example
//!
//! Before (dual paths):
//! ```ignore
//! let is_default_design = self.current_design == DesignVariant::Default;
//! let empty_text_color = if is_default_design {
//!     theme.colors.text.muted
//! } else {
//!     design_colors.text_muted
//! };
//! ```
//!
//! After (unified):
//! ```ignore
//! let resolver = ColorResolver::new(&theme, self.current_design);
//! let empty_text_color = resolver.text_muted();
//! ```
//!
//! ## API Visibility Contract
//!
//! Resolver internals are implementation details; call sites should use
//! semantic accessor methods instead of reading struct fields directly.
//! ```compile_fail
//! use script_kit_gpui::designs::DesignVariant;
//! use script_kit_gpui::theme::{ColorResolver, Theme, TypographyResolver};
//!
//! let theme = Theme::default();
//! let colors = ColorResolver::new(&theme, DesignVariant::Default);
//! let _ = colors.text_primary;
//!
//! let typography = TypographyResolver::new(&theme, DesignVariant::Default);
//! let _ = typography.font_family;
//! ```

use crate::designs::{get_tokens, DesignVariant};
use crate::theme::types::Theme;
use tracing::debug;

/// Unified color resolution that works with both theme and design tokens
///
/// This struct provides a single API for color access that automatically
/// routes to the correct source (theme or design tokens) based on the
/// current design variant.
#[derive(Debug, Clone, Copy)]
pub struct ColorResolver {
    // Cached colors extracted from theme or design tokens
    // All colors are stored as u32 hex values (0xRRGGBB)

    // Background colors
    background: u32,
    background_secondary: u32,

    // Text colors
    text_primary: u32,
    text_muted: u32,

    // Accent colors
    accent: u32,

    // Border colors
    border: u32,
}

impl ColorResolver {
    /// Create a new color resolver for the given theme and design variant
    ///
    /// This automatically selects colors from either the theme (for Default variant)
    /// or from design tokens (for all other variants).
    pub fn new(theme: &Theme, variant: DesignVariant) -> Self {
        let (source, resolver) = if variant == DesignVariant::Default {
            ("theme", Self::from_theme(theme))
        } else {
            ("design_tokens", Self::from_design_tokens(variant))
        };

        debug!(
            variant = %variant.name(),
            source,
            background = %format_args!("{:#08x}", resolver.background),
            text_primary = %format_args!("{:#08x}", resolver.text_primary),
            accent = %format_args!("{:#08x}", resolver.accent),
            border = %format_args!("{:#08x}", resolver.border),
            "ColorResolver initialized"
        );

        resolver
    }

    /// Create a resolver from theme colors (Default variant)
    fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        Self {
            background: colors.background.main,
            background_secondary: colors.background.title_bar,

            text_primary: colors.text.primary,
            text_muted: colors.text.muted,

            accent: colors.accent.selected,

            border: colors.ui.border,
        }
    }

    /// Create a resolver from design tokens (all other variants)
    fn from_design_tokens(variant: DesignVariant) -> Self {
        let tokens = get_tokens(variant);
        let colors = tokens.colors();

        Self {
            background: colors.background,
            background_secondary: colors.background_secondary,

            text_primary: colors.text_primary,
            text_muted: colors.text_muted,

            accent: colors.accent,

            border: colors.border,
        }
    }

    // Convenience methods for semantic access

    /// Get the empty state text color (muted text)
    pub fn empty_text_color(&self) -> u32 {
        self.text_muted
    }

    /// Get the primary text color
    pub fn primary_text_color(&self) -> u32 {
        self.text_primary
    }

    /// Get the main background color
    #[cfg(test)]
    pub fn main_background(&self) -> u32 {
        self.background
    }

    /// Get the primary accent color
    pub fn primary_accent(&self) -> u32 {
        self.accent
    }

    /// Get the border color
    pub fn border_color(&self) -> u32 {
        self.border
    }

    /// Get the secondary background color
    pub fn secondary_background_color(&self) -> u32 {
        self.background_secondary
    }
}

/// Unified typography resolution that works with both theme and design tokens
#[derive(Debug, Clone, Copy)]
pub struct TypographyResolver {
    font_family: &'static str,
    font_family_mono: &'static str,
    font_size_xl: f32,
}

impl TypographyResolver {
    /// Create a new typography resolver for the given theme and design variant
    pub fn new(_theme: &Theme, variant: DesignVariant) -> Self {
        let tokens = get_tokens(variant);
        let typography = tokens.typography();
        Self {
            font_family: typography.font_family,
            font_family_mono: typography.font_family_mono,
            font_size_xl: typography.font_size_xl,
        }
    }

    /// Get the primary font family
    pub fn primary_font(&self) -> &'static str {
        self.font_family
    }

    /// Get the monospace font family
    pub fn mono_font(&self) -> &'static str {
        self.font_family_mono
    }

    /// Get extra-large font size token
    pub fn font_size_xl(&self) -> f32 {
        self.font_size_xl
    }
}

/// Unified spacing resolution that works with both theme and design tokens
#[derive(Debug, Clone, Copy)]
pub struct SpacingResolver {
    margin_lg: f32,
}

impl SpacingResolver {
    /// Create a new spacing resolver for the given design variant
    pub fn new(variant: DesignVariant) -> Self {
        let tokens = get_tokens(variant);
        let spacing = tokens.spacing();
        Self {
            margin_lg: spacing.margin_lg,
        }
    }

    /// Get large margin token
    pub fn margin_lg(&self) -> f32 {
        self.margin_lg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_resolver_default_variant() {
        let theme = Theme::default();
        let resolver = ColorResolver::new(&theme, DesignVariant::Default);

        // Should use theme colors
        assert_eq!(resolver.primary_text_color(), theme.colors.text.primary);
        assert_eq!(resolver.empty_text_color(), theme.colors.text.muted);
        assert_eq!(resolver.primary_accent(), theme.colors.accent.selected);
    }

    #[test]
    fn test_color_resolver_minimal_variant() {
        let theme = Theme::default();
        let resolver = ColorResolver::new(&theme, DesignVariant::Minimal);

        // Should use design token colors, not theme colors
        // Minimal has different colors from default
        let tokens = get_tokens(DesignVariant::Minimal);
        let design_colors = tokens.colors();

        assert_eq!(resolver.primary_text_color(), design_colors.text_primary);
        assert_eq!(resolver.empty_text_color(), design_colors.text_muted);
        assert_eq!(resolver.primary_accent(), design_colors.accent);
    }

    #[test]
    fn test_color_resolver_semantic_methods() {
        let theme = Theme::default();
        let resolver = ColorResolver::new(&theme, DesignVariant::Default);

        assert_eq!(resolver.empty_text_color(), theme.colors.text.muted);
        assert_eq!(resolver.primary_text_color(), theme.colors.text.primary);
        assert_eq!(resolver.main_background(), theme.colors.background.main);
    }

    #[test]
    fn test_typography_resolver_default() {
        let theme = Theme::default();
        let resolver = TypographyResolver::new(&theme, DesignVariant::Default);

        assert_eq!(resolver.primary_font(), ".AppleSystemUIFont");
        assert_eq!(resolver.mono_font(), "Menlo");
    }

    #[test]
    fn test_typography_resolver_retro_terminal() {
        let theme = Theme::default();
        let resolver = TypographyResolver::new(&theme, DesignVariant::RetroTerminal);

        // RetroTerminal uses Menlo for everything
        assert_eq!(resolver.primary_font(), "Menlo");
        assert_eq!(resolver.mono_font(), "Menlo");
    }

    #[test]
    fn test_all_variants_have_valid_colors() {
        let theme = Theme::default();
        for variant in DesignVariant::all() {
            let resolver = ColorResolver::new(&theme, *variant);

            // All variants should have different bg and text for contrast
            assert_ne!(
                resolver.main_background(),
                resolver.primary_text_color(),
                "Variant {:?} has no contrast",
                variant
            );

            // All colors should be valid hex values
            assert!(resolver.primary_text_color() <= 0xFFFFFF);
            assert!(resolver.primary_accent() <= 0xFFFFFF);
        }
    }
}
