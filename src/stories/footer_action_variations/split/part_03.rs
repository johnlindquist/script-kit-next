
/// 6. Keyboard hints prominent (badges)
fn footer_kbd_prominent(colors: FooterColors) -> impl IntoElement {
    let badge_bg = (colors.border << 8) | 0x50;

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
        .child(logo_component(colors, 20.))
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
                                .text_color(colors.accent.to_rgb())
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .bg(rgba(badge_bg))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(colors.text_secondary.to_rgb())
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
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
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
                                        .bg(rgba(badge_bg))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("⌘"),
                                )
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba(badge_bg))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("K"),
                                ),
                        ),
                ),
        )
}

/// 7. Icon-style Run button
fn footer_icon_run(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

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
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                // Play icon button
                .child(
                    div()
                        .id("icon-run")
                        .w(px(28.))
                        .h(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_base()
                                .text_color(colors.accent.to_rgb())
                                .child("▶"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                // Actions text
                .child(
                    div()
                        .id("actions-text")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(4.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
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

/// 8. Ghost button style
fn footer_ghost_buttons(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;
    let border_color = (colors.accent << 8) | 0x40;

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
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Run button (ghost)
                .child(
                    div()
                        .id("ghost-run")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgba(border_color))
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
                // Actions button (ghost, no border)
                .child(
                    div()
                        .id("ghost-actions")
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

/// 9. Primary Run + ghost Actions
fn footer_primary_run(colors: FooterColors) -> impl IntoElement {
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
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Actions button (ghost)
                .child(
                    div()
                        .id("secondary-actions")
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
                )
                // Run button (primary)
                .child(
                    div()
                        .id("primary-run")
                        .px(px(12.))
                        .py(px(5.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(6.))
                        .cursor_pointer()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x000000))
                                        .child("Run Script"),
                                )
                                .child(div().text_xs().text_color(rgba(0x00000080)).child("↵")),
                        ),
                ),
        )
}
