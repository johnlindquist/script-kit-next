//! Theme module - Color schemes and styling
//!
//! This module provides functionality for:
//! - Loading theme from ~/.scriptkit/kit/theme.json
//! - Color scheme definitions (dark/light mode)
//! - Focus-aware color variations
//! - Terminal ANSI color palette
//! - gpui-component theme integration
//! - Global theme service for multi-window theme sync
//!
//! # Module Structure
//!
//! - `hex_color` - Hex color parsing and serialization
//! - `types` - Theme struct definitions
//! - `helpers` - Lightweight color extraction for render closures
//! - `gpui_integration` - gpui-component theme mapping
//! - `service` - Global theme watcher service

mod color_resolver;
mod gpui_integration;
mod helpers;
pub mod hex_color;
pub mod prelude;
pub mod presets;
pub mod service;
mod types;
#[cfg(test)]
#[allow(dead_code)]
pub mod validation;

// Re-export types used externally
pub(crate) use types::relative_luminance_srgb;
pub use types::{ColorScheme, FontConfig, Theme, VibrancyMaterial};

// Re-export helper types for lightweight color extraction
pub use helpers::{
    accent_color_name, hover_overlay_bg, modal_overlay_bg, ListItemColors, PromptColors,
    ACCENT_PALETTE,
};

// Re-export color resolver for unified color access
pub use color_resolver::{ColorResolver, SpacingResolver, TypographyResolver};

// Re-export loader functions
pub use types::load_theme;

// Re-export cached theme access (use in render code instead of load_theme)
pub use types::{get_cached_theme, init_theme_cache, reload_theme_cache};

// Re-export appearance cache invalidation (called when system appearance changes)
pub use types::invalidate_appearance_cache;

// Re-export gpui integration
pub use gpui_integration::sync_gpui_component_theme;

// Keep cross-target theme exports reachable in both lib and binary builds.
const _: fn(u32) -> f32 = relative_luminance_srgb;
const _: fn() -> &'static [presets::ThemePreset] = presets::presets_cached;
const _: fn() -> &'static [presets::PresetPreviewColors] = presets::preset_preview_colors_cached;
const _: usize = core::mem::size_of::<ListItemColors>();

#[cfg(test)]
#[path = "lightweight_colors_test.rs"]
mod legacy_lightweight_colors_test;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
