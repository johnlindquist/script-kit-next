/// Apple HIG design tokens
#[derive(Debug, Clone, Copy)]
pub struct AppleHIGDesignTokens;

impl DesignTokens for AppleHIGDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0x1c1c1e, // iOS dark background
            background_secondary: 0x2c2c2e,
            background_tertiary: 0x3a3a3c,
            background_selected: 0x3a3a3c,
            background_hover: 0x2c2c2e,

            text_primary: 0xffffff,
            text_secondary: 0xebebf5, // iOS secondary label
            text_muted: 0x8e8e93,     // iOS tertiary label
            text_dimmed: 0x636366,    // iOS quaternary label
            text_on_accent: 0xffffff,

            accent: 0x0a84ff,           // iOS blue
            accent_secondary: 0x5e5ce6, // iOS indigo
            success: 0x30d158,          // iOS green
            warning: 0xff9f0a,          // iOS orange
            error: 0xff453a,            // iOS red

            border: 0x38383a,
            border_subtle: 0x2c2c2e,
            border_focus: 0x0a84ff,

            shadow: 0x00000040,
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
            padding_xs: 4.0,
            padding_sm: 8.0,
            padding_md: 12.0,
            padding_lg: 16.0,
            padding_xl: 20.0,

            gap_sm: 4.0,
            gap_md: 8.0,
            gap_lg: 12.0,

            margin_sm: 4.0,
            margin_md: 8.0,
            margin_lg: 16.0,

            item_padding_x: 16.0,
            item_padding_y: 11.0,
            icon_text_gap: 12.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        44.0 // iOS standard row height
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::AppleHIG
    }
}

/// Material Design 3 tokens
#[derive(Debug, Clone, Copy)]
pub struct Material3DesignTokens;

impl DesignTokens for Material3DesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0x1c1b1f, // M3 dark surface
            background_secondary: 0x2b2930,
            background_tertiary: 0x36343b,
            background_selected: 0x4f378b, // M3 primary container
            background_hover: 0x36343b,

            text_primary: 0xe6e1e5,   // M3 on-surface
            text_secondary: 0xcac4d0, // M3 on-surface-variant
            text_muted: 0x938f99,
            text_dimmed: 0x79747e,
            text_on_accent: 0xeaddff, // M3 on-primary-container

            accent: 0xd0bcff,           // M3 primary
            accent_secondary: 0xccc2dc, // M3 secondary
            success: 0xa5d6a7,
            warning: 0xffcc80,
            error: 0xf2b8b5, // M3 error

            border: 0x49454f, // M3 outline
            border_subtle: 0x36343b,
            border_focus: 0xd0bcff,

            shadow: 0x00000040,
        }
    }

    fn spacing(&self) -> DesignSpacing {
        DesignSpacing {
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
            item_padding_y: 12.0,
            icon_text_gap: 16.0, // M3 uses larger icon gaps
        }
    }

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

    fn item_height(&self) -> f32 {
        56.0 // M3 list item height
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Material3
    }
}

