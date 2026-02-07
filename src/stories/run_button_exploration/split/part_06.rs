
fn render_split_button(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let divider_color = (colors.accent << 8) | 0x40;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Split button
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
                        .id("split-run")
                        .px(px(8.))
                        .py(px(3.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        ),
                )
                .child(div().w(px(1.)).h(px(16.)).bg(rgba(divider_color)))
                .child(
                    div()
                        .id("split-more")
                        .px(px(4.))
                        .py(px(3.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_pill_plus_more(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let pill_bg = (colors.accent << 8) | 0x1A;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("pill-run")
                .px(px(10.))
                .py(px(3.))
                .bg(rgba(pill_bg))
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run ↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("pill-more")
                .w(px(22.))
                .h(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("⋯"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_contextual_primary(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("ctx-primary")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .px(px(8.))
                .py(px(3.))
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
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("ctx-more")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("More ⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_two_part(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

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
                .gap(px(2.))
                .child(
                    div()
                        .id("two-icon")
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(div().text_sm().text_color(rgb(0x000000)).child("▶")),
                )
                .child(
                    div()
                        .id("two-dropdown")
                        .w(px(20.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(4.))
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

fn render_expandable_hover(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Shows icon normally, expands to "Run ↵" on hover
            div()
                .id("expandable")
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
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"),
                )
                .child(
                    // This would be hidden by default, shown on hover
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_cycle_actions(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Click cycles: Run → Edit → Copy → Delete → Run...
            div()
                .id("cycle")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(px(8.))
                .py(px(3.))
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x40))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("⇧ cycle"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_quick_plus_menu(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(
                    div()
                        .id("quick-run")
                        .px(px(6.))
                        .py(px(3.))
                        .rounded_l(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(12.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .id("quick-menu")
                        .px(px(6.))
                        .py(px(3.))
                        .rounded_r(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
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
                                        .child("⌘K"),
                                ),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 6: CONTEXTUAL ICONS
// =============================================================================

fn context_icon_button(
    colors: PromptHeaderColors,
    icon: &'static str,
    id: &'static str,
) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .id(id)
        .w(px(28.))
        .h(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
        .child(
            div()
                .text_base()
                .text_color(colors.accent.to_rgb())
                .child(icon),
        )
}
