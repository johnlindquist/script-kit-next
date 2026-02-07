pub struct HeaderRaycastVariationsStory;

impl Story for HeaderRaycastVariationsStory {
    fn id(&self) -> &'static str {
        "header-raycast-variations"
    }

    fn name(&self) -> &'static str {
        "Header Raycast Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Separator Styles (1-5)")
                    .child(variation_item(
                        "1. Pipe Separators (Current)",
                        render_v1_pipe_separators(colors),
                    ))
                    .child(variation_item(
                        "2. No Separators",
                        render_v2_no_separators(colors),
                    ))
                    .child(variation_item(
                        "3. Dot Separators",
                        render_v3_dot_separators(colors),
                    ))
                    .child(variation_item(
                        "4. Slash Separators",
                        render_v4_slash_separators(colors),
                    ))
                    .child(variation_item(
                        "5. Thin Line Separators",
                        render_v5_line_separators(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Spacing Variations (6-10)")
                    .child(variation_item(
                        "6. Tight Spacing",
                        render_v6_tight_spacing(colors),
                    ))
                    .child(variation_item(
                        "7. Loose Spacing",
                        render_v7_loose_spacing(colors),
                    ))
                    .child(variation_item(
                        "8. Grouped Buttons",
                        render_v8_grouped_buttons(colors),
                    ))
                    .child(variation_item(
                        "9. Spread Layout",
                        render_v9_spread_layout(colors),
                    ))
                    .child(variation_item(
                        "10. Compact All",
                        render_v10_compact_all(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Logo Variations (11-15)")
                    .child(variation_item(
                        "11. Logo Left",
                        render_v11_logo_left(colors),
                    ))
                    .child(variation_item(
                        "12. Logo Right",
                        render_v12_logo_right(colors),
                    ))
                    .child(variation_item(
                        "13. Logo Larger",
                        render_v13_logo_larger(colors),
                    ))
                    .child(variation_item(
                        "14. Logo with Border",
                        render_v14_logo_border(colors),
                    ))
                    .child(variation_item(
                        "15. Logo Circular",
                        render_v15_logo_circular(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Button Styles (16-20)")
                    .child(variation_item(
                        "16. Pill Buttons",
                        render_v16_pill_buttons(colors),
                    ))
                    .child(variation_item(
                        "17. Ghost Buttons",
                        render_v17_ghost_buttons(colors),
                    ))
                    .child(variation_item(
                        "18. Icon Only",
                        render_v18_icon_only(colors),
                    ))
                    .child(variation_item(
                        "19. Text Only",
                        render_v19_text_only(colors),
                    ))
                    .child(variation_item(
                        "20. Badge Style",
                        render_v20_badge_style(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "separator-pipe".into(),
                description: Some("1. Pipe Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-none".into(),
                description: Some("2. No Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-dot".into(),
                description: Some("3. Dot Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-slash".into(),
                description: Some("4. Slash Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-line".into(),
                description: Some("5. Line Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-tight".into(),
                description: Some("6. Tight Spacing".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-loose".into(),
                description: Some("7. Loose Spacing".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-grouped".into(),
                description: Some("8. Grouped Buttons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-spread".into(),
                description: Some("9. Spread Layout".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-compact".into(),
                description: Some("10. Compact All".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-left".into(),
                description: Some("11. Logo Left".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-right".into(),
                description: Some("12. Logo Right".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-larger".into(),
                description: Some("13. Logo Larger".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-border".into(),
                description: Some("14. Logo with Border".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-circular".into(),
                description: Some("15. Logo Circular".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-pill".into(),
                description: Some("16. Pill Buttons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-ghost".into(),
                description: Some("17. Ghost Buttons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-icon".into(),
                description: Some("18. Icon Only".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-text".into(),
                description: Some("19. Text Only".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-badge".into(),
                description: Some("20. Badge Style".into()),
                ..Default::default()
            },
        ]
    }
}

/// Wrapper for variation items with label
fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .mb_4()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x888888))
                .child(SharedString::from(label.to_string())),
        )
        .child(content)
}

/// Reusable header container with consistent styling
fn header_container(colors: PromptHeaderColors) -> Div {
    div()
        .w_full()
        .h(px(52.))
        .px_4()
        .flex()
        .flex_row()
        .items_center()
        .bg(colors.background.to_rgb())
        .border_b_1()
        .border_color(colors.border.to_rgb())
}

/// Script Kit logo as black icon inside yellow rounded rectangle
fn logo_component(size: f32, corner_radius: f32) -> impl IntoElement {
    div()
        .w(px(size))
        .h(px(size))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xffc821)) // Script Kit yellow
        .rounded(px(corner_radius))
        .child(
            svg()
                .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(size * 0.65))
                .text_color(rgb(0x000000)), // Black logo inside yellow
        )
}

/// "Script Kit" text label
fn script_kit_label(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_base()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors.text_primary.to_rgb())
        .child("Script Kit")
}

/// Ask AI hint with Tab badge
fn ask_ai_hint(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb()) // Yellow text
                .child("Ask AI"),
        )
        .child(
            div()
                .px(px(6.))
                .py(px(2.))
                .bg(colors.search_box_bg.to_rgb())
                .rounded(px(4.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb()) // Grey shortcut
                .child("Tab"),
        )
}

/// Run button with Enter hint
fn run_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.accent.to_rgb())
                .child("Run"),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("↵"),
        )
}

/// Actions button with shortcut
fn actions_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb()) // Yellow text
                .child("Actions"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb()) // Grey shortcut
                .child("⌘K"),
        )
}

/// Pipe separator
fn pipe_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_color(colors.text_dimmed.to_rgb())
        .mx_2()
        .child("|")
}

/// Dot separator
fn dot_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_color(colors.text_dimmed.to_rgb())
        .mx_2()
        .child("·")
}

/// Slash separator
fn slash_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_color(colors.text_dimmed.to_rgb())
        .mx_2()
        .child("/")
}
