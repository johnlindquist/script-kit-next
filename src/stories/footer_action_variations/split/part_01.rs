pub struct FooterActionVariationsStory;

impl Story for FooterActionVariationsStory {
    fn id(&self) -> &'static str {
        "footer-action-variations"
    }

    fn name(&self) -> &'static str {
        "Footer Actions (10 variations)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = FooterColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Footer Action Variations")
                    .child(variation_item(
                        "1. Base - Logo left, Run + Actions right",
                        full_layout(colors, footer_base(colors)),
                    ))
                    .child(variation_item(
                        "2. No divider between actions",
                        full_layout(colors, footer_no_divider(colors)),
                    ))
                    .child(variation_item(
                        "3. Compact - smaller text",
                        full_layout(colors, footer_compact(colors)),
                    ))
                    .child(variation_item(
                        "4. With item count",
                        full_layout(colors, footer_with_count(colors)),
                    ))
                    .child(variation_item(
                        "5. Selected item preview",
                        full_layout(colors, footer_with_preview(colors)),
                    ))
                    .child(variation_item(
                        "6. Keyboard hints prominent",
                        full_layout(colors, footer_kbd_prominent(colors)),
                    ))
                    .child(variation_item(
                        "7. Icon-style Run button",
                        full_layout(colors, footer_icon_run(colors)),
                    ))
                    .child(variation_item(
                        "8. Ghost button style",
                        full_layout(colors, footer_ghost_buttons(colors)),
                    ))
                    .child(variation_item(
                        "9. Primary Run + ghost Actions",
                        full_layout(colors, footer_primary_run(colors)),
                    ))
                    .child(variation_item(
                        "10. Taller footer with more spacing",
                        full_layout(colors, footer_tall(colors)),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant {
            name: "base".into(),
            description: Some("Base footer layout".into()),
            ..Default::default()
        }]
    }
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct FooterColors {
    background: u32,
    background_elevated: u32,
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    border: u32,
}

impl FooterColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            background_elevated: theme.colors.background.title_bar,
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

/// Window container with header, list preview, and footer
fn full_layout(colors: FooterColors, footer: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .h(px(280.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        // Header
        .child(header_clean(colors))
        // Divider
        .child(
            div()
                .mx(px(16.))
                .h(px(1.))
                .bg(rgba((colors.border << 8) | 0x40)),
        )
        // List preview (simplified)
        .child(list_preview(colors))
        // Footer
        .child(footer)
}

/// Clean header - just input and Ask AI
fn header_clean(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;
    let tab_bg = (colors.border << 8) | 0x40;

    div()
        .w_full()
        .px(px(16.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        // Input
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(colors.text_primary.to_rgb())
                .child("clipboard"),
        )
        // Ask AI button
        .child(
            div()
                .id("ask-ai")
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
                        .text_color(colors.accent.to_rgb())
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(rgba(tab_bg))
                        .rounded(px(4.))
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Tab"),
                ),
        )
}

/// Simplified list preview
fn list_preview(colors: FooterColors) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .px(px(8.))
        .py(px(4.))
        // Selected item
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .bg(rgba((colors.accent << 8) | 0x12))
                .rounded(px(6.))
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.accent.to_rgb())
                        .rounded(px(5.))
                        .child(div().text_sm().text_color(rgb(0x000000)).child("📋")),
                )
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
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.text_primary.to_rgb())
                                .child("Clipboard History"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("View and manage clipboard"),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Built-in"),
                ),
        )
        // Another item (unselected)
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.accent.to_rgb())
                        .rounded(px(5.))
                        .child(div().text_sm().text_color(rgb(0x000000)).child("🔍")),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child("Search Files"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script"),
                ),
        )
}

/// Logo component using actual SVG
fn logo_component(_colors: FooterColors, size: f32) -> impl IntoElement {
    div()
        .w(px(size))
        .h(px(size))
        .flex()
        .items_center()
        .justify_center()
        // .bg(rgba((colors.accent << 8) | 0xD9)) // 85% opacity
        // .rounded(px(4.))
        .child(
            svg()
                .external_path(utils::get_logo_path())
                .size(px(size * 0.65))
                .text_color(rgb(0x000000)),
        )
}
