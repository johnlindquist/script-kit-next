
/// V5: Secret input with value (masked)
fn render_secret_with_value(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;

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
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_lock_icon(text_muted))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("****************"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V6: API key style with branding
fn render_api_key_style(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let accent = theme.colors.accent.selected;

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
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded(px(4.))
                        .bg(rgba((accent << 8) | 0x33)) // ~0.2 opacity
                        .text_xs()
                        .text_color(rgb(accent))
                        .child("OPENAI_API_KEY"),
                )
                .child(render_cursor(text_primary))
                .child(div().text_lg().text_color(rgb(text_muted)).child("sk-...")),
        )
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

// ============================================================================
// VARIATION 7-9: Validation States
// ============================================================================

/// V7: Valid state with checkmark
fn render_valid_state(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let success_color = 0x4ade80; // Green

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .border_l_2()
        .border_color(rgb(success_color))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_check_icon(success_color))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("ghp_validtoken12345"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(success_color))
                .child("Valid token format"),
        )
        .child(sep_pipe(theme))
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V8: Error state
fn render_error_state(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let error_color = 0xf87171; // Red

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .border_l_2()
        .border_color(rgb(error_color))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_error_icon(error_color))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("invalid-token"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(error_color))
                .child("Token must start with 'ghp_'"),
        )
        .child(sep_pipe(theme))
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

/// V9: Warning state
fn render_warning_state(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let warning_color = 0xfbbf24; // Amber

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .border_l_2()
        .border_color(rgb(warning_color))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_warning_icon(warning_color))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(text_primary))
                        .child("ghp_shorttoken"),
                )
                .child(render_cursor(text_primary)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(warning_color))
                .child("Token appears short - verify it's complete"),
        )
        .child(sep_pipe(theme))
        .child(render_submit_button(theme))
        .child(sep_pipe(theme))
        .child(render_logo(theme))
}

// ============================================================================
// VARIATION 10-12: With Suggestions/Hints
// ============================================================================

/// V10: With hint text below
fn render_with_hint(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let text_dimmed = theme.colors.text.dimmed;

    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
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
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(render_cursor(text_primary))
                        .child(
                            div()
                                .text_lg()
                                .text_color(rgb(text_muted))
                                .child("Enter DATABASE_URL"),
                        ),
                )
                .child(render_submit_button(theme))
                .child(sep_pipe(theme))
                .child(render_logo(theme)),
        )
        .child(
            div()
                .px_4()
                .pb_2()
                .text_xs()
                .text_color(rgb(text_dimmed))
                .child("Tip: Get this from your database provider's connection settings"),
        )
}

/// V11: With example value
fn render_with_example(theme: &Theme) -> impl IntoElement {
    let text_primary = theme.colors.text.primary;
    let text_muted = theme.colors.text.muted;
    let text_dimmed = theme.colors.text.dimmed;
    let border = theme.colors.ui.border;

    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
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
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(render_cursor(text_primary))
                        .child(
                            div()
                                .text_lg()
                                .text_color(rgb(text_muted))
                                .child("Enter WEBHOOK_URL"),
                        ),
                )
                .child(render_submit_button(theme))
                .child(sep_pipe(theme))
                .child(render_logo(theme)),
        )
        .child(
            div()
                .px_4()
                .pb_2()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child("Example:"),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded(px(4.))
                        .bg(rgba((border << 8) | 0x4D)) // ~0.3 opacity
                        .font_family("Menlo")
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .child("https://hooks.slack.com/services/T00/B00/XXX"),
                ),
        )
}
