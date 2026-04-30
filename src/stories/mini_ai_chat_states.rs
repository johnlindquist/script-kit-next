use gpui::*;

use crate::storybook::{
    mini_ai_chat_state_story_variants, render_mini_ai_chat_state_compare_thumbnail,
    render_mini_ai_chat_state_preview, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct MiniAiChatStatesStory;

impl Story for MiniAiChatStatesStory {
    fn id(&self) -> &'static str {
        "mini-ai-chat-states"
    }

    fn name(&self) -> &'static str {
        "Mini Agent Chat States"
    }

    fn category(&self) -> &'static str {
        "AI"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render(&self) -> AnyElement {
        render_mini_ai_chat_state_preview("welcome")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mini_ai_chat_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mini_ai_chat_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        mini_ai_chat_state_story_variants()
    }
}
