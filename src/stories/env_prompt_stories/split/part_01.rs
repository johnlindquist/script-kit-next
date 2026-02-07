pub struct EnvPromptStory;

impl Story for EnvPromptStory {
    fn id(&self) -> &'static str {
        "env-prompt"
    }

    fn name(&self) -> &'static str {
        "Environment Prompt"
    }

    fn category(&self) -> &'static str {
        "Prompts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();

        story_container()
            .child(
                story_section("Basic Environment Variable Input")
                    .child(variation_item("1. Simple Key", render_simple_key(&theme)))
                    .child(variation_item(
                        "2. With Placeholder Text",
                        render_with_placeholder(&theme),
                    ))
                    .child(variation_item(
                        "3. With Value Entered",
                        render_with_value(&theme),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Masked/Secret Values")
                    .child(variation_item(
                        "4. Secret Input (Empty)",
                        render_secret_empty(&theme),
                    ))
                    .child(variation_item(
                        "5. Secret Input (With Value)",
                        render_secret_with_value(&theme),
                    ))
                    .child(variation_item(
                        "6. API Key Style",
                        render_api_key_style(&theme),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Validation States")
                    .child(variation_item("7. Valid State", render_valid_state(&theme)))
                    .child(variation_item("8. Error State", render_error_state(&theme)))
                    .child(variation_item(
                        "9. Warning State",
                        render_warning_state(&theme),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Suggestions/Hints")
                    .child(variation_item(
                        "10. With Hint Text",
                        render_with_hint(&theme),
                    ))
                    .child(variation_item(
                        "11. With Example Value",
                        render_with_example(&theme),
                    ))
                    .child(variation_item(
                        "12. With Format Suggestion",
                        render_with_format(&theme),
                    )),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "simple".into(),
                description: Some("Simple environment variable input".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "secret".into(),
                description: Some("Masked secret input".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "validation".into(),
                description: Some("With validation states".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hints".into(),
                description: Some("With suggestions and hints".into()),
                ..Default::default()
            },
        ]
    }
}

/// Wrapper for each variation
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

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

/// Lock icon for secrets (using emoji since we don't have the SVG)
fn render_lock_icon(color: u32) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w_5()
        .h_5()
        .text_sm()
        .text_color(rgb(color))
        .child("🔒")
}

/// Check icon for valid state
fn render_check_icon(color: u32) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w_5()
        .h_5()
        .text_sm()
        .text_color(rgb(color))
        .child("✓")
}

/// Error icon
fn render_error_icon(color: u32) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w_5()
        .h_5()
        .text_sm()
        .text_color(rgb(color))
        .child("✕")
}

/// Warning icon
fn render_warning_icon(color: u32) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w_5()
        .h_5()
        .text_sm()
        .text_color(rgb(color))
        .child("⚠")
}

/// Cursor element
fn render_cursor(color: u32) -> Div {
    div().w(px(2.)).h(px(20.)).bg(rgb(color))
}

/// Submit button with Enter key hint
fn render_submit_button(theme: &Theme) -> Div {
    let accent = theme.colors.accent.selected;
    let text_muted = theme.colors.text.muted;
    let border = theme.colors.ui.border;

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(div().text_sm().text_color(rgb(accent)).child("Submit"))
        .child(
            div()
                .ml(px(4.))
                .px(px(4.))
                .py(px(2.))
                .rounded(px(4.))
                .bg(rgba((border << 8) | 0x4D)) // ~0.3 opacity
                .text_color(rgb(text_muted))
                .text_xs()
                .child("Enter"),
        )
}

/// Pipe separator
fn sep_pipe(theme: &Theme) -> Div {
    let text_dimmed = theme.colors.text.dimmed;
    div()
        .text_sm()
        .text_color(rgba((text_dimmed << 8) | 0x99)) // ~0.6 opacity
        .child("|")
}

/// Script Kit logo
fn render_logo(theme: &Theme) -> impl IntoElement {
    let accent = theme.colors.accent.selected;
    svg()
        .path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
        .size(px(16.))
        .text_color(rgb(accent))
}

// ============================================================================
// VARIATION 1-3: Basic Environment Variable Input
// ============================================================================

/// V1: Simple key prompt
fn render_simple_key(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(render_cursor(text_primary))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_muted))
                        .child("Enter GITHUB_TOKEN"),
                ),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V2: With custom placeholder text
fn render_with_placeholder(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(render_cursor(text_primary))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_muted))
                        .child("Enter your OpenAI API key to enable AI features"),
                ),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V3: With value entered
fn render_with_value(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .text_lg()
                .text_color(rgb(text_primary))
                .child("ghp_1234567890abcdef")
                .child(render_cursor(text_primary)),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

// ============================================================================
// VARIATION 4-6: Masked/Secret Values
// ============================================================================

/// V4: Secret input empty
fn render_secret_empty(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_lock_icon(text_muted))
                .child(render_cursor(text_primary))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_muted))
                        .child("Enter SECRET_KEY"),
                ),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}
