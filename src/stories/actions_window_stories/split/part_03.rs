
/// 7. Larger Keycaps
fn actions_window_large_keycaps(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(380.))
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
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    div()
                        .w_full()
                        .h(px(44.))
                        .px(px(8.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(selection_bg)
                        .rounded(px(6.))
                        .child(
                            div()
                                .w(px(22.))
                                .h(px(22.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_base()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_base()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(3.));
                            for key in action.shortcut {
                                row = row.child(
                                    div()
                                        .min_w(px(26.))
                                        .h(px(26.))
                                        .px(px(8.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(colors.keycap_bg.to_rgb())
                                        .border_1()
                                        .border_color(colors.keycap_border.to_rgb())
                                        .rounded(px(5.))
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 8. Monochrome Icons
fn actions_window_mono_icons(colors: ActionColors) -> impl IntoElement {
    let mono_actions = [
        ActionItem {
            icon: "◯",
            label: "Open Application",
            shortcut: &["↵"],
        },
        ActionItem {
            icon: "◎",
            label: "Show in Finder",
            shortcut: &["⌘", "↵"],
        },
        ActionItem {
            icon: "◉",
            label: "Show Info in Finder",
            shortcut: &["⌘", "I"],
        },
        ActionItem {
            icon: "◍",
            label: "Show Package Contents",
            shortcut: &["⌥", "⌘", "I"],
        },
        ActionItem {
            icon: "☆",
            label: "Add to Favorites",
            shortcut: &["⇧", "⌘", "F"],
        },
    ];

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
                .children(
                    mono_actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
}

/// 9. With Descriptions
fn actions_window_with_descriptions(colors: ActionColors) -> impl IntoElement {
    let described_actions = [
        (
            "□",
            "Open Application",
            "Launch the selected app",
            &["↵"][..],
        ),
        (
            "🔍",
            "Show in Finder",
            "Reveal in Finder window",
            &["⌘", "↵"][..],
        ),
        (
            "ℹ",
            "Show Info in Finder",
            "Open Get Info panel",
            &["⌘", "I"][..],
        ),
    ];

    div()
        .w(px(400.))
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
                .children(described_actions.iter().enumerate().map(
                    |(i, (icon, label, desc, shortcut))| {
                        let is_selected = i == 0;
                        let selection_bg = if is_selected {
                            rgba((colors.selection << 8) | 0x20)
                        } else {
                            rgba(0x00000000)
                        };

                        div()
                            .w_full()
                            .h(px(52.))
                            .px(px(8.))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.))
                            .bg(selection_bg)
                            .rounded(px(6.))
                            .child(
                                div()
                                    .w(px(24.))
                                    .h(px(24.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_base()
                                    .text_color(colors.text_secondary.to_rgb())
                                    .child(*icon),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors.text_primary.to_rgb())
                                            .child(*label),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors.text_muted.to_rgb())
                                            .child(*desc),
                                    ),
                            )
                            .child(shortcut_badge(shortcut, colors))
                    },
                )),
        )
        .child(search_input("Search for actions...", colors))
}

/// 10. Minimal Footer
fn actions_window_minimal_footer(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(340.))
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
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
        // Minimal footer - just shortcuts
        .child(
            div()
                .w_full()
                .h(px(32.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .gap(px(12.))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div().flex().flex_row().items_center().gap(px(4.)).child(
                        div()
                            .text_xs()
                            .text_color(colors.text_muted.to_rgb())
                            .child("↵ Run"),
                    ),
                )
                .child(
                    div().flex().flex_row().items_center().gap(px(4.)).child(
                        div()
                            .text_xs()
                            .text_color(colors.text_muted.to_rgb())
                            .child("⌘K More"),
                    ),
                ),
        )
}
