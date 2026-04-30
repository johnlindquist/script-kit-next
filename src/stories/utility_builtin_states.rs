use gpui::*;

use crate::storybook::{
    render_utility_builtin_state_compare_thumbnail, render_utility_builtin_state_preview,
    utility_builtin_state_story_variants, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct UtilityBuiltinStatesStory;

impl Story for UtilityBuiltinStatesStory {
    fn id(&self) -> &'static str {
        "utility-builtin-states"
    }

    fn name(&self) -> &'static str {
        "Utility Built-In States"
    }

    fn category(&self) -> &'static str {
        "Built-ins"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Shell
    }

    fn render(&self) -> AnyElement {
        render_utility_builtin_state_preview("emoji-picker")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_utility_builtin_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_utility_builtin_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        utility_builtin_state_story_variants()
    }
}
