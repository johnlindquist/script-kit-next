//! Storybook - Component preview system for script-kit-gpui
//!
//! This module provides a component preview system for GPUI components.
//!
//! # Components
//!
//! - [`Story`] - Trait for defining previewable stories
//! - [`StoryBrowser`] - Main UI for browsing stories
//! - [`story_container`], [`story_section`], etc. - Layout helpers
//!

mod browser;
mod layout;
mod registry;
mod selection;
mod story;

pub use browser::StoryBrowser;
pub use layout::{code_block, story_container, story_divider, story_item, story_section};
pub use registry::{
    all_categories, all_stories, first_story_with_multiple_variants, stories_by_category,
    stories_by_surface, StoryEntry,
};
pub use selection::{
    load_selected_story_variant, load_story_selections, save_story_selections,
    StorySelectionStore,
};
pub use story::{Story, StorySurface, StoryVariant};
