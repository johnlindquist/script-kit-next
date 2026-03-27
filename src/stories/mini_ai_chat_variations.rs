//! Mini AI Chat — Design Variations
//!
//! 5 distinct aesthetics for the mini AI chat window.
//! Each variant is rendered through the shared `render_mini_ai_chat_presentation`
//! presenter, guaranteeing visual parity between storybook and the live window.

use gpui::*;

use crate::storybook::{
    mini_ai_chat_presenter::{
        render_mini_ai_chat_presentation, MiniAiChatPresentationMessage,
        MiniAiChatPresentationModel, MiniAiChatRole, MiniAiChatSuggestion,
    },
    mini_ai_chat_variations::mini_ai_chat_story_variants,
    mini_ai_chat_variations::resolve_mini_ai_chat_style,
    Story, StorySurface, StoryVariant,
};

pub struct MiniAiChatVariationsStory;

impl Story for MiniAiChatVariationsStory {
    fn id(&self) -> &'static str {
        "mini-ai-chat-variations"
    }

    fn name(&self) -> &'static str {
        "Mini AI Chat Redesign (5)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let (style, _resolution) = resolve_mini_ai_chat_style(Some(variant.stable_id().as_str()));
        let theme = crate::theme::get_cached_theme();

        // Show conversation state for all variants in compare mode
        let model = storybook_conversation_model();
        render_mini_ai_chat_presentation(&model, style, &theme)
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        let theme = crate::theme::get_cached_theme();

        let mut sections = crate::storybook::story_container();

        // Conversation state
        sections = sections.child(
            crate::storybook::story_section("Mini AI Chat — Conversation").children(
                variants.iter().enumerate().map(|(i, v)| {
                    let (style, _) = resolve_mini_ai_chat_style(Some(v.stable_id().as_str()));
                    let model = storybook_conversation_model();
                    crate::storybook::story_item(
                        &format!("{}. {}", i + 1, v.name),
                        render_mini_ai_chat_presentation(&model, style, &theme),
                    )
                }),
            ),
        );

        // Welcome state
        sections = sections.child(
            crate::storybook::story_section("Mini AI Chat — Welcome").children(
                variants.iter().enumerate().map(|(i, v)| {
                    let (style, _) = resolve_mini_ai_chat_style(Some(v.stable_id().as_str()));
                    let model = storybook_welcome_model();
                    crate::storybook::story_item(
                        &format!("{}. {} (welcome)", i + 1, v.name),
                        render_mini_ai_chat_presentation(&model, style, &theme),
                    )
                }),
            ),
        );

        sections.into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        mini_ai_chat_story_variants()
    }
}

// ─── Fixture models ───────────────────────────────────────────────────

fn storybook_conversation_model() -> MiniAiChatPresentationModel {
    MiniAiChatPresentationModel {
        title: SharedString::from("Rust Help"),
        is_streaming: false,
        model_name: SharedString::from("Sonnet"),
        input_text: SharedString::from(""),
        input_placeholder: SharedString::from("Ask anything..."),
        messages: vec![
            MiniAiChatPresentationMessage {
                role: MiniAiChatRole::User,
                content: SharedString::from("What is Rust's ownership model?"),
            },
            MiniAiChatPresentationMessage {
                role: MiniAiChatRole::Assistant,
                content: SharedString::from(
                    "Each value has exactly one owner. When the owner goes out of scope, the value is dropped.",
                ),
            },
            MiniAiChatPresentationMessage {
                role: MiniAiChatRole::User,
                content: SharedString::from("How does borrowing work?"),
            },
        ],
        show_welcome: false,
        welcome_suggestions: vec![],
    }
}

fn storybook_welcome_model() -> MiniAiChatPresentationModel {
    MiniAiChatPresentationModel {
        title: SharedString::from("New Chat"),
        is_streaming: false,
        model_name: SharedString::from("Sonnet"),
        input_text: SharedString::from(""),
        input_placeholder: SharedString::from("Ask anything..."),
        messages: vec![],
        show_welcome: true,
        welcome_suggestions: vec![
            MiniAiChatSuggestion {
                title: SharedString::from("Summarize clipboard"),
                shortcut: SharedString::from("\u{2318}1"),
            },
            MiniAiChatSuggestion {
                title: SharedString::from("Explain selected code"),
                shortcut: SharedString::from("\u{2318}2"),
            },
            MiniAiChatSuggestion {
                title: SharedString::from("Draft a reply"),
                shortcut: SharedString::from("\u{2318}3"),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::MiniAiChatVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn mini_ai_chat_story_is_compare_ready() {
        let story = MiniAiChatVariationsStory;
        assert_eq!(story.surface(), StorySurface::MiniAiChat);
        assert_eq!(story.variants().len(), 5);
    }

    #[test]
    fn all_variants_use_shared_presenter() {
        let story = MiniAiChatVariationsStory;
        for variant in story.variants() {
            let _element = story.render_variant(&variant);
        }
    }
}
