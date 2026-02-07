
// ============================================================================
// VARIATION 7: Pill Buttons
// [Input] ............ [Ask AI Tab] [(Run ↵)] [(Actions ⌘K)] [Logo]
// ============================================================================
fn render_variation_7(colors: PromptHeaderColors) -> impl IntoElement {
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
        .child(render_pill_button("Run ↵", colors, false))
        .child(render_pill_button("Actions ⌘K", colors, true))
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 8: Minimal - Input + Enter Only
// [Input] .................................................. [↵]
// ============================================================================
fn render_variation_8(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_xl()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(
            div()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(colors.accent.rgba8(0x20))
                .text_color(colors.accent.to_rgb())
                .text_sm()
                .child("↵"),
        )
}

// ============================================================================
// VARIATION 9: Search Box Style (outlined input)
// [🔍 Input ...........................] [Ask AI] [Run] [⋮]
// ============================================================================
fn render_variation_9(colors: PromptHeaderColors) -> impl IntoElement {
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
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .bg(colors.search_box_bg.to_rgb())
                .border_1()
                .border_color(colors.border.to_rgb())
                .rounded_lg()
                .child(div().text_color(colors.text_dimmed.to_rgb()).child("🔍"))
                .child(
                    div()
                        .flex_1()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
        .child(render_text_button("Ask AI", colors))
        .child(render_text_button("Run", colors))
        .child(div().text_color(colors.text_dimmed.to_rgb()).child("⋮"))
}

// ============================================================================
// VARIATION 10: Tab Bar Style
// [Script Kit ▾] | [Input ......................] | [⌘K] [↵]
// ============================================================================
fn render_variation_10(colors: PromptHeaderColors) -> impl IntoElement {
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
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_md()
                .child(render_logo(colors))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child("Script Kit"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("▾"),
                ),
        )
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .text_color(colors.text_muted.to_rgb())
                .child("Type to search..."),
        )
        .child(render_separator(colors))
        .child(render_kbd("⌘K", colors))
        .child(render_kbd("↵", colors))
}

// ============================================================================
// VARIATION 11: Floating Actions (actions in a separate container)
// [Input] ........................ [Ask AI Tab] | [  Run ↵  Actions ⌘K  ]
// ============================================================================
fn render_variation_11(colors: PromptHeaderColors) -> impl IntoElement {
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
        .child(render_separator(colors))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_lg()
                .child(render_button("Run", "↵", colors))
                .child(render_button("Actions", "⌘K", colors)),
        )
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 12: Breadcrumb Style
// [Logo] > [Scripts] > [Input ...................] [Ask AI] [Run ↵]
// ============================================================================
fn render_variation_12(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        .child(render_logo(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .text_sm()
                .child(">"),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Scripts"),
        )
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .text_sm()
                .child(">"),
        )
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Search..."),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
}

// ============================================================================
// VARIATION 13: Command Palette Style (VS Code inspired)
// [>] [Input ........................................] [Esc to close]
// ============================================================================
fn render_variation_13(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_2()
        .child(
            div()
                .text_lg()
                .text_color(colors.accent.to_rgb())
                .child(">"),
        )
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Type a command..."),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Esc to close"),
        )
}

// ============================================================================
// VARIATION 14: Toolbar Style (with icon buttons)
// [Logo] | [🏠] [📁] [⚙️] | [Input ..........] | [Ask AI] [Run ↵]
// ============================================================================
fn render_variation_14(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        .child(render_logo(colors))
        .child(render_separator(colors))
        .child(render_icon_button("🏠", colors))
        .child(render_icon_button("📁", colors))
        .child(render_icon_button("⚙️", colors))
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_separator(colors))
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
}

// ============================================================================
// VARIATION 15: Split Header (left/right sections)
// [Logo] [Input ............] || [Ask AI Tab] [Run ↵] [Actions ⌘K]
// ============================================================================
fn render_variation_15(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_4()
        // Left section
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_logo(colors))
                .child(
                    div()
                        .flex_1()
                        .text_lg()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
        // Thick separator
        .child(div().w_px().h_6().bg(colors.border.to_rgb()))
        // Right section
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(render_ask_ai_hint(colors))
                .child(render_button("Run", "↵", colors))
                .child(render_button("Actions", "⌘K", colors)),
        )
}

// ============================================================================
// VARIATION 16: Icon-Only Buttons
// [Input] .......................... [Ask AI Tab] [▶] [⚡] [⋯] [Logo]
// ============================================================================
fn render_variation_16(colors: PromptHeaderColors) -> impl IntoElement {
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
        .child(render_icon_button("▶", colors))
        .child(render_icon_button("⚡", colors))
        .child(render_icon_button("⋯", colors))
        .child(render_logo(colors))
}
