
// =============================================================================
// VARIATIONS
// =============================================================================

/// 1. Base Raycast Design
fn actions_window_base(colors: ActionColors) -> impl IntoElement {
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
        // Header
        .child(header_bar("Activity Monitor", colors))
        // Actions list
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
        // Search
        .child(search_input("Search for actions...", colors))
        // Footer
        .child(footer_bar("Open Application", colors))
}

/// 2. Search at Top
fn actions_window_search_top(colors: ActionColors) -> impl IntoElement {
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
        // Header
        .child(header_bar("Activity Monitor", colors))
        // Search at top
        .child(
            div().w_full().px(px(12.)).py(px(8.)).child(
                div()
                    .w_full()
                    .h(px(32.))
                    .px(px(10.))
                    .bg(rgba((colors.border << 8) | 0x30))
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.text_muted.to_rgb())
                            .child("🔍"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.text_muted.to_rgb())
                            .child("Search for actions..."),
                    ),
            ),
        )
        // Actions list
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
        // Footer
        .child(footer_bar("Open Application", colors))
}

/// 3. Compact Items
fn actions_window_compact(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(320.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(10.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(2.))
                .px(px(4.))
                .flex()
                .flex_col()
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 32.)),
                ),
        )
        .child(search_input("Search...", colors))
}

/// 4. Without Icons
fn actions_window_no_icons(colors: ActionColors) -> impl IntoElement {
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
                .px(px(8.))
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
                        .h(px(36.))
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .bg(selection_bg)
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors))
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 5. With Section Headers
fn actions_window_with_sections(colors: ActionColors) -> impl IntoElement {
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
        // Section: Application
        .child(
            div().w_full().px(px(12.)).pt(px(8.)).pb(px(4.)).child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.text_muted.to_rgb())
                    .child("APPLICATION"),
            ),
        )
        .child(
            div()
                .w_full()
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(action_row(&sample_actions()[0], true, colors, 40.))
                .child(action_row(&sample_actions()[1], false, colors, 40.)),
        )
        // Section: Finder
        .child(
            div().w_full().px(px(12.)).pt(px(12.)).pb(px(4.)).child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.text_muted.to_rgb())
                    .child("FINDER"),
            ),
        )
        .child(
            div()
                .w_full()
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(action_row(&sample_actions()[2], false, colors, 40.))
                .child(action_row(&sample_actions()[3], false, colors, 40.)),
        )
        .child(search_input("Search for actions...", colors))
}

/// 6. Pill-style Selection
fn actions_window_pill_selection(colors: ActionColors) -> impl IntoElement {
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
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let bg: Hsla = if is_selected {
                        colors.accent.to_rgb()
                    } else {
                        rgba(0x00000000).into()
                    };
                    let text_color: Hsla = if is_selected {
                        rgb(0x000000).into()
                    } else {
                        colors.text_primary.to_rgb()
                    };
                    let keycap_text: Hsla = if is_selected {
                        rgb(0x000000).into()
                    } else {
                        colors.text_secondary.to_rgb()
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(bg)
                        .rounded(px(20.)) // Pill shape
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
                                    rgba(0x00000020).into()
                                } else {
                                    colors.keycap_bg.to_rgb()
                                };
                                let keycap_border: Hsla = if is_selected {
                                    rgba(0x00000040).into()
                                } else {
                                    colors.keycap_border.to_rgb()
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
                                        .border_1()
                                        .border_color(keycap_border)
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(keycap_text)
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        .child(search_input("Search for actions...", colors))
}
