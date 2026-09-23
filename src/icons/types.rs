//! Core types for the unified icon system

mod embedded;
mod icon_ref;
mod icon_style;
mod lucide_mapping;

pub use embedded::EmbeddedIcon;
pub use icon_ref::IconRef;
pub use icon_style::{ColorToken, IconColor, IconSize, IconStyle};
pub use lucide_mapping::lucide_from_str;

// Re-export gpui_component types we use
pub use super::lucide_catalog::LucideIcon;
pub use gpui_component::IconNamed;
