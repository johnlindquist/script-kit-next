#![allow(unused_imports)]

//! Design System Module
//!
//! This module provides a pluggable design system for the script list UI.
//! Each design variant implements the `DesignRenderer` trait to provide
//! its own visual style while maintaining the same functionality.
//!

pub mod apple_hig;
pub mod brutalist;
pub mod compact;
mod glassmorphism;
pub mod group_header_variations;
pub mod icon_variations;
pub mod material3;
mod minimal;
pub mod neon_cyberpunk;
pub mod paper;
pub mod playful;
pub mod retro_terminal;
pub mod separator_variations;
mod traits;

// Re-export the trait and types
pub use apple_hig::{
    render_apple_hig_header, render_apple_hig_log_panel, render_apple_hig_preview_panel,
    render_apple_hig_window_container, AppleHIGRenderer, ITEM_HEIGHT as APPLE_HIG_ITEM_HEIGHT,
};
pub use brutalist::{
    render_brutalist_header, render_brutalist_list, render_brutalist_log_panel,
    render_brutalist_preview_panel, render_brutalist_window_container, BrutalistColors,
    BrutalistRenderer,
};
pub use compact::{
    render_compact_header, render_compact_log_panel, render_compact_preview_panel,
    render_compact_window_container, CompactListItem, CompactRenderer, COMPACT_ITEM_HEIGHT,
};
pub use glassmorphism::{
    render_glassmorphism_header, render_glassmorphism_log_panel,
    render_glassmorphism_preview_panel, render_glassmorphism_window_container, GlassColors,
    GlassmorphismRenderer,
};
pub use material3::{
    render_material3_header, render_material3_log_panel, render_material3_preview_panel,
    render_material3_window_container, Material3Renderer,
};
pub use minimal::{
    render_minimal_action_button, render_minimal_divider, render_minimal_empty_state,
    render_minimal_header, render_minimal_list, render_minimal_log_panel,
    render_minimal_preview_panel, render_minimal_search_bar, render_minimal_status,
    render_minimal_window_container, MinimalColors, MinimalConstants, MinimalRenderer,
    MinimalWindowConfig, MINIMAL_ITEM_HEIGHT,
};
pub use neon_cyberpunk::{
    render_neon_cyberpunk_header, render_neon_cyberpunk_log_panel,
    render_neon_cyberpunk_preview_panel, render_neon_cyberpunk_window_container,
    NeonCyberpunkRenderer,
};
pub use paper::{
    render_paper_header, render_paper_log_panel, render_paper_preview_panel,
    render_paper_window_container, PaperRenderer,
};
pub use playful::{
    render_playful_header, render_playful_log_panel, render_playful_preview_panel,
    render_playful_window_container, PlayfulColors, PlayfulRenderer,
};
pub use retro_terminal::{RetroTerminalRenderer, TerminalColors, TERMINAL_ITEM_HEIGHT};
pub use traits::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    DesignColors, DesignSpacing, DesignTokens, DesignTokensBox, DesignTypography, DesignVisual,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};
pub use traits::{DesignRenderer, DesignRendererBox};

mod core;

pub use core::*;
