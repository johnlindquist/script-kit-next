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

/// Strategy for how the color resolver picks its color source.
///
/// - `VariantAware`: Default variant → theme colors; other variants → design tokens.
/// - `ThemeFirst`: Always uses the active theme colors regardless of design variant.
///   Use this for shell surfaces that must visually track the active theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceColorStrategy {
    /// Default variant uses theme colors; non-default variants use design tokens.
    VariantAware,
    /// Always uses theme colors — design variant only affects spacing/shape/typography.
    ThemeFirst,
}

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
    /// Create a color resolver with an explicit strategy.
    ///
    /// This is the canonical entry point — all other constructors delegate here.
    pub fn new_with_strategy(
        theme: &Theme,
        variant: DesignVariant,
        strategy: SurfaceColorStrategy,
    ) -> Self {
        let (source, resolver) = match strategy {
            SurfaceColorStrategy::ThemeFirst => ("theme_first", Self::from_theme(theme)),
            SurfaceColorStrategy::VariantAware => {
                if variant == DesignVariant::Default {
                    ("theme", Self::from_theme(theme))
                } else {
                    ("design_tokens", Self::from_design_tokens(variant))
                }
            }
        };

        debug!(
            variant = %variant.name(),
            strategy = ?strategy,
            source,
            background = %format_args!("{:#08x}", resolver.background),
            text_primary = %format_args!("{:#08x}", resolver.text_primary),
            accent = %format_args!("{:#08x}", resolver.accent),
            border = %format_args!("{:#08x}", resolver.border),
            "ColorResolver initialized"
        );

        resolver
    }

    /// Create a new color resolver for the given theme and design variant
    ///
    /// This automatically selects colors from either the theme (for Default variant)
    /// or from design tokens (for all other variants).
    pub fn new(theme: &Theme, variant: DesignVariant) -> Self {
        Self::new_with_strategy(theme, variant, SurfaceColorStrategy::VariantAware)
    }

    /// Create a theme-first color resolver that always uses the active theme colors,
    /// regardless of the current design variant.
    ///
    /// Use this for surfaces that must visually track the active theme (e.g. prompt
    /// shells, theme chooser-adjacent views, windows that should match the live
    /// chooser preview). Changing the active theme will update text, background,
    /// accent, and border colors on these surfaces even when the design variant
    /// is not `Default`.
    pub fn new_theme_first(theme: &Theme, variant: DesignVariant) -> Self {
        Self::new_with_strategy(theme, variant, SurfaceColorStrategy::ThemeFirst)
    }

    /// Create a theme-first color resolver for shell chrome.
    ///
    /// Shell surfaces (main menu, prompt frames) should always follow the active
    /// theme. Non-default design variants still control spacing, density, and shape
    /// via `SpacingResolver` — only colors come from the theme.
    pub fn new_for_shell(theme: &Theme, variant: DesignVariant) -> Self {
        Self::new_with_strategy(theme, variant, SurfaceColorStrategy::ThemeFirst)
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
    #[allow(dead_code)]
    pub fn border_color(&self) -> u32 {
        self.border
    }

    /// Get the secondary background color
    #[allow(dead_code)]
    pub fn secondary_background_color(&self) -> u32 {
        self.background_secondary
    }
}

/// Unified typography resolution that works with both theme and design tokens
#[derive(Debug, Clone)]
pub struct TypographyResolver {
    font_family: String,
    #[cfg_attr(not(test), allow(dead_code))]
    font_family_mono: String,
    font_size_xl: f32,
}

impl TypographyResolver {
    /// Create a new typography resolver for the given theme and design variant
    pub fn new(theme: &Theme, variant: DesignVariant) -> Self {
        match variant {
            DesignVariant::Default => Self::from_theme(theme),
            _ => Self::from_design_tokens(variant),
        }
    }

    /// Create a theme-first typography resolver that always uses the active
    /// theme's font configuration, regardless of the current design variant.
    ///
    /// Use this alongside `ColorResolver::new_for_shell` so that shell chrome
    /// fully tracks the active theme's visual identity.
    pub fn new_theme_first(theme: &Theme, variant: DesignVariant) -> Self {
        let resolver = Self::from_theme(theme);
        debug!(
            variant = %variant.name(),
            source = "theme_first",
            font_family = %resolver.font_family,
            mono_font_family = %resolver.font_family_mono,
            font_size_xl = resolver.font_size_xl,
            "TypographyResolver initialized"
        );
        resolver
    }

    /// Create a resolver from theme fonts (Default variant)
    fn from_theme(theme: &Theme) -> Self {
        let fonts = theme.get_fonts();
        Self {
            font_family: fonts.ui_family,
            font_family_mono: fonts.mono_family,
            font_size_xl: (fonts.ui_size * 1.25).clamp(1.0, 200.0),
        }
    }

    /// Create a resolver from design tokens (all other variants)
    fn from_design_tokens(variant: DesignVariant) -> Self {
        let t = get_tokens(variant).typography();

        Self {
            font_family: t.font_family.to_string(),
            font_family_mono: t.font_family_mono.to_string(),
            font_size_xl: t.font_size_xl,
        }
    }

    /// Get the primary font family
    pub fn primary_font(&self) -> &str {
        &self.font_family
    }

    /// Get the monospace font family
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn mono_font(&self) -> &str {
        &self.font_family_mono
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
        let fonts = theme.get_fonts();
        let resolver = TypographyResolver::new(&theme, DesignVariant::Default);

        assert_eq!(resolver.primary_font(), fonts.ui_family.as_str());
        assert_eq!(resolver.mono_font(), fonts.mono_family.as_str());
    }

    #[test]
    fn test_typography_resolver_retro_terminal() {
        let theme = Theme::default();
        let fonts = theme.get_fonts();
        let default_resolver = TypographyResolver::new(&theme, DesignVariant::Default);
        let retro_resolver = TypographyResolver::new(&theme, DesignVariant::RetroTerminal);

        assert_eq!(default_resolver.primary_font(), fonts.ui_family.as_str());
        assert_eq!(default_resolver.mono_font(), fonts.mono_family.as_str());
        assert_eq!(retro_resolver.primary_font(), "Menlo");
        assert_eq!(retro_resolver.mono_font(), "Menlo");
    }

    #[test]
    fn test_typography_resolver_paper() {
        let theme = Theme::default();
        let paper_resolver = TypographyResolver::new(&theme, DesignVariant::Paper);

        assert_eq!(paper_resolver.primary_font(), "Georgia");
    }

    #[test]
    fn test_typography_resolver_brutalist() {
        let theme = Theme::default();
        let brutalist_resolver = TypographyResolver::new(&theme, DesignVariant::Brutalist);

        assert_eq!(brutalist_resolver.primary_font(), "Helvetica Neue");
    }

    #[test]
    fn test_theme_first_ignores_design_tokens() {
        let theme = Theme::default();

        // Standard resolver for Minimal uses design tokens
        let standard = ColorResolver::new(&theme, DesignVariant::Minimal);
        let tokens = get_tokens(DesignVariant::Minimal);
        let design_colors = tokens.colors();
        assert_eq!(standard.primary_text_color(), design_colors.text_primary);

        // Theme-first resolver for Minimal still uses theme colors
        let theme_first = ColorResolver::new_theme_first(&theme, DesignVariant::Minimal);
        assert_eq!(theme_first.primary_text_color(), theme.colors.text.primary);
        assert_eq!(theme_first.empty_text_color(), theme.colors.text.muted);
        assert_eq!(theme_first.primary_accent(), theme.colors.accent.selected);
        assert_eq!(theme_first.main_background(), theme.colors.background.main);
        assert_eq!(theme_first.border_color(), theme.colors.ui.border);
    }

    #[test]
    fn test_theme_first_matches_default_variant() {
        let theme = Theme::default();
        let default_resolver = ColorResolver::new(&theme, DesignVariant::Default);
        let theme_first = ColorResolver::new_theme_first(&theme, DesignVariant::Minimal);

        // Both should produce identical colors since both use theme
        assert_eq!(
            default_resolver.primary_text_color(),
            theme_first.primary_text_color()
        );
        assert_eq!(
            default_resolver.primary_accent(),
            theme_first.primary_accent()
        );
        assert_eq!(
            default_resolver.main_background(),
            theme_first.main_background()
        );
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

    #[test]
    fn test_new_with_strategy_variant_aware_matches_new() {
        let theme = Theme::default();
        for variant in DesignVariant::all() {
            let via_new = ColorResolver::new(&theme, *variant);
            let via_strategy = ColorResolver::new_with_strategy(
                &theme,
                *variant,
                SurfaceColorStrategy::VariantAware,
            );
            assert_eq!(
                via_new.primary_text_color(),
                via_strategy.primary_text_color(),
                "VariantAware strategy should match new() for {:?}",
                variant
            );
            assert_eq!(via_new.primary_accent(), via_strategy.primary_accent());
            assert_eq!(via_new.main_background(), via_strategy.main_background());
        }
    }

    #[test]
    fn test_new_with_strategy_theme_first_matches_new_theme_first() {
        let theme = Theme::default();
        for variant in DesignVariant::all() {
            let via_old = ColorResolver::new_theme_first(&theme, *variant);
            let via_strategy = ColorResolver::new_with_strategy(
                &theme,
                *variant,
                SurfaceColorStrategy::ThemeFirst,
            );
            assert_eq!(
                via_old.primary_text_color(),
                via_strategy.primary_text_color(),
            );
            assert_eq!(via_old.primary_accent(), via_strategy.primary_accent());
            assert_eq!(via_old.main_background(), via_strategy.main_background());
        }
    }

    #[test]
    fn test_new_for_shell_keeps_theme_colors_under_non_default_variant() {
        // This is the key acceptance criterion: shell chrome must use theme
        // colors even when a non-default design variant is active.
        let theme = Theme::default();

        for variant in DesignVariant::all() {
            let shell = ColorResolver::new_for_shell(&theme, *variant);

            // Shell always resolves to the active theme's colors
            assert_eq!(
                shell.primary_text_color(),
                theme.colors.text.primary,
                "new_for_shell should use theme text.primary for {:?}",
                variant
            );
            assert_eq!(
                shell.primary_accent(),
                theme.colors.accent.selected,
                "new_for_shell should use theme accent for {:?}",
                variant
            );
            assert_eq!(
                shell.main_background(),
                theme.colors.background.main,
                "new_for_shell should use theme background for {:?}",
                variant
            );
        }
    }

    #[test]
    fn test_typography_new_theme_first_uses_theme_fonts() {
        let theme = Theme::default();
        let fonts = theme.get_fonts();

        // For a non-default variant, regular new() uses design tokens
        let regular = TypographyResolver::new(&theme, DesignVariant::RetroTerminal);
        assert_eq!(regular.primary_font(), "Menlo");

        // Theme-first always uses the theme's fonts
        let theme_first = TypographyResolver::new_theme_first(&theme, DesignVariant::RetroTerminal);
        assert_eq!(
            theme_first.primary_font(),
            fonts.ui_family.as_str(),
            "new_theme_first should use theme font, not design token font"
        );
    }
}
