use gpui::*;

use crate::storybook::{
    input_story_variants, render_input_story_preview, story_container, story_item, story_section,
    Story, StorySurface, StoryVariant,
};

pub struct InputDesignVariationsStory;

impl Story for InputDesignVariationsStory {
    fn id(&self) -> &'static str {
        "input-design-variations"
    }

    fn name(&self) -> &'static str {
        "Input Design Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Input
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_input_story_preview(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        let variants = input_story_variants();

        story_container()
            .child(story_section("Input Variations").children(
                variants.into_iter().enumerate().map(|(index, variant)| {
                    story_item(
                        &format!("{}. {}", index + 1, variant.name),
                        render_input_story_preview(&variant.stable_id()),
                    )
                }),
            ))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        input_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::InputDesignVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn input_story_is_compare_ready() {
        let story = InputDesignVariationsStory;
        assert_eq!(story.surface(), StorySurface::Input);
        assert!(story.variants().len() > 1);
    }
}
