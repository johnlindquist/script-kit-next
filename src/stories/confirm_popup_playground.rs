//! Story wrapper for the confirm popup playground.

use gpui::*;

use crate::storybook::{
    confirm_popup_playground_story_variants, render_confirm_popup_playground_story_preview,
    story_container, story_item, story_section, Story, StorySurface, StoryVariant,
};

pub struct ConfirmPopupPlaygroundStory;

impl Story for ConfirmPopupPlaygroundStory {
    fn id(&self) -> &'static str {
        "confirm-popup-playground"
    }

    fn name(&self) -> &'static str {
        "Confirm Popup Playground"
    }

    fn category(&self) -> &'static str {
        "Integrated Surfaces"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Shell
    }

    fn render(&self) -> gpui::AnyElement {
        let variants = self.variants();
        story_container()
            .child(story_section("Integrated confirm scenes").children(
                variants.iter().enumerate().map(|(i, variant)| {
                    story_item(
                        &format!("{}. {}", i + 1, variant.name),
                        self.render_variant(variant),
                    )
                }),
            ))
            .into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> gpui::AnyElement {
        tracing::info!(
            event = "storybook_shell_story_rendered",
            story_id = self.id(),
            variant_id = %variant.stable_id(),
            surface = self.surface().label(),
            "Rendered shell story variant"
        );
        render_confirm_popup_playground_story_preview(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        confirm_popup_playground_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::ConfirmPopupPlaygroundStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn confirm_popup_playground_story_is_compare_ready() {
        let story = ConfirmPopupPlaygroundStory;
        assert_eq!(story.surface(), StorySurface::Shell);
        assert!(story.variants().len() > 1);
    }
}
