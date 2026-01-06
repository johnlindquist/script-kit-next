//! ArgPrompt Component Stories
//!
//! Showcases ArgPrompt variations including:
//! - Basic text input
//! - With placeholder text
//! - With validation states
//! - With hints
//! - Focused/unfocused states

use gpui::*;

use crate::storybook::{
    code_block, story_container, story_divider, story_section, Story, StoryVariant,
};
use crate::theme::Theme;

/// Story showcasing ArgPrompt component variations
pub struct ArgPromptStory;

impl Story for ArgPromptStory {
    fn id(&self) -> &'static str {
        "arg-prompt"
    }

    fn name(&self) -> &'static str {
        "ArgPrompt"
    }

    fn category(&self) -> &'static str {
        "Prompts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = &theme.colors;

        story_container()
            .child(
                story_section("Basic Text Input")
                    .child(variation_item(
                        "Default state",
                        render_basic_input(colors, "", "Type a command..."),
                    ))
                    .child(variation_item(
                        "With typed text",
                        render_basic_input(colors, "hello world", "Type a command..."),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Placeholder Variations")
                    .child(variation_item(
                        "Short placeholder",
                        render_basic_input(colors, "", "Search..."),
                    ))
                    .child(variation_item(
                        "Long placeholder",
                        render_basic_input(
                            colors,
                            "",
                            "Enter your script name or search for existing scripts...",
                        ),
                    ))
                    .child(variation_item(
                        "With icon",
                        render_input_with_icon(colors, "", "Search scripts"),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Validation States")
                    .child(variation_item(
                        "Valid input",
                        render_validation_state(colors, "valid-script.ts", ValidationState::Valid),
                    ))
                    .child(variation_item(
                        "Invalid input",
                        render_validation_state(colors, "invalid name!", ValidationState::Invalid),
                    ))
                    .child(variation_item(
                        "Warning",
                        render_validation_state(colors, "deprecated-api", ValidationState::Warning),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Hints")
                    .child(variation_item(
                        "Keyboard hint",
                        render_with_hint(colors, "", "Search...", "Press Enter to submit"),
                    ))
                    .child(variation_item(
                        "Tab completion hint",
                        render_with_hint(colors, "sc", "Search...", "Tab to autocomplete"),
                    ))
                    .child(variation_item(
                        "Shortcut hint",
                        render_with_shortcut(colors, "", "Search...", "⌘K"),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Focus States")
                    .child(variation_item(
                        "Unfocused",
                        render_focus_state(colors, false),
                    ))
                    .child(variation_item("Focused", render_focus_state(colors, true)))
                    .child(variation_item(
                        "Focused with selection",
                        render_focused_with_selection(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Choice List")
                    .child(variation_item(
                        "Empty filter",
                        render_with_choices(colors, "", vec!["Apple", "Banana", "Cherry"]),
                    ))
                    .child(variation_item(
                        "Filtered list",
                        render_with_choices(colors, "a", vec!["Apple", "Banana"]),
                    ))
                    .child(variation_item(
                        "No matches",
                        render_no_matches(colors, "xyz"),
                    )),
            )
            .child(story_divider())
            .child(story_section("Usage").child(code_block(
                r#"
use crate::prompts::ArgPrompt;
use crate::protocol::Choice;

let choices = vec![
    Choice::new("apple", "Apple"),
    Choice::new("banana", "Banana"),
];

let prompt = ArgPrompt::new(
    "fruit-picker".to_string(),
    "Select a fruit...".to_string(),
    choices,
    focus_handle,
    on_submit,
    theme,
);
"#,
            )))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "basic".into(),
                description: Some("Basic text input".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "placeholder".into(),
                description: Some("With placeholder text".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "validation".into(),
                description: Some("Validation states".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hints".into(),
                description: Some("With hints and shortcuts".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "focus".into(),
                description: Some("Focus states".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "choices".into(),
                description: Some("With choice list".into()),
                ..Default::default()
            },
        ]
    }
}

// ============================================================================
// HELPER TYPES
// ============================================================================

#[derive(Clone, Copy)]
enum ValidationState {
    Valid,
    Invalid,
    Warning,
}

// ============================================================================
// VARIATION HELPERS
// ============================================================================

/// Wrapper for each variation item
fn variation_item(label: &str, content: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(
            div()
                .w_full()
                .bg(rgb(0x252526))
                .rounded_md()
                .overflow_hidden()
                .child(content),
        )
}

/// Basic input field
fn render_basic_input(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    placeholder: &str,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        placeholder.to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(text_color)
                .child(display_text),
        )
}

/// Input with search icon
fn render_input_with_icon(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    placeholder: &str,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        placeholder.to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .child(div().text_color(rgb(colors.text.muted)).child("🔍"))
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(text_color)
                .child(display_text),
        )
}

/// Input with validation state
fn render_validation_state(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    state: ValidationState,
) -> impl IntoElement {
    let (border_color, icon, message) = match state {
        ValidationState::Valid => (rgb(0x4ec9b0), "✓", "Valid script name"),
        ValidationState::Invalid => (rgb(0xf14c4c), "✗", "Invalid characters in name"),
        ValidationState::Warning => (rgb(0xdcdcaa), "⚠", "This API is deprecated"),
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .px_4()
                .py_3()
                .bg(rgb(colors.background.search_box))
                .border_b_2()
                .border_color(border_color)
                .child(
                    div()
                        .flex_1()
                        .text_base()
                        .text_color(rgb(colors.text.secondary))
                        .child(input_text.to_string()),
                )
                .child(div().text_color(border_color).child(icon)),
        )
        .child(
            div()
                .px_4()
                .py_1()
                .text_xs()
                .text_color(border_color)
                .child(message),
        )
}

/// Input with hint text
fn render_with_hint(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    placeholder: &str,
    hint: &str,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        placeholder.to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(text_color)
                .child(display_text),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(colors.text.muted))
                .child(hint.to_string()),
        )
}

/// Input with keyboard shortcut badge
fn render_with_shortcut(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    placeholder: &str,
    shortcut: &str,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        placeholder.to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(text_color)
                .child(display_text),
        )
        .child(
            div()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .bg(rgb(colors.background.main))
                .border_1()
                .border_color(rgb(colors.ui.border))
                .text_xs()
                .text_color(rgb(colors.text.muted))
                .child(shortcut.to_string()),
        )
}

/// Focus state visualization
fn render_focus_state(colors: &crate::theme::ColorScheme, is_focused: bool) -> impl IntoElement {
    let border_color = if is_focused {
        rgb(colors.accent.selected)
    } else {
        rgb(colors.ui.border)
    };

    let border_width = if is_focused { px(2.) } else { px(1.) };

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .border(border_width)
        .border_color(border_color)
        .rounded_md()
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(rgb(colors.text.dimmed))
                .child("Type something..."),
        );

    if is_focused {
        container = container.child(
            div()
                .w(px(2.))
                .h(px(18.))
                .bg(rgb(colors.text.primary))
                .ml_1(),
        );
    }

    container
}

/// Focused with text selection
fn render_focused_with_selection(colors: &crate::theme::ColorScheme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .border_2()
        .border_color(rgb(colors.accent.selected))
        .rounded_md()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(colors.text.secondary))
                        .child("hello "),
                )
                .child(
                    div()
                        .px_1()
                        .bg(rgb(colors.accent.selected))
                        .text_base()
                        .text_color(rgb(colors.text.primary))
                        .child("world"),
                ),
        )
}

/// With choice list
fn render_with_choices(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    choices: Vec<&str>,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        "Search...".to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    let mut container = div().flex().flex_col().w_full().child(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .w_full()
            .px_4()
            .py_3()
            .bg(rgb(colors.background.search_box))
            .border_b_1()
            .border_color(rgb(colors.ui.border))
            .child(div().text_color(rgb(colors.text.muted)).child("🔍"))
            .child(
                div()
                    .flex_1()
                    .text_base()
                    .text_color(text_color)
                    .child(display_text),
            ),
    );

    for (idx, choice) in choices.iter().enumerate() {
        let is_selected = idx == 0;
        let bg = if is_selected {
            rgb(colors.accent.selected)
        } else {
            rgb(colors.background.main)
        };
        let name_color = if is_selected {
            rgb(colors.text.primary)
        } else {
            rgb(colors.text.secondary)
        };

        container = container.child(
            div()
                .w_full()
                .px_4()
                .py_2()
                .bg(bg)
                .border_b_1()
                .border_color(rgb(colors.ui.border))
                .text_base()
                .text_color(name_color)
                .child(choice.to_string()),
        );
    }

    container
}

/// No matching choices state
fn render_no_matches(colors: &crate::theme::ColorScheme, input_text: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .w_full()
                .px_4()
                .py_3()
                .bg(rgb(colors.background.search_box))
                .border_b_1()
                .border_color(rgb(colors.ui.border))
                .child(div().text_color(rgb(colors.text.muted)).child("🔍"))
                .child(
                    div()
                        .flex_1()
                        .text_base()
                        .text_color(rgb(colors.text.secondary))
                        .child(input_text.to_string()),
                ),
        )
        .child(
            div()
                .w_full()
                .px_4()
                .py_4()
                .text_color(rgb(colors.text.dimmed))
                .child("No choices match your filter"),
        )
}

// Story is registered in stories/mod.rs via get_all_stories()
