use gpui::*;

use crate::storybook::{
    context_picker_popup_playground_story_variants,
    render_context_picker_popup_playground_story_preview, story_container, story_item,
    story_section, Story, StorySurface, StoryVariant,
};

pub struct ContextPickerPopupPlaygroundStory;

impl Story for ContextPickerPopupPlaygroundStory {
    fn id(&self) -> &'static str {
        "context-picker-popup-playground"
    }

    fn name(&self) -> &'static str {
        "Context Picker Popup Playground"
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
            .child(
                story_section("Integrated slash / mention popup scenes").children(
                    variants.iter().enumerate().map(|(i, variant)| {
                        story_item(
                            &format!("{}. {}", i + 1, variant.name),
                            self.render_variant(variant),
                        )
                    }),
                ),
            )
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
        render_context_picker_popup_playground_story_preview(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        context_picker_popup_playground_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::ContextPickerPopupPlaygroundStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn context_picker_popup_playground_story_is_compare_ready() {
        let story = ContextPickerPopupPlaygroundStory;
        assert_eq!(story.surface(), StorySurface::Shell);
        assert!(story.variants().len() > 1);
    }
}
