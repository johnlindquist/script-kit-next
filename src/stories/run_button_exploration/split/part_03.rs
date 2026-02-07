
// =============================================================================
// HELPER COMPONENTS
// =============================================================================

fn variation_label(text: &str) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(rgb(0x666666))
        .italic()
        .mb_2()
        .child(text.to_string())
}

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mb_3()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

fn header_container(colors: PromptHeaderColors) -> Div {
    div()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .bg(colors.background.to_rgb())
        .rounded(px(8.))
}

fn script_kit_label(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors.text_primary.to_rgb())
        .child("Script Kit")
}

fn logo_box() -> impl IntoElement {
    div()
        .w(px(19.))
        .h(px(19.))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0xFFD60AD9))
        .rounded(px(4.))
        .child(
            svg()
                .external_path(utils::get_logo_path())
                .size(px(12.))
                .text_color(rgb(0x000000)),
        )
}

fn ask_ai_button(colors: PromptHeaderColors) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;
    let tab_bg = (colors.search_box_bg << 8) | 0x4D;

    div()
        .id("ask-ai")
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
                .child("Ask AI"),
        )
        .child(
            div()
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(tab_bg))
                .rounded(px(3.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

fn actions_button(colors: PromptHeaderColors) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .id("actions")
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
                .child("⌘K"),
        )
}

// =============================================================================
// SECTION 1: NO RUN BUTTON
// =============================================================================

fn render_no_run_minimal(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("↵ Enter"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_ask_ai_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_actions_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_hint_in_input(colors: PromptHeaderColors) -> impl IntoElement {
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
            // Simulated input with hint
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
                        .child("Type to search..."),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵"),
                ),
        )
}

fn render_no_run_enter_far_right(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(12.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("↵"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_floating_hint(colors: PromptHeaderColors) -> impl IntoElement {
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
            div().w_full().flex().justify_end().pr(px(20.)).child(
                div()
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("Press Enter to run"),
            ),
        )
}

// =============================================================================
// SECTION 2: ICON-ONLY
// =============================================================================

fn icon_button(colors: PromptHeaderColors, icon: &'static str, id: &'static str) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .id(id)
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
                .child(icon),
        )
}

fn render_icon_only_play(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "▶", "play-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_arrow(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "→", "arrow-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_check(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "✓", "check-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_return(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "↵", "return-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_circle(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "●", "circle-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_double_arrow(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "»", "double-arrow-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}
