pub struct FooterLayoutVariationsStory;

impl Story for FooterLayoutVariationsStory {
    fn id(&self) -> &'static str {
        "footer-layout-variations"
    }

    fn name(&self) -> &'static str {
        "Footer Layout (Raycast-style)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = LayoutColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Raycast-style Footer Layouts")
                    .child(variation_label("Header for input, footer for actions"))
                    .child(variation_item(
                        "1. Exact Raycast clone",
                        render_raycast_exact(colors),
                    ))
                    .child(variation_item(
                        "2. Script Kit branding (yellow accent)",
                        render_scriptkit_branded(colors),
                    ))
                    .child(variation_item(
                        "3. Minimal footer (just shortcuts)",
                        render_minimal_footer(colors),
                    ))
                    .child(variation_item(
                        "4. Footer with breadcrumb context",
                        render_breadcrumb_footer(colors),
                    ))
                    .child(variation_item(
                        "5. Centered action in footer",
                        render_centered_action(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Footer Action Variations")
                    .child(variation_label("Different ways to show the primary action"))
                    .child(variation_item(
                        "6. Icon + text action",
                        render_icon_text_action(colors),
                    ))
                    .child(variation_item(
                        "7. Primary button style",
                        render_primary_button_footer(colors),
                    ))
                    .child(variation_item(
                        "8. Ghost button style",
                        render_ghost_button_footer(colors),
                    ))
                    .child(variation_item(
                        "9. Split action (Run | More)",
                        render_split_action_footer(colors),
                    ))
                    .child(variation_item(
                        "10. Contextual actions row",
                        render_contextual_footer(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "raycast-exact".into(),
                description: Some("Exact Raycast layout".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "scriptkit-branded".into(),
                description: Some("Script Kit styling".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "minimal".into(),
                description: Some("Minimal footer".into()),
                ..Default::default()
            },
        ]
    }
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone, Copy)]
struct LayoutColors {
    background: u32,
    background_elevated: u32,
    background_selected: u32,
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    border: u32,
}

impl LayoutColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            background_elevated: theme.colors.background.title_bar,
            background_selected: theme.colors.accent.selected_subtle,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.muted,
            text_muted: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

fn variation_label(text: &str) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(rgb(0x666666))
        .italic()
        .mb_2()
        .child(text.to_string())
}

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

/// Full window container
fn window_container(colors: LayoutColors) -> Div {
    div()
        .w_full()
        .h(px(320.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
}

/// Clean header with input (Raycast-style)
fn header_input(colors: LayoutColors, placeholder: &str) -> Div {
    div()
        .w_full()
        .px(px(16.))
        .py(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .border_b_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(
            // Input area (simulated)
            div()
                .flex_1()
                .text_base()
                .text_color(colors.text_primary.to_rgb())
                .child(placeholder.to_string()),
        )
}

/// Header with Ask AI button
fn header_with_ask_ai(colors: LayoutColors, placeholder: &str) -> Div {
    let hover_bg = (colors.accent << 8) | 0x20;

    header_input(colors, placeholder).child(
        div()
            .id("ask-ai-header")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba(hover_bg)))
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_secondary.to_rgb())
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .bg(rgba((colors.border << 8) | 0x60))
                    .rounded(px(4.))
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Tab"),
            ),
    )
}

/// Results section label
fn results_label(colors: LayoutColors) -> impl IntoElement {
    div()
        .px(px(16.))
        .py(px(6.))
        .text_xs()
        .text_color(colors.text_muted.to_rgb())
        .child("Results")
}

/// List item (Raycast-style)
fn list_item(
    colors: LayoutColors,
    icon: &'static str,
    icon_bg: u32,
    name: &str,
    subtitle: &str,
    item_type: &str,
    is_selected: bool,
) -> impl IntoElement {
    let bg = if is_selected {
        Some(colors.background_selected.to_rgb())
    } else {
        None
    };

    let mut item = div()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .rounded(px(8.))
        .mx(px(4.));

    if let Some(bg_color) = bg {
        item = item.bg(bg_color);
    }

    item
        // Icon
        .child(
            div()
                .w(px(28.))
                .h(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .bg(icon_bg.to_rgb())
                .rounded(px(6.))
                .child(div().text_sm().text_color(rgb(0xFFFFFF)).child(icon)),
        )
        // Name + subtitle
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_secondary.to_rgb())
                        .child(subtitle.to_string()),
                ),
        )
        // Type badge
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child(item_type.to_string()),
        )
}

/// Sample list content
fn sample_list(colors: LayoutColors) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(results_label(colors))
        .child(list_item(
            colors,
            "⚙",
            0x5856D6,
            "System Settings",
            "",
            "Application",
            true,
        ))
        .child(list_item(
            colors,
            "W",
            0x5856D6,
            "Rewrite Selected Text",
            "AI Writing Assistant",
            "Command",
            false,
        ))
        .child(list_item(
            colors,
            "⬆",
            0x5856D6,
            "Export Settings & Data",
            "Raycast",
            "Command",
            false,
        ))
        .child(list_item(
            colors,
            "●",
            0x007AFF,
            "Top Center Sixth",
            "Window Management",
            "Command",
            false,
        ))
}

// =============================================================================
// VARIATION 1: Exact Raycast Clone
// =============================================================================

fn render_raycast_exact(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_raycast_exact(colors))
}
