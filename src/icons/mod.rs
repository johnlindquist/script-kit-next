//! Unified Icon System
//!
//! This module provides a unified icon API that works across all icon sources:
//! - gpui_component::IconName (Lucide icons from the component library)
//! - Embedded SVGs (Script Kit's custom icons)
//! - SF Symbols (macOS 11+)
//! - App bundle icons (macOS app icons by bundle ID)
//! - File paths (script-local SVGs)
//! - URLs (opt-in remote icons)
//!
//! # Design Principles
//!
//! 1. **Don't duplicate enums**: Use gpui_component::IconName directly for Lucide
//! 2. **Type erasure for render**: Return AnyElement to unify vector and raster icons
//! 3. **Tintable vs full-color**: Vector icons are tintable, app icons are not
//! 4. **Leverage gpui-component**: Route vectors through gpui_component::Icon
//! 5. **Central fallback policy**: Source-specific fallbacks, not call-site decisions
//!
//! # Icon String Format (for scripts)
//!
//! Scripts can specify icons using a URI-like format:
//! - `lucide:trash` - Lucide icon
//! - `sf:gear` - SF Symbol (macOS only)
//! - `app:com.apple.finder` - App bundle icon
//! - `file:icons/custom.svg` - Script-relative file
//! - `embedded:terminal` - Script Kit's embedded icons
//!
//! # Example
//!
//! ```ignore
//! use crate::icons::{IconRef, IconStyle, IconColor};
//!
//! // From gpui_component IconName
//! let icon = IconRef::from(gpui_component::IconName::Check);
//!
//! // From string (scripts)
//! let icon = IconRef::parse("lucide:trash");
//!
//! // With styling
//! let styled = icon.with_style(IconStyle {
//!     size: IconSize::Medium,
//!     color: IconColor::Token(ColorToken::Accent),
//!     ..Default::default()
//! });
//! ```

mod render;
mod types;

pub use render::{render_icon, render_image, IconView, ThemeColorProvider};
pub use types::{
    lucide_from_str, ColorToken, EmbeddedIcon, IconColor, IconNamed, IconRef, IconSize, IconStyle,
    LucideIcon,
};

#[cfg(test)]
mod tests;
