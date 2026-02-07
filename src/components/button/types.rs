use crate::designs::DesignColors;
use crate::theme::Theme;

/// Canonical height for prompt action buttons (Run/Actions/Save/Cancel/etc.).
pub const BUTTON_GHOST_HEIGHT: f32 = 28.0;
/// Canonical horizontal padding for ghost buttons.
pub const BUTTON_GHOST_PADDING_X: f32 = 8.0;
/// Canonical vertical padding for ghost buttons.
pub const BUTTON_GHOST_PADDING_Y: f32 = 4.0;
/// Canonical horizontal padding for primary buttons.
pub const BUTTON_PRIMARY_PADDING_X: f32 = 12.0;
/// Canonical vertical padding for primary buttons.
pub const BUTTON_PRIMARY_PADDING_Y: f32 = 6.0;
/// Canonical horizontal padding for icon buttons.
pub const BUTTON_ICON_PADDING_X: f32 = 6.0;
/// Canonical vertical padding for icon buttons.
pub const BUTTON_ICON_PADDING_Y: f32 = 6.0;
/// Canonical spacing between button label/shortcut and inline content.
pub const BUTTON_CONTENT_GAP_PX: f32 = 2.0;
/// Canonical margin between the main label and shortcut text.
pub const BUTTON_SHORTCUT_MARGIN_LEFT_PX: f32 = 4.0;
/// Canonical button corner radius.
pub const BUTTON_RADIUS_PX: f32 = 6.0;

/// Button variant determines the visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Primary button with filled background (accent color)
    #[default]
    Primary,
    /// Ghost button with text only (no background)
    Ghost,
    /// Icon button (compact, for icons)
    Icon,
}

/// Pre-computed colors for Button rendering
///
/// This struct holds the primitive color values needed for button rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct ButtonColors {
    /// Text color for the button label
    pub text_color: u32,
    /// Text color when hovering (reserved for future use)
    #[allow(dead_code)]
    pub text_hover: u32,
    /// Background color (for Primary variant)
    pub background: u32,
    /// Background color when hovering
    pub background_hover: u32,
    /// Accent color for highlights
    pub accent: u32,
    /// Border color
    pub border: u32,
    /// Focus ring color (shown when button is focused)
    pub focus_ring: u32,
    /// Subtle background tint when focused
    pub focus_tint: u32,
    /// Hover overlay color with alpha (theme-aware: white for dark, black for light)
    /// Format: 0xRRGGBBAA
    pub hover_overlay: u32,
}

impl ButtonColors {
    fn overlay_with_alpha(base_color: u32, alpha: u8) -> u32 {
        ((base_color & 0x00ff_ffff) << 8) | (alpha as u32)
    }

    /// Create ButtonColors from theme reference
    /// Uses accent.selected (yellow/gold) to match logo and selected item highlights
    pub fn from_theme(theme: &Theme) -> Self {
        let hover_overlay = Self::overlay_with_alpha(theme.colors.accent.selected_subtle, 0x26);

        Self {
            text_color: theme.colors.accent.selected, // Yellow/gold - matches logo & highlights
            text_hover: theme.colors.text.primary,
            background: theme.colors.accent.selected_subtle,
            background_hover: theme.colors.accent.selected_subtle,
            accent: theme.colors.accent.selected, // Yellow/gold - matches logo & highlights
            border: theme.colors.ui.border,
            focus_ring: theme.colors.accent.selected, // Accent color for focus ring
            focus_tint: theme.colors.accent.selected_subtle, // Subtle tint when focused
            hover_overlay,
        }
    }

    /// Create ButtonColors from design colors for design system support
    /// Uses the primary accent color to match the design's brand
    ///
    /// NOTE: This defaults to dark mode hover overlay (white at 15%).
    /// For light mode support, use `from_design_with_theme()` instead.
    pub fn from_design(colors: &DesignColors) -> Self {
        // Default to dark mode (white hover overlay)
        Self::from_design_with_dark_mode(colors, true)
    }

    /// Create ButtonColors from design colors with explicit dark/light mode
    ///
    /// # Arguments
    /// * `colors` - Design color tokens
    /// * `is_dark` - True for dark mode (white hover), false for light mode (black hover)
    pub fn from_design_with_dark_mode(colors: &DesignColors, _is_dark: bool) -> Self {
        let hover_overlay = Self::overlay_with_alpha(colors.background_selected, 0x26);

        Self {
            text_color: colors.accent, // Primary accent (yellow/gold for default)
            text_hover: colors.text_primary,
            background: colors.background_selected,
            background_hover: colors.background_hover,
            accent: colors.accent, // Primary accent (yellow/gold for default)
            border: colors.border,
            focus_ring: colors.accent, // Accent color for focus ring
            focus_tint: colors.background_selected, // Subtle tint when focused
            hover_overlay,
        }
    }
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}
