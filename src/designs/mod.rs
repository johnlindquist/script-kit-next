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

pub use minimal::{
    render_minimal_action_button, render_minimal_divider, render_minimal_empty_state,
    render_minimal_header, render_minimal_list, render_minimal_log_panel,
    render_minimal_preview_panel, render_minimal_search_bar, render_minimal_status,
    render_minimal_window_container, MinimalColors, MinimalConstants, MinimalRenderer,
    MinimalWindowConfig, MINIMAL_ITEM_HEIGHT,
};
pub use retro_terminal::{RetroTerminalRenderer, TerminalColors, TERMINAL_ITEM_HEIGHT};
pub use traits::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    DesignColors, DesignSpacing, DesignTokens, DesignTypography, DesignVisual,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};

mod core;

pub use core::*;
