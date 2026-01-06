//! Story Definitions for Script Kit Components
//!
//! This module contains all the story definitions for the storybook.
//! Stories are manually registered in get_all_stories().

mod arg_prompt_stories;
mod button_stories;
mod design_token_stories;
mod drop_prompt_stories;
mod env_prompt_stories;
mod form_field_stories;
mod header_design_variations;
mod header_logo_variations;
mod header_raycast_variations;
mod header_stories;
mod list_item_stories;
mod logo_centering_stories;
mod path_prompt_stories;
mod scrollbar_stories;
mod select_prompt_stories;
mod toast_stories;

use crate::storybook::StoryEntry;
use std::sync::OnceLock;

// Re-export story types
pub use arg_prompt_stories::ArgPromptStory;
pub use button_stories::ButtonStory;
pub use design_token_stories::DesignTokenStory;
pub use drop_prompt_stories::DropPromptStory;
pub use env_prompt_stories::EnvPromptStory;
pub use form_field_stories::FormFieldStory;
pub use header_design_variations::HeaderDesignVariationsStory;
pub use header_logo_variations::HeaderLogoVariationsStory;
pub use header_raycast_variations::HeaderRaycastVariationsStory;
pub use header_stories::HeaderVariationsStory;
pub use list_item_stories::ListItemStory;
pub use logo_centering_stories::LogoCenteringStory;
pub use path_prompt_stories::PathPromptStory;
pub use scrollbar_stories::ScrollbarStory;
pub use select_prompt_stories::SelectPromptStory;
pub use toast_stories::ToastStory;

/// Static storage for all stories
static ALL_STORIES: OnceLock<Vec<StoryEntry>> = OnceLock::new();

/// Get all registered stories
pub fn get_all_stories() -> &'static Vec<StoryEntry> {
    ALL_STORIES.get_or_init(|| {
        vec![
            // Foundation
            StoryEntry::new(Box::new(DesignTokenStory)),
            // Components
            StoryEntry::new(Box::new(ButtonStory)),
            StoryEntry::new(Box::new(ToastStory)),
            StoryEntry::new(Box::new(FormFieldStory)),
            StoryEntry::new(Box::new(ListItemStory)),
            StoryEntry::new(Box::new(ScrollbarStory)),
            // Layouts
            StoryEntry::new(Box::new(HeaderVariationsStory)),
            StoryEntry::new(Box::new(HeaderDesignVariationsStory)),
            StoryEntry::new(Box::new(HeaderRaycastVariationsStory)),
            StoryEntry::new(Box::new(HeaderLogoVariationsStory)),
            StoryEntry::new(Box::new(LogoCenteringStory)),
            // Prompts
            StoryEntry::new(Box::new(ArgPromptStory)),
            StoryEntry::new(Box::new(DropPromptStory)),
            StoryEntry::new(Box::new(EnvPromptStory)),
            StoryEntry::new(Box::new(PathPromptStory)),
            StoryEntry::new(Box::new(SelectPromptStory)),
        ]
    })
}
