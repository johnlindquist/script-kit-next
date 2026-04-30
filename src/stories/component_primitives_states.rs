use gpui::*;

use crate::storybook::{
    component_primitive_state_story_variants, render_component_primitive_state_compare_thumbnail,
    render_component_primitive_state_preview, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct ComponentPrimitivesStatesStory;

impl Story for ComponentPrimitivesStatesStory {
    fn id(&self) -> &'static str {
        "component-primitives-states"
    }

    fn name(&self) -> &'static str {
        "Component Primitives States"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render(&self) -> AnyElement {
        render_component_primitive_state_preview("list-items")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_component_primitive_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_component_primitive_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        component_primitive_state_story_variants()
    }
}
