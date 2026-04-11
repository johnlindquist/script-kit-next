//! Story Definitions for Script Kit Components
//!
//! The storybook design lab exposes multiple live/adoptable surfaces for
//! compare-mode iteration and screenshot-driven verification.

mod dictation_ui_variations;
mod main_menu_variations;
mod mention_picker_redesigns;
mod slash_picker_redesigns;

use crate::storybook::StoryEntry;
use std::sync::LazyLock;

pub use dictation_ui_variations::DictationUiVariationsStory;
pub use main_menu_variations::MainMenuStory;
pub use mention_picker_redesigns::MentionPickerRedesignsStory;
pub use slash_picker_redesigns::SlashPickerRedesignsStory;

/// Static storage for all stories.
static ALL_STORIES: LazyLock<Vec<StoryEntry>> = LazyLock::new(|| {
    vec![
        StoryEntry::new(Box::new(MainMenuStory)),
        StoryEntry::new(Box::new(DictationUiVariationsStory)),
        StoryEntry::new(Box::new(MentionPickerRedesignsStory)),
        StoryEntry::new(Box::new(SlashPickerRedesignsStory)),
    ]
});

/// Get all registered stories.
pub fn get_all_stories() -> &'static Vec<StoryEntry> {
    &ALL_STORIES
}
