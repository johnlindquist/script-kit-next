//! Semantic theme types
//!
//! This module provides semantic abstraction over raw theme colors:
//! - `FocusAware<T>` - Generic wrapper for focus-dependent values
//! - `SemanticColors` - Semantic color tokens for UI components
//! - `Surface` - Surface type for vibrancy-aware background rendering
//! - `SurfaceStyle` - Computed style for a surface including bg, border, shadow
//!
//! These types are designed for incremental adoption - existing code continues
//! to work while new code can use the semantic layer.

#![allow(dead_code)] // Types are designed for incremental adoption

use gpui::Hsla;
use serde::{Deserialize, Serialize};
use tracing::debug;

// ============================================================================
// FocusAware<T> - Generic focus state wrapper
// ============================================================================

/// Generic wrapper for values that differ based on window focus state.
///
/// This pattern allows any type to have focused and unfocused variants,
/// with a simple accessor method that picks the right one.
///
/// # Example
///
/// ```ignore
/// let colors = FocusAware {
///     focused: SemanticColors::dark(),
///     unfocused: SemanticColors::dark().dimmed(),
/// };
///
/// // In render, pick based on window focus
/// let current = colors.for_focus(window.is_focused());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FocusAware<T> {
    /// Value when window is focused
    pub focused: T,
    /// Value when window is unfocused
    pub unfocused: T,
}

impl<T> FocusAware<T> {
    /// Create a new FocusAware with both variants
    pub fn new(focused: T, unfocused: T) -> Self {
        Self { focused, unfocused }
    }

    /// Get the appropriate value based on focus state
    #[inline]
    pub fn for_focus(&self, is_focused: bool) -> &T {
        if is_focused {
            &self.focused
        } else {
            &self.unfocused
        }
    }

    /// Get a mutable reference based on focus state
    #[inline]
    pub fn for_focus_mut(&mut self, is_focused: bool) -> &mut T {
        if is_focused {
            &mut self.focused
        } else {
            &mut self.unfocused
        }
    }
}

impl<T: Clone> FocusAware<T> {
    /// Create a FocusAware where both states use the same value
    pub fn uniform(value: T) -> Self {
        Self {
            focused: value.clone(),
            unfocused: value,
        }
    }
}

impl<T: Default> Default for FocusAware<T> {
    fn default() -> Self {
        Self {
            focused: T::default(),
            unfocused: T::default(),
        }
    }
}

impl<T: Copy> Copy for FocusAware<T> {}

// ============================================================================
// Surface - UI surface types for vibrancy-aware rendering
// ============================================================================

/// UI surface type for selecting appropriate background styling.
///
/// Each surface type maps to specific vibrancy and opacity settings,
/// ensuring consistent appearance across the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    /// Main application background
    #[default]
    App,
    /// Sidebar/navigation background
    Sidebar,
    /// Panel/container background
    Panel,
    /// Input field background
    Input,
    /// Elevated surface (dialogs, popovers, dropdowns)
    Elevated,
    /// List item background
    ListItem,
    /// Header/toolbar background
    Header,
}

/// Computed style for a surface.
///
/// Contains the resolved background, border, and shadow for a given surface
/// and focus state. Components use this instead of computing vibrancy manually.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceStyle {
    /// Background color (with vibrancy opacity applied if enabled)
    pub background: Hsla,
    /// Border color
    pub border: Hsla,
    /// Whether to show shadow for this surface
    pub has_shadow: bool,
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        Self {
            background: gpui::hsla(0.0, 0.0, 0.1, 1.0),
            border: gpui::hsla(0.0, 0.0, 0.2, 1.0),
            has_shadow: false,
        }
    }
}

// ============================================================================
// SemanticColors - Semantic color token system
// ============================================================================

/// Semantic color tokens for UI components.
///
/// These tokens provide meaning-based color naming rather than
/// appearance-based, allowing themes to maintain semantic consistency
/// while varying the actual colors.
///
/// # Categories
///
/// - `bg_*` - Background colors
/// - `text_*` - Text colors
/// - `border_*` - Border colors
/// - `status_*` - Status/feedback colors
/// - `overlay_*` - Modal/overlay colors
#[derive(Debug, Clone)]
pub struct SemanticColors {
    // Background tokens
    /// Primary background (main content area)
    pub bg_primary: Hsla,
    /// Secondary background (sidebars, panels)
    pub bg_secondary: Hsla,
    /// Tertiary background (nested panels)
    pub bg_tertiary: Hsla,
    /// Elevated background (dialogs, popovers)
    pub bg_elevated: Hsla,
    /// Input field background
    pub bg_input: Hsla,
    /// Selected item background
    pub bg_selected: Hsla,
    /// Hovered item background
    pub bg_hover: Hsla,

    // Text tokens
    /// Primary text (high contrast, main content)
    pub text_primary: Hsla,
    /// Secondary text (lower contrast, descriptions)
    pub text_secondary: Hsla,
    /// Muted text (hints, placeholders)
    pub text_muted: Hsla,
    /// Disabled text
    pub text_disabled: Hsla,
    /// Accent text (links, highlights)
    pub text_accent: Hsla,

    // Border tokens
    /// Default border
    pub border_default: Hsla,
    /// Subtle border (dividers)
    pub border_subtle: Hsla,
    /// Focused border
    pub border_focus: Hsla,
    /// Selected item border
    pub border_selected: Hsla,

    // Status tokens
    /// Success state
    pub status_success: Hsla,
    /// Error state
    pub status_error: Hsla,
    /// Warning state
    pub status_warning: Hsla,
    /// Info state
    pub status_info: Hsla,

    // Overlay tokens
    /// Modal scrim/backdrop
    pub overlay_scrim: Hsla,
    /// Selection highlight glow
    pub overlay_highlight: Hsla,
    /// Drop shadow color
    pub shadow_color: Hsla,
    /// Focus ring color
    pub focus_ring: Hsla,
}

impl Default for SemanticColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl SemanticColors {
    /// Dark mode semantic colors (Script Kit default)
    pub fn dark() -> Self {
        let colors = Self {
            // Backgrounds
            bg_primary: gpui::hsla(0.0, 0.0, 0.118, 1.0), // #1e1e1e
            bg_secondary: gpui::hsla(0.0, 0.0, 0.176, 1.0), // #2d2d30
            bg_tertiary: gpui::hsla(0.0, 0.0, 0.235, 1.0), // #3c3c3c
            bg_elevated: gpui::hsla(0.0, 0.0, 0.15, 1.0), // slightly lighter than primary
            bg_input: gpui::hsla(0.0, 0.0, 0.235, 1.0),   // #3c3c3c
            bg_selected: gpui::hsla(0.0, 0.0, 0.165, 1.0), // #2a2a2a
            bg_hover: gpui::hsla(0.0, 0.0, 0.165, 0.5),   // #2a2a2a at 50%

            // Text
            text_primary: gpui::hsla(0.0, 0.0, 1.0, 1.0), // #ffffff
            text_secondary: gpui::hsla(0.0, 0.0, 0.8, 1.0), // #cccccc
            text_muted: gpui::hsla(0.0, 0.0, 0.5, 1.0),   // #808080
            text_disabled: gpui::hsla(0.0, 0.0, 0.4, 1.0), // #666666
            text_accent: gpui::hsla(43.0 / 360.0, 0.96, 0.56, 1.0), // #fbbf24 (Script Kit gold)

            // Borders
            border_default: gpui::hsla(0.0, 0.0, 0.275, 1.0), // #464647
            border_subtle: gpui::hsla(0.0, 0.0, 0.2, 1.0),
            border_focus: gpui::hsla(43.0 / 360.0, 0.96, 0.56, 1.0), // accent
            border_selected: gpui::hsla(43.0 / 360.0, 0.96, 0.56, 0.5),

            // Status
            status_success: gpui::hsla(120.0 / 360.0, 1.0, 0.5, 1.0), // #00ff00
            status_error: gpui::hsla(0.0, 0.84, 0.60, 1.0),           // #ef4444
            status_warning: gpui::hsla(38.0 / 360.0, 0.92, 0.50, 1.0), // #f59e0b
            status_info: gpui::hsla(217.0 / 360.0, 0.91, 0.60, 1.0),  // #3b82f6

            // Overlays
            overlay_scrim: gpui::hsla(0.0, 0.0, 0.0, 0.5), // black at 50%
            overlay_highlight: gpui::hsla(43.0 / 360.0, 0.96, 0.56, 0.2), // accent at 20%
            shadow_color: gpui::hsla(0.0, 0.0, 0.0, 0.25), // black at 25%
            focus_ring: gpui::hsla(43.0 / 360.0, 0.96, 0.56, 0.5), // accent at 50%
        };

        debug!(
            mode = "dark",
            base_hue = colors.bg_primary.h,
            base_saturation = colors.bg_primary.s,
            accent_hue = colors.text_accent.h,
            accent_saturation = colors.text_accent.s,
            "SemanticColors palette resolved"
        );

        colors
    }

    /// Light mode semantic colors
    pub fn light() -> Self {
        let colors = Self {
            // Backgrounds
            bg_primary: gpui::hsla(0.0, 0.0, 1.0, 1.0), // #ffffff
            bg_secondary: gpui::hsla(0.0, 0.0, 0.953, 1.0), // #f3f3f3
            bg_tertiary: gpui::hsla(0.0, 0.0, 0.925, 1.0), // #ececec
            bg_elevated: gpui::hsla(0.0, 0.0, 1.0, 1.0),
            bg_input: gpui::hsla(0.0, 0.0, 0.925, 1.0),
            bg_selected: gpui::hsla(0.0, 0.0, 0.91, 1.0), // #e8e8e8
            bg_hover: gpui::hsla(0.0, 0.0, 0.91, 0.5),

            // Text
            text_primary: gpui::hsla(0.0, 0.0, 0.0, 1.0), // #000000
            text_secondary: gpui::hsla(0.0, 0.0, 0.2, 1.0), // #333333
            text_muted: gpui::hsla(0.0, 0.0, 0.6, 1.0),   // #999999
            text_disabled: gpui::hsla(0.0, 0.0, 0.8, 1.0), // #cccccc
            text_accent: gpui::hsla(210.0 / 360.0, 1.0, 0.42, 1.0), // #0078d4 (blue for light)

            // Borders
            border_default: gpui::hsla(0.0, 0.0, 0.816, 1.0), // #d0d0d0
            border_subtle: gpui::hsla(0.0, 0.0, 0.9, 1.0),
            border_focus: gpui::hsla(210.0 / 360.0, 1.0, 0.42, 1.0),
            border_selected: gpui::hsla(210.0 / 360.0, 1.0, 0.42, 0.5),

            // Status
            status_success: gpui::hsla(120.0 / 360.0, 1.0, 0.31, 1.0), // #00a000
            status_error: gpui::hsla(0.0, 0.72, 0.51, 1.0),            // #dc2626
            status_warning: gpui::hsla(38.0 / 360.0, 0.84, 0.44, 1.0), // #d97706
            status_info: gpui::hsla(217.0 / 360.0, 0.83, 0.53, 1.0),   // #2563eb

            // Overlays
            overlay_scrim: gpui::hsla(0.0, 0.0, 0.0, 0.3),
            overlay_highlight: gpui::hsla(210.0 / 360.0, 1.0, 0.42, 0.15),
            shadow_color: gpui::hsla(0.0, 0.0, 0.0, 0.15),
            focus_ring: gpui::hsla(210.0 / 360.0, 1.0, 0.42, 0.4),
        };

        debug!(
            mode = "light",
            base_hue = colors.bg_primary.h,
            base_saturation = colors.bg_primary.s,
            accent_hue = colors.text_accent.h,
            accent_saturation = colors.text_accent.s,
            "SemanticColors palette resolved"
        );

        colors
    }

    /// Create a dimmed version of these colors (for unfocused state)
    pub fn dimmed(&self) -> Self {
        const SATURATION_SCALE: f32 = 0.7;
        const ALPHA_SCALE: f32 = 0.9;

        let dim = |c: Hsla| -> Hsla {
            // Reduce saturation and alpha slightly
            gpui::hsla(c.h, c.s * SATURATION_SCALE, c.l, c.a * ALPHA_SCALE)
        };

        debug!(
            mode = "dimmed",
            saturation_scale = SATURATION_SCALE,
            alpha_scale = ALPHA_SCALE,
            base_hue = self.bg_primary.h,
            base_saturation = self.bg_primary.s,
            "SemanticColors dimmed palette generated"
        );

        Self {
            bg_primary: dim(self.bg_primary),
            bg_secondary: dim(self.bg_secondary),
            bg_tertiary: dim(self.bg_tertiary),
            bg_elevated: dim(self.bg_elevated),
            bg_input: dim(self.bg_input),
            bg_selected: dim(self.bg_selected),
            bg_hover: dim(self.bg_hover),

            text_primary: dim(self.text_primary),
            text_secondary: dim(self.text_secondary),
            text_muted: dim(self.text_muted),
            text_disabled: dim(self.text_disabled),
            text_accent: dim(self.text_accent),

            border_default: dim(self.border_default),
            border_subtle: dim(self.border_subtle),
            border_focus: dim(self.border_focus),
            border_selected: dim(self.border_selected),

            status_success: dim(self.status_success),
            status_error: dim(self.status_error),
            status_warning: dim(self.status_warning),
            status_info: dim(self.status_info),

            overlay_scrim: self.overlay_scrim, // Keep scrim unchanged
            overlay_highlight: dim(self.overlay_highlight),
            shadow_color: dim(self.shadow_color),
            focus_ring: dim(self.focus_ring),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // FocusAware tests
    #[test]
    fn test_focus_aware_for_focus() {
        let fa = FocusAware::new(10, 5);
        assert_eq!(*fa.for_focus(true), 10);
        assert_eq!(*fa.for_focus(false), 5);
    }

    #[test]
    fn test_focus_aware_for_focus_mut() {
        let mut fa = FocusAware::new(10, 5);
        *fa.for_focus_mut(true) = 20;
        *fa.for_focus_mut(false) = 3;
        assert_eq!(fa.focused, 20);
        assert_eq!(fa.unfocused, 3);
    }

    #[test]
    fn test_focus_aware_uniform() {
        let fa = FocusAware::uniform(42);
        assert_eq!(*fa.for_focus(true), 42);
        assert_eq!(*fa.for_focus(false), 42);
    }

    #[test]
    fn test_focus_aware_default() {
        let fa: FocusAware<i32> = FocusAware::default();
        assert_eq!(fa.focused, 0);
        assert_eq!(fa.unfocused, 0);
    }

    #[test]
    fn test_focus_aware_copy_for_copyable_types() {
        // FocusAware<T: Copy> should be Copy
        let fa = FocusAware::new(1u32, 2u32);
        let fa2 = fa; // Copy, not move
        assert_eq!(*fa.for_focus(true), 1); // Original still usable
        assert_eq!(*fa2.for_focus(true), 1);
    }

    // Surface tests
    #[test]
    fn test_surface_default() {
        let s = Surface::default();
        assert!(matches!(s, Surface::App));
    }

    #[test]
    fn test_surface_serialization() {
        let s = Surface::Sidebar;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"sidebar\"");
    }

    #[test]
    fn test_surface_deserialization() {
        let s: Surface = serde_json::from_str("\"elevated\"").unwrap();
        assert!(matches!(s, Surface::Elevated));
    }

    // SemanticColors tests
    #[test]
    fn test_semantic_colors_dark_default() {
        let colors = SemanticColors::dark();
        // Primary text should be white (high lightness)
        assert!(colors.text_primary.l > 0.9);
        // Primary background should be dark (low lightness)
        assert!(colors.bg_primary.l < 0.2);
    }

    #[test]
    fn test_semantic_colors_light() {
        let colors = SemanticColors::light();
        // Primary text should be black (low lightness)
        assert!(colors.text_primary.l < 0.1);
        // Primary background should be white (high lightness)
        assert!(colors.bg_primary.l > 0.9);
    }

    #[test]
    fn test_semantic_colors_dimmed() {
        let colors = SemanticColors::dark();
        let dimmed = colors.dimmed();

        // Dimmed version should have lower alpha values
        assert!(dimmed.text_primary.a <= colors.text_primary.a);
    }

    #[test]
    fn test_semantic_colors_has_all_status_colors() {
        let colors = SemanticColors::dark();
        // All status colors should be fully opaque
        assert_eq!(colors.status_success.a, 1.0);
        assert_eq!(colors.status_error.a, 1.0);
        assert_eq!(colors.status_warning.a, 1.0);
        assert_eq!(colors.status_info.a, 1.0);
    }

    #[test]
    fn test_focus_aware_with_semantic_colors() {
        // The main use case: FocusAware<SemanticColors>
        let focused = SemanticColors::dark();
        let unfocused = focused.clone().dimmed();

        let colors = FocusAware::new(focused, unfocused);

        // Focused should have higher alpha
        assert!(colors.for_focus(true).text_primary.a >= colors.for_focus(false).text_primary.a);
    }

    // SurfaceStyle tests
    #[test]
    fn test_surface_style_default() {
        let style = SurfaceStyle::default();
        assert!(!style.has_shadow);
    }
}
