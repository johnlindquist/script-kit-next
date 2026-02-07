//! UnifiedListItem - A presentational list item component for all list views.
//!
//! See types.rs for type definitions and render.rs for implementation.

mod render;
mod types;

pub use render::{SectionHeader, UnifiedListItem};
pub use types::{
    Density, ItemState, LeadingContent, ListItemLayout, TextContent, TrailingContent,
    UnifiedListItemColors, SECTION_HEADER_HEIGHT,
};

// Re-export from existing list_item for backwards compatibility
#[allow(unused_imports)]
pub use crate::list_item::{GroupedListItem, GroupedListState, LIST_ITEM_HEIGHT};
