
fn render_search_results(colors: PathPromptColors) -> impl IntoElement {
    let results = vec![
        ("~/.config/", "config", true, true),
        ("~/project/", "config.json", false, false),
        ("~/app/", "config.ts", false, false),
        ("~/.zsh/", "config.zsh", false, false),
    ];

    div()
        .flex()
        .flex_col()
        .px_2()
        .py_1()
        .gap_px()
        .children(results.into_iter().map(|(path, name, is_dir, selected)| {
            let bg = if selected {
                rgb(colors.selected_bg)
            } else {
                rgb(0x00000000)
            };
            let icon = if is_dir { "📁" } else { "📄" };

            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_2()
                .py_1()
                .bg(bg)
                .rounded_sm()
                .child(div().text_sm().child(icon.to_string()))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(colors.text_primary))
                                .child(name.to_string()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_muted))
                                .child(path.to_string()),
                        ),
                )
        }))
}

fn render_path_prompt_container(
    colors: PathPromptColors,
    path_prefix: &str,
    filter_text: &str,
    entries: &[(&str, bool, bool)],
    hint: Option<&str>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(render_header_simple(colors, path_prefix, filter_text))
        .child(div().w_full().h_px().bg(rgb(colors.border)))
        .child(render_path_list(colors, entries))
        .when_some(hint, |d, h| d.child(render_footer(colors, h)))
}

fn render_header_simple(
    colors: PathPromptColors,
    path_prefix: &str,
    filter_text: &str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_3()
        .py_2()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .flex_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_muted))
                        .child(path_prefix.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_primary))
                        .child(filter_text.to_string()),
                )
                .when(filter_text.is_empty(), |d| {
                    d.child(
                        div()
                            .text_sm()
                            .text_color(rgb(colors.text_muted))
                            .child("Type to filter..."),
                    )
                }),
        )
        .child(render_header_buttons(colors))
}

fn render_header_with_breadcrumbs(
    colors: PathPromptColors,
    breadcrumbs: &[&str],
    filter_text: &str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .px_3()
        .py_2()
        .gap_1()
        .child(div().flex().flex_row().items_center().gap_1().children(
            breadcrumbs.iter().enumerate().map(|(i, crumb)| {
                let is_last = i == breadcrumbs.len() - 1;
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_last {
                                rgb(colors.text_primary)
                            } else {
                                rgb(colors.accent)
                            })
                            .when(!is_last, |d| d.cursor_pointer())
                            .child(crumb.to_string()),
                    )
                    .when(!is_last, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.breadcrumb_separator))
                                .child("/"),
                        )
                    })
            }),
        ))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(if filter_text.is_empty() {
                            rgb(colors.text_muted)
                        } else {
                            rgb(colors.text_primary)
                        })
                        .child(if filter_text.is_empty() {
                            "Type to filter...".to_string()
                        } else {
                            filter_text.to_string()
                        }),
                )
                .child(render_header_buttons(colors)),
        )
}

fn render_header_buttons(colors: PathPromptColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.accent))
                        .child("Select"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_muted))
                        .child("↵"),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgba((colors.text_muted << 8) | 0x66))
                .child("|"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.accent))
                        .child("Actions"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_muted))
                        .child("⌘K"),
                ),
        )
}

fn render_path_list(colors: PathPromptColors, entries: &[(&str, bool, bool)]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .px_2()
        .py_1()
        .gap_px()
        .min_h(px(200.))
        .children(entries.iter().map(|(name, is_dir, is_selected)| {
            render_path_entry(colors, name, *is_dir, *is_selected)
        }))
}

fn render_path_entry(
    colors: PathPromptColors,
    name: &str,
    is_dir: bool,
    is_selected: bool,
) -> impl IntoElement {
    let bg = if is_selected {
        rgb(colors.selected_bg)
    } else {
        rgb(0x00000000)
    };
    let text_color = if is_selected {
        rgb(colors.text_primary)
    } else {
        rgb(colors.text_secondary)
    };
    let icon = if is_dir { "📁" } else { "📄" };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .bg(bg)
        .rounded_sm()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(colors.hover_bg)))
        .when(is_selected, |d| {
            d.child(div().w(px(3.)).h_4().rounded_sm().bg(rgb(colors.accent)))
        })
        .child(
            div()
                .text_sm()
                .text_color(if is_dir {
                    rgb(colors.icon_folder)
                } else {
                    rgb(colors.icon_file)
                })
                .child(icon.to_string()),
        )
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(text_color)
                .child(name.to_string()),
        )
        .when(is_dir, |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_muted))
                    .child("→"),
            )
        })
}

fn render_footer(colors: PathPromptColors, hint: &str) -> impl IntoElement {
    div()
        .w_full()
        .px_3()
        .py_2()
        .border_t_1()
        .border_color(rgb(colors.border))
        .child(
            div()
                .text_xs()
                .text_color(rgb(colors.text_muted))
                .child(hint.to_string()),
        )
}
