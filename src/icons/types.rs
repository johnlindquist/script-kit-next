//! Core types for the unified icon system

mod embedded;
mod icon_ref;
mod icon_style;
mod lucide_mapping;

pub use embedded::*;
pub use icon_ref::*;
pub use icon_style::*;
pub use lucide_mapping::*;

// Re-export gpui_component types we use
pub use gpui_component::IconName as LucideIcon;
pub use gpui_component::IconNamed;
