
/// 11. Dense Layout
fn actions_window_dense(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(300.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(8.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(6.))
                .border_b_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_primary.to_rgb())
                        .child("Activity Monitor"),
                ),
        )
        .child(
            div()
                .w_full()
                .py(px(2.))
                .px(px(2.))
                .flex()
                .flex_col()
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    div()
                        .w_full()
                        .h(px(26.))
                        .px(px(6.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .bg(selection_bg)
                        .rounded(px(4.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(1.));
                            for key in action.shortcut {
                                row = row.child(
                                    div()
                                        .min_w(px(14.))
                                        .h(px(14.))
                                        .px(px(3.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(colors.keycap_bg.to_rgb())
                                        .rounded(px(2.))
                                        .text_color(colors.text_muted.to_rgb())
                                        .child(
                                            div()
                                                .text_color(colors.text_muted.to_rgb())
                                                .child(key.to_string()),
                                        ),
                                );
                            }
                            row
                        })
                })),
        )
        .child(
            div()
                .w_full()
                .px(px(6.))
                .py(px(4.))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .w_full()
                        .h(px(22.))
                        .px(px(6.))
                        .bg(rgba((colors.border << 8) | 0x30))
                        .rounded(px(4.))
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Search..."),
                        ),
                ),
        )
}

/// 12. With Action Count
fn actions_window_with_count(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .px(px(12.))
                .py(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_primary.to_rgb())
                        .child("Activity Monitor"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(rgba((colors.accent << 8) | 0x30))
                        .rounded(px(10.))
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("5 actions"),
                ),
        )
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
}

/// 13. Accent Colored Selection
fn actions_window_accent_selection(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let (bg, text_color, border): (Hsla, Hsla, Hsla) = if is_selected {
                        (
                            colors.accent.to_rgb(),
                            rgb(0x000000).into(),
                            colors.accent.to_rgb(),
                        )
                    } else {
                        (
                            rgba(0x00000000).into(),
                            colors.text_primary.to_rgb(),
                            rgba(0x00000000).into(),
                        )
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(8.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(bg)
                        .border_1()
                        .border_color(border)
                        .rounded(px(6.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(text_color)
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(text_color)
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(2.));
                            for key in action.shortcut {
                                let keycap_bg: Hsla = if is_selected {
                                    rgba(0x00000030).into()
                                } else {
                                    colors.keycap_bg.to_rgb()
                                };
                                row = row.child(
                                    div()
                                        .min_w(px(20.))
                                        .h(px(20.))
                                        .px(px(6.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(keycap_bg)
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(text_color)
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 14. Bordered Items
fn actions_window_bordered(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(8.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let border: Hsla = if is_selected {
                        colors.accent.to_rgb()
                    } else {
                        rgba((colors.border << 8) | 0x60).into()
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(10.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .border_1()
                        .border_color(border)
                        .rounded(px(8.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors))
                })),
        )
        .child(search_input("Search for actions...", colors))
}
