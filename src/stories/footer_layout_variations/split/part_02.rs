
fn footer_raycast_exact(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Logo/icon
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child("🖐"),
                ),
        )
        // Right: Actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Primary action
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Open Application"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                // Divider
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                // Actions
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(2.))
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba((colors.border << 8) | 0x60))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘"),
                                )
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba((colors.border << 8) | 0x60))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("K"),
                                ),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 2: Script Kit Branded
// =============================================================================

fn render_scriptkit_branded(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_scriptkit(colors, "testing"))
        .child(sample_list_scriptkit(colors))
        .child(footer_scriptkit(colors))
}

fn header_scriptkit(colors: LayoutColors, placeholder: &str) -> Div {
    let hover_bg = (colors.accent << 8) | 0x20;
    let tab_bg = (colors.border << 8) | 0x40;

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
            div()
                .flex_1()
                .text_base()
                .text_color(colors.text_primary.to_rgb())
                .child(placeholder.to_string()),
        )
        .child(
            div()
                .id("ask-ai-sk")
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
                        .text_color(colors.accent.to_rgb()) // Yellow accent
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

fn sample_list_scriptkit(colors: LayoutColors) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(results_label(colors))
        .child(list_item(
            colors,
            "📋",
            0xFBBF24,
            "Clipboard History",
            "View and manage clipboard",
            "Built-in",
            true,
        ))
        .child(list_item(
            colors,
            "🔍",
            0xFBBF24,
            "Search Files",
            "Find files on your system",
            "Script",
            false,
        ))
        .child(list_item(
            colors,
            "⌨️",
            0xFBBF24,
            "Snippet Manager",
            "Quick text expansion",
            "Built-in",
            false,
        ))
        .child(list_item(
            colors,
            "🚀",
            0xFBBF24,
            "App Launcher",
            "Launch applications",
            "Built-in",
            false,
        ))
}

fn footer_scriptkit(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Script Kit logo
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(0xFBBF24D9)) // Yellow with alpha
                .rounded(px(4.))
                .child(div().text_xs().text_color(rgb(0x000000)).child("SK")),
        )
        // Right: Actions with yellow accent
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb()) // Yellow
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb()) // Yellow
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 3: Minimal Footer
// =============================================================================

fn render_minimal_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_minimal(colors))
}

fn footer_minimal(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(32.))
        .px(px(16.))
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x20))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("↵ Open  •  ⌘K Actions  •  Tab AI"),
        )
}

// =============================================================================
// VARIATION 4: Breadcrumb Footer
// =============================================================================

fn render_breadcrumb_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_breadcrumb(colors))
}
