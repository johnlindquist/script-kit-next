use gpui::*;

use crate::storybook::context_picker_popup_playground::{
    mention_picker_redesign_story_variants, render_mention_picker_redesign_compare_thumbnail,
    render_mention_picker_redesign_gallery, render_mention_picker_redesign_story_preview,
};
use crate::storybook::{Story, StorySurface, StoryVariant};

pub struct MentionPickerRedesignsStory;

impl Story for MentionPickerRedesignsStory {
    fn id(&self) -> &'static str {
        "mention-picker-redesigns"
    }

    fn name(&self) -> &'static str {
        "Mention Picker Redesigns (7)"
    }

    fn category(&self) -> &'static str {
        "AI"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render(&self) -> AnyElement {
        render_mention_picker_redesign_gallery()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mention_picker_redesign_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mention_picker_redesign_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        mention_picker_redesign_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::MentionPickerRedesignsStory;
    use crate::storybook::Story;

    #[test]
    fn mention_picker_redesign_story_exposes_seven_variants() {
        let story = MentionPickerRedesignsStory;
        assert_eq!(story.variants().len(), 7);
    }
}
