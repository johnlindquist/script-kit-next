//! Curated theme API surface for modules that want explicit, stable imports.
#![allow(unused_imports)]

pub use super::color_resolver::{ColorResolver, SpacingResolver, TypographyResolver};
pub use super::gpui_integration::sync_gpui_component_theme;
pub use super::helpers::{
    hover_overlay_bg, modal_overlay_bg, InputFieldColors, ListItemColors, PromptColors,
};
pub use super::semantic::{FocusAware, SemanticColors, Surface, SurfaceStyle};
pub use super::types::{
    get_cached_theme, init_theme_cache, invalidate_appearance_cache, invalidate_theme_cache,
    load_theme, reload_theme_cache, ColorScheme, FontConfig, Theme, VibrancyMaterial,
};
pub use super::validation::{
    validate_theme_json, Diagnostic, DiagnosticSeverity, ThemeDiagnostics,
};
