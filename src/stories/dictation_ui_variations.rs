use gpui::*;

use crate::storybook::{
    dictation_ui_story_variants, render_dictation_ui_compare_thumbnail,
    render_dictation_ui_gallery, render_dictation_ui_story_preview, Story, StorySurface,
    StoryVariant,
};

pub struct DictationUiVariationsStory;

impl Story for DictationUiVariationsStory {
    fn id(&self) -> &'static str {
        "dictation-ui-variations"
    }

    fn name(&self) -> &'static str {
        "Dictation UI Variations (21)"
    }

    fn category(&self) -> &'static str {
        "Voice"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render(&self) -> AnyElement {
        render_dictation_ui_gallery()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_dictation_ui_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_dictation_ui_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        dictation_ui_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::DictationUiVariationsStory;
    use crate::storybook::Story;

    #[test]
    fn dictation_ui_story_exposes_twenty_one_variants() {
        let story = DictationUiVariationsStory;
        assert_eq!(story.variants().len(), 21);
    }
}
