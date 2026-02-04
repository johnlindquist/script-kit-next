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
#[allow(dead_code)]
pub mod presets;
pub mod semantic;
pub mod service;
mod types;
pub mod validation;

#[cfg(test)]
#[path = "validation_tests.rs"]
mod validation_tests;

// Re-export types used externally
pub use types::{ColorScheme, FontConfig, Theme, VibrancyMaterial};

// Re-export helper types for lightweight color extraction (allow unused - designed for incremental adoption)
#[allow(unused_imports)]
pub use helpers::{
    hover_overlay_bg, modal_overlay_bg, InputFieldColors, ListItemColors, PromptColors,
};

// Re-export semantic types (allow unused - designed for incremental adoption)
#[allow(unused_imports)]
pub use semantic::{FocusAware, SemanticColors, Surface, SurfaceStyle};

// Re-export color resolver for unified color access
#[allow(unused_imports)]
pub use color_resolver::{ColorResolver, SpacingResolver, TypographyResolver};

// Re-export validation types
#[allow(unused_imports)]
pub use validation::{validate_theme_json, Diagnostic, DiagnosticSeverity, ThemeDiagnostics};

// Re-export loader functions
pub use types::load_theme;

// Re-export cached theme access (use in render code instead of load_theme)
#[allow(unused_imports)]
pub use types::{get_cached_theme, init_theme_cache, invalidate_theme_cache, reload_theme_cache};

// Re-export appearance cache invalidation (called when system appearance changes)
pub use types::invalidate_appearance_cache;

// Re-export gpui integration
pub use gpui_integration::sync_gpui_component_theme;

// Additional exports for tests
#[cfg(test)]
pub use hex_color::{hex_color_serde, HexColor};

#[cfg(test)]
#[allow(unused_imports)]
pub use types::{
    detect_system_appearance, AppearanceMode, BackgroundOpacity, BackgroundRole, DropShadow,
    VibrancySettings,
};

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "lightweight_colors_test.rs"]
mod lightweight_colors_test;
