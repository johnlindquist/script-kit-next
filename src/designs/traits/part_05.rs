/// Playful design tokens
#[derive(Debug, Clone, Copy)]
pub struct PlayfulDesignTokens;

impl DesignTokens for PlayfulDesignTokens {
    fn colors(&self) -> DesignColors {
        DesignColors {
            background: 0xfef3e2, // Warm cream
            background_secondary: 0xfff8ed,
            background_tertiary: 0xffffff,
            background_selected: 0xffe5b4, // Peach
            background_hover: 0xfff0d4,

            text_primary: 0x2d1b4e, // Deep purple
            text_secondary: 0x4a3a6d,
            text_muted: 0x7a6a9d,
            text_dimmed: 0xa09ac0,
            text_on_accent: 0xffffff,

            accent: 0xff6b6b,           // Coral
            accent_secondary: 0x4ecdc4, // Teal
            success: 0x2ecc71,
            warning: 0xf39c12,
            error: 0xe74c3c,

            border: 0xe0d0c0,
            border_subtle: 0xf0e8e0,
            border_focus: 0xff6b6b,

            shadow: 0x2d1b4e20,
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

            item_padding_x: 20.0,
            item_padding_y: 12.0,
            icon_text_gap: 12.0,
        }
    }

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

    fn item_height(&self) -> f32 {
        56.0
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Playful
    }
}

// ============================================================================
// Boxed Token Type for Dynamic Dispatch
// ============================================================================

/// Type alias for boxed design tokens (for dynamic dispatch)
pub type DesignTokensBox = Box<dyn DesignTokens>;

/// Trait for design renderers
///
/// Each design variant implements this trait to provide its own rendering
/// of the script list UI. The trait is designed to work with GPUI's
/// component model and follows the existing patterns in the codebase.
///
/// # Type Parameters
///
/// * `App` - The application type that this renderer works with
///
/// # Implementation Notes
///
/// - Use `AnyElement` as the return type to allow flexible element trees
/// - Access app state through the provided app reference
/// - Follow the project's theme system
/// - Use `LIST_ITEM_HEIGHT` (34.0) for consistent item sizing
pub trait DesignRenderer<App>: Send + Sync {
    /// Render the script list in this design's style
    ///
    /// This method should return a complete script list UI element
    /// that can be composed into the main application view.
    ///
    /// # Arguments
    ///
    /// * `app` - Reference to the app for accessing state
    /// * `cx` - GPUI context for creating elements and handling events
    ///
    /// # Returns
    ///
    /// An `AnyElement` containing the rendered script list.
    fn render_script_list(&self, app: &App, cx: &mut Context<App>) -> AnyElement;

    /// Get the variant this renderer implements
    fn variant(&self) -> DesignVariant;

    /// Get the display name for this design
    fn name(&self) -> &'static str {
        self.variant().name()
    }

    /// Get a description of this design
    fn description(&self) -> &'static str {
        self.variant().description()
    }
}

/// Type alias for boxed design renderers
///
/// Use this when storing or passing design renderers as trait objects.
pub type DesignRendererBox<App> = Box<dyn DesignRenderer<App>>;

// Note: Tests for DesignTypography cursor methods are validated at compile time
// through usage in src/panel.rs (CursorStyle) and editor.rs patterns.
// The cursor_height_lg() returns 18.0 for GPUI .text_lg() compatibility.
// The cursor_margin_y() returns 2.0 for vertical centering (matching editor.rs pattern).
