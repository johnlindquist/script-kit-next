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
// ArgPrompt is rendered inline via AppView::ArgPrompt in main.rs
// and rendered by render_prompts/arg.rs

// In SDK scripts:
const choice = await arg("Select a fruit", ["Apple", "Banana", "Cherry"]);

// Or with Choice objects:
const choice = await arg("Select a fruit", [
    { name: "Apple", value: "apple" },
    { name: "Banana", value: "banana" },
]);
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
