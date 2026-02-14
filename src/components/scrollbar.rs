//! Minimal native-style scrollbar component for GPUI
//!
//! This module provides a semi-transparent scrollbar that can overlay on uniform_list
//! or other scrollable containers. The scrollbar is designed to be thin and unobtrusive,
//! matching the native macOS aesthetic.
//!
//! # Features
//!
//! - Thin vertical bar (4-6px width) on the right edge
//! - Shows thumb position/size based on scroll state
//! - Semi-transparent and only visible when content overflows
//! - Theme-aware colors
//!

#![allow(dead_code)]

use gpui::{prelude::*, *};

/// Width of the scrollbar track in pixels
pub const SCROLLBAR_WIDTH: f32 = 6.0;

/// Minimum thumb height in pixels (prevents thumb from becoming too small)
pub const MIN_THUMB_HEIGHT: f32 = 20.0;

/// Padding from container edge
pub const SCROLLBAR_PADDING: f32 = 2.0;

/// Pre-computed colors for scrollbar rendering
///
/// This struct holds the color values needed for scrollbar rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct ScrollbarColors {
    /// Track background color (very subtle, semi-transparent)
    pub track: u32,
    /// Track opacity (0.0 - 1.0)
    pub track_opacity: f32,
    /// Thumb color (the draggable part)
    pub thumb: u32,
    /// Thumb opacity (0.0 - 1.0)
    pub thumb_opacity: f32,
    /// Thumb color when hovered
    pub thumb_hover: u32,
    /// Thumb hover opacity
    pub thumb_hover_opacity: f32,
}

impl ScrollbarColors {
    /// Create ScrollbarColors from theme reference
    ///
    /// Uses muted/border colors for a subtle, native appearance.
    /// Opacity values are theme-aware: light mode uses higher opacity for visibility.
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        let is_dark = theme.is_dark_mode();
        let (track_opacity, thumb_opacity, thumb_hover_opacity) = if is_dark {
            (0.1, 0.4, 0.6) // Dark mode: lower opacity works well
        } else {
            (0.15, 0.5, 0.7) // Light mode: higher opacity for visibility
        };

        Self {
            track: theme.colors.ui.border,
            track_opacity,
            thumb: theme.colors.text.muted,
            thumb_opacity,
            thumb_hover: theme.colors.text.secondary,
            thumb_hover_opacity,
        }
    }

    /// Create ScrollbarColors from design colors
    ///
    /// NOTE: This defaults to dark mode opacity values. For light mode support,
    /// use `from_design_with_dark_mode()` instead.
    pub fn from_design(colors: &crate::designs::DesignColors) -> Self {
        // Default to dark mode
        Self::from_design_with_dark_mode(colors, true)
    }

    /// Create ScrollbarColors from design colors with explicit dark/light mode
    ///
    /// Light mode needs higher opacity values because low opacity on light backgrounds
    /// is too subtle to be visible. Dark mode uses lower opacity because overlays
    /// are more visible on dark backgrounds.
    ///
    /// # Arguments
    /// * `colors` - Design colors to use
    /// * `is_dark` - True for dark mode (lower opacity), false for light mode (higher opacity)
    pub fn from_design_with_dark_mode(
        colors: &crate::designs::DesignColors,
        is_dark: bool,
    ) -> Self {
        let (track_opacity, thumb_opacity, thumb_hover_opacity) = if is_dark {
            (0.1, 0.4, 0.6) // Dark mode: lower opacity works well
        } else {
            (0.15, 0.5, 0.7) // Light mode: higher opacity for visibility
        };

        Self {
            track: colors.border,
            track_opacity,
            thumb: colors.text_secondary,
            thumb_opacity,
            thumb_hover: colors.text_primary,
            thumb_hover_opacity,
        }
    }
}

impl Default for ScrollbarColors {
    /// Default scrollbar colors (dark mode)
    ///
    /// For light mode, use `from_design_with_dark_mode(colors, false)` or
    /// `from_theme()` which auto-detects the mode.
    fn default() -> Self {
        Self::from_theme(&crate::theme::get_cached_theme())
    }
}

/// A minimal native-style scrollbar component
///
/// The scrollbar is designed to:
/// - Overlay on content (absolute positioned on right edge)
/// - Show thumb proportional to visible/total content ratio
/// - Be semi-transparent and unobtrusive
/// - Only render when content overflows (total > visible)
///
#[derive(IntoElement)]
pub struct Scrollbar {
    /// Total number of items in the list
    total_items: usize,
    /// Number of items visible at once
    visible_items: usize,
    /// Index of first visible item (scroll offset)
    scroll_offset: usize,
    /// Pre-computed colors
    colors: ScrollbarColors,
    /// Container height in pixels (for calculating thumb position)
    container_height: Option<f32>,
    /// Scrollbar visibility factor (for scroll-activity-aware fade)
    /// When Some(v), base opacities are multiplied by v (clamped 0.0..1.0)
    /// When None, the scrollbar uses base opacities unchanged
    visibility: Option<f32>,
}

impl Scrollbar {
    /// Create a new scrollbar
    ///
    /// # Arguments
    /// * `total_items` - Total number of items in the scrollable list
    /// * `visible_items` - Number of items visible in the viewport
    /// * `scroll_offset` - Index of the first visible item
    /// * `colors` - Pre-computed colors for rendering
    pub fn new(
        total_items: usize,
        visible_items: usize,
        scroll_offset: usize,
        colors: ScrollbarColors,
    ) -> Self {
        Self {
            total_items,
            visible_items,
            scroll_offset,
            colors,
            container_height: None,
            visibility: None,
        }
    }

    /// Set the container height for precise thumb positioning
    ///
    /// If not set, the scrollbar will use percentage-based positioning
    pub fn container_height(mut self, height: f32) -> Self {
        self.container_height = Some(height);
        self
    }

    /// Set a visibility opacity multiplier for scroll-activity-aware fade.
    ///
    /// Value is clamped to 0.0..1.0 and multiplies the base theme opacities.
    /// If not called, the scrollbar uses default behavior (opacity factor 1.0).
    pub fn visibility_opacity(mut self, visibility: f32) -> Self {
        self.visibility = Some(visibility.clamp(0.0, 1.0));
        self
    }

    /// Check if scrollbar should be visible (content overflows)
    fn should_show(&self) -> bool {
        self.total_items > self.visible_items && self.total_items > 0
    }

    /// Calculate thumb height as a ratio of visible/total items
    fn thumb_height_ratio(&self) -> f32 {
        if self.total_items == 0 {
            return 1.0;
        }
        (self.visible_items as f32 / self.total_items as f32).clamp(0.05, 1.0)
    }

    /// Calculate thumb position as a ratio of scroll_offset/(total-visible)
    ///
    /// Uses a tolerance-based approach to ensure the thumb reaches the bottom
    /// even when visible_items is slightly underestimated. When scroll_offset
    /// is within 2 items of the estimated maximum, we snap to 1.0.
    fn thumb_position_ratio(&self) -> f32 {
        if self.total_items <= self.visible_items {
            return 0.0;
        }
        let max_offset = self.total_items.saturating_sub(self.visible_items);
        if max_offset == 0 {
            return 0.0;
        }

        // Snap to 1.0 if we're within 2 items of the estimated max
        // This handles cases where visible_items estimate is slightly off
        let tolerance = 2;
        if self.scroll_offset + tolerance >= max_offset {
            return 1.0;
        }

        (self.scroll_offset as f32 / max_offset as f32).clamp(0.0, 1.0)
    }
}

impl RenderOnce for Scrollbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Don't render if content doesn't overflow
        if !self.should_show() {
            return div().into_any_element();
        }

        // Handle scroll-activity-aware visibility using an opacity multiplier
        let visibility = self.visibility.unwrap_or(1.0);
        let thumb_opacity = self.colors.thumb_opacity * visibility;
        let thumb_hover_opacity = self.colors.thumb_hover_opacity * visibility;

        let colors = self.colors;
        let thumb_height_ratio = self.thumb_height_ratio();
        let thumb_position_ratio = self.thumb_position_ratio();

        // Calculate actual pixel values if container height is known
        let (thumb_height_px, thumb_top_px) = if let Some(container_h) = self.container_height {
            let available_height = container_h - (SCROLLBAR_PADDING * 2.0);
            let thumb_h = (available_height * thumb_height_ratio).max(MIN_THUMB_HEIGHT);
            let scrollable_range = available_height - thumb_h;
            let thumb_top = SCROLLBAR_PADDING + (scrollable_range * thumb_position_ratio);
            (Some(thumb_h), Some(thumb_top))
        } else {
            (None, None)
        };

        // Build the scrollbar container (absolute positioned on right edge)
        let mut scrollbar = div()
            .absolute()
            .top_0()
            .bottom_0()
            .right(px(SCROLLBAR_PADDING))
            .w(px(SCROLLBAR_WIDTH))
            .flex()
            .flex_col();

        // Build the thumb element
        let thumb = if let (Some(height), Some(top)) = (thumb_height_px, thumb_top_px) {
            // Precise pixel positioning
            div()
                .absolute()
                .top(px(top))
                .left_0()
                .right_0()
                .h(px(height))
                .rounded(px(SCROLLBAR_WIDTH / 2.0))
                .bg(rgba((colors.thumb << 8) | ((thumb_opacity * 255.0) as u32)))
                .hover(move |s| {
                    s.bg(rgba(
                        (colors.thumb_hover << 8) | ((thumb_hover_opacity * 255.0) as u32),
                    ))
                })
        } else {
            // Percentage-based positioning (fallback)
            // Use flex layout to position thumb
            let top_spacer_flex = thumb_position_ratio * (1.0 - thumb_height_ratio);
            let thumb_flex = thumb_height_ratio;
            let bottom_spacer_flex = (1.0 - thumb_position_ratio) * (1.0 - thumb_height_ratio);

            // Create a flex container for percentage-based positioning
            scrollbar = scrollbar
                .child(
                    div()
                        .flex_grow()
                        .flex_shrink_0()
                        .min_h_0()
                        .map(move |d: Div| {
                            if top_spacer_flex > 0.001 {
                                d.flex_basis(relative(top_spacer_flex))
                            } else {
                                d
                            }
                        }),
                )
                .child(
                    div()
                        .flex_grow()
                        .flex_shrink_0()
                        .min_h(px(MIN_THUMB_HEIGHT))
                        .flex_basis(relative(thumb_flex))
                        .rounded(px(SCROLLBAR_WIDTH / 2.0))
                        .bg(rgba((colors.thumb << 8) | ((thumb_opacity * 255.0) as u32)))
                        .hover(move |s| {
                            s.bg(rgba(
                                (colors.thumb_hover << 8) | ((thumb_hover_opacity * 255.0) as u32),
                            ))
                        }),
                )
                .child(
                    div()
                        .flex_grow()
                        .flex_shrink_0()
                        .min_h_0()
                        .map(move |d: Div| {
                            if bottom_spacer_flex > 0.001 {
                                d.flex_basis(relative(bottom_spacer_flex))
                            } else {
                                d
                            }
                        }),
                );

            return scrollbar.into_any_element();
        };

        scrollbar.child(thumb).into_any_element()
    }
}

// Note: Tests omitted for this module due to GPUI macro recursion limit issues.
// The Scrollbar component is integration-tested via the main application's
// list rendering.
//
// Verified traits:
// - ScrollbarColors: Copy, Clone, Debug, Default
// - Scrollbar: builder pattern with .container_height()
//
// Logic verification (manual):
// - should_show(): returns true when total_items > visible_items && total_items > 0
// - thumb_height_ratio(): returns visible_items / total_items, clamped to [0.05, 1.0]
// - thumb_position_ratio(): returns scroll_offset / max_offset, clamped to [0.0, 1.0]

#[cfg(test)]
mod tests {
    use super::{Scrollbar, ScrollbarColors};

    #[test]
    fn test_scrollbar_colors_default_uses_cached_theme_tokens() {
        let resolved = ScrollbarColors::default();
        let expected = ScrollbarColors::from_theme(&crate::theme::get_cached_theme());

        assert_eq!(resolved.track, expected.track);
        assert_eq!(resolved.track_opacity, expected.track_opacity);
        assert_eq!(resolved.thumb, expected.thumb);
        assert_eq!(resolved.thumb_opacity, expected.thumb_opacity);
        assert_eq!(resolved.thumb_hover, expected.thumb_hover);
        assert_eq!(resolved.thumb_hover_opacity, expected.thumb_hover_opacity);
    }

    #[test]
    fn test_visibility_opacity_does_clamp_to_zero_when_below_range() {
        let scrollbar =
            Scrollbar::new(10, 5, 0, ScrollbarColors::default()).visibility_opacity(-1.5);
        assert_eq!(scrollbar.visibility, Some(0.0));
    }

    #[test]
    fn test_visibility_opacity_does_clamp_to_one_when_above_range() {
        let scrollbar =
            Scrollbar::new(10, 5, 0, ScrollbarColors::default()).visibility_opacity(2.5);
        assert_eq!(scrollbar.visibility, Some(1.0));
    }

    #[test]
    fn test_visibility_opacity_does_preserve_value_when_in_range() {
        let scrollbar =
            Scrollbar::new(10, 5, 0, ScrollbarColors::default()).visibility_opacity(0.42);
        assert_eq!(scrollbar.visibility, Some(0.42));
    }
}
