use super::{
    colors::{DesignColorTokens, DesignColors},
    spacing::{DesignSpacing, DesignSpacingTokens},
    typography::{DesignTypography, DesignTypographyTokens},
    visual::{DesignVisual, DesignVisualTokens},
};

/// Trait for design token providers.
///
/// Each design variant implements this trait to provide its complete set of
/// design tokens. This enables consistent theming across the entire application
/// while allowing each design to have its own unique visual identity.
pub trait DesignTokens: Send + Sync {
    /// Get the color tokens for this design.
    fn colors(&self) -> DesignColors;

    /// Get the spacing tokens for this design.
    fn spacing(&self) -> DesignSpacing;

    /// Get the typography tokens for this design.
    fn typography(&self) -> DesignTypography;

    /// Get the visual effect tokens for this design.
    fn visual(&self) -> DesignVisual;

    /// Get the list item height for this design (in pixels).
    ///
    /// This is used by uniform_list for virtualization.
    fn item_height(&self) -> f32;
}

/// Default token implementation for the standard design.
#[derive(Debug, Clone, Copy)]
pub struct DefaultDesignTokens;

/// Minimal design tokens.
#[derive(Debug, Clone, Copy)]
pub struct MinimalDesignTokens;

/// Retro Terminal design tokens.
#[derive(Debug, Clone, Copy)]
pub struct RetroTerminalDesignTokens;

/// Glassmorphism design tokens.
#[derive(Debug, Clone, Copy)]
pub struct GlassmorphismDesignTokens;

/// Brutalist design tokens.
#[derive(Debug, Clone, Copy)]
pub struct BrutalistDesignTokens;

/// Compact design tokens (for power users).
#[derive(Debug, Clone, Copy)]
pub struct CompactDesignTokens;

/// Neon Cyberpunk design tokens.
#[derive(Debug, Clone, Copy)]
pub struct NeonCyberpunkDesignTokens;

/// Paper design tokens.
#[derive(Debug, Clone, Copy)]
pub struct PaperDesignTokens;

/// Apple HIG design tokens.
#[derive(Debug, Clone, Copy)]
pub struct AppleHIGDesignTokens;

/// Material Design 3 tokens.
#[derive(Debug, Clone, Copy)]
pub struct Material3DesignTokens;

/// Playful design tokens.
#[derive(Debug, Clone, Copy)]
pub struct PlayfulDesignTokens;

macro_rules! impl_design_tokens {
    ($token:ty, $item_height:expr) => {
        impl DesignTokens for $token {
            fn colors(&self) -> DesignColors {
                DesignColorTokens::colors(self)
            }

            fn spacing(&self) -> DesignSpacing {
                DesignSpacingTokens::spacing(self)
            }

            fn typography(&self) -> DesignTypography {
                DesignTypographyTokens::typography(self)
            }

            fn visual(&self) -> DesignVisual {
                DesignVisualTokens::visual(self)
            }

            fn item_height(&self) -> f32 {
                $item_height
            }
        }
    };
}

impl_design_tokens!(DefaultDesignTokens, 40.0);
impl_design_tokens!(MinimalDesignTokens, 64.0);
impl_design_tokens!(RetroTerminalDesignTokens, 28.0);
impl_design_tokens!(GlassmorphismDesignTokens, 56.0);
impl_design_tokens!(BrutalistDesignTokens, 40.0);
impl_design_tokens!(CompactDesignTokens, 24.0);
impl_design_tokens!(NeonCyberpunkDesignTokens, 34.0);
impl_design_tokens!(PaperDesignTokens, 34.0);
impl_design_tokens!(AppleHIGDesignTokens, 44.0);
impl_design_tokens!(Material3DesignTokens, 56.0);
impl_design_tokens!(PlayfulDesignTokens, 56.0);
