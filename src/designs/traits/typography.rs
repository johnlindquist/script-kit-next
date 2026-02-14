use gpui::FontWeight;

use super::tokens::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};

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

pub(crate) trait DesignTypographyTokens {
    fn typography(&self) -> DesignTypography;
}

impl DesignTypographyTokens for DefaultDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography::default()
    }
}

impl DesignTypographyTokens for MinimalDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: ".AppleSystemUIFont",
            font_family_mono: "Menlo",

            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 16.0, // Larger base for minimal
            font_size_lg: 18.0,
            font_size_xl: 22.0,
            font_size_title: 28.0,

            // Minimal uses thin/light weights
            font_weight_thin: FontWeight::THIN,
            font_weight_light: FontWeight::LIGHT,
            font_weight_normal: FontWeight::THIN, // Default to thin
            font_weight_medium: FontWeight::LIGHT,
            font_weight_semibold: FontWeight::NORMAL,
            font_weight_bold: FontWeight::MEDIUM,

            line_height_tight: 1.3,
            line_height_normal: 1.6,
            line_height_relaxed: 1.8,
        }
    }
}

impl DesignTypographyTokens for RetroTerminalDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: "Menlo", // Monospace for terminal
            font_family_mono: "Menlo",

            font_size_xs: 10.0,
            font_size_sm: 12.0, // Terminal uses smaller text
            font_size_md: 13.0,
            font_size_lg: 14.0,
            font_size_xl: 16.0,
            font_size_title: 18.0,

            font_weight_thin: FontWeight::NORMAL,
            font_weight_light: FontWeight::NORMAL,
            font_weight_normal: FontWeight::NORMAL,
            font_weight_medium: FontWeight::NORMAL,
            font_weight_semibold: FontWeight::BOLD,
            font_weight_bold: FontWeight::BOLD,

            line_height_tight: 1.1,
            line_height_normal: 1.3,
            line_height_relaxed: 1.5,
        }
    }
}

impl DesignTypographyTokens for GlassmorphismDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography::default()
    }
}

impl DesignTypographyTokens for BrutalistDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: "Helvetica Neue",
            font_family_mono: "Courier",

            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 18.0,
            font_size_xl: 24.0,
            font_size_title: 32.0,

            // Bold typography for brutalist
            font_weight_thin: FontWeight::NORMAL,
            font_weight_light: FontWeight::NORMAL,
            font_weight_normal: FontWeight::MEDIUM,
            font_weight_medium: FontWeight::SEMIBOLD,
            font_weight_semibold: FontWeight::BOLD,
            font_weight_bold: FontWeight::BLACK,

            line_height_tight: 1.1,
            line_height_normal: 1.4,
            line_height_relaxed: 1.6,
        }
    }
}

impl DesignTypographyTokens for CompactDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: ".AppleSystemUIFont",
            font_family_mono: "SF Mono",

            // Smaller text sizes
            font_size_xs: 9.0,
            font_size_sm: 10.0,
            font_size_md: 11.0,
            font_size_lg: 12.0,
            font_size_xl: 14.0,
            font_size_title: 16.0,

            font_weight_thin: FontWeight::LIGHT,
            font_weight_light: FontWeight::LIGHT,
            font_weight_normal: FontWeight::NORMAL,
            font_weight_medium: FontWeight::MEDIUM,
            font_weight_semibold: FontWeight::SEMIBOLD,
            font_weight_bold: FontWeight::BOLD,

            // Tighter line heights
            line_height_tight: 1.1,
            line_height_normal: 1.2,
            line_height_relaxed: 1.3,
        }
    }
}

impl DesignTypographyTokens for NeonCyberpunkDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography::default()
    }
}

impl DesignTypographyTokens for PaperDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: "Georgia",
            font_family_mono: "Courier New",

            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_title: 24.0,

            font_weight_thin: FontWeight::NORMAL,
            font_weight_light: FontWeight::NORMAL,
            font_weight_normal: FontWeight::NORMAL,
            font_weight_medium: FontWeight::MEDIUM,
            font_weight_semibold: FontWeight::SEMIBOLD,
            font_weight_bold: FontWeight::BOLD,

            line_height_tight: 1.3,
            line_height_normal: 1.6,
            line_height_relaxed: 1.8,
        }
    }
}

impl DesignTypographyTokens for AppleHIGDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: ".AppleSystemUIFont",
            font_family_mono: "SF Mono",

            font_size_xs: 11.0,
            font_size_sm: 13.0,
            font_size_md: 15.0, // iOS body
            font_size_lg: 17.0, // iOS headline
            font_size_xl: 20.0,
            font_size_title: 28.0, // iOS title1

            font_weight_thin: FontWeight::THIN,
            font_weight_light: FontWeight::LIGHT,
            font_weight_normal: FontWeight::NORMAL,
            font_weight_medium: FontWeight::MEDIUM,
            font_weight_semibold: FontWeight::SEMIBOLD,
            font_weight_bold: FontWeight::BOLD,

            line_height_tight: 1.2,
            line_height_normal: 1.4,
            line_height_relaxed: 1.6,
        }
    }
}

impl DesignTypographyTokens for Material3DesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: ".AppleSystemUIFont", // Would be Roboto on Android
            font_family_mono: "Roboto Mono",

            font_size_xs: 11.0,
            font_size_sm: 12.0, // M3 label-small
            font_size_md: 14.0, // M3 body-medium
            font_size_lg: 16.0, // M3 title-medium
            font_size_xl: 22.0, // M3 headline-small
            font_size_title: 28.0,

            font_weight_thin: FontWeight::THIN,
            font_weight_light: FontWeight::LIGHT,
            font_weight_normal: FontWeight::NORMAL,
            font_weight_medium: FontWeight::MEDIUM,
            font_weight_semibold: FontWeight::MEDIUM, // M3 uses medium more
            font_weight_bold: FontWeight::BOLD,

            line_height_tight: 1.2,
            line_height_normal: 1.5,
            line_height_relaxed: 1.75,
        }
    }
}

impl DesignTypographyTokens for PlayfulDesignTokens {
    fn typography(&self) -> DesignTypography {
        DesignTypography {
            font_family: ".AppleSystemUIFont",
            font_family_mono: "Menlo",

            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_title: 24.0,

            font_weight_thin: FontWeight::LIGHT,
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
