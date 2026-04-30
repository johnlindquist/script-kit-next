use gpui::*;

use crate::storybook::{
    actions_dialog_state_story_variants, render_actions_dialog_state_compare_thumbnail,
    render_actions_dialog_state_preview, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct ActionsDialogStatesStory;

impl Story for ActionsDialogStatesStory {
    fn id(&self) -> &'static str {
        "actions-dialog-states"
    }

    fn name(&self) -> &'static str {
        "Actions Dialog States"
    }

    fn category(&self) -> &'static str {
        "Popups"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ActionDialog
    }

    fn render(&self) -> AnyElement {
        render_actions_dialog_state_preview("default-list")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_actions_dialog_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_actions_dialog_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        actions_dialog_state_story_variants()
    }
}
