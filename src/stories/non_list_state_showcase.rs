use gpui::*;

use crate::storybook::{
    non_list_state_showcase_story_variants, render_non_list_state_showcase_compare_thumbnail,
    render_non_list_state_showcase_preview, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct NonListStateShowcaseStory;

impl Story for NonListStateShowcaseStory {
    fn id(&self) -> &'static str {
        "non-list-state-showcase"
    }

    fn name(&self) -> &'static str {
        "Non-List State Language"
    }

    fn category(&self) -> &'static str {
        "Design Language"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::NonListState
    }

    fn render(&self) -> AnyElement {
        render_non_list_state_showcase_preview("empty")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_non_list_state_showcase_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_non_list_state_showcase_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        non_list_state_showcase_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_list_state_showcase_story_is_canonical() {
        let story = NonListStateShowcaseStory;

        assert_eq!(story.catalog_role(), StoryCatalogRole::CanonicalState);
        assert_eq!(story.surface(), StorySurface::NonListState);
        assert_eq!(story.variants().len(), 8);
    }
}
