//! EnvPrompt Story Variations
//!
//! Showcases the EnvPrompt component in various configurations:
//! - Basic environment variable input
//! - Masked/hidden value input (for secrets)
//! - With validation states
//! - With suggestions/hints

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;

/// Story showcasing EnvPrompt variations
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

/// V5: Secret input with value (masked)
fn render_secret_with_value(theme: &Theme) -> impl IntoElement {
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
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("****************"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V6: API key style with branding
fn render_api_key_style(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let accent = theme.colors.accent.selected;

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
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded(px(4.))
                        .bg(rgba((accent << 8) | 0x33)) // ~0.2 opacity
                        .text_xs()
                        .text_color(rgb(accent))
                        .child("OPENAI_API_KEY"),
                )
                .child(render_cursor(text_primary))
                .child(div().text_lg().text_color(rgb(text_muted)).child("sk-...")),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

// ============================================================================
// VARIATION 7-9: Validation States
// ============================================================================

/// V7: Valid state with checkmark
fn render_valid_state(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let success_color = 0x4ade80; // Green

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .border_l_2()
        .border_color(rgb(success_color))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_check_icon(success_color))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("ghp_validtoken12345"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(success_color))
                .child("Valid token format"),
        )
        .child(sep_pipe(theme))
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V8: Error state
fn render_error_state(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let error_color = 0xf87171; // Red

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .border_l_2()
        .border_color(rgb(error_color))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_error_icon(error_color))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("invalid-token"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(error_color))
                .child("Token must start with 'ghp_'"),
        )
        .child(sep_pipe(theme))
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V9: Warning state
fn render_warning_state(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let warning_color = 0xfbbf24; // Amber

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .border_l_2()
        .border_color(rgb(warning_color))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_warning_icon(warning_color))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("ghp_shorttoken"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(warning_color))
                .child("Token appears short - verify it's complete"),
        )
        .child(sep_pipe(theme))
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

// ============================================================================
// VARIATION 10-12: With Suggestions/Hints
// ============================================================================

/// V10: With hint text below
fn render_with_hint(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let text_dimmed = theme.colors.text.dimmed;

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
                                .child("Enter DATABASE_URL"),
                        ),
                )
                .child(render_submit_button(theme))
                .child(sep_pipe(theme))
                .child(render_logo(theme)),
        )
        .child(
            div()
                .px_4()
                .pb_2()
                .text_xs()
                .text_color(rgb(text_dimmed))
                .child("Tip: Get this from your database provider's connection settings"),
        )
}

/// V11: With example value
fn render_with_example(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let text_dimmed = theme.colors.text.dimmed;
    let border = theme.colors.ui.border;

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
                                .child("Enter WEBHOOK_URL"),
                        ),
                )
                .child(render_submit_button(theme))
                .child(sep_pipe(theme))
                .child(render_logo(theme)),
        )
        .child(
            div()
                .px_4()
                .pb_2()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child("Example:"),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded(px(4.))
                        .bg(rgba((border << 8) | 0x4D)) // ~0.3 opacity
                        .font_family("Menlo")
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .child("https://hooks.slack.com/services/T00/B00/XXX"),
                ),
        )
}

/// V12: With format suggestion
fn render_with_format(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let text_dimmed = theme.colors.text.dimmed;
    let accent = theme.colors.accent.selected;

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
                                .child("Enter AWS_ACCESS_KEY_ID"),
                        ),
                )
                .child(render_submit_button(theme))
                .child(sep_pipe(theme))
                .child(render_logo(theme)),
        )
        .child(
            div()
                .px_4()
                .pb_2()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child("Format:"),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded(px(4.))
                                .bg(rgba((accent << 8) | 0x26)) // ~0.15 opacity
                                .text_xs()
                                .text_color(rgb(accent))
                                .child("AKIA"),
                        )
                        .child(div().text_xs().text_color(rgb(text_dimmed)).child("+"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child("16 alphanumeric characters"),
                        ),
                ),
        )
}
