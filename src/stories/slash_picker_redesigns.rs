use gpui::*;

use crate::storybook::context_picker_popup_playground::{
    render_slash_picker_redesign_compare_thumbnail, render_slash_picker_redesign_gallery,
    render_slash_picker_redesign_story_preview, slash_picker_redesign_story_variants,
};
use crate::storybook::{Story, StorySurface, StoryVariant};

pub struct SlashPickerRedesignsStory;

impl Story for SlashPickerRedesignsStory {
    fn id(&self) -> &'static str {
        "slash-picker-redesigns"
    }

    fn name(&self) -> &'static str {
        "Slash Picker Redesigns (7)"
    }

    fn category(&self) -> &'static str {
        "AI"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render(&self) -> AnyElement {
        render_slash_picker_redesign_gallery()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_slash_picker_redesign_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_slash_picker_redesign_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        slash_picker_redesign_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::SlashPickerRedesignsStory;
    use crate::storybook::Story;

    #[test]
    fn slash_picker_redesign_story_exposes_seven_variants() {
        let story = SlashPickerRedesignsStory;
        assert_eq!(story.variants().len(), 7);
    }
}
