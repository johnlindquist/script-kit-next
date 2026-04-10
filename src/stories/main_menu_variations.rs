use gpui::*;

use crate::storybook::{
    main_menu_story_variants, render_main_menu_compare_thumbnail, render_main_menu_story_preview,
    Story, StorySurface, StoryVariant,
};

pub struct MainMenuStory;

impl Story for MainMenuStory {
    fn id(&self) -> &'static str {
        "main-menu"
    }

    fn name(&self) -> &'static str {
        "Main Menu"
    }

    fn category(&self) -> &'static str {
        "Launcher"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MainMenu
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_main_menu_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_main_menu_compare_thumbnail(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        render_main_menu_story_preview("current-main-menu")
    }

    fn variants(&self) -> Vec<StoryVariant> {
        main_menu_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::MainMenuStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn main_menu_story_is_compare_ready() {
        let story = MainMenuStory;
        assert_eq!(story.surface(), StorySurface::MainMenu);
        assert_eq!(story.variants().len(), 3);
    }
}
