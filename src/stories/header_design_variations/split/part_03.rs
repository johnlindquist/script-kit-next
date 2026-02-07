
// ============================================================================
// VARIATION 17: Grouped Actions (with background)
// [Input] .......... | [Ask AI] | [ Run  |  Actions ] | [Logo]
// ============================================================================
fn render_variation_17(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_separator(colors))
        .child(render_text_button("Ask AI", colors))
        .child(render_separator(colors))
        // Grouped buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_md()
                .overflow_hidden()
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .hover(|s| s.bg(colors.accent.rgba8(0x20)))
                        .child("Run ↵"),
                )
                .child(div().w_px().h_4().bg(colors.border.to_rgb()))
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .hover(|s| s.bg(colors.accent.rgba8(0x20)))
                        .child("Actions ⌘K"),
                ),
        )
        .child(render_separator(colors))
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 18: Spotlight Style (Apple Spotlight inspired)
// [ 🔍  Input .................................................. ]
// ============================================================================
fn render_variation_18(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_6()
        .py_4()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_xl()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("🔍"),
                )
                .child(
                    div()
                        .flex_1()
                        .text_2xl()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
}

// ============================================================================
// VARIATION 19: Alfred Style
// [Input ...................] [↵] .......... [⌘1] [⌘2] [⌘3]
// ============================================================================
fn render_variation_19(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_kbd("↵", colors))
        .child(div().flex_1()) // Spacer
        .child(render_kbd("⌘1", colors))
        .child(render_kbd("⌘2", colors))
        .child(render_kbd("⌘3", colors))
}

// ============================================================================
// VARIATION 20: Raycast Style (current production look)
// [Input] ............ [Ask AI Tab] [Run ↵] | [Actions ⌘K] | [▶]
// ============================================================================
fn render_variation_20(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_separator(colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(render_separator(colors))
        .child(
            div()
                .w_6()
                .h_6()
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .bg(colors.accent.to_rgb())
                .text_color(rgb(0x000000))
                .text_sm()
                .child("▶"),
        )
}

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

/// Render the "Ask AI [Tab]" hint
fn render_ask_ai_hint(colors: PromptHeaderColors) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .flex_shrink_0()
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Ask AI"),
        )
        .child(
            div()
                .px_1()
                .py_px()
                .rounded(px(3.))
                .border_1()
                .border_color(colors.border.to_rgb())
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

/// Render a separator
fn render_separator(colors: PromptHeaderColors) -> Div {
    div()
        .text_sm()
        .text_color(colors.text_dimmed.rgba8(0x60))
        .child("|")
}

/// Render a text button (label + shortcut)
fn render_button(label: &str, shortcut: &str, colors: PromptHeaderColors) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .text_sm()
        .text_color(colors.accent.to_rgb())
        .child(label.to_string())
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .child(shortcut.to_string()),
        )
}

/// Render a pill-style button
fn render_pill_button(label: &str, colors: PromptHeaderColors, outlined: bool) -> Div {
    let base = div()
        .px_3()
        .py_1()
        .rounded_full()
        .text_sm()
        .text_color(colors.accent.to_rgb());

    if outlined {
        base.border_1().border_color(colors.accent.rgba8(0x40))
    } else {
        base.bg(colors.accent.rgba8(0x20))
    }
    .child(label.to_string())
}

/// Render a text-only button
fn render_text_button(label: &str, colors: PromptHeaderColors) -> Div {
    div()
        .text_sm()
        .text_color(colors.accent.to_rgb())
        .child(label.to_string())
}

/// Render an icon button
fn render_icon_button(icon: &str, colors: PromptHeaderColors) -> Div {
    div()
        .w_7()
        .h_7()
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .hover(|s| s.bg(colors.search_box_bg.to_rgb()))
        .text_color(colors.text_muted.to_rgb())
        .child(icon.to_string())
}

/// Render a keyboard shortcut badge
fn render_kbd(key: &str, colors: PromptHeaderColors) -> Div {
    div()
        .px_2()
        .py_1()
        .rounded(px(4.))
        .bg(colors.search_box_bg.to_rgb())
        .border_1()
        .border_color(colors.border.to_rgb())
        .text_xs()
        .text_color(colors.text_muted.to_rgb())
        .child(key.to_string())
}

/// Render the logo
fn render_logo(colors: PromptHeaderColors) -> Div {
    div()
        .w_4()
        .h_4()
        .flex()
        .items_center()
        .justify_center()
        .text_color(colors.accent.to_rgb())
        .child("▶") // Placeholder for actual SVG logo
}
