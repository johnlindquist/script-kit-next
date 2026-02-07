
// =============================================================================
// BUTTON STYLES (16-20)
// =============================================================================

/// 16. Pill Buttons
fn render_v16_pill_buttons(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_3()
                .py(px(6.))
                .bg(rgba(0xffffff10))
                .rounded_full()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(colors.search_box_bg.to_rgb())
                        .rounded_full()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb()) // Grey shortcut
                        .child("Tab"),
                ),
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .px_3()
                .py(px(6.))
                .bg(colors.accent.to_rgb())
                .rounded_full()
                .text_sm()
                .text_color(rgb(0x000000))
                .font_weight(FontWeight::MEDIUM)
                .child("Run ↵"),
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_3()
                .py(px(6.))
                .bg(rgba(0xffffff10))
                .rounded_full()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("Actions"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb()) // Grey shortcut
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_component(28., 6.))
}

/// 17. Ghost Buttons (transparent hover)
fn render_v17_ghost_buttons(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb()) // Grey shortcut
                        .child("⇥"),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("Run"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb()) // Grey shortcut
                        .child("↵"),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("Actions"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb()) // Grey shortcut
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_component(28., 6.))
}

/// 18. Icon Only
fn render_v18_icon_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(logo_component(28., 6.))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(rgba(0xffffff08))
                .text_color(colors.text_muted.to_rgb())
                .child("✨"), // AI icon
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(colors.accent.to_rgb())
                .text_color(rgb(0x000000))
                .child("▶"),
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(rgba(0xffffff08))
                .text_color(colors.text_muted.to_rgb())
                .child("⋯"),
        )
}

/// 19. Text Only (no icons, minimal)
fn render_v19_text_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffc821))
                .child("⌘"),
        )
        .child(div().w(px(8.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_dimmed.to_rgb()) // Grey shortcut
                        .child("Tab"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("for AI"),
                ),
        )
        .child(div().w(px(16.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_dimmed.to_rgb()) // Grey shortcut
                        .child("Enter"),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("to Run"),
                ),
        )
        .child(div().w(px(16.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_dimmed.to_rgb()) // Grey shortcut
                        .child("⌘K"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow text
                        .child("Actions"),
                ),
        )
}

/// 20. Badge Style
fn render_v20_badge_style(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(logo_component(28., 6.))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            // Badge-style buttons
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py(px(4.))
                        .bg(rgba(0x6366f120))
                        .border_1()
                        .border_color(rgba(0x6366f140))
                        .rounded(px(4.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(div().text_xs().text_color(rgb(0x818cf8)).child("AI"))
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .bg(rgba(0x6366f130))
                                .rounded(px(2.))
                                .text_xs()
                                .text_color(rgb(0x818cf8))
                                .child("⇥"),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(4.))
                        .bg(rgba(0x22c55e20))
                        .border_1()
                        .border_color(rgba(0x22c55e40))
                        .rounded(px(4.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(div().text_xs().text_color(rgb(0x4ade80)).child("Run"))
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .bg(rgba(0x22c55e30))
                                .rounded(px(2.))
                                .text_xs()
                                .text_color(rgb(0x4ade80))
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(4.))
                        .bg(rgba(0xffffff10))
                        .border_1()
                        .border_color(rgba(0xffffff20))
                        .rounded(px(4.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("More"),
                        )
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .bg(rgba(0xffffff10))
                                .rounded(px(2.))
                                .text_xs()
                                .text_color(colors.text_dimmed.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}
