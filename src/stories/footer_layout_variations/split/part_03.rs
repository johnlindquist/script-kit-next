
fn footer_breadcrumb(colors: LayoutColors) -> impl IntoElement {
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
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Breadcrumb path
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Main Menu"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("›"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("System Settings"),
                ),
        )
        // Right: Actions
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
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Open"),
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
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("⌘K"),
                ),
        )
}

// =============================================================================
// VARIATION 5: Centered Action
// =============================================================================

fn render_centered_action(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_centered(colors))
}

fn footer_centered(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        .child(
            div()
                .id("centered-action")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(16.))
                .px(px(16.))
                .py(px(6.))
                .rounded(px(8.))
                .cursor_pointer()
                .hover(|s| s.bg(rgba(0xFFFFFF10)))
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
                                .text_color(colors.text_primary.to_rgb())
                                .child("Open Application"),
                        )
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .bg(rgba((colors.border << 8) | 0x60))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
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
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("More"),
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

// =============================================================================
// VARIATION 6: Icon + Text Action
// =============================================================================

fn render_icon_text_action(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_icon_text(colors))
}

fn footer_icon_text(colors: LayoutColors) -> impl IntoElement {
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
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Item count
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("4 results"),
        )
        // Right: Icon + text actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .id("icon-action-1")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_base()
                                .text_color(colors.accent.to_rgb())
                                .child("▶"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Run"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
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
                        .id("icon-action-2")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_base()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("⚡"),
                        )
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
}

// =============================================================================
// VARIATION 7: Primary Button Footer
// =============================================================================

fn render_primary_button_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_primary_button(colors))
}

fn footer_primary_button(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(48.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Esc to close
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Esc to close"),
        )
        // Right: Buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Secondary
                .child(
                    div()
                        .id("secondary-btn")
                        .px(px(12.))
                        .py(px(6.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgba((colors.border << 8) | 0x60))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xFFFFFF10)))
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
                // Primary
                .child(
                    div()
                        .id("primary-btn")
                        .px(px(12.))
                        .py(px(6.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(6.))
                        .cursor_pointer()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x000000))
                                        .child("Open"),
                                )
                                .child(div().text_xs().text_color(rgba(0x00000080)).child("↵")),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 8: Ghost Button Footer
// =============================================================================

fn render_ghost_button_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_ghost_button(colors))
}
