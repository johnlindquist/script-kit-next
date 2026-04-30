use gpui::*;

use crate::storybook::{
    built_in_browser_state_story_variants, render_built_in_browser_state_compare_thumbnail,
    render_built_in_browser_state_preview, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct BuiltInBrowserStatesStory;

impl Story for BuiltInBrowserStatesStory {
    fn id(&self) -> &'static str {
        "built-in-browser-states"
    }

    fn name(&self) -> &'static str {
        "Built-In Browser States"
    }

    fn category(&self) -> &'static str {
        "Built-ins"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::BuiltInBrowser
    }

    fn render(&self) -> AnyElement {
        render_built_in_browser_state_preview("file-search")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_built_in_browser_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_built_in_browser_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        built_in_browser_state_story_variants()
    }
}
