use gpui::*;

use crate::storybook::{
    notes_window_story_variants, render_notes_window_story_preview, Story, StorySurface,
    StoryVariant,
};

pub struct NotesWindowVariationsStory;

impl Story for NotesWindowVariationsStory {
    fn id(&self) -> &'static str {
        "notes-window"
    }

    fn name(&self) -> &'static str {
        "Notes Window"
    }

    fn category(&self) -> &'static str {
        "Windows"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::NotesWindow
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_notes_window_story_preview(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        render_notes_window_story_preview("current")
    }

    fn variants(&self) -> Vec<StoryVariant> {
        notes_window_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::NotesWindowVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn notes_window_story_has_correct_surface() {
        let story = NotesWindowVariationsStory;
        assert_eq!(story.surface(), StorySurface::NotesWindow);
    }

    #[test]
    fn notes_window_story_has_three_variants() {
        let story = NotesWindowVariationsStory;
        assert_eq!(story.variants().len(), 3);
    }

    #[test]
    fn notes_window_story_id_matches_adoptable_surface() {
        let story = NotesWindowVariationsStory;
        assert_eq!(story.id(), "notes-window");
    }
}
