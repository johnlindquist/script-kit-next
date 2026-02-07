
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
