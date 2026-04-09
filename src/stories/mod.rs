//! Story Definitions for Script Kit Components
//!
//! The storybook has been reset to a single canonical story that represents
//! the current main menu surface.

mod main_menu_variations;

use crate::storybook::StoryEntry;
use std::sync::LazyLock;

pub use main_menu_variations::MainMenuStory;

/// Static storage for all stories.
static ALL_STORIES: LazyLock<Vec<StoryEntry>> =
    LazyLock::new(|| vec![StoryEntry::new(Box::new(MainMenuStory))]);

/// Get all registered stories.
pub fn get_all_stories() -> &'static Vec<StoryEntry> {
    &ALL_STORIES
}
