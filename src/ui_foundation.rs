//! UI Foundation - Shared UI patterns for consistent vibrancy and layout
//!
//! This module extracts common UI patterns from the main menu (render_script_list.rs)
//! into reusable helpers. The main menu is the "gold standard" for vibrancy support.
//!
//! NOTE: Many items are currently unused as this is a foundation module.
//! They will be used as other modules are refactored to use the shared patterns.
#![allow(dead_code)]
//!
//! # Key Vibrancy Pattern (from render_script_list.rs:699-707)
//!
//! ```ignore
//! // VIBRANCY: Remove background from content div - let gpui-component Root's
//! // semi-transparent background handle vibrancy effect. Content areas should NOT
//! // have their own backgrounds to allow blur to show through.
//! let _bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
//!
//! let mut main_div = div()
//!     .flex()
//!     .flex_col()
//!     // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
//!     .shadow(box_shadows)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui_foundation::{get_vibrancy_background, container_div, content_div};
//!
//! // In your render function:
//! let bg = get_vibrancy_background(&theme);
//! let container = container_div()
//!     .when_some(bg, |d, bg| d.bg(bg))
//!     .child(content_div().child(...));
//! ```

use gpui::{px, Div, Hsla, Rgba, Styled};

use crate::designs::{get_tokens, DesignColors, DesignSpacing, DesignVariant};
use crate::theme::{ColorScheme, Theme};

/// Convert a hex color (u32) to RGBA with specified opacity.
///
/// This is the standard way to create semi-transparent colors for vibrancy support.
/// The hex color provides RGB values, and opacity controls the alpha channel.
///
/// # Arguments
/// * `hex` - A u32 hex color (e.g., 0x1E1E1E for dark gray)
/// * `opacity` - Alpha value from 0.0 (transparent) to 1.0 (opaque)
///
/// # Returns
/// A u32 suitable for use with `gpui::rgba()` - format is 0xRRGGBBAA
///
/// # Example (from main menu)
/// ```ignore
/// let bg_hex = theme.colors.background.main; // 0x1E1E1E
/// let opacity = theme.get_opacity().main;     // 0.30
/// let bg_with_alpha = hex_to_rgba_with_opacity(bg_hex, opacity);
/// // Result: 0x1E1E1E4D (30% opacity)
/// ```
#[inline]
pub fn hex_to_rgba_with_opacity(hex: u32, opacity: f32) -> u32 {
    // Convert opacity (0.0-1.0) to alpha byte (0x00-0xFF)
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
    // Shift hex left 8 bits and add alpha
    (hex << 8) | alpha
}

/// Convert a hex color to HSLA with specified alpha.
///
/// Used when GPUI components expect Hsla instead of Rgba.
///
/// # Arguments
/// * `hex` - A u32 hex color
/// * `alpha` - Alpha value from 0.0 to 1.0
///
/// # Returns
/// An Hsla color with the specified alpha
#[inline]
pub fn hex_to_hsla_with_alpha(hex: u32, alpha: f32) -> Hsla {
    let rgba = gpui::rgb(hex);
    let hsla: Hsla = rgba.into();
    Hsla {
        h: hsla.h,
        s: hsla.s,
        l: hsla.l,
        a: alpha.clamp(0.0, 1.0),
    }
}

/// Get the background color for vibrancy-aware containers.
///
/// **CRITICAL VIBRANCY PATTERN:** When vibrancy is enabled, content divs should NOT
/// have their own backgrounds. Instead, they rely on the gpui-component Root wrapper
/// to provide a semi-transparent background that allows blur to show through.
///
/// # Arguments
/// * `theme` - The current theme
///
/// # Returns
/// * `None` when vibrancy is enabled (let Root handle the background)
/// * `Some(Rgba)` when vibrancy is disabled (use solid background)
///
/// # Example (from main menu render_script_list.rs)
/// ```ignore
/// let bg = get_vibrancy_background(&self.theme);
/// let main_div = div()
///     .flex()
///     .flex_col()
///     .when_some(bg, |d, bg| d.bg(bg)) // Only apply bg when vibrancy disabled
///     .shadow(box_shadows);
/// ```
pub fn get_vibrancy_background(theme: &Theme) -> Option<Rgba> {
    if theme.is_vibrancy_enabled() {
        // VIBRANCY: Let Root's semi-transparent background handle blur
        None
    } else {
        // No vibrancy: use solid background
        Some(gpui::rgb(theme.colors.background.main))
    }
}

/// Get container background with optional opacity for semi-transparent areas.
///
/// Use this for inner containers that need subtle backgrounds even with vibrancy.
/// For example, log panels or input fields that need slight visual separation.
///
/// # Arguments
/// * `theme` - The current theme
/// * `opacity` - Opacity to apply (0.0-1.0)
///
/// # Returns
/// An Rgba color with the specified opacity applied
///
/// # Example
/// ```ignore
/// let log_bg = get_container_background(&theme, theme.get_opacity().log_panel);
/// div().bg(log_bg).child(logs)
/// ```
pub fn get_container_background(theme: &Theme, opacity: f32) -> Rgba {
    let hex = theme.colors.background.main;
    let rgba_u32 = hex_to_rgba_with_opacity(hex, opacity);
    gpui::rgba(rgba_u32)
}

/// Design colors extracted from tokens, ready for use in UI rendering.
///
/// This provides a consistent interface whether using the Default design
/// (which uses theme.colors) or other design variants (which use design tokens).
#[derive(Clone, Copy)]
pub struct UIDesignColors {
    /// Background color (hex)
    pub background: u32,
    /// Primary text color (hex)
    pub text_primary: u32,
    /// Secondary/muted text color (hex)
    pub text_muted: u32,
    /// Dimmed text color (hex)
    pub text_dimmed: u32,
    /// Accent/highlight color (hex)
    pub accent: u32,
    /// Border color (hex)
    pub border: u32,
}

impl UIDesignColors {
    /// Create design colors from theme (for Default design variant)
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            text_primary: theme.colors.text.primary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
        }
    }

    /// Create design colors from design tokens
    pub fn from_design(colors: &DesignColors) -> Self {
        Self {
            background: colors.background,
            text_primary: colors.text_primary,
            text_muted: colors.text_muted,
            text_dimmed: colors.text_dimmed,
            accent: colors.accent,
            border: colors.border,
        }
    }

    /// Get design colors based on variant - uses theme for Default, tokens for others
    pub fn for_variant(variant: DesignVariant, theme: &Theme) -> Self {
        if variant == DesignVariant::Default {
            Self::from_theme(theme)
        } else {
            let tokens = get_tokens(variant);
            Self::from_design(&tokens.colors())
        }
    }
}

/// Get design colors for the current design variant.
///
/// This abstracts the pattern of choosing between theme.colors (Default design)
/// and design tokens (other designs).
///
/// # Arguments
/// * `variant` - The current design variant
/// * `theme` - The current theme
///
/// # Returns
/// Design colors appropriate for the variant
///
/// # Example (from main menu)
/// ```ignore
/// let design_colors = get_design_colors(self.current_design, &self.theme);
/// let text_color = rgb(design_colors.text_primary);
/// ```
pub fn get_design_colors(variant: DesignVariant, theme: &Theme) -> UIDesignColors {
    UIDesignColors::for_variant(variant, theme)
}

/// Get design spacing values for the current design variant.
///
/// # Arguments
/// * `variant` - The current design variant
///
/// # Returns
/// Design spacing tokens
pub fn get_design_spacing(variant: DesignVariant) -> DesignSpacing {
    let tokens = get_tokens(variant);
    tokens.spacing()
}

/// Opacity configuration extracted from theme, with helper methods.
///
/// This wraps the theme's BackgroundOpacity with convenient accessors.
#[derive(Clone, Copy)]
pub struct OpacityConfig {
    /// Main background opacity
    pub main: f32,
    /// Title bar opacity
    pub title_bar: f32,
    /// Search box/input opacity
    pub search_box: f32,
    /// Log panel opacity
    pub log_panel: f32,
    /// Selected item opacity
    pub selected: f32,
    /// Hovered item opacity
    pub hover: f32,
    /// Preview panel opacity
    pub preview: f32,
    /// Dialog/popup opacity
    pub dialog: f32,
    /// Input field opacity
    pub input: f32,
    /// Panel/container opacity
    pub panel: f32,
    /// Input inactive state opacity
    pub input_inactive: f32,
    /// Input active state opacity
    pub input_active: f32,
    /// Border inactive state opacity
    pub border_inactive: f32,
    /// Border active state opacity
    pub border_active: f32,
}

impl OpacityConfig {
    /// Create from theme
    pub fn from_theme(theme: &Theme) -> Self {
        let o = theme.get_opacity();
        Self {
            main: o.main,
            title_bar: o.title_bar,
            search_box: o.search_box,
            log_panel: o.log_panel,
            selected: o.selected,
            hover: o.hover,
            preview: o.preview,
            dialog: o.dialog,
            input: o.input,
            panel: o.panel,
            input_inactive: o.input_inactive,
            input_active: o.input_active,
            border_inactive: o.border_inactive,
            border_active: o.border_active,
        }
    }
}

/// Get opacity configuration from theme.
///
/// # Example
/// ```ignore
/// let opacity = get_opacity_config(&theme);
/// let bg = hex_to_rgba_with_opacity(bg_hex, opacity.main);
/// ```
pub fn get_opacity_config(theme: &Theme) -> OpacityConfig {
    OpacityConfig::from_theme(theme)
}

// ============================================================================
// Layout Primitives
// ============================================================================

/// Create a standard container div with flex column layout.
///
/// This is the base pattern for main content containers, matching the
/// main menu's structure.
///
/// # Returns
/// A `Div` configured with:
/// - `flex()` - Enable flexbox
/// - `flex_col()` - Column direction
/// - `w_full()` - Full width
/// - `h_full()` - Full height
///
/// # Example (from main menu)
/// ```ignore
/// let main_div = container_div()
///     .shadow(box_shadows)
///     .rounded(px(border_radius))
///     .child(...);
/// ```
pub fn container_div() -> Div {
    gpui::div().flex().flex_col().w_full().h_full()
}

/// Create a content area div with proper overflow handling.
///
/// Use this for content areas that may need scrolling or contain lists.
/// The `min_h(px(0.))` is critical for proper flex shrinking.
///
/// # Returns
/// A `Div` configured with:
/// - `flex()` - Enable flexbox
/// - `flex_col()` - Column direction
/// - `flex_1()` - Grow to fill available space
/// - `w_full()` - Full width
/// - `min_h(px(0.))` - Critical: allows flex container to shrink properly
/// - `overflow_hidden()` - Clip overflow content
///
/// # Example (from main menu)
/// ```ignore
/// main_div = main_div.child(
///     content_div()
///         .flex_row() // Override to row for split layout
///         .child(list_panel)
///         .child(preview_panel)
/// );
/// ```
pub fn content_div() -> Div {
    gpui::div()
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .min_h(px(0.)) // Critical: allows flex container to shrink properly
        .overflow_hidden()
}

/// Create a panel div for split-view layouts (like list/preview).
///
/// # Arguments
/// * `width_fraction` - The width as a fraction (e.g., 0.5 for half width)
///
/// # Returns
/// A `Div` configured for panel layout with proper shrinking
pub fn panel_div() -> Div {
    gpui::div()
        .h_full()
        .min_h(px(0.)) // Allow shrinking
        .overflow_hidden()
}

// ============================================================================
// Color Scheme Helpers
// ============================================================================

/// Extension trait for ColorScheme to provide convenient color access.
pub trait ColorSchemeExt {
    /// Get text color for selection state
    fn text_for_selection(&self, is_selected: bool) -> u32;

    /// Get description color for selection state
    fn description_for_selection(&self, is_selected: bool) -> u32;
}

impl ColorSchemeExt for ColorScheme {
    fn text_for_selection(&self, is_selected: bool) -> u32 {
        if is_selected {
            self.text.primary
        } else {
            self.text.secondary
        }
    }

    fn description_for_selection(&self, is_selected: bool) -> u32 {
        if is_selected {
            self.accent.selected // Use accent color for selected item description
        } else {
            self.text.secondary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgba_with_opacity() {
        // Test 30% opacity (0.30 * 255 = 76.5 -> truncates to 76 = 0x4C)
        let result = hex_to_rgba_with_opacity(0x1E1E1E, 0.30);
        assert_eq!(result, 0x1E1E1E4C);

        // Test full opacity
        let result = hex_to_rgba_with_opacity(0xFFFFFF, 1.0);
        assert_eq!(result, 0xFFFFFFFF);

        // Test zero opacity
        let result = hex_to_rgba_with_opacity(0x000000, 0.0);
        assert_eq!(result, 0x00000000);

        // Test 50% opacity (0.5 * 255 = 127.5 -> truncates to 127 = 0x7F)
        let result = hex_to_rgba_with_opacity(0xABCDEF, 0.5);
        assert_eq!(result, 0xABCDEF7F);
    }

    #[test]
    fn test_opacity_clamping() {
        // Test opacity > 1.0 gets clamped
        let result = hex_to_rgba_with_opacity(0x123456, 1.5);
        assert_eq!(result, 0x123456FF);

        // Test opacity < 0.0 gets clamped
        let result = hex_to_rgba_with_opacity(0x123456, -0.5);
        assert_eq!(result, 0x12345600);
    }

    #[test]
    fn test_vibrancy_background_with_default_theme() {
        let theme = Theme::default();
        // Default theme has vibrancy enabled
        let bg = get_vibrancy_background(&theme);
        // Should return None when vibrancy is enabled
        assert!(bg.is_none());
    }
}
