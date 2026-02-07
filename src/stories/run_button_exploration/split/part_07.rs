
fn render_context_terminal(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "⌘", "ctx-terminal"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_send(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "➤", "ctx-send"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_check(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "✓", "ctx-check"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_launch(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "↗", "ctx-launch"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_folder(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "📁", "ctx-folder"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_globe(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "🌐", "ctx-globe"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_gear(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "⚙", "ctx-gear"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_clipboard(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "📋", "ctx-clipboard"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 7: TIGHTER BUTTONS
// =============================================================================

fn render_tight_micro(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("tight-ai")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .px(px(2.))
                .py(px(2.))
                .rounded(px(2.))
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
                .id("tight-run")
                .px(px(2.))
                .py(px(2.))
                .rounded(px(2.))
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
                .id("tight-actions")
                .px(px(2.))
                .py(px(2.))
                .rounded(px(2.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(6.)))
        .child(logo_box())
}

fn render_tight_small(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("small-ai")
                .px(px(4.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI Tab"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("small-run")
                .px(px(4.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("small-actions")
                .px(px(4.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(6.)))
        .child(logo_box())
}

fn render_tight_compact(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("compact-ai")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Ask AI"),
                ),
        )
        .child(div().w(px(3.)))
        .child(
            div()
                .id("compact-run")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(3.)))
        .child(
            div()
                .id("compact-actions")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Actions"),
                ),
        )
        .child(div().w(px(6.)))
        .child(logo_box())
}

fn render_tight_text_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("AI"),
        )
        .child(
            div().w(px(8.)).child(
                div()
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("·"),
            ),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("Run"),
        )
        .child(
            div().w(px(8.)).child(
                div()
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("·"),
            ),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("Actions"),
        )
        .child(div().w(px(12.)))
        .child(logo_box())
}

fn render_tight_underline(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div().id("ul-ai").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .hover(|s| s.border_b_1().border_color(rgb(0xFBBF24)))
                    .child("Ask AI"),
            ),
        )
        .child(div().w(px(8.)))
        .child(
            div().id("ul-run").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Run"),
            ),
        )
        .child(div().w(px(8.)))
        .child(
            div().id("ul-actions").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Actions"),
            ),
        )
        .child(div().w(px(12.)))
        .child(logo_box())
}
