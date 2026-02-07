
/// 15. With Dividers
fn actions_window_with_dividers(colors: ActionColors) -> impl IntoElement {
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
                .px(px(8.))
                .py(px(4.))
                .flex()
                .flex_col()
                .children(actions.iter().enumerate().flat_map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    let row = div()
                        .w_full()
                        .h(px(40.))
                        .px(px(8.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(selection_bg)
                        .rounded(px(6.))
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
                        .child(shortcut_badge(action.shortcut, colors));

                    let mut items: Vec<Div> = vec![row];
                    if i < actions.len() - 1 {
                        items.push(
                            div()
                                .w_full()
                                .h(px(1.))
                                .mx(px(8.))
                                .my(px(2.))
                                .bg(rgba((colors.border << 8) | 0x30)),
                        );
                    }
                    items
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 16. Keyboard-Only (no mouse hints)
fn actions_window_keyboard_only(colors: ActionColors) -> impl IntoElement {
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
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        // No search input - keyboard navigation only
        .child(
            div()
                .w_full()
                .h(px(36.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(16.))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(keycap("↑", colors))
                        .child(keycap("↓", colors))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Navigate"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(keycap("↵", colors))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Select"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(keycap("Esc", colors))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Close"),
                        ),
                ),
        )
}

/// 17. Extra Wide
fn actions_window_wide(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(480.))
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
                .px(px(6.))
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
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(12.))
                        .bg(selection_bg)
                        .rounded(px(8.))
                        .child(
                            div()
                                .w(px(28.))
                                .h(px(28.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgba((colors.accent << 8) | 0x20))
                                .rounded(px(6.))
                                .text_base()
                                .text_color(colors.accent.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_base()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors))
                })),
        )
        .child(search_input("Search for actions...", colors))
        .child(footer_bar("Open Application", colors))
}

/// 18. Floating Style (more shadow, subtle border)
fn actions_window_floating(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(16.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x20))
        .shadow_xl()
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 42.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
        .child(footer_bar("Open Application", colors))
}

/// 19. With Categories Sidebar
fn actions_window_categories(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(480.))
        .h(px(300.))
        .flex()
        .flex_row()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        // Left sidebar
        .child(
            div()
                .w(px(120.))
                .h_full()
                .flex()
                .flex_col()
                .border_r_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .bg(rgba((colors.background_elevated << 8) | 0x60))
                .py(px(8.))
                .px(px(6.))
                .gap(px(2.))
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .bg(rgba((colors.accent << 8) | 0x20))
                        .rounded(px(6.))
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.accent.to_rgb())
                        .child("All"),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Application"),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Finder"),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Favorites"),
                ),
        )
        // Right content
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .child(header_bar("Activity Monitor", colors))
                .child(
                    div()
                        .flex_1()
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
                .child(search_input("Search for actions...", colors)),
        )
}
