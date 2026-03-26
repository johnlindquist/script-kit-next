use gpui::*;

use crate::storybook::{
    main_menu_story_variants, render_main_menu_story_preview, story_container, story_item,
    story_section, Story, StorySurface, StoryVariant,
};

pub struct MainMenuVariationsStory;

impl Story for MainMenuVariationsStory {
    fn id(&self) -> &'static str {
        "main-menu-variations"
    }

    fn name(&self) -> &'static str {
        "Main Menu Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MainMenu
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_main_menu_story_preview(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        let variants = main_menu_story_variants();

        story_container()
            .child(
                story_section("Main Menu Compositions").children(
                    variants.into_iter().enumerate().map(|(index, variant)| {
                        story_item(
                            &format!("{}. {}", index + 1, variant.name),
                            render_main_menu_story_preview(&variant.stable_id()),
                        )
                    }),
                ),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        main_menu_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::MainMenuVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn main_menu_story_is_compare_ready() {
        let story = MainMenuVariationsStory;
        assert_eq!(story.surface(), StorySurface::MainMenu);
        assert!(story.variants().len() > 1);
    }
}
