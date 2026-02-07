
fn render_rec_contextual_icon(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Changes icon based on context: ▶ for scripts, ✓ for select, ➤ for send
            div()
                .id("rec-ctx")
                .w(px(24.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"), // Would change dynamically
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_merged(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(
            // Run is first item in Actions menu, button just shows ⌘K/↵
            div()
                .id("rec-merged")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Actions"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵ ⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_split_compact(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let divider_color = (colors.accent << 8) | 0x30;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .border_1()
                .border_color(rgba(divider_color))
                .rounded(px(4.))
                .overflow_hidden()
                .child(
                    div()
                        .id("split-action")
                        .w(px(24.))
                        .h(px(22.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("▶"),
                        ),
                )
                .child(div().w(px(1.)).h(px(14.)).bg(rgba(divider_color)))
                .child(
                    div()
                        .id("split-menu")
                        .w(px(20.))
                        .h(px(22.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}
