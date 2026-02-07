
/// 10. Taller footer with more spacing
fn footer_tall(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(52.))
        .px(px(16.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        // Left: Logo with breathing room
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .child(logo_component(colors, 24.))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
        // Right: Spacious actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.accent.to_rgb())
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
                        .h(px(20.))
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
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}
