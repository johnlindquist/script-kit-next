
/// Thin line separator
fn line_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w(px(1.))
        .h(px(20.))
        .mx_2()
        .bg(colors.text_dimmed.to_rgb())
}

// =============================================================================
// SEPARATOR STYLES (1-5)
// =============================================================================

/// 1. Pipe Separators (Current style)
fn render_v1_pipe_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(logo_component(28., 6.))
}

/// 2. No Separators
fn render_v2_no_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_component(28., 6.))
}

/// 3. Dot Separators
fn render_v3_dot_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(dot_sep(colors))
        .child(run_button(colors))
        .child(dot_sep(colors))
        .child(actions_button(colors))
        .child(dot_sep(colors))
        .child(logo_component(28., 6.))
}

/// 4. Slash Separators
fn render_v4_slash_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(slash_sep(colors))
        .child(run_button(colors))
        .child(slash_sep(colors))
        .child(actions_button(colors))
        .child(slash_sep(colors))
        .child(logo_component(28., 6.))
}

/// 5. Thin Line Separators
fn render_v5_line_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(line_sep(colors))
        .child(run_button(colors))
        .child(line_sep(colors))
        .child(actions_button(colors))
        .child(line_sep(colors))
        .child(logo_component(28., 6.))
}

// =============================================================================
// SPACING VARIATIONS (6-10)
// =============================================================================

/// 6. Tight Spacing
fn render_v6_tight_spacing(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .gap_1()
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_1()
                .child("|"),
        )
        .child(run_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_1()
                .child("|"),
        )
        .child(actions_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_1()
                .child("|"),
        )
        .child(logo_component(24., 4.))
}

/// 7. Loose Spacing
fn render_v7_loose_spacing(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .gap_4()
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_4()
                .child("|"),
        )
        .child(run_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_4()
                .child("|"),
        )
        .child(actions_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_4()
                .child("|"),
        )
        .child(logo_component(32., 8.))
}

/// 8. Grouped Buttons
fn render_v8_grouped_buttons(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(24.)))
        .child(
            // Button group with background
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .bg(rgba(0xffffff10))
                .rounded(px(6.))
                .child(run_button(colors))
                .child(div().w(px(1.)).h(px(16.)).bg(colors.text_dimmed.to_rgb()))
                .child(actions_button(colors)),
        )
        .child(div().w(px(12.)))
        .child(logo_component(28., 6.))
}

/// 9. Spread Layout
fn render_v9_spread_layout(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .justify_between()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(logo_component(28., 6.))
                .child(script_kit_label(colors)),
        )
        .child(ask_ai_hint(colors))
        .child(run_button(colors))
        .child(actions_button(colors))
}

/// 10. Compact All
fn render_v10_compact_all(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px_3()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .bg(colors.background.to_rgb())
        .border_b_1()
        .border_color(colors.border.to_rgb())
        .child(logo_component(22., 4.))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.text_primary.to_rgb())
                .child("Script Kit"),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb()) // Yellow text
                .child("AI"),
        )
        .child(
            div()
                .px(px(4.))
                .py(px(1.))
                .bg(colors.search_box_bg.to_rgb())
                .rounded(px(3.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("⇥"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("↵"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("⌘K"),
        )
}

// =============================================================================
// LOGO VARIATIONS (11-15)
// =============================================================================

/// 11. Logo Left
fn render_v11_logo_left(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(logo_component(28., 6.))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
}

/// 12. Logo Right (far right)
fn render_v12_logo_right(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_component(28., 6.))
}

/// 13. Logo Larger
fn render_v13_logo_larger(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(logo_component(36., 8.))
}

/// 14. Logo with Border
fn render_v14_logo_border(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .border_2()
                .border_color(rgb(0xffc821))
                .rounded(px(8.))
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgb(0xffc821))
                        .rounded(px(4.))
                        .child(
                            svg()
                                .external_path(concat!(
                                    env!("CARGO_MANIFEST_DIR"),
                                    "/assets/logo.svg"
                                ))
                                .size(px(14.))
                                .text_color(rgb(0x000000)), // Black logo
                        ),
                ),
        )
}

/// 15. Logo Circular
fn render_v15_logo_circular(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(
            div()
                .w(px(28.))
                .h(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0xffc821))
                .rounded_full()
                .child(
                    svg()
                        .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                        .size(px(16.))
                        .text_color(rgb(0x000000)), // Black logo
                ),
        )
}
