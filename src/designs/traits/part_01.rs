use gpui::*;

use super::DesignVariant;

// ============================================================================
// Design Token Structs (Copy/Clone for efficient closure use)
// ============================================================================

/// Color tokens for a design variant
///
/// All colors are stored as u32 hex values (0xRRGGBB format).
/// Use `gpui::rgb()` to convert to GPUI colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesignColors {
    // Background colors
    /// Primary background color
    pub background: u32,
    /// Secondary/surface background (for cards, panels)
    pub background_secondary: u32,
    /// Tertiary background (for nested elements)
    pub background_tertiary: u32,
    /// Background for selected items
    pub background_selected: u32,
    /// Background for hovered items
    pub background_hover: u32,

    // Text colors
    /// Primary text color (headings, names)
    pub text_primary: u32,
    /// Secondary text color (descriptions, labels)
    pub text_secondary: u32,
    /// Muted text color (placeholders, hints)
    pub text_muted: u32,
    /// Dimmed text color (disabled, inactive)
    pub text_dimmed: u32,
    /// Text color on selected/accent backgrounds
    pub text_on_accent: u32,

    // Accent colors
    /// Primary accent color (selection highlight, links)
    pub accent: u32,
    /// Secondary accent color (buttons, interactive)
    pub accent_secondary: u32,
    /// Success state color
    pub success: u32,
    /// Warning state color
    pub warning: u32,
    /// Error state color
    pub error: u32,

    // Border colors
    /// Primary border color
    pub border: u32,
    /// Subtle/light border color
    pub border_subtle: u32,
    /// Focused element border color
    pub border_focus: u32,

    // Shadow color (with alpha in 0xRRGGBBAA format)
    /// Shadow color (typically black with alpha)
    pub shadow: u32,
}

impl DesignColors {
    /// Combine a hex color (0xRRGGBB) with an alpha value (0-255)
    /// Returns a value suitable for gpui::rgba() in 0xRRGGBBAA format
    #[inline]
    pub fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
        (hex << 8) | (alpha as u32)
    }
}

impl Default for DesignColors {
    fn default() -> Self {
        // Default dark theme colors
        Self {
            background: 0x1e1e1e,
            background_secondary: 0x2d2d30,
            background_tertiary: 0x3c3c3c,
            background_selected: 0xffffff, // White - subtle brightening like Raycast
            background_hover: 0xffffff,    // White - barely visible hover

            text_primary: 0xffffff,
            text_secondary: 0xcccccc,
            text_muted: 0x808080,
            text_dimmed: 0x666666,
            text_on_accent: 0x000000,

            accent: 0xfbbf24,           // Script Kit yellow/gold
            accent_secondary: 0xfbbf24, // Same as primary for consistency
            success: 0x00ff00,
            warning: 0xf59e0b,
            error: 0xef4444,

            border: 0x464647,
            border_subtle: 0x3a3a3a,
            border_focus: 0x007acc,

            shadow: 0x00000040,
        }
    }
}

/// Spacing tokens for a design variant
///
/// All values are in pixels (f32). Use `gpui::px()` to convert.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesignSpacing {
    // Padding variants
    /// Extra small padding (4px)
    pub padding_xs: f32,
    /// Small padding (8px)
    pub padding_sm: f32,
    /// Medium/base padding (12px)
    pub padding_md: f32,
    /// Large padding (16px)
    pub padding_lg: f32,
    /// Extra large padding (24px)
    pub padding_xl: f32,

    // Gap variants (for flexbox)
    /// Small gap between items (4px)
    pub gap_sm: f32,
    /// Medium gap between items (8px)
    pub gap_md: f32,
    /// Large gap between items (16px)
    pub gap_lg: f32,

    // Margin variants
    /// Small margin (4px)
    pub margin_sm: f32,
    /// Medium margin (8px)
    pub margin_md: f32,
    /// Large margin (16px)
    pub margin_lg: f32,

    // Component-specific spacing
    /// Horizontal padding for list items
    pub item_padding_x: f32,
    /// Vertical padding for list items
    pub item_padding_y: f32,
    /// Gap between icon and text in list items
    pub icon_text_gap: f32,
}

impl Default for DesignSpacing {
    fn default() -> Self {
        Self {
            padding_xs: 4.0,
            padding_sm: 8.0,
            padding_md: 12.0,
            padding_lg: 16.0,
            padding_xl: 24.0,

            gap_sm: 4.0,
            gap_md: 8.0,
            gap_lg: 16.0,

            margin_sm: 4.0,
            margin_md: 8.0,
            margin_lg: 16.0,

            item_padding_x: 16.0,
            item_padding_y: 8.0,
            icon_text_gap: 8.0,
        }
    }
}

/// Typography tokens for a design variant
#[derive(Debug, Clone, PartialEq)]
pub struct DesignTypography {
    // Font families
    /// Primary font family (for UI text)
    pub font_family: &'static str,
    /// Monospace font family (for code, terminal)
    pub font_family_mono: &'static str,

    // Font sizes (in pixels)
    /// Extra small text size (10px)
    pub font_size_xs: f32,
    /// Small text size (12px)
    pub font_size_sm: f32,
    /// Base/medium text size (14px)
    pub font_size_md: f32,
    /// Large text size (16px)
    pub font_size_lg: f32,
    /// Extra large text size (20px)
    pub font_size_xl: f32,
    /// Title text size (24px)
    pub font_size_title: f32,

    // Font weights
    /// Thin font weight (100)
    pub font_weight_thin: FontWeight,
    /// Light font weight (300)
    pub font_weight_light: FontWeight,
    /// Normal font weight (400)
    pub font_weight_normal: FontWeight,
    /// Medium font weight (500)
    pub font_weight_medium: FontWeight,
    /// Semibold font weight (600)
    pub font_weight_semibold: FontWeight,
    /// Bold font weight (700)
    pub font_weight_bold: FontWeight,

    // Line heights (as multipliers)
    /// Tight line height (1.2)
    pub line_height_tight: f32,
    /// Normal line height (1.5)
    pub line_height_normal: f32,
    /// Relaxed line height (1.75)
    pub line_height_relaxed: f32,
}

impl Default for DesignTypography {
    fn default() -> Self {
        Self {
            font_family: ".AppleSystemUIFont",
            font_family_mono: "Menlo",

            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_title: 24.0,

            font_weight_thin: FontWeight::THIN,
            font_weight_light: FontWeight::LIGHT,
            font_weight_normal: FontWeight::NORMAL,
            font_weight_medium: FontWeight::MEDIUM,
            font_weight_semibold: FontWeight::SEMIBOLD,
            font_weight_bold: FontWeight::BOLD,

            line_height_tight: 1.2,
            line_height_normal: 1.5,
            line_height_relaxed: 1.75,
        }
    }
}

// Implement Copy for DesignTypography by storing only static str references
impl Copy for DesignTypography {}

impl DesignTypography {
    /// Calculate the cursor height for text input fields.
    ///
    /// The cursor should be slightly shorter than the line height for visual balance.
    /// This follows the pattern from editor.rs where cursor_height = line_height - 4.0
    ///
    /// # Arguments
    /// * `font_size` - The font size in pixels (e.g., font_size_lg for .text_lg())
    ///
    /// # Returns
    /// The cursor height in pixels, slightly shorter than the line height.
    #[inline]
    pub fn cursor_height_for_font(&self, font_size: f32) -> f32 {
        // Calculate line height based on normal multiplier
        let line_height = font_size * self.line_height_normal;
        // Subtract 4px for visual balance (matches editor.rs pattern)
        // This leaves 2px margin on top and bottom for vertical centering
        (line_height - 4.0).max(12.0) // Minimum 12px for visibility
    }

    /// Calculate cursor height for large text (used with .text_lg())
    ///
    /// GPUI's .text_lg() is approximately 18px font with ~1.55 line height.
    /// Returns a cursor height that aligns properly with GPUI's text rendering.
    #[inline]
    pub fn cursor_height_lg(&self) -> f32 {
        // For GPUI .text_lg() compatibility:
        // - GPUI text_lg is ~18px font size
        // - Natural line height ~28px (1.55 multiplier)
        // - Cursor should be ~20px with 4px margin for centering
        //
        // We use 18px as a good middle ground that works with various line heights
        18.0
    }

    /// Calculate vertical margin for cursor centering within text line
    ///
    /// Returns the top/bottom margin needed to vertically center the cursor.
    #[inline]
    pub fn cursor_margin_y(&self) -> f32 {
        2.0
    }
}

/// Visual effect tokens for a design variant
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesignVisual {
    // Border radius variants
    /// No border radius (0px)
    pub radius_none: f32,
    /// Small border radius (4px)
    pub radius_sm: f32,
    /// Medium border radius (8px)
    pub radius_md: f32,
    /// Large border radius (12px)
    pub radius_lg: f32,
    /// Extra large border radius (16px)
    pub radius_xl: f32,
    /// Full/pill border radius (9999px)
    pub radius_full: f32,

    // Shadow properties
    /// Shadow blur radius
    pub shadow_blur: f32,
    /// Shadow spread radius
    pub shadow_spread: f32,
    /// Shadow X offset
    pub shadow_offset_x: f32,
    /// Shadow Y offset
    pub shadow_offset_y: f32,
    /// Shadow opacity (0.0 - 1.0)
    pub shadow_opacity: f32,

    // Opacity variants
    /// Disabled element opacity
    pub opacity_disabled: f32,
    /// Hover state opacity
    pub opacity_hover: f32,
    /// Pressed/active state opacity
    pub opacity_pressed: f32,
    /// Background overlay opacity (for modals, dialogs)
    pub opacity_overlay: f32,

    // Animation durations (ms)
    /// Fast animation (100ms)
    pub animation_fast: u32,
    /// Normal animation (200ms)
    pub animation_normal: u32,
    /// Slow animation (300ms)
    pub animation_slow: u32,

    // Border widths
    /// Thin border (1px)
    pub border_thin: f32,
    /// Normal border (2px)
    pub border_normal: f32,
    /// Thick border (4px)
    pub border_thick: f32,
}

impl Default for DesignVisual {
    fn default() -> Self {
        Self {
            radius_none: 0.0,
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,
            radius_xl: 16.0,
            radius_full: 9999.0,

            shadow_blur: 8.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 4.0,
            shadow_opacity: 0.25,

            opacity_disabled: 0.5,
            opacity_hover: 0.8,
            opacity_pressed: 0.6,
            opacity_overlay: 0.5,

            animation_fast: 100,
            animation_normal: 200,
            animation_slow: 300,

            border_thin: 1.0,
            border_normal: 2.0,
            border_thick: 4.0,
        }
    }
}

// ============================================================================
// DesignTokens Trait
// ============================================================================

/// Trait for design token providers
///
/// Each design variant implements this trait to provide its complete set of
/// design tokens. This enables consistent theming across the entire application
/// while allowing each design to have its own unique visual identity.
///
pub trait DesignTokens: Send + Sync {
    /// Get the color tokens for this design
    fn colors(&self) -> DesignColors;

    /// Get the spacing tokens for this design
    fn spacing(&self) -> DesignSpacing;

    /// Get the typography tokens for this design
    fn typography(&self) -> DesignTypography;

    /// Get the visual effect tokens for this design
    fn visual(&self) -> DesignVisual;

    /// Get the list item height for this design (in pixels)
    ///
    /// This is used by uniform_list for virtualization.
    fn item_height(&self) -> f32;

    /// Get the design variant this token set represents
    fn variant(&self) -> DesignVariant;
}

/// Default token implementation for the standard design
#[derive(Debug, Clone, Copy)]
pub struct DefaultDesignTokens;

impl DesignTokens for DefaultDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors::default()
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing::default()
    }

    fn typography(&self) -> DesignTypography {
        DesignTypography::default()
    }

    fn visual(&self) -> DesignVisual {
        DesignVisual::default()
    }

    fn item_height(&self) -> f32 {
        40.0 // Standard list item height matching LIST_ITEM_HEIGHT constant
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Default
    }
}

// ============================================================================
// Design-Specific Token Implementations
// ============================================================================

