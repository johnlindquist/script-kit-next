//! Story Definitions for Script Kit Components
//!
//! The storybook design lab exposes multiple live/adoptable surfaces for
//! compare-mode iteration and screenshot-driven verification.

mod about_surface;
mod actions_dialog_states;
mod adoptable_surface_stories;
mod built_in_browser_states;
mod component_primitives_states;
mod dictation_states;
mod main_menu_variations;
mod mini_ai_chat_states;
mod popup_component_states;
mod utility_builtin_states;

use crate::storybook::StoryEntry;
use std::sync::LazyLock;

pub use about_surface::AboutSurfaceStory;
pub use actions_dialog_states::ActionsDialogStatesStory;
pub use adoptable_surface_stories::{
    ActionsDialogVariationsStory, FooterVariationsStory, InputVariationsStory,
    MiniAiChatVariationsStory,
};
pub use built_in_browser_states::BuiltInBrowserStatesStory;
pub use component_primitives_states::ComponentPrimitivesStatesStory;
pub use dictation_states::DictationStatesStory;
pub use main_menu_variations::MainMenuStory;
pub use mini_ai_chat_states::MiniAiChatStatesStory;
pub use popup_component_states::{
    AcpChatStatesStory, ConfirmPopupStatesStory, ContextPickerPopupStatesStory,
    NotesWindowStatesStory, ShortcutRecorderStatesStory,
};
pub use utility_builtin_states::UtilityBuiltinStatesStory;

/// Static storage for all stories.
static ALL_STORIES: LazyLock<Vec<StoryEntry>> = LazyLock::new(|| {
    vec![
        StoryEntry::new(Box::new(MainMenuStory)),
        StoryEntry::new(Box::new(AboutSurfaceStory)),
        StoryEntry::new(Box::new(FooterVariationsStory)),
        StoryEntry::new(Box::new(InputVariationsStory)),
        StoryEntry::new(Box::new(ActionsDialogVariationsStory)),
        StoryEntry::new(Box::new(ActionsDialogStatesStory)),
        StoryEntry::new(Box::new(MiniAiChatVariationsStory)),
        StoryEntry::new(Box::new(MiniAiChatStatesStory)),
        StoryEntry::new(Box::new(DictationStatesStory)),
        StoryEntry::new(Box::new(ConfirmPopupStatesStory)),
        StoryEntry::new(Box::new(ContextPickerPopupStatesStory)),
        StoryEntry::new(Box::new(ShortcutRecorderStatesStory)),
        StoryEntry::new(Box::new(NotesWindowStatesStory)),
        StoryEntry::new(Box::new(AcpChatStatesStory)),
        StoryEntry::new(Box::new(BuiltInBrowserStatesStory)),
        StoryEntry::new(Box::new(ComponentPrimitivesStatesStory)),
        StoryEntry::new(Box::new(UtilityBuiltinStatesStory)),
    ]
});

/// Get all registered stories.
pub fn get_all_stories() -> &'static Vec<StoryEntry> {
    &ALL_STORIES
}
