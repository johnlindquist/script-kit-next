use gpui::*;

use crate::storybook::{
    dictation_state_story_variants, render_dictation_state_compare_thumbnail,
    render_dictation_state_gallery, render_dictation_state_story_preview, Story, StoryCatalogRole,
    StorySurface, StoryVariant,
};

pub struct DictationStatesStory;

impl Story for DictationStatesStory {
    fn id(&self) -> &'static str {
        "dictation-states"
    }

    fn name(&self) -> &'static str {
        "Dictation States"
    }

    fn category(&self) -> &'static str {
        "Voice"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::DictationOverlay
    }

    fn render(&self) -> AnyElement {
        render_dictation_state_gallery()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_dictation_state_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_dictation_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        dictation_state_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::DictationStatesStory;
    use crate::storybook::{Story, StoryCatalogRole, StorySurface};

    #[test]
    fn dictation_states_story_is_canonical_overlay_coverage() {
        let story = DictationStatesStory;
        assert_eq!(story.catalog_role(), StoryCatalogRole::CanonicalState);
        assert_eq!(story.surface(), StorySurface::DictationOverlay);
        assert_eq!(story.variants().len(), 10);
    }
}
