        // Build preview panel content - matching main menu labeled section pattern
        let preview_content = if let Some(file) = &selected_file {
            let file_type_str = match file.file_type {
                FileType::Directory => "Folder",
                FileType::Image => "Image",
                FileType::Audio => "Audio",
                FileType::Video => "Video",
                FileType::Document => "Document",
                FileType::Application => "Application",
                FileType::File => "File",
                FileType::Other => "File",
            };

            div()
                .flex_1()
                .flex()
                .flex_col()
                .p(px(design_spacing.padding_lg))
                .gap(px(design_spacing.gap_md))
                .overflow_y_hidden()
                // Name section (labeled like main menu)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Name"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(design_spacing.gap_sm))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child(file.name.clone()),
                                )
                                .child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(rgba((ui_border << 8) | 0x40))
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child(file_type_str),
                                ),
                        ),
                )
                // Path section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Path"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child(file.path.clone()),
                        ),
                )
                // Divider (like main menu)
                .child(
                    div()
                        .w_full()
                        .h(px(design_visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .my(px(design_spacing.padding_sm)),
                )
                // Details section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Details"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(design_spacing.gap_sm))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Size: {}",
                                    file_search::format_file_size(file.size)
                                )))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Modified: {}",
                                    file_search::format_relative_time(file.modified)
                                )))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_dimmed))
                                        .child(format!("Type: {}", file_type_str)),
                                ),
                        ),
                )
        } else if is_loading {
            // When loading, show empty preview (no distracting message)
            div().flex_1()
        } else {
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_sm()
                    .text_color(rgb(text_dimmed))
                    .child("No file selected"),
            )
        };
