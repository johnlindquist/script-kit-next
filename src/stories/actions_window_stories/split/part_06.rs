
/// 20. Full Raycast Clone - exact Raycast reproduction
fn actions_window_raycast_clone(_colors: ActionColors) -> impl IntoElement {
    // Raycast exact colors (dark theme)
    let raycast_bg = 0x1e1e1e;
    let raycast_border = 0x3a3a3c;
    let raycast_text = 0xffffff;
    let raycast_text_secondary = 0xa0a0a0;
    let raycast_selection = 0x3a3a3c;

    let actions = [
        ActionItem {
            icon: "□",
            label: "Open Application",
            shortcut: &["↵"],
        },
        ActionItem {
            icon: "📁",
            label: "Show in Finder",
            shortcut: &["⌘", "↵"],
        },
        ActionItem {
            icon: "ℹ️",
            label: "Show Info in Finder",
            shortcut: &["⌘", "I"],
        },
        ActionItem {
            icon: "📦",
            label: "Show Package Contents",
            shortcut: &["⌥", "⌘", "I"],
        },
        ActionItem {
            icon: "⭐",
            label: "Add to Favorites",
            shortcut: &["⇧", "⌘", "F"],
        },
    ];

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(raycast_bg.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((raycast_border << 8) | 0x80))
        .shadow_2xl()
        .overflow_hidden()
        // Header
        .child(
            div()
                .w_full()
                .px(px(14.))
                .py(px(10.))
                .border_b_1()
                .border_color(rgba((raycast_border << 8) | 0x60))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(raycast_text.to_rgb())
                        .child("Activity Monitor"),
                ),
        )
        // Actions list
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let bg: Hsla = if is_selected {
                        raycast_selection.to_rgb()
                    } else {
                        rgba(0x00000000).into()
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(10.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(bg)
                        .rounded(px(8.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(raycast_text.to_rgb())
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(3.));
                            for key in action.shortcut {
                                row = row.child(
                                    div()
                                        .min_w(px(22.))
                                        .h(px(22.))
                                        .px(px(6.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(rgb(0x4a4a4c))
                                        .border_1()
                                        .border_color(rgb(0x5a5a5c))
                                        .rounded(px(5.))
                                        .text_xs()
                                        .text_color(raycast_text_secondary.to_rgb())
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        // Search input (Raycast style - at bottom)
        .child(
            div()
                .w_full()
                .px(px(10.))
                .py(px(8.))
                .border_t_1()
                .border_color(rgba((raycast_border << 8) | 0x60))
                .child(
                    div()
                        .w_full()
                        .h(px(32.))
                        .px(px(12.))
                        .bg(rgba((raycast_border << 8) | 0x40))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgba((raycast_text_secondary << 8) | 0xA0))
                                .child("Search for actions..."),
                        ),
                ),
        )
        // Footer (Raycast style)
        .child(
            div()
                .w_full()
                .h(px(44.))
                .px(px(14.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .border_t_1()
                .border_color(rgba((raycast_border << 8) | 0x60))
                .bg(rgba((raycast_bg << 8) | 0xE0))
                // Left: Primary action
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(raycast_text.to_rgb())
                                .child("Open Application"),
                        )
                        .child(
                            div()
                                .min_w(px(22.))
                                .h(px(22.))
                                .px(px(6.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(0x4a4a4c))
                                .border_1()
                                .border_color(rgb(0x5a5a5c))
                                .rounded(px(5.))
                                .text_xs()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("↵"),
                        ),
                )
                // Right: Actions button
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .px(px(10.))
                        .py(px(5.))
                        .bg(rgb(0x4a4a4c))
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .min_w(px(18.))
                                .h(px(18.))
                                .px(px(4.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(0x5a5a5c))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("⌘"),
                        )
                        .child(
                            div()
                                .min_w(px(18.))
                                .h(px(18.))
                                .px(px(4.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(0x5a5a5c))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("K"),
                        ),
                ),
        )
}
