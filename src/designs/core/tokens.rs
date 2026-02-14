use super::variant::DesignVariant;
use crate::designs::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    DesignTokens, GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};

/// Check if a variant uses the default renderer
///
/// When true, ScriptListApp should use its built-in render_script_list()
/// instead of delegating to a custom design renderer.
///
/// Currently all variants use the default renderer until custom implementations
/// are added. In the future, only DesignVariant::Default will return true here.
#[allow(dead_code)]
pub fn uses_default_renderer(variant: DesignVariant) -> bool {
    // When a custom renderer is added for a variant, remove it from this match
    // Minimal, RetroTerminal now have custom renderers
    matches!(
        variant,
        DesignVariant::Default
            | DesignVariant::Glassmorphism
            | DesignVariant::Brutalist
            | DesignVariant::NeonCyberpunk
            | DesignVariant::Paper
            | DesignVariant::AppleHIG
            | DesignVariant::Material3
            | DesignVariant::Compact
            | DesignVariant::Playful
    )
}

/// Get the item height for a design variant
///
/// Different designs use different item heights for their aesthetic.
/// This should be used when setting up uniform_list.
///
/// Note: This function now uses the DesignTokens system for consistency.
/// The constants MINIMAL_ITEM_HEIGHT, TERMINAL_ITEM_HEIGHT, etc. are
/// kept for backward compatibility with existing renderers.
#[allow(dead_code)]
pub fn get_item_height(variant: DesignVariant) -> f32 {
    // Use tokens for authoritative item heights
    get_tokens(variant).item_height()
}

/// Get design tokens for a design variant
///
/// Returns a boxed trait object that provides the complete design token set
/// for the specified variant. Use this when you need dynamic dispatch.
pub fn get_tokens(variant: DesignVariant) -> Box<dyn DesignTokens> {
    match variant {
        DesignVariant::Default => Box::new(DefaultDesignTokens),
        DesignVariant::Minimal => Box::new(MinimalDesignTokens),
        DesignVariant::RetroTerminal => Box::new(RetroTerminalDesignTokens),
        DesignVariant::Glassmorphism => Box::new(GlassmorphismDesignTokens),
        DesignVariant::Brutalist => Box::new(BrutalistDesignTokens),
        DesignVariant::NeonCyberpunk => Box::new(NeonCyberpunkDesignTokens),
        DesignVariant::Paper => Box::new(PaperDesignTokens),
        DesignVariant::AppleHIG => Box::new(AppleHIGDesignTokens),
        DesignVariant::Material3 => Box::new(Material3DesignTokens),
        DesignVariant::Compact => Box::new(CompactDesignTokens),
        DesignVariant::Playful => Box::new(PlayfulDesignTokens),
    }
}

/// Get design tokens for a design variant (static dispatch version)
///
/// Returns the concrete token type for the specified variant.
/// Use this when you know the variant at compile time for better performance.
#[allow(dead_code)]
pub fn get_tokens_static<T: DesignTokens + Copy + Default>() -> T {
    T::default()
}
