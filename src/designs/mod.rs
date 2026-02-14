//! Design System Module
//!
//! This module provides a pluggable design system for the script list UI.
//! Each design variant implements the `DesignRenderer` trait to provide
//! its own visual style while maintaining the same functionality.
//!

pub mod group_header_variations;
pub mod icon_variations;
mod minimal;
pub mod retro_terminal;
pub mod separator_variations;
mod traits;

pub use minimal::{MinimalColors, MinimalRenderer};
pub use retro_terminal::RetroTerminalRenderer;
pub use traits::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    DesignColors, DesignSpacing, DesignTokens, DesignTypography, DesignVisual,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};

mod core;

pub use core::*;
