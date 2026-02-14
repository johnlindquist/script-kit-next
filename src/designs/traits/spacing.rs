use super::tokens::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};

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

pub(crate) trait DesignSpacingTokens {
    fn spacing(&self) -> DesignSpacing;
}

impl DesignSpacingTokens for DefaultDesignTokens {
    fn spacing(&self) -> DesignSpacing {
        DesignSpacing::default()
    }
}

impl DesignSpacingTokens for MinimalDesignTokens {
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
}

impl DesignSpacingTokens for RetroTerminalDesignTokens {
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
}

impl DesignSpacingTokens for GlassmorphismDesignTokens {
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
}

impl DesignSpacingTokens for BrutalistDesignTokens {
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
}

impl DesignSpacingTokens for CompactDesignTokens {
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
}

impl DesignSpacingTokens for NeonCyberpunkDesignTokens {
    fn spacing(&self) -> DesignSpacing {
        DesignSpacing::default()
    }
}

impl DesignSpacingTokens for PaperDesignTokens {
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
}

impl DesignSpacingTokens for AppleHIGDesignTokens {
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
}

impl DesignSpacingTokens for Material3DesignTokens {
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
}

impl DesignSpacingTokens for PlayfulDesignTokens {
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
}
