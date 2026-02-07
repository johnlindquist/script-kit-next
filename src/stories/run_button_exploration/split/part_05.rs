
fn render_pos_in_input(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .mx(px(12.))
                .px(px(12.))
                .py(px(8.))
                .bg(rgba((colors.search_box_bg << 8) | 0x80))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Search..."),
                )
                .child(
                    div()
                        .id("pos-in-input")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(rgba((colors.accent << 8) | 0x20))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        ),
                ),
        )
}

fn render_pos_overlap_input(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .relative()
                .mx(px(12.))
                .child(
                    div()
                        .px(px(12.))
                        .py(px(8.))
                        .pr(px(60.))
                        .bg(rgba((colors.search_box_bg << 8) | 0x80))
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Search..."),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .right(px(4.))
                        .top(px(4.))
                        .id("pos-overlap")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
}

fn render_pos_below_header(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .h(px(28.))
                .px(px(12.))
                .bg(rgba((colors.background << 8) | 0x80))
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .child(
                    div()
                        .id("pos-below")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run ↵"),
                        ),
                ),
        )
}

fn render_pos_floating_br(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(100.))
        .relative()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .absolute()
                .bottom(px(8.))
                .right(px(12.))
                .id("pos-floating")
                .px(px(12.))
                .py(px(6.))
                .bg(colors.accent.to_rgb())
                .rounded(px(6.))
                .cursor_pointer()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x000000))
                        .child("Run Script"),
                ),
        )
}

fn render_pos_in_list(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            // Simulated list item with run button
            div()
                .mx(px(8.))
                .px(px(12.))
                .py(px(10.))
                .bg(rgba((colors.accent << 8) | 0x15))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Clipboard History"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("View and manage clipboard"),
                        ),
                )
                .child(
                    div()
                        .id("pos-list")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(rgba((colors.accent << 8) | 0x30))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        ),
                ),
        )
}

fn render_pos_sticky_footer(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(120.))
        .flex()
        .flex_col()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(div().flex_1())
        .child(
            div()
                .h(px(36.))
                .px(px(12.))
                .bg(rgba((colors.background << 8) | 0xE0))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Selected: Clipboard History"),
                )
                .child(
                    div()
                        .id("pos-footer")
                        .px(px(12.))
                        .py(px(4.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
}

// =============================================================================
// SECTION 5: COMBINE WITH ACTIONS
// =============================================================================

fn render_actions_merged(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(
            // Combined Actions/Run button
            div()
                .id("merged-actions")
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
                        .child("↵/⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}
