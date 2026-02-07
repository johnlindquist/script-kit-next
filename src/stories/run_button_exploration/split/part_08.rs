
fn render_tight_badge(colors: PromptHeaderColors) -> impl IntoElement {
    let badge_bg = (colors.accent << 8) | 0x20;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("badge-ai")
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(badge_bg))
                .rounded(px(2.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("badge-run")
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(badge_bg))
                .rounded(px(2.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("badge-actions")
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(badge_bg))
                .rounded(px(2.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_tight_link(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div().id("link-ai").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Ask AI"),
            ),
        )
        .child(div().w(px(6.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("|"),
        )
        .child(div().w(px(6.)))
        .child(
            div().id("link-run").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Run"),
            ),
        )
        .child(div().w(px(6.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("|"),
        )
        .child(div().w(px(6.)))
        .child(
            div().id("link-actions").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Actions"),
            ),
        )
        .child(div().w(px(12.)))
        .child(logo_box())
}

fn render_tight_pill(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("pill-ai")
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("pill-run")
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("pill-actions")
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 8: ALTERNATIVE PLACEMENTS
// =============================================================================

fn render_alt_in_list_item(colors: PromptHeaderColors) -> impl IntoElement {
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
                        .w(px(20.))
                        .h(px(20.))
                        .mr(px(12.))
                        .rounded(px(4.))
                        .bg(rgba((colors.accent << 8) | 0x30)),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child("Clipboard History"),
                )
                .child(
                    div()
                        .id("list-run")
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

fn render_alt_hover_overlay(colors: PromptHeaderColors) -> impl IntoElement {
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
                .mx(px(8.))
                .child(
                    div()
                        .px(px(12.))
                        .py(px(10.))
                        .bg(rgba((colors.accent << 8) | 0x15))
                        .rounded(px(6.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(12.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .rounded(px(4.))
                                .bg(rgba((colors.accent << 8) | 0x30)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Clipboard History (hover shows Run overlay)"),
                        ),
                )
                .child(
                    // Overlay that would appear on hover
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .bottom_0()
                        .w(px(80.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba((colors.background << 8) | 0xE0))
                        .rounded_r(px(6.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run ↵"),
                        ),
                ),
        )
}

fn render_alt_keyboard_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
    // Note: No visual Run button - just Enter key
}

fn render_alt_status_bar(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(100.))
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
                .h(px(24.))
                .px(px(12.))
                .bg(rgba((colors.background << 8) | 0x80))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵ Run • Tab AI • ⌘K Actions"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("3 items"),
                ),
        )
}

fn render_alt_right_click(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Right-click for actions"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_alt_gesture(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("→ swipe to run"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}
