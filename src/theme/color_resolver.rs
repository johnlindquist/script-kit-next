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
#[allow(dead_code)] // Internal token cache; not all fields are consumed yet.
pub struct ColorResolver {
    // Cached colors extracted from theme or design tokens
    // All colors are stored as u32 hex values (0xRRGGBB)

    // Background colors
    background: u32,
    background_secondary: u32,
    background_tertiary: u32,
    background_selected: u32,
    background_hover: u32,

    // Text colors
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    text_dimmed: u32,
    text_on_accent: u32,

    // Accent colors
    accent: u32,
    accent_secondary: u32,
    success: u32,
    warning: u32,
    error: u32,

    // Border colors
    border: u32,
    border_subtle: u32,
    border_focus: u32,

    // Shadow
    shadow: u32,
}

#[allow(dead_code)]
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
            success = %format_args!("{:#08x}", resolver.success),
            warning = %format_args!("{:#08x}", resolver.warning),
            error = %format_args!("{:#08x}", resolver.error),
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
            background_tertiary: colors.background.search_box,
            background_selected: colors.accent.selected_subtle,
            background_hover: colors.accent.selected_subtle,

            text_primary: colors.text.primary,
            text_secondary: colors.text.secondary,
            text_muted: colors.text.muted,
            text_dimmed: colors.text.dimmed,
            text_on_accent: colors.text.on_accent,

            accent: colors.accent.selected,
            accent_secondary: colors.accent.selected,
            success: colors.ui.success,
            warning: colors.ui.warning,
            error: colors.ui.error,

            border: colors.ui.border,
            border_subtle: colors.ui.border,
            border_focus: colors.accent.selected,

            shadow: 0x00000040, // Default shadow
        }
    }

    /// Create a resolver from design tokens (all other variants)
    fn from_design_tokens(variant: DesignVariant) -> Self {
        let tokens = get_tokens(variant);
        let colors = tokens.colors();

        Self {
            background: colors.background,
            background_secondary: colors.background_secondary,
            background_tertiary: colors.background_tertiary,
            background_selected: colors.background_selected,
            background_hover: colors.background_hover,

            text_primary: colors.text_primary,
            text_secondary: colors.text_secondary,
            text_muted: colors.text_muted,
            text_dimmed: colors.text_dimmed,
            text_on_accent: colors.text_on_accent,

            accent: colors.accent,
            accent_secondary: colors.accent_secondary,
            success: colors.success,
            warning: colors.warning,
            error: colors.error,

            border: colors.border,
            border_subtle: colors.border_subtle,
            border_focus: colors.border_focus,

            shadow: colors.shadow,
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

    /// Get the secondary text color
    pub fn secondary_text_color(&self) -> u32 {
        self.text_secondary
    }

    /// Get the main background color
    pub fn main_background(&self) -> u32 {
        self.background
    }

    /// Get the selection background color
    pub fn selection_background(&self) -> u32 {
        self.background_selected
    }

    /// Get the primary accent color
    pub fn primary_accent(&self) -> u32 {
        self.accent
    }

    /// Get the border color
    pub fn border_color(&self) -> u32 {
        self.border
    }

    /// Get the dimmed text color
    pub fn dimmed_text_color(&self) -> u32 {
        self.text_dimmed
    }

    /// Get the secondary background color
    pub fn secondary_background_color(&self) -> u32 {
        self.background_secondary
    }
}

/// Unified typography resolution that works with both theme and design tokens
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Incremental migration away from direct design token access.
pub struct TypographyResolver {
    font_family: &'static str,
    font_family_mono: &'static str,
    font_size_xs: f32,
    font_size_sm: f32,
    font_size_md: f32,
    font_size_lg: f32,
    font_size_xl: f32,
}

#[allow(dead_code)]
impl TypographyResolver {
    /// Create a new typography resolver for the given theme and design variant
    pub fn new(_theme: &Theme, variant: DesignVariant) -> Self {
        let tokens = get_tokens(variant);
        let typography = tokens.typography();
        Self {
            font_family: typography.font_family,
            font_family_mono: typography.font_family_mono,
            font_size_xs: typography.font_size_xs,
            font_size_sm: typography.font_size_sm,
            font_size_md: typography.font_size_md,
            font_size_lg: typography.font_size_lg,
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
#[allow(dead_code)] // Incremental migration away from direct design token access.
pub struct SpacingResolver {
    padding_xs: f32,
    padding_sm: f32,
    padding_md: f32,
    padding_lg: f32,
    padding_xl: f32,
    gap_sm: f32,
    gap_md: f32,
    gap_lg: f32,
    margin_sm: f32,
    margin_md: f32,
    margin_lg: f32,
}

#[allow(dead_code)]
impl SpacingResolver {
    /// Create a new spacing resolver for the given design variant
    pub fn new(variant: DesignVariant) -> Self {
        let tokens = get_tokens(variant);
        let spacing = tokens.spacing();
        Self {
            padding_xs: spacing.padding_xs,
            padding_sm: spacing.padding_sm,
            padding_md: spacing.padding_md,
            padding_lg: spacing.padding_lg,
            padding_xl: spacing.padding_xl,
            gap_sm: spacing.gap_sm,
            gap_md: spacing.gap_md,
            gap_lg: spacing.gap_lg,
            margin_sm: spacing.margin_sm,
            margin_md: spacing.margin_md,
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
