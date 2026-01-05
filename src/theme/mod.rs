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

mod gpui_integration;
mod helpers;
pub mod hex_color;
pub mod service;
mod types;

// Re-export types used externally
pub use types::{ColorScheme, Theme};

// Re-export loader functions
pub use types::load_theme;

// Re-export gpui integration
pub use gpui_integration::sync_gpui_component_theme;

// Additional exports for tests
#[cfg(test)]
pub use hex_color::{hex_color_serde, HexColor};

#[cfg(test)]
#[allow(unused_imports)]
pub use types::{
    detect_system_appearance, BackgroundOpacity, BackgroundRole, DropShadow, FontConfig,
    VibrancyMaterial, VibrancySettings,
};

#[cfg(test)]
pub use helpers::{InputFieldColors, ListItemColors};

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
