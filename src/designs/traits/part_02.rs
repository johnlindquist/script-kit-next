/// Minimal design tokens
#[derive(Debug, Clone, Copy)]
pub struct MinimalDesignTokens;

impl DesignTokens for MinimalDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            // Same base colors, but minimal uses more transparency
            background: 0x1e1e1e,
            background_secondary: 0x1e1e1e, // Same as bg for minimal look
            background_tertiary: 0x1e1e1e,
            background_selected: 0x1e1e1e, // No visible selection bg
            background_hover: 0x1e1e1e,

            text_primary: 0xffffff,
            text_secondary: 0xcccccc,
            text_muted: 0x808080,
            text_dimmed: 0x666666,
            text_on_accent: 0x000000,

            accent: 0xfbbf24,           // Gold accent for selected text
            accent_secondary: 0xfbbf24, // Same as primary for consistency
            success: 0x00ff00,
            warning: 0xf59e0b,
            error: 0xef4444,

            border: 0x1e1e1e, // No visible borders
            border_subtle: 0x1e1e1e,
            border_focus: 0xfbbf24,

            shadow: 0x00000000, // No shadows
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            // Generous spacing for minimal design
            padding_xs: 8.0,
            padding_sm: 16.0,
            padding_md: 24.0,
            padding_lg: 32.0,
            padding_xl: 48.0,

            gap_sm: 8.0,
            gap_md: 16.0,
            gap_lg: 24.0,

            margin_sm: 8.0,
            margin_md: 16.0,
            margin_lg: 24.0,

            item_padding_x: 80.0, // Very generous horizontal padding
            item_padding_y: 24.0, // Tall items
            icon_text_gap: 16.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        64.0 // Taller items for minimal
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Minimal
    }
}

/// Retro Terminal design tokens
#[derive(Debug, Clone, Copy)]
pub struct RetroTerminalDesignTokens;

impl DesignTokens for RetroTerminalDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0x000000, // Pure black
            background_secondary: 0x001100,
            background_tertiary: 0x002200,
            background_selected: 0x00ff00, // Inverted: green bg when selected
            background_hover: 0x003300,

            text_primary: 0x00ff00, // Phosphor green
            text_secondary: 0x00cc00,
            text_muted: 0x00aa00,
            text_dimmed: 0x008800,
            text_on_accent: 0x000000, // Black on green

            accent: 0x00ff00,
            accent_secondary: 0x00cc00,
            success: 0x00ff00,
            warning: 0xffff00, // Yellow for terminal warnings
            error: 0xff0000,

            border: 0x00aa00, // Dim green borders
            border_subtle: 0x003300,
            border_focus: 0x00ff00,

            shadow: 0x00ff0040, // Green glow
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            // Dense terminal spacing
            padding_xs: 2.0,
            padding_sm: 4.0,
            padding_md: 8.0,
            padding_lg: 12.0,
            padding_xl: 16.0,

            gap_sm: 2.0,
            gap_md: 4.0,
            gap_lg: 8.0,

            margin_sm: 2.0,
            margin_md: 4.0,
            margin_lg: 8.0,

            item_padding_x: 8.0, // Tight horizontal
            item_padding_y: 4.0, // Dense vertical
            icon_text_gap: 8.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        28.0 // Dense terminal items
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::RetroTerminal
    }
}

/// Glassmorphism design tokens
#[derive(Debug, Clone, Copy)]
pub struct GlassmorphismDesignTokens;

impl DesignTokens for GlassmorphismDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0xffffff20, // White with transparency
            background_secondary: 0xffffff30,
            background_tertiary: 0xffffff40,
            background_selected: 0xffffff50,
            background_hover: 0xffffff40,

            text_primary: 0xffffff,
            text_secondary: 0xffffffcc,
            text_muted: 0xffffff99,
            text_dimmed: 0xffffff66,
            text_on_accent: 0x000000,

            accent: 0x007aff,           // iOS blue
            accent_secondary: 0x5856d6, // iOS purple
            success: 0x34c759,          // iOS green
            warning: 0xff9500,          // iOS orange
            error: 0xff3b30,            // iOS red

            border: 0xffffff30,
            border_subtle: 0xffffff20,
            border_focus: 0x007aff,

            shadow: 0x00000020,
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            padding_xs: 6.0,
            padding_sm: 12.0,
            padding_md: 16.0,
            padding_lg: 20.0,
            padding_xl: 28.0,

            gap_sm: 6.0,
            gap_md: 12.0,
            gap_lg: 20.0,

            margin_sm: 6.0,
            margin_md: 12.0,
            margin_lg: 20.0,

            item_padding_x: 20.0,
            item_padding_y: 14.0,
            icon_text_gap: 12.0,
        }
    }

    fn typography(&self) -> DesignTypography {
        DesignTypography::default()
    }

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

    fn item_height(&self) -> f32 {
        56.0
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Glassmorphism
    }
}

