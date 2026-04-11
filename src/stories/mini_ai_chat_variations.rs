//! Mini AI Chat — Design Variations (Runtime Fixture Host)
//!
//! 5 distinct aesthetics for the mini AI chat window.
//! Each variant is rendered through the shared runtime-fixture host,
//! matching the same deterministic fixture pattern used by Main Menu and Notes.

use gpui::*;

use crate::storybook::{
    mini_ai_chat_story_variants, render_mini_ai_chat_compare_thumbnail,
    render_mini_ai_chat_story_preview, Story, StorySurface, StoryVariant,
};

pub struct MiniAiChatVariationsStory;

impl Story for MiniAiChatVariationsStory {
    fn id(&self) -> &'static str {
        "mini-ai-chat-variations"
    }

    fn name(&self) -> &'static str {
        "Mini AI Chat Redesign (5)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mini_ai_chat_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mini_ai_chat_compare_thumbnail(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        render_mini_ai_chat_story_preview("current")
    }

    fn variants(&self) -> Vec<StoryVariant> {
        mini_ai_chat_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::MiniAiChatVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn mini_ai_chat_story_is_compare_ready() {
        let story = MiniAiChatVariationsStory;
        assert_eq!(story.surface(), StorySurface::MiniAiChat);
        assert_eq!(story.variants().len(), 5);
    }
}
