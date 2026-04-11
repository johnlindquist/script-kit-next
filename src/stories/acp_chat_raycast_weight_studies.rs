use gpui::*;

use crate::storybook::{
    acp_chat_raycast_weight_story_variants, render_acp_chat_raycast_weight_compare_thumbnail,
    render_acp_chat_raycast_weight_gallery, render_acp_chat_raycast_weight_story_preview, Story,
    StorySurface, StoryVariant,
};

pub struct AcpChatRaycastWeightStudiesStory;

impl Story for AcpChatRaycastWeightStudiesStory {
    fn id(&self) -> &'static str {
        "acp-chat-raycast-weight-studies"
    }

    fn name(&self) -> &'static str {
        "ACP Chat Raycast Weight Studies (15)"
    }

    fn category(&self) -> &'static str {
        "AI"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render(&self) -> AnyElement {
        render_acp_chat_raycast_weight_gallery()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_acp_chat_raycast_weight_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_acp_chat_raycast_weight_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        acp_chat_raycast_weight_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::AcpChatRaycastWeightStudiesStory;
    use crate::storybook::Story;

    #[test]
    fn acp_chat_raycast_story_has_fifteen_variants() {
        let story = AcpChatRaycastWeightStudiesStory;
        assert_eq!(story.variants().len(), 15);
    }
}
