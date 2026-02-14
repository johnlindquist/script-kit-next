//! Design token stories for the storybook - showcases colors, spacing, typography

use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;

/// Story showcasing design tokens (colors, spacing, typography)
pub struct DesignTokenStory;

impl Story for DesignTokenStory {
    fn id(&self) -> &'static str {
        "design-tokens"
    }

    fn name(&self) -> &'static str {
        "Design Tokens"
    }

    fn category(&self) -> &'static str {
        "Foundation"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = &theme.colors;

        story_container()
            .child(
                story_section("Background Colors").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .flex_wrap()
                        .child(color_swatch("Main", colors.background.main))
                        .child(color_swatch("Title Bar", colors.background.title_bar))
                        .child(color_swatch("Search Box", colors.background.search_box))
                        .child(color_swatch("Log Panel", colors.background.log_panel)),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Text Colors").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .flex_wrap()
                        .child(color_swatch("Primary", colors.text.primary))
                        .child(color_swatch("Secondary", colors.text.secondary))
                        .child(color_swatch("Tertiary", colors.text.tertiary))
                        .child(color_swatch("Muted", colors.text.muted))
                        .child(color_swatch("Dimmed", colors.text.dimmed)),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Accent Colors").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .flex_wrap()
                        .child(color_swatch("Selected", colors.accent.selected))
                        .child(color_swatch(
                            "Selected Subtle",
                            colors.accent.selected_subtle,
                        )), // Note: button_text removed - use accent.selected for button text
                ),
            )
            .child(story_divider())
            .child(
                story_section("UI Colors").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .flex_wrap()
                        .child(color_swatch("Border", colors.ui.border))
                        .child(color_swatch("Success", colors.ui.success))
                        .child(color_swatch("Warning", colors.ui.warning))
                        .child(color_swatch("Error", colors.ui.error))
                        .child(color_swatch("Info", colors.ui.info)),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Spacing Scale").child(
                    div()
                        .flex()
                        .flex_row()
                        .items_end()
                        .gap_2()
                        .child(spacing_box("4px", 4.0))
                        .child(spacing_box("8px", 8.0))
                        .child(spacing_box("12px", 12.0))
                        .child(spacing_box("16px", 16.0))
                        .child(spacing_box("24px", 24.0))
                        .child(spacing_box("32px", 32.0))
                        .child(spacing_box("48px", 48.0)),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Typography").child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(colors.text.primary))
                                .child("Heading XL - Bold"),
                        )
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(colors.text.primary))
                                .child("Heading Large - Semibold"),
                        )
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(colors.text.primary))
                                .child("Body Base - Medium"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(colors.text.secondary))
                                .child("Body Small - Regular"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text.muted))
                                .child("Caption XS - Regular"),
                        ),
                ),
            )
            .child(story_divider())
            .child(
                story_section("Border Radius").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .child(radius_box("None", 0.0))
                        .child(radius_box("SM", 4.0))
                        .child(radius_box("MD", 6.0))
                        .child(radius_box("LG", 8.0))
                        .child(radius_box("XL", 12.0))
                        .child(radius_box("Full", 50.0)),
                ),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant {
            name: "default".into(),
            description: Some("Default theme tokens".into()),
            ..Default::default()
        }]
    }
}

/// Render a color swatch with hex value
fn color_swatch(name: &str, color: u32) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(
            div()
                .w(px(60.))
                .h(px(60.))
                .rounded_md()
                .bg(rgb(color))
                .border_1()
                .border_color(rgba(0xffffff22)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xcccccc))
                .child(name.to_string()),
        )
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(rgb(0x888888))
                .child(format!("#{:06x}", color)),
        )
}

/// Render a spacing demonstration box
fn spacing_box(label: &str, size: f32) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(div().w(px(size)).h(px(size)).bg(rgb(0x4a90d9)).rounded_sm())
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
}

/// Render a border radius demonstration
fn radius_box(label: &str, radius: f32) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(
            div()
                .w(px(50.))
                .h(px(50.))
                .bg(rgb(0x4a90d9))
                .rounded(px(radius)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
}
