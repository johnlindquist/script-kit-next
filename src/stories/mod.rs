//! Story Definitions for Script Kit Components
//!
//! This module contains all the story definitions for the storybook.
//! Stories are manually registered in get_all_stories().

mod actions_mini_variations;
mod actions_window_stories;
mod arg_prompt_stories;
mod at_mention_picker_variations;
mod button_stories;
mod context_indicator_variations;
mod design_token_stories;
mod drop_prompt_stories;
mod env_prompt_stories;
mod footer_action_variations;
mod footer_layout_variations;
mod form_field_stories;
mod frosted_surface_variations;
mod header_button_variations;
mod header_design_variations;
mod header_logo_variations;
mod header_raycast_variations;
mod header_stories;
mod header_tab_spacing_variations;
mod hint_button_variations;
mod input_design_variations;
mod list_item_state_variations;
mod list_item_stories;
mod logo_centering_stories;
mod main_menu_variations;
mod mini_ai_chat_variations;
mod path_prompt_stories;
mod run_button_exploration;
mod scrollbar_stories;
mod select_prompt_stories;
mod slash_command_menu_variations;
mod toast_stories;

use crate::storybook::StoryEntry;
use std::sync::LazyLock;

// Re-export story types
pub use actions_mini_variations::ActionsMiniVariationsStory;
pub use actions_window_stories::ActionsWindowStory;
pub use arg_prompt_stories::ArgPromptStory;
pub use at_mention_picker_variations::AtMentionPickerVariationsStory;
pub use button_stories::ButtonStory;
pub use context_indicator_variations::ContextIndicatorVariationsStory;
pub use design_token_stories::DesignTokenStory;
pub use drop_prompt_stories::DropPromptStory;
pub use env_prompt_stories::EnvPromptStory;
pub use footer_action_variations::FooterActionVariationsStory;
pub use footer_layout_variations::FooterLayoutVariationsStory;
pub use form_field_stories::FormFieldStory;
pub use frosted_surface_variations::FrostedSurfaceVariationsStory;
pub use header_button_variations::HeaderButtonVariationsStory;
pub use header_design_variations::HeaderDesignVariationsStory;
pub use header_logo_variations::HeaderLogoVariationsStory;
pub use header_raycast_variations::HeaderRaycastVariationsStory;
pub use header_stories::HeaderVariationsStory;
pub use header_tab_spacing_variations::HeaderTabSpacingVariationsStory;
pub use hint_button_variations::HintButtonVariationsStory;
pub use input_design_variations::InputDesignVariationsStory;
pub use list_item_state_variations::ListItemStateVariationsStory;
pub use list_item_stories::ListItemStory;
pub use logo_centering_stories::LogoCenteringStory;
pub use main_menu_variations::MainMenuVariationsStory;
pub use mini_ai_chat_variations::MiniAiChatVariationsStory;
pub use path_prompt_stories::PathPromptStory;
pub use run_button_exploration::RunButtonExplorationStory;
pub use scrollbar_stories::ScrollbarStory;
pub use select_prompt_stories::SelectPromptStory;
pub use slash_command_menu_variations::SlashCommandMenuVariationsStory;
pub use toast_stories::ToastStory;

/// Static storage for all stories
static ALL_STORIES: LazyLock<Vec<StoryEntry>> = LazyLock::new(|| {
    vec![
        // Foundation
        StoryEntry::new(Box::new(DesignTokenStory)),
        // Components
        StoryEntry::new(Box::new(ButtonStory)),
        StoryEntry::new(Box::new(ContextIndicatorVariationsStory)),
        StoryEntry::new(Box::new(ToastStory)),
        StoryEntry::new(Box::new(FormFieldStory)),
        StoryEntry::new(Box::new(ListItemStory)),
        StoryEntry::new(Box::new(ScrollbarStory)),
        StoryEntry::new(Box::new(FrostedSurfaceVariationsStory)),
        // Layouts
        StoryEntry::new(Box::new(HeaderVariationsStory)),
        StoryEntry::new(Box::new(HeaderDesignVariationsStory)),
        StoryEntry::new(Box::new(HeaderRaycastVariationsStory)),
        StoryEntry::new(Box::new(HeaderLogoVariationsStory)),
        StoryEntry::new(Box::new(HeaderTabSpacingVariationsStory)),
        StoryEntry::new(Box::new(HeaderButtonVariationsStory)),
        StoryEntry::new(Box::new(ListItemStateVariationsStory)),
        StoryEntry::new(Box::new(RunButtonExplorationStory)),
        StoryEntry::new(Box::new(LogoCenteringStory)),
        StoryEntry::new(Box::new(FooterLayoutVariationsStory)),
        StoryEntry::new(Box::new(FooterActionVariationsStory)),
        StoryEntry::new(Box::new(HintButtonVariationsStory)),
        StoryEntry::new(Box::new(InputDesignVariationsStory)),
        StoryEntry::new(Box::new(MainMenuVariationsStory)),
        StoryEntry::new(Box::new(ActionsMiniVariationsStory)),
        StoryEntry::new(Box::new(MiniAiChatVariationsStory)),
        StoryEntry::new(Box::new(ActionsWindowStory)),
        // Prompts
        StoryEntry::new(Box::new(ArgPromptStory)),
        StoryEntry::new(Box::new(DropPromptStory)),
        StoryEntry::new(Box::new(EnvPromptStory)),
        StoryEntry::new(Box::new(PathPromptStory)),
        StoryEntry::new(Box::new(SelectPromptStory)),
        // AI Chat
        StoryEntry::new(Box::new(AtMentionPickerVariationsStory)),
        StoryEntry::new(Box::new(SlashCommandMenuVariationsStory)),
    ]
});

/// Get all registered stories
pub fn get_all_stories() -> &'static Vec<StoryEntry> {
    &ALL_STORIES
}
