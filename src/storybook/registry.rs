//! Story registry - manual registration for compile-time story collection
//!
//! Instead of using inventory (which has const-fn requirements in newer Rust),
//! we use a manual registration approach where stories are collected in the
//! stories module and returned via get_all_stories().

use super::{Story, StorySurface};

/// Entry for a registered story
pub struct StoryEntry {
    pub story: Box<dyn Story>,
}

impl StoryEntry {
    pub fn new(story: Box<dyn Story>) -> Self {
        Self { story }
    }
}

/// Get all registered stories
/// This function is implemented in the stories module
pub fn all_stories() -> impl Iterator<Item = &'static StoryEntry> {
    crate::stories::get_all_stories().iter()
}

/// Find stories by category
pub fn stories_by_category(category: &str) -> Vec<&'static StoryEntry> {
    all_stories()
        .filter(|e| e.story.category() == category)
        .collect()
}

/// Find stories whose `surface()` matches the requested `StorySurface`.
pub fn stories_by_surface(surface: StorySurface) -> Vec<&'static StoryEntry> {
    all_stories()
        .filter(|e| e.story.surface() == surface)
        .collect()
}

/// Pick a deterministic startup story for compare mode.
/// Returns the first story whose `variants().len() > 1`, or `None`.
pub fn first_story_with_multiple_variants() -> Option<&'static StoryEntry> {
    all_stories().find(|e| e.story.variants().len() > 1)
}

/// Get unique categories
pub fn all_categories() -> Vec<&'static str> {
    let mut categories: Vec<_> = all_stories().map(|e| e.story.category()).collect();
    categories.sort();
    categories.dedup();
    categories
}

#[cfg(test)]
mod tests {
    use super::{all_categories, first_story_with_multiple_variants, stories_by_surface};
    use crate::storybook::StorySurface;

    #[test]
    fn surface_queries_are_safe_for_known_surfaces() {
        let _ = stories_by_surface(StorySurface::Footer);
        let _ = stories_by_surface(StorySurface::Header);
        let _ = stories_by_surface(StorySurface::ActionDialog);
    }

    #[test]
    fn comparable_story_helper_only_returns_valid_entries() {
        assert!(
            first_story_with_multiple_variants().is_none(),
            "the reset storybook should not expose compare-ready stories"
        );
    }

    #[test]
    fn main_menu_surface_has_single_story() {
        assert!(
            stories_by_surface(StorySurface::MainMenu)
                .into_iter()
                .any(|entry| entry.story.id() == "main-menu"),
            "Main Menu surface should expose the canonical main-menu story"
        );
    }

    #[test]
    fn other_surfaces_are_empty_after_storybook_reset() {
        assert!(
            stories_by_surface(StorySurface::Footer).is_empty(),
            "Footer stories should be removed from the reset storybook"
        );
        assert!(
            stories_by_surface(StorySurface::Header).is_empty(),
            "Header stories should be removed from the reset storybook"
        );
        assert!(
            stories_by_surface(StorySurface::ActionDialog).is_empty(),
            "Action dialog stories should be removed from the reset storybook"
        );
    }

    #[test]
    fn reset_storybook_exposes_single_launcher_category() {
        assert_eq!(all_categories(), vec!["Launcher"]);
    }
}
