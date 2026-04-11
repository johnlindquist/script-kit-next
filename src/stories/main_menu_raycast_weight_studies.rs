use gpui::*;

use crate::storybook::{
    main_menu_raycast_weight_story_variants, render_main_menu_raycast_weight_compare_thumbnail,
    render_main_menu_raycast_weight_gallery, render_main_menu_raycast_weight_story_preview, Story,
    StorySurface, StoryVariant,
};

pub struct MainMenuRaycastWeightStudiesStory;

impl Story for MainMenuRaycastWeightStudiesStory {
    fn id(&self) -> &'static str {
        "main-menu-raycast-weight-studies"
    }

    fn name(&self) -> &'static str {
        "Main Menu Raycast Weight Studies (15)"
    }

    fn category(&self) -> &'static str {
        "Launcher"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MainMenu
    }

    fn render(&self) -> AnyElement {
        render_main_menu_raycast_weight_gallery()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_main_menu_raycast_weight_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_main_menu_raycast_weight_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        main_menu_raycast_weight_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::MainMenuRaycastWeightStudiesStory;
    use crate::storybook::Story;

    #[test]
    fn main_menu_raycast_story_has_fifteen_variants() {
        let story = MainMenuRaycastWeightStudiesStory;
        assert_eq!(story.variants().len(), 15);
    }
}
