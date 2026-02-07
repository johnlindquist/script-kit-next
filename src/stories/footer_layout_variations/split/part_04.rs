
fn footer_ghost_button(colors: LayoutColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Logo
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(0xFBBF24D9))
                .rounded(px(4.))
                .child(div().text_xs().text_color(rgb(0x000000)).child("SK")),
        )
        // Right: Ghost buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .child(
                    div()
                        .id("ghost-action")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgba((colors.accent << 8) | 0x40))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Run Script"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ghost-more")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
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
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘K"),
                                ),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 9: Split Action Footer
// =============================================================================

fn render_split_action_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_split_action(colors))
}

fn footer_split_action(colors: LayoutColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Selected item info
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .w(px(16.))
                        .h(px(16.))
                        .bg(rgb(0x5856D6))
                        .rounded(px(4.)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("System Settings"),
                ),
        )
        // Right: Split button
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x40))
                .rounded(px(6.))
                .overflow_hidden()
                .child(
                    div()
                        .id("split-main")
                        .px(px(10.))
                        .py(px(5.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Open"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(20.))
                        .bg(rgba((colors.accent << 8) | 0x40)),
                )
                .child(
                    div()
                        .id("split-more")
                        .px(px(8.))
                        .py(px(5.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 10: Contextual Actions Row
// =============================================================================

fn render_contextual_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_contextual(colors))
}

fn footer_contextual(colors: LayoutColors) -> impl IntoElement {
    let hover_bg = 0xFFFFFF15;

    div()
        .w_full()
        .h(px(40.))
        .px(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Multiple contextual actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(
                    div()
                        .id("ctx-1")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Open"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ctx-2")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
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
                                        .child("Edit"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘E"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ctx-3")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
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
                                        .child("Copy"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘C"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ctx-4")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
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
                                        .child("Delete"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘⌫"),
                                ),
                        ),
                ),
        )
        // More actions
        .child(
            div()
                .id("ctx-more")
                .px(px(8.))
                .py(px(4.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("More"),
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
