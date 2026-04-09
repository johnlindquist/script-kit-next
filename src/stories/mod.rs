//! Story Definitions for Script Kit Components
//!
//! The storybook design lab exposes multiple live/adoptable surfaces for
//! compare-mode iteration and screenshot-driven verification.

mod actions_mini_variations;
mod footer_layout_variations;
mod input_design_variations;
mod main_menu_variations;
mod mini_ai_chat_variations;

use crate::storybook::StoryEntry;
use std::sync::LazyLock;

pub use actions_mini_variations::ActionsMiniVariationsStory;
pub use footer_layout_variations::FooterLayoutVariationsStory;
pub use input_design_variations::InputDesignVariationsStory;
pub use main_menu_variations::MainMenuStory;
pub use mini_ai_chat_variations::MiniAiChatVariationsStory;

/// Static storage for all stories.
static ALL_STORIES: LazyLock<Vec<StoryEntry>> = LazyLock::new(|| {
    vec![
        StoryEntry::new(Box::new(MainMenuStory)),
        StoryEntry::new(Box::new(ActionsMiniVariationsStory)),
        StoryEntry::new(Box::new(FooterLayoutVariationsStory)),
        StoryEntry::new(Box::new(InputDesignVariationsStory)),
        StoryEntry::new(Box::new(MiniAiChatVariationsStory)),
    ]
});

/// Get all registered stories.
pub fn get_all_stories() -> &'static Vec<StoryEntry> {
    &ALL_STORIES
}
