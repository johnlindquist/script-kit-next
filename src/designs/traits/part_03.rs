/// Brutalist design tokens
#[derive(Debug, Clone, Copy)]
pub struct BrutalistDesignTokens;

impl DesignTokens for BrutalistDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0xffffff, // White
            background_secondary: 0xf5f5f5,
            background_tertiary: 0xeeeeee,
            background_selected: 0x000000, // Black selection
            background_hover: 0xf0f0f0,

            text_primary: 0x000000, // Black
            text_secondary: 0x333333,
            text_muted: 0x666666,
            text_dimmed: 0x999999,
            text_on_accent: 0xffffff, // White on black

            accent: 0x000000,
            accent_secondary: 0xff0000, // Red accent
            success: 0x00ff00,
            warning: 0xffff00,
            error: 0xff0000,

            border: 0x000000, // Black borders
            border_subtle: 0x333333,
            border_focus: 0x000000,

            shadow: 0x00000040,
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            padding_xs: 4.0,
            padding_sm: 8.0,
            padding_md: 16.0,
            padding_lg: 24.0,
            padding_xl: 32.0,

            gap_sm: 4.0,
            gap_md: 8.0,
            gap_lg: 16.0,

            margin_sm: 4.0,
            margin_md: 8.0,
            margin_lg: 16.0,

            item_padding_x: 16.0,
            item_padding_y: 12.0,
            icon_text_gap: 12.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        40.0 // Standard list item height matching LIST_ITEM_HEIGHT constant
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Brutalist
    }
}

/// Compact design tokens (for power users)
#[derive(Debug, Clone, Copy)]
pub struct CompactDesignTokens;

impl DesignTokens for CompactDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors::default() // Use default colors
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            // Very tight spacing for density
            padding_xs: 2.0,
            padding_sm: 4.0,
            padding_md: 6.0,
            padding_lg: 8.0,
            padding_xl: 12.0,

            gap_sm: 2.0,
            gap_md: 4.0,
            gap_lg: 8.0,

            margin_sm: 2.0,
            margin_md: 4.0,
            margin_lg: 8.0,

            item_padding_x: 8.0,
            item_padding_y: 2.0,
            icon_text_gap: 6.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        24.0 // Very compact
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Compact
    }
}

// ============================================================================
// Placeholder implementations for remaining variants
// ============================================================================

/// Neon Cyberpunk design tokens
#[derive(Debug, Clone, Copy)]
pub struct NeonCyberpunkDesignTokens;

impl DesignTokens for NeonCyberpunkDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0x0a0a0f, // Near black
            background_secondary: 0x12121a,
            background_tertiary: 0x1a1a24,
            background_selected: 0x1e1e2e,
            background_hover: 0x16161f,

            text_primary: 0xffffff,
            text_secondary: 0xb0b0d0,
            text_muted: 0x8080a0,
            text_dimmed: 0x606080,
            text_on_accent: 0x000000,

            accent: 0x00ffff,           // Cyan neon
            accent_secondary: 0xff00ff, // Magenta neon
            success: 0x00ff88,
            warning: 0xffaa00,
            error: 0xff0055,

            border: 0x00ffff40, // Neon border with glow
            border_subtle: 0x00ffff20,
            border_focus: 0x00ffff,

            shadow: 0x00ffff30, // Cyan glow
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing::default()
    }

    fn typography(&self) -> DesignTypography {
        DesignTypography::default()
    }

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

    fn item_height(&self) -> f32 {
        34.0 // Compact list item height matching LIST_ITEM_HEIGHT constant
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::NeonCyberpunk
    }
}

/// Paper design tokens
#[derive(Debug, Clone, Copy)]
pub struct PaperDesignTokens;

impl DesignTokens for PaperDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0xfaf8f5, // Warm off-white
            background_secondary: 0xf5f3f0,
            background_tertiary: 0xf0ede8,
            background_selected: 0xe8e5e0,
            background_hover: 0xf0ede8,

            text_primary: 0x2c2825, // Warm dark brown
            text_secondary: 0x4a4540,
            text_muted: 0x78736e,
            text_dimmed: 0xa09a95,
            text_on_accent: 0xffffff,

            accent: 0xc04030,           // Warm red
            accent_secondary: 0x2060a0, // Ink blue
            success: 0x408040,
            warning: 0xc08020,
            error: 0xc04040,

            border: 0xd0ccc5,
            border_subtle: 0xe0dcd5,
            border_focus: 0xc04030,

            shadow: 0x20180010,
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            padding_xs: 6.0,
            padding_sm: 10.0,
            padding_md: 14.0,
            padding_lg: 20.0,
            padding_xl: 28.0,

            gap_sm: 6.0,
            gap_md: 10.0,
            gap_lg: 18.0,

            margin_sm: 6.0,
            margin_md: 10.0,
            margin_lg: 18.0,

            item_padding_x: 18.0,
            item_padding_y: 10.0,
            icon_text_gap: 10.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        34.0 // Compact list item height matching LIST_ITEM_HEIGHT constant
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Paper
    }
}

