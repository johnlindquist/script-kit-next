use gpui::*;

use crate::storybook::{
    actions_dialog_story_variants, footer_story_variants, input_story_variants,
    mini_ai_chat_story_variants, render_actions_dialog_presentation, render_footer_story_preview,
    render_input_story_preview, render_mini_ai_chat_compare_thumbnail,
    render_mini_ai_chat_story_preview, resolve_actions_dialog_style,
    ActionsDialogPresentationAction, ActionsDialogPresentationItem, ActionsDialogPresentationModel,
    Story, StoryCatalogRole, StorySurface, StoryVariant,
};
use crate::theme::get_cached_theme;

pub struct FooterVariationsStory;

impl Story for FooterVariationsStory {
    fn id(&self) -> &'static str {
        "footer-layout-variations"
    }

    fn name(&self) -> &'static str {
        "Footer Layout Variations"
    }

    fn category(&self) -> &'static str {
        "Adoptable Surfaces"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::AdoptableVariation
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Footer
    }

    fn render(&self) -> AnyElement {
        render_footer_story_preview("raycast-exact")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_footer_story_preview(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        footer_story_variants()
    }
}

pub struct InputVariationsStory;

impl Story for InputVariationsStory {
    fn id(&self) -> &'static str {
        "input-design-variations"
    }

    fn name(&self) -> &'static str {
        "Input Layout Variations"
    }

    fn category(&self) -> &'static str {
        "Adoptable Surfaces"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::AdoptableVariation
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Input
    }

    fn render(&self) -> AnyElement {
        render_input_story_preview("bare")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_input_story_preview(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        input_story_variants()
    }
}

pub struct ActionsDialogVariationsStory;

impl Story for ActionsDialogVariationsStory {
    fn id(&self) -> &'static str {
        "actions-mini-variations"
    }

    fn name(&self) -> &'static str {
        "Actions Dialog Variations"
    }

    fn category(&self) -> &'static str {
        "Adoptable Surfaces"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::AdoptableVariation
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ActionDialog
    }

    fn render(&self) -> AnyElement {
        render_actions_dialog_story_preview("current")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_actions_dialog_story_preview(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        actions_dialog_story_variants()
    }
}

pub struct MiniAiChatVariationsStory;

impl Story for MiniAiChatVariationsStory {
    fn id(&self) -> &'static str {
        "mini-ai-chat-variations"
    }

    fn name(&self) -> &'static str {
        "Mini Agent Chat Variations"
    }

    fn category(&self) -> &'static str {
        "Adoptable Surfaces"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::AdoptableVariation
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render(&self) -> AnyElement {
        render_mini_ai_chat_story_preview("current")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mini_ai_chat_story_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_mini_ai_chat_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        mini_ai_chat_story_variants()
    }
}

fn render_actions_dialog_story_preview(stable_id: &str) -> AnyElement {
    let (style, _) = resolve_actions_dialog_style(Some(stable_id));
    let theme = get_cached_theme();
    let model = sample_actions_dialog_model();

    div()
        .w(px(420.))
        .child(render_actions_dialog_presentation(&model, style, &theme))
        .into_any_element()
}

fn sample_actions_dialog_model() -> ActionsDialogPresentationModel {
    ActionsDialogPresentationModel {
        context_title: Some("Current script".into()),
        search_text: "".into(),
        search_placeholder: "Search actions".into(),
        cursor_visible: true,
        show_search: true,
        search_at_top: true,
        show_footer: false,
        selected_index: 0,
        hovered_index: None,
        input_mode_mouse: false,
        items: vec![
            ActionsDialogPresentationItem::SectionHeader("Script".into()),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: "Run".into(),
                subtitle: Some("Execute the selected script".into()),
                shortcut: Some("Enter".into()),
                icon_svg_path: None,
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: "Open in Editor".into(),
                subtitle: Some("Edit the script source".into()),
                shortcut: Some("Cmd+O".into()),
                icon_svg_path: None,
                is_destructive: false,
            }),
            ActionsDialogPresentationItem::SectionHeader("Danger".into()),
            ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                title: "Move to Trash".into(),
                subtitle: Some("Remove this script".into()),
                shortcut: None,
                icon_svg_path: None,
                is_destructive: true,
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adoptable_surface_stories_have_expected_roles() {
        let stories: Vec<&dyn Story> = vec![
            &FooterVariationsStory,
            &InputVariationsStory,
            &ActionsDialogVariationsStory,
            &MiniAiChatVariationsStory,
        ];

        for story in stories {
            assert_eq!(story.catalog_role(), StoryCatalogRole::AdoptableVariation);
            assert!(
                story.variants().len() > 1,
                "{} should expose comparable variants",
                story.id()
            );
        }
    }
}
