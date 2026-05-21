use gpui::*;

use crate::storybook::{
    render_tahoe_design_system_compare_thumbnail, render_tahoe_design_system_preview,
    tahoe_design_system_story_variants, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct TahoeDesignSystemStatesStory;

impl Story for TahoeDesignSystemStatesStory {
    fn id(&self) -> &'static str {
        "tahoe-design-system-states"
    }

    fn name(&self) -> &'static str {
        "Tahoe Design System States"
    }

    fn category(&self) -> &'static str {
        "Adoptable Surfaces"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Shell
    }

    fn render(&self) -> AnyElement {
        render_tahoe_design_system_preview("tahoe-main-menu")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_tahoe_design_system_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_tahoe_design_system_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        tahoe_design_system_story_variants()
    }
}
