/// Semantic color roles used by separator styles.
///
/// These roles intentionally avoid concrete RGB values so separator presets
/// stay aligned with theme tokens and can be remapped per design system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum SeparatorColorRole {
    UiBorder,
    UiBorderSubtle,
    UiBorderMuted,
    UiSurface,
    UiSurfaceElevated,
    UiSurfaceOverlay,
    TextMuted,
    TextSecondary,
    TextPrimary,
    TextHighContrast,
    AccentWarning,
    AccentTerminal,
    AccentTerminalMuted,
    AccentNeon,
}

impl SeparatorColorRole {
    /// Fallback RGB values used when a renderer does not provide token mapping.
    #[allow(dead_code)]
    pub const fn fallback_hex(self) -> u32 {
        match self {
            SeparatorColorRole::UiBorder => 0x464647,
            SeparatorColorRole::UiBorderSubtle => 0x3a3a3a,
            SeparatorColorRole::UiBorderMuted => 0x555555,
            SeparatorColorRole::UiSurface => 0x2a2a2a,
            SeparatorColorRole::UiSurfaceElevated => 0x3a3a3a,
            SeparatorColorRole::UiSurfaceOverlay => 0x1e1e1e,
            SeparatorColorRole::TextMuted => 0x808080,
            SeparatorColorRole::TextSecondary => 0xa0a0a0,
            SeparatorColorRole::TextPrimary => 0xaaaaaa,
            SeparatorColorRole::TextHighContrast => 0xcccccc,
            SeparatorColorRole::AccentWarning => 0xfbbf24,
            SeparatorColorRole::AccentTerminal => 0x00ff00,
            SeparatorColorRole::AccentTerminalMuted => 0x00aa00,
            SeparatorColorRole::AccentNeon => 0x00ffff,
        }
    }
}

/// Configuration parameters for rendering a separator.
///
/// Each separator style uses these parameters differently based on its
/// visual approach. Some fields may be ignored by simpler styles.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct SeparatorConfig {
    // Dimensions
    /// Total height of the separator (including padding)
    pub height: f32,

    /// Thickness of line elements (for line-based styles)
    pub line_thickness: f32,

    /// Horizontal padding from container edges
    pub padding_x: f32,

    /// Vertical padding above the separator
    pub padding_top: f32,

    /// Vertical padding below the separator
    pub padding_bottom: f32,

    /// Indent from left edge (for indented styles)
    pub indent: f32,

    // Color roles (resolved via theme tokens at render time)
    /// Primary color role for lines and decorations
    pub color_primary: SeparatorColorRole,

    /// Secondary/muted color role for subtle elements
    pub color_secondary: SeparatorColorRole,

    /// Background color role for filled styles
    pub color_background: SeparatorColorRole,

    /// Text color role for labels
    pub color_text: SeparatorColorRole,

    // Typography
    /// Font size for label text
    pub font_size: f32,

    /// Whether label should be uppercase
    pub uppercase: bool,

    /// Whether label should be bold
    pub bold: bool,

    /// Whether label should be italic
    pub italic: bool,

    /// Letter spacing adjustment (0.0 = normal)
    pub letter_spacing: f32,

    // Visual Effects
    /// Corner radius for rounded elements
    pub border_radius: f32,

    /// Opacity (0.0 - 1.0)
    pub opacity: f32,

    /// Shadow blur radius (0.0 = no shadow)
    pub shadow_blur: f32,

    /// Shadow offset Y
    pub shadow_offset_y: f32,

    /// Whether to show decorative elements
    pub show_decorations: bool,

    /// Gap between decorations and label
    pub decoration_gap: f32,
}

impl Default for SeparatorConfig {
    fn default() -> Self {
        Self {
            // Dimensions
            height: 24.0,
            line_thickness: 1.0,
            padding_x: 16.0,
            padding_top: 8.0,
            padding_bottom: 4.0,
            indent: 0.0,

            // Color roles
            color_primary: SeparatorColorRole::UiBorder,
            color_secondary: SeparatorColorRole::UiBorderSubtle,
            color_background: SeparatorColorRole::UiSurface,
            color_text: SeparatorColorRole::TextMuted,

            // Typography
            font_size: 11.0,
            uppercase: true,
            bold: false,
            italic: false,
            letter_spacing: 1.0,

            // Visual effects
            border_radius: 0.0,
            opacity: 1.0,
            shadow_blur: 0.0,
            shadow_offset_y: 0.0,
            show_decorations: true,
            decoration_gap: 8.0,
        }
    }
}
