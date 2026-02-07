
/// Input with keyboard shortcut badge
fn render_with_shortcut(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    placeholder: &str,
    shortcut: &str,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        placeholder.to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(text_color)
                .child(display_text),
        )
        .child(
            div()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .bg(rgb(colors.background.main))
                .border_1()
                .border_color(rgb(colors.ui.border))
                .text_xs()
                .text_color(rgb(colors.text.muted))
                .child(shortcut.to_string()),
        )
}

/// Focus state visualization
fn render_focus_state(colors: &crate::theme::ColorScheme, is_focused: bool) -> impl IntoElement {
    let border_color = if is_focused {
        rgb(colors.accent.selected)
    } else {
        rgb(colors.ui.border)
    };

    let border_width = if is_focused { px(2.) } else { px(1.) };

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .border(border_width)
        .border_color(border_color)
        .rounded_md()
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(rgb(colors.text.dimmed))
                .child("Type something..."),
        );

    if is_focused {
        container = container.child(
            div()
                .w(px(2.))
                .h(px(18.))
                .bg(rgb(colors.text.primary))
                .ml_1(),
        );
    }

    container
}

/// Focused with text selection
fn render_focused_with_selection(colors: &crate::theme::ColorScheme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(colors.background.search_box))
        .border_2()
        .border_color(rgb(colors.accent.selected))
        .rounded_md()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(colors.text.secondary))
                        .child("hello "),
                )
                .child(
                    div()
                        .px_1()
                        .bg(rgb(colors.accent.selected))
                        .text_base()
                        .text_color(rgb(colors.text.primary))
                        .child("world"),
                ),
        )
}

/// With choice list
fn render_with_choices(
    colors: &crate::theme::ColorScheme,
    input_text: &str,
    choices: Vec<&str>,
) -> impl IntoElement {
    let display_text = if input_text.is_empty() {
        "Search...".to_string()
    } else {
        input_text.to_string()
    };

    let text_color = if input_text.is_empty() {
        rgb(colors.text.dimmed)
    } else {
        rgb(colors.text.secondary)
    };

    let mut container = div().flex().flex_col().w_full().child(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .w_full()
            .px_4()
            .py_3()
            .bg(rgb(colors.background.search_box))
            .border_b_1()
            .border_color(rgb(colors.ui.border))
            .child(div().text_color(rgb(colors.text.muted)).child("🔍"))
            .child(
                div()
                    .flex_1()
                    .text_base()
                    .text_color(text_color)
                    .child(display_text),
            ),
    );

    for (idx, choice) in choices.iter().enumerate() {
        let is_selected = idx == 0;
        let bg = if is_selected {
            rgb(colors.accent.selected)
        } else {
            rgb(colors.background.main)
        };
        let name_color = if is_selected {
            rgb(colors.text.primary)
        } else {
            rgb(colors.text.secondary)
        };

        container = container.child(
            div()
                .w_full()
                .px_4()
                .py_2()
                .bg(bg)
                .border_b_1()
                .border_color(rgb(colors.ui.border))
                .text_base()
                .text_color(name_color)
                .child(choice.to_string()),
        );
    }

    container
}

/// No matching choices state
fn render_no_matches(colors: &crate::theme::ColorScheme, input_text: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .w_full()
                .px_4()
                .py_3()
                .bg(rgb(colors.background.search_box))
                .border_b_1()
                .border_color(rgb(colors.ui.border))
                .child(div().text_color(rgb(colors.text.muted)).child("🔍"))
                .child(
                    div()
                        .flex_1()
                        .text_base()
                        .text_color(rgb(colors.text.secondary))
                        .child(input_text.to_string()),
                ),
        )
        .child(
            div()
                .w_full()
                .px_4()
                .py_4()
                .text_color(rgb(colors.text.dimmed))
                .child("No choices match your filter"),
        )
}

// Story is registered in stories/mod.rs via get_all_stories()
