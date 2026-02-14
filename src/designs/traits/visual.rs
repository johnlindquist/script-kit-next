use super::tokens::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};

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

pub(crate) trait DesignVisualTokens {
    fn visual(&self) -> DesignVisual;
}

impl DesignVisualTokens for DefaultDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual::default()
    }
}

impl DesignVisualTokens for MinimalDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            // No rounded corners for minimal
            radius_none: 0.0,
            radius_sm: 0.0,
            radius_md: 0.0,
            radius_lg: 0.0,
            radius_xl: 0.0,
            radius_full: 0.0,

            // No shadows
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_opacity: 0.0,

            // Subtle opacity effects
            opacity_disabled: 0.4,
            opacity_hover: 0.8,
            opacity_pressed: 0.6,
            opacity_overlay: 0.3,

            animation_fast: 150,
            animation_normal: 250,
            animation_slow: 350,

            // No visible borders
            border_thin: 0.0,
            border_normal: 0.0,
            border_thick: 0.0,
        }
    }
}

impl DesignVisualTokens for RetroTerminalDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            // No rounded corners for terminal
            radius_none: 0.0,
            radius_sm: 0.0,
            radius_md: 0.0,
            radius_lg: 0.0,
            radius_xl: 0.0,
            radius_full: 0.0,

            // Green glow effect
            shadow_blur: 8.0,
            shadow_spread: 2.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_opacity: 0.6,

            opacity_disabled: 0.5,
            opacity_hover: 1.0,
            opacity_pressed: 0.9,
            opacity_overlay: 0.8,

            animation_fast: 0, // Instant for terminal feel
            animation_normal: 0,
            animation_slow: 100,

            border_thin: 1.0,
            border_normal: 1.0,
            border_thick: 2.0,
        }
    }
}

impl DesignVisualTokens for GlassmorphismDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            // Large rounded corners for glass effect
            radius_none: 0.0,
            radius_sm: 8.0,
            radius_md: 16.0,
            radius_lg: 24.0,
            radius_xl: 32.0,
            radius_full: 9999.0,

            // Soft shadows
            shadow_blur: 20.0,
            shadow_spread: -2.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 10.0,
            shadow_opacity: 0.2,

            opacity_disabled: 0.4,
            opacity_hover: 0.9,
            opacity_pressed: 0.7,
            opacity_overlay: 0.6,

            animation_fast: 150,
            animation_normal: 300,
            animation_slow: 500,

            border_thin: 1.0,
            border_normal: 1.0,
            border_thick: 2.0,
        }
    }
}

impl DesignVisualTokens for BrutalistDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            // No rounded corners - raw edges
            radius_none: 0.0,
            radius_sm: 0.0,
            radius_md: 0.0,
            radius_lg: 0.0,
            radius_xl: 0.0,
            radius_full: 0.0,

            // Hard shadows
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_offset_x: 4.0,
            shadow_offset_y: 4.0,
            shadow_opacity: 1.0,

            opacity_disabled: 0.5,
            opacity_hover: 1.0,
            opacity_pressed: 0.9,
            opacity_overlay: 0.9,

            animation_fast: 0, // No animations
            animation_normal: 0,
            animation_slow: 0,

            // Thick borders
            border_thin: 2.0,
            border_normal: 4.0,
            border_thick: 8.0,
        }
    }
}

impl DesignVisualTokens for CompactDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            // Small radius
            radius_none: 0.0,
            radius_sm: 2.0,
            radius_md: 4.0,
            radius_lg: 6.0,
            radius_xl: 8.0,
            radius_full: 9999.0,

            // Minimal shadows
            shadow_blur: 2.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 1.0,
            shadow_opacity: 0.15,

            opacity_disabled: 0.5,
            opacity_hover: 0.9,
            opacity_pressed: 0.7,
            opacity_overlay: 0.5,

            animation_fast: 50,
            animation_normal: 100,
            animation_slow: 150,

            border_thin: 1.0,
            border_normal: 1.0,
            border_thick: 2.0,
        }
    }
}

impl DesignVisualTokens for NeonCyberpunkDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            radius_none: 0.0,
            radius_sm: 2.0,
            radius_md: 4.0,
            radius_lg: 8.0,
            radius_xl: 12.0,
            radius_full: 9999.0,

            // Neon glow
            shadow_blur: 15.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_opacity: 0.8,

            opacity_disabled: 0.4,
            opacity_hover: 1.0,
            opacity_pressed: 0.8,
            opacity_overlay: 0.7,

            animation_fast: 100,
            animation_normal: 200,
            animation_slow: 300,

            border_thin: 1.0,
            border_normal: 2.0,
            border_thick: 3.0,
        }
    }
}

impl DesignVisualTokens for PaperDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            radius_none: 0.0,
            radius_sm: 2.0,
            radius_md: 4.0,
            radius_lg: 6.0,
            radius_xl: 8.0,
            radius_full: 9999.0,

            // Soft paper shadows
            shadow_blur: 12.0,
            shadow_spread: -2.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 4.0,
            shadow_opacity: 0.1,

            opacity_disabled: 0.5,
            opacity_hover: 0.95,
            opacity_pressed: 0.85,
            opacity_overlay: 0.4,

            animation_fast: 150,
            animation_normal: 250,
            animation_slow: 400,

            border_thin: 1.0,
            border_normal: 1.0,
            border_thick: 2.0,
        }
    }
}

impl DesignVisualTokens for AppleHIGDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            radius_none: 0.0,
            radius_sm: 6.0,
            radius_md: 10.0, // iOS standard
            radius_lg: 14.0,
            radius_xl: 20.0,
            radius_full: 9999.0,

            shadow_blur: 10.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 4.0,
            shadow_opacity: 0.3,

            opacity_disabled: 0.38, // iOS disabled
            opacity_hover: 0.85,
            opacity_pressed: 0.75,
            opacity_overlay: 0.5,

            animation_fast: 150,
            animation_normal: 250,
            animation_slow: 350,

            border_thin: 0.5, // iOS uses hairline borders
            border_normal: 1.0,
            border_thick: 2.0,
        }
    }
}

impl DesignVisualTokens for Material3DesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            radius_none: 0.0,
            radius_sm: 8.0,
            radius_md: 12.0, // M3 uses larger radius
            radius_lg: 16.0,
            radius_xl: 28.0, // M3 extra-large
            radius_full: 9999.0,

            // M3 elevation shadows
            shadow_blur: 8.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 2.0,
            shadow_opacity: 0.3,

            opacity_disabled: 0.38, // M3 standard
            opacity_hover: 0.08,    // M3 state layer
            opacity_pressed: 0.12,
            opacity_overlay: 0.5,

            animation_fast: 100,
            animation_normal: 200,
            animation_slow: 300,

            border_thin: 1.0,
            border_normal: 1.0,
            border_thick: 2.0,
        }
    }
}

impl DesignVisualTokens for PlayfulDesignTokens {
    fn visual(&self) -> DesignVisual {
        DesignVisual {
            // Very rounded for playful feel
            radius_none: 0.0,
            radius_sm: 8.0,
            radius_md: 16.0,
            radius_lg: 24.0,
            radius_xl: 32.0,
            radius_full: 9999.0,

            // Colorful soft shadows
            shadow_blur: 16.0,
            shadow_spread: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 6.0,
            shadow_opacity: 0.15,

            opacity_disabled: 0.5,
            opacity_hover: 0.95,
            opacity_pressed: 0.85,
            opacity_overlay: 0.4,

            // Bouncy animations
            animation_fast: 150,
            animation_normal: 300,
            animation_slow: 450,

            border_thin: 2.0,
            border_normal: 3.0,
            border_thick: 4.0,
        }
    }
}
