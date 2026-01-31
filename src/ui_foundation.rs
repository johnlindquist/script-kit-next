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

/// Opacity for vibrancy window backgrounds in dark mode.
/// Lower value (37%) allows more blur to show through while maintaining readability.
pub const VIBRANCY_DARK_OPACITY: f32 = 0.37;

/// Opacity for vibrancy window backgrounds in light mode.
/// Higher value (85%) needed for visibility - matches POC's rgba(0xFAFAFAD9).
pub const VIBRANCY_LIGHT_OPACITY: f32 = 0.85;

/// Get the vibrancy background for window root containers.
///
/// This is used by the main window outer div to tint the macOS blur effect with the
/// theme's background color. Each window (main, AI, Notes) must call this to apply
/// the semi-transparent background that filters the vibrancy blur.
///
/// **NOTE:** Root no longer provides this background (was removed for vibrancy support).
/// Windows must explicitly apply this to their outermost container div.
///
/// # Returns
/// An Rgba color with appropriate opacity for the current theme mode:
/// - Light mode: 85% opacity (matches POC's proven appearance)
/// - Dark mode: 37% opacity (more blur visibility)
///
/// # Example
/// ```ignore
/// let vibrancy_bg = get_window_vibrancy_background();
/// div()
///     .size_full()
///     .bg(vibrancy_bg)  // Tints the blur effect with theme color
///     .child(content)
/// ```
pub fn get_window_vibrancy_background() -> Rgba {
    let theme = crate::theme::load_theme();
    let opacity = if theme.has_dark_colors() {
        VIBRANCY_DARK_OPACITY
    } else {
        VIBRANCY_LIGHT_OPACITY
    };
    let bg_hex = theme.colors.background.main;
    gpui::rgba(hex_to_rgba_with_opacity(bg_hex, opacity))
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

    /// Get design colors based on variant - always uses theme for consistent styling
    pub fn for_variant(_variant: DesignVariant, theme: &Theme) -> Self {
        // Always use theme for consistent styling across all prompts
        Self::from_theme(theme)
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

// ============================================================================
// HexColorExt - Extension trait for u32 hex colors
// ============================================================================

/// Extension trait for u32 hex colors to provide convenient color conversion.
///
/// This eliminates the need for manual `rgb(colors.*)` calls and `<< 8 | alpha`
/// packing throughout the codebase.
///
/// # Example
/// ```ignore
/// use crate::ui_foundation::HexColorExt;
///
/// let colors = theme.colors;
/// // Instead of: rgb(colors.text.primary)
/// // Use: colors.text.primary.to_rgb()
///
/// // Instead of: rgba((colors.border << 8) | 0x80)
/// // Use: colors.border.rgba8(0x80)
///
/// // Instead of manual opacity calculation:
/// // Use: colors.background.with_opacity(0.5)
/// ```
pub trait HexColorExt {
    /// Convert hex color to GPUI Hsla (fully opaque).
    ///
    /// Replaces `rgb(color)` calls.
    fn to_rgb(self) -> Hsla;

    /// Convert hex color to GPUI Hsla with alpha byte (0-255).
    ///
    /// Replaces `rgba((color << 8) | alpha)` patterns.
    fn rgba8(self, alpha: u8) -> Hsla;

    /// Convert hex color to GPUI Hsla with opacity float (0.0-1.0).
    ///
    /// More readable than manual alpha calculation.
    fn with_opacity(self, opacity: f32) -> Hsla;
}

impl HexColorExt for u32 {
    #[inline]
    fn to_rgb(self) -> Hsla {
        gpui::rgb(self).into()
    }

    #[inline]
    fn rgba8(self, alpha: u8) -> Hsla {
        gpui::rgba((self << 8) | alpha as u32).into()
    }

    #[inline]
    fn with_opacity(self, opacity: f32) -> Hsla {
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
        gpui::rgba((self << 8) | alpha as u32).into()
    }
}

// ============================================================================
// Layout Primitives - Free functions for common patterns
// ============================================================================

/// Create a vertical stack (flex column).
///
/// Replaces `div().flex().flex_col()` pattern.
#[inline]
pub fn vstack() -> Div {
    gpui::div().flex().flex_col()
}

/// Create a horizontal stack (flex row with centered items).
///
/// Replaces `div().flex().flex_row().items_center()` pattern.
#[inline]
pub fn hstack() -> Div {
    gpui::div().flex().flex_row().items_center()
}

/// Create a centered container (items centered both axes).
///
/// Replaces `div().flex().items_center().justify_center()` pattern.
#[inline]
pub fn centered() -> Div {
    gpui::div().flex().items_center().justify_center()
}

/// Create a flexible spacer that fills available space.
///
/// Replaces `div().flex_1()` pattern.
#[inline]
pub fn spacer() -> Div {
    gpui::div().flex_1()
}

// ============================================================================
// Key Normalization - Allocation-free key matching
// ============================================================================
//
// IMPORTANT: These helpers use eq_ignore_ascii_case() instead of to_lowercase()
// to avoid allocations on every keystroke. This is a hot path optimization.

/// Check if key is an up arrow (handles both "up" and "arrowup" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_up(key: &str) -> bool {
    key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("arrowup")
}

/// Check if key is a down arrow (handles both "down" and "arrowdown" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_down(key: &str) -> bool {
    key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("arrowdown")
}

/// Check if key is a left arrow (handles both "left" and "arrowleft" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_left(key: &str) -> bool {
    key.eq_ignore_ascii_case("left") || key.eq_ignore_ascii_case("arrowleft")
}

/// Check if key is a right arrow (handles both "right" and "arrowright" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_right(key: &str) -> bool {
    key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright")
}

/// Check if key is Enter/Return.
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_enter(key: &str) -> bool {
    key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("return")
}

/// Check if key is Escape.
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_escape(key: &str) -> bool {
    key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("esc")
}

/// Check if key is Backspace.
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_backspace(key: &str) -> bool {
    key.eq_ignore_ascii_case("backspace")
}

/// Check if key is the "k" key (for Cmd+K shortcut).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_k(key: &str) -> bool {
    key.eq_ignore_ascii_case("k")
}

/// Extract printable character from a KeyDownEvent's key_char field.
///
/// Returns Some(char) if the key_char contains a non-control character,
/// None otherwise (for special keys like arrows, escape, etc.).
#[inline]
pub fn printable_char(key_char: Option<&str>) -> Option<char> {
    key_char
        .and_then(|s| s.chars().next())
        .filter(|ch| !ch.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // HexColorExt Tests
    // ========================================================================

    #[test]
    fn test_hex_color_to_rgb() {
        // White should convert correctly
        let white = 0xFFFFFFu32.to_rgb();
        assert!(
            (white.l - 1.0).abs() < 0.01,
            "White should have lightness ~1.0"
        );

        // Black should convert correctly
        let black = 0x000000u32.to_rgb();
        assert!(black.l < 0.01, "Black should have lightness ~0.0");

        // Alpha should be 1.0 (fully opaque)
        assert!(
            (white.a - 1.0).abs() < 0.001,
            "to_rgb should be fully opaque"
        );
        assert!(
            (black.a - 1.0).abs() < 0.001,
            "to_rgb should be fully opaque"
        );
    }

    #[test]
    fn test_hex_color_rgba8() {
        // Test with 50% alpha (0x80 = 128)
        let semi = 0xFFFFFFu32.rgba8(0x80);
        // Alpha should be approximately 128/255 = 0.502
        assert!(
            (semi.a - 0.502).abs() < 0.01,
            "rgba8(0x80) should have ~50% alpha, got {}",
            semi.a
        );

        // Test with 0 alpha
        let transparent = 0xFFFFFFu32.rgba8(0x00);
        assert!(
            transparent.a < 0.01,
            "rgba8(0x00) should be fully transparent"
        );

        // Test with full alpha
        let opaque = 0xFFFFFFu32.rgba8(0xFF);
        assert!(
            (opaque.a - 1.0).abs() < 0.01,
            "rgba8(0xFF) should be fully opaque"
        );
    }

    #[test]
    fn test_hex_color_with_opacity() {
        // 50% opacity
        let half = 0xFFFFFFu32.with_opacity(0.5);
        assert!(
            (half.a - 0.5).abs() < 0.02,
            "with_opacity(0.5) should have ~50% alpha, got {}",
            half.a
        );

        // 0% opacity
        let transparent = 0xFFFFFFu32.with_opacity(0.0);
        assert!(
            transparent.a < 0.01,
            "with_opacity(0.0) should be fully transparent"
        );

        // 100% opacity
        let opaque = 0xFFFFFFu32.with_opacity(1.0);
        assert!(
            (opaque.a - 1.0).abs() < 0.01,
            "with_opacity(1.0) should be fully opaque"
        );
    }

    #[test]
    fn test_hex_color_opacity_clamping() {
        // Opacity > 1.0 should clamp to 1.0
        let over = 0xFFFFFFu32.with_opacity(1.5);
        assert!(
            (over.a - 1.0).abs() < 0.01,
            "with_opacity(1.5) should clamp to 1.0"
        );

        // Opacity < 0.0 should clamp to 0.0
        let under = 0xFFFFFFu32.with_opacity(-0.5);
        assert!(under.a < 0.01, "with_opacity(-0.5) should clamp to 0.0");
    }

    // ========================================================================
    // Key Normalization Tests (Allocation-free helpers)
    // ========================================================================

    #[test]
    fn test_is_key_up() {
        // All valid forms
        assert!(is_key_up("up"));
        assert!(is_key_up("Up"));
        assert!(is_key_up("UP"));
        assert!(is_key_up("arrowup"));
        assert!(is_key_up("ArrowUp"));
        assert!(is_key_up("ARROWUP"));
        // Invalid
        assert!(!is_key_up("down"));
        assert!(!is_key_up("left"));
        assert!(!is_key_up("enter"));
    }

    #[test]
    fn test_is_key_down() {
        assert!(is_key_down("down"));
        assert!(is_key_down("Down"));
        assert!(is_key_down("DOWN"));
        assert!(is_key_down("arrowdown"));
        assert!(is_key_down("ArrowDown"));
        assert!(is_key_down("ARROWDOWN"));
        assert!(!is_key_down("up"));
        assert!(!is_key_down("right"));
    }

    #[test]
    fn test_is_key_left() {
        assert!(is_key_left("left"));
        assert!(is_key_left("Left"));
        assert!(is_key_left("arrowleft"));
        assert!(is_key_left("ArrowLeft"));
        assert!(!is_key_left("right"));
        assert!(!is_key_left("up"));
    }

    #[test]
    fn test_is_key_right() {
        assert!(is_key_right("right"));
        assert!(is_key_right("Right"));
        assert!(is_key_right("arrowright"));
        assert!(is_key_right("ArrowRight"));
        assert!(!is_key_right("left"));
        assert!(!is_key_right("down"));
    }

    #[test]
    fn test_is_key_enter() {
        assert!(is_key_enter("enter"));
        assert!(is_key_enter("Enter"));
        assert!(is_key_enter("ENTER"));
        assert!(is_key_enter("return"));
        assert!(is_key_enter("Return"));
        assert!(!is_key_enter("escape"));
        assert!(!is_key_enter("space"));
    }

    #[test]
    fn test_is_key_escape() {
        assert!(is_key_escape("escape"));
        assert!(is_key_escape("Escape"));
        assert!(is_key_escape("ESCAPE"));
        assert!(is_key_escape("esc"));
        assert!(is_key_escape("Esc"));
        assert!(!is_key_escape("enter"));
    }

    #[test]
    fn test_is_key_backspace() {
        assert!(is_key_backspace("backspace"));
        assert!(is_key_backspace("Backspace"));
        assert!(is_key_backspace("BACKSPACE"));
        assert!(!is_key_backspace("delete"));
        assert!(!is_key_backspace("enter"));
    }

    #[test]
    fn test_is_key_k() {
        assert!(is_key_k("k"));
        assert!(is_key_k("K"));
        assert!(!is_key_k("j"));
        assert!(!is_key_k("enter"));
    }

    #[test]
    fn test_printable_char() {
        // Normal printable chars
        assert_eq!(printable_char(Some("a")), Some('a'));
        assert_eq!(printable_char(Some("A")), Some('A'));
        assert_eq!(printable_char(Some("1")), Some('1'));
        assert_eq!(printable_char(Some("!")), Some('!'));
        assert_eq!(printable_char(Some(" ")), Some(' '));

        // Control characters should return None
        assert_eq!(printable_char(Some("\n")), None);
        assert_eq!(printable_char(Some("\t")), None);
        assert_eq!(printable_char(Some("\x1b")), None); // ESC

        // Empty/None cases
        assert_eq!(printable_char(None), None);
        assert_eq!(printable_char(Some("")), None);
    }

    // ========================================================================
    // Original Tests
    // ========================================================================

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
