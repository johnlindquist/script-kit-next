
fn render_alt_double_click(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Double-click or ↵"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_alt_long_press(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Hold for actions"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 9: VISUAL HIERARCHY
// =============================================================================

fn render_hier_ghost(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x15;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("ghost-run")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba((colors.accent << 8) | 0x60))
                        .child("Run ↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_muted(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Muted by default, brighter on hover (concept - actual would need state)
            div()
                .id("muted-run")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_primary(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("primary-run")
                .px(px(10.))
                .py(px(4.))
                .bg(colors.accent.to_rgb())
                .rounded(px(4.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x000000))
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_accent_bg(colors: PromptHeaderColors) -> impl IntoElement {
    let accent_bg = (colors.accent << 8) | 0x30;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("accent-run")
                .px(px(8.))
                .py(px(3.))
                .bg(rgba(accent_bg))
                .rounded(px(4.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run ↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_outline(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x15;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("outline-run")
                .px(px(8.))
                .py(px(3.))
                .border_1()
                .border_color(colors.accent.to_rgb())
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
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

fn render_hier_gradient(colors: PromptHeaderColors) -> impl IntoElement {
    // Simulated gradient with solid color (GPUI doesn't do gradients easily)
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("gradient-run")
                .px(px(10.))
                .py(px(4.))
                .bg(rgb(0xF59E0B)) // Amber gradient approximation
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
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_glow(colors: PromptHeaderColors) -> impl IntoElement {
    // Simulated glow with background
    let glow_bg = (colors.accent << 8) | 0x40;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("glow-run")
                .px(px(10.))
                .py(px(5.))
                .bg(rgba(glow_bg))
                .rounded(px(6.))
                .cursor_pointer()
                .child(
                    div()
                        .px(px(8.))
                        .py(px(3.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_pulse(colors: PromptHeaderColors) -> impl IntoElement {
    // Simulated pulse (static in storybook, would animate)
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .relative()
                .child(
                    // Pulse ring (would animate opacity)
                    div()
                        .absolute()
                        .inset_0()
                        .bg(rgba((colors.accent << 8) | 0x30))
                        .rounded(px(4.)),
                )
                .child(
                    div()
                        .id("pulse-run")
                        .relative()
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
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 10: RECOMMENDED
// =============================================================================

fn render_rec_icon_tooltip(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Icon-only, tooltip on hover would show full action
            div()
                .id("rec-icon")
                .w(px(24.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_fixed_ghost(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("rec-ghost")
                .w(px(50.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x30))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .overflow_hidden()
                        .text_ellipsis()
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_no_button(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("↵"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}
