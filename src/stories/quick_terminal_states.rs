use gpui::*;

use crate::storybook::{
    quick_terminal_state_story_variants, render_quick_terminal_state_compare_thumbnail,
    render_quick_terminal_state_preview, Story, StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct QuickTerminalStatesStory;

impl Story for QuickTerminalStatesStory {
    fn id(&self) -> &'static str {
        "quick-terminal-states"
    }

    fn name(&self) -> &'static str {
        "Quick Terminal States"
    }

    fn category(&self) -> &'static str {
        "Built-ins"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::QuickTerminal
    }

    fn render(&self) -> AnyElement {
        render_quick_terminal_state_preview("cold-empty")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_quick_terminal_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_quick_terminal_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        quick_terminal_state_story_variants()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storybook::{Story, StoryCatalogRole, StorySurface};

    #[test]
    fn quick_terminal_story_is_canonical_and_compare_ready() {
        let story = QuickTerminalStatesStory;
        assert_eq!(story.id(), "quick-terminal-states");
        assert_eq!(story.catalog_role(), StoryCatalogRole::CanonicalState);
        assert_eq!(story.surface(), StorySurface::QuickTerminal);
        assert!(story.variants().len() >= 4);
        for variant in story.variants() {
            assert_eq!(
                variant.props.get("surface").map(String::as_str),
                Some("quickTerminal")
            );
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("presenterFixture")
            );
        }
    }
}
