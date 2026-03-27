//! Actions Dialog — Mini Redesign Variations
//!
//! 8 distilled, minimalistic compositions for the actions dialog.
//! Each variant is rendered through the shared `render_actions_dialog_presentation`
//! presenter, guaranteeing visual parity between storybook and the live dialog.

use gpui::*;

use crate::storybook::{
    actions_dialog_story_variants, render_actions_dialog_presentation,
    resolve_actions_dialog_style, ActionsDialogPresentationAction, ActionsDialogPresentationItem,
    ActionsDialogPresentationModel, Story, StorySurface, StoryVariant,
};

pub struct ActionsMiniVariationsStory;

impl Story for ActionsMiniVariationsStory {
    fn id(&self) -> &'static str {
        "actions-mini-variations"
    }

    fn name(&self) -> &'static str {
        "Actions Mini Redesign (8)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ActionDialog
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let (style, _resolution) = resolve_actions_dialog_style(Some(variant.stable_id().as_str()));
        let theme = crate::theme::get_cached_theme();
        let model = storybook_actions_dialog_model();
        render_actions_dialog_presentation(&model, style, &theme)
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        crate::storybook::story_container()
            .child(
                crate::storybook::story_section("Actions Mini Redesign").children(
                    variants.into_iter().enumerate().map(|(i, v)| {
                        crate::storybook::story_item(
                            &format!("{}. {}", i + 1, v.name),
                            self.render_variant(&v),
                        )
                    }),
                ),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        actions_dialog_story_variants()
    }
}

// ─── Shared presentation model for storybook ────────────────────────────

fn storybook_actions_dialog_model() -> ActionsDialogPresentationModel {
    ActionsDialogPresentationModel {
        context_title: Some(SharedString::from("Actions")),
        search_text: SharedString::from(""),
        search_placeholder: SharedString::from("Search actions..."),
        cursor_visible: true,
        show_search: true,
        search_at_top: false,
        show_footer: true,
        items: vec![
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: SharedString::from("Open Application"),
                subtitle: None,
                shortcut: Some(SharedString::from("↵")),
                icon_svg_path: None,
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: SharedString::from("Show in Finder"),
                subtitle: None,
                shortcut: Some(SharedString::from("⌘↵")),
                icon_svg_path: Some(SharedString::from("🔍")),
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: SharedString::from("Show Info"),
                subtitle: None,
                shortcut: Some(SharedString::from("⌘I")),
                icon_svg_path: None,
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: SharedString::from("Package Contents"),
                subtitle: None,
                shortcut: Some(SharedString::from("⌥⌘I")),
                icon_svg_path: None,
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: SharedString::from("Add to Favorites"),
                subtitle: None,
                shortcut: Some(SharedString::from("⇧⌘F")),
                icon_svg_path: None,
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: SharedString::from("Copy Path"),
                subtitle: None,
                shortcut: Some(SharedString::from("⇧⌘C")),
                icon_svg_path: None,
                is_destructive: false,
            }),
        ],
        selected_index: 0,
        hovered_index: None,
        input_mode_mouse: false,
    }
}

#[cfg(test)]
mod tests {
    use super::ActionsMiniVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn actions_mini_story_is_compare_ready() {
        let story = ActionsMiniVariationsStory;
        assert_eq!(story.surface(), StorySurface::ActionDialog);
        assert_eq!(story.variants().len(), 8);
    }

    #[test]
    fn all_variants_use_shared_presenter() {
        let story = ActionsMiniVariationsStory;
        for variant in story.variants() {
            // Each variant renders without panic through the shared presenter
            let _element = story.render_variant(&variant);
        }
    }
}
