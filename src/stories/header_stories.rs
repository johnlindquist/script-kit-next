//! Header component stories for the storybook
//!
//! This module showcases the PromptHeader component with various configurations
//! to explore different header layouts and states.

use gpui::*;

use crate::components::{PromptHeader, PromptHeaderColors, PromptHeaderConfig};
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;

/// Custom story item for headers - gives full width to the header component
fn header_story_item(label: &str, header: PromptHeader) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x666666))
                .child(label.to_string()),
        )
        .child(
            div()
                .w_full()
                .bg(rgb(0x252526)) // Slightly different bg to show header bounds
                .rounded_md()
                .overflow_hidden()
                .child(header),
        )
}

/// Story showcasing the PromptHeader component
pub struct HeaderVariationsStory;

impl Story for HeaderVariationsStory {
    fn id(&self) -> &'static str {
        "header-variations"
    }

    fn name(&self) -> &'static str {
        "Header Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Header States")
                    .child(header_story_item(
                        "Current Production Header",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .cursor_visible(true)
                                .focused(true)
                                .show_ask_ai_hint(true)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "With Filter Text",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .filter_text("hello world")
                                .cursor_visible(true)
                                .focused(true)
                                .show_ask_ai_hint(true)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "With Path Prefix",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .path_prefix(Some("/Users/john/scripts/".to_string()))
                                .filter_text("my-script")
                                .cursor_visible(true)
                                .focused(true)
                                .show_ask_ai_hint(true)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "Unfocused (Cursor Hidden)",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .filter_text("some text")
                                .cursor_visible(false)
                                .focused(false)
                                .show_ask_ai_hint(true)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true),
                            colors,
                        ),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Button Configurations")
                    .child(header_story_item(
                        "Run + Actions (Default)",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .show_ask_ai_hint(true)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true)
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "Select Mode",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Select an option...")
                                .show_ask_ai_hint(true)
                                .primary_button_label("Select")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true)
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "No Actions Button",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Enter value...")
                                .show_ask_ai_hint(false)
                                .primary_button_label("Submit")
                                .primary_button_shortcut("↵")
                                .show_actions_button(false)
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "Custom Shortcut",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Filename...")
                                .show_ask_ai_hint(false)
                                .primary_button_label("Save")
                                .primary_button_shortcut("⌘S")
                                .show_actions_button(true)
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Ask AI Hint")
                    .child(header_story_item(
                        "With Ask AI Hint",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .show_ask_ai_hint(true)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true)
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "Without Ask AI Hint",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .show_ask_ai_hint(false)
                                .primary_button_label("Run")
                                .primary_button_shortcut("↵")
                                .show_actions_button(true)
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Actions Mode")
                    .child(header_story_item(
                        "Actions Search (Empty)",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .filter_text("Clipboard History")
                                .actions_mode(true)
                                .actions_search_text("")
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    ))
                    .child(header_story_item(
                        "Actions Search (With Text)",
                        PromptHeader::new(
                            PromptHeaderConfig::new()
                                .placeholder("Script Kit")
                                .filter_text("Clipboard History")
                                .actions_mode(true)
                                .actions_search_text("copy path")
                                .cursor_visible(true)
                                .focused(true),
                            colors,
                        ),
                    )),
            )
            .child(story_divider())
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "default".into(),
                description: Some("Default empty state".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "with-text".into(),
                description: Some("With filter text".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "actions-mode".into(),
                description: Some("Actions search mode".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ask-ai".into(),
                description: Some("With Ask AI hint".into()),
                ..Default::default()
            },
        ]
    }
}
