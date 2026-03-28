        // Header: bare input + file count (scaffold adds padding/layout)
        let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);
        let header_element = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(HEADER_GAP))
            .child(
                div().flex_1().flex().flex_row().items_center().child(
                    Input::new(&self.gpui_input_state)
                        .w_full()
                        .h(px(input_height))
                        .px(px(0.))
                        .py(px(0.))
                        .with_size(Size::Size(px(design_typography.font_size_xl)))
                        .appearance(false)
                        .bordered(false)
                        .focus_bordered(false),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .py(px(4.))
                    .w(px(70.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} files", filtered_len)),
                    ),
            );

        // List pane: loading/empty/results with scrollbar overlay
        let list_pane = if is_loading && filtered_len == 0 {
            div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_dimmed))
                        .child("Searching..."),
                )
        } else if filtered_len == 0 {
            div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div().flex().flex_col().items_center().gap(px(8.)).child(
                        div()
                            .text_color(rgb(text_dimmed))
                            .child(if query.is_empty() {
                                "Type to search files"
                            } else {
                                "No files found"
                            }),
                    ),
                )
        } else {
            div()
                .relative()
                .w_full()
                .h_full()
                .child(list_element)
                .child(list_scrollbar)
        };

        // Preview pane: file detail or placeholder
        let preview_pane = preview_content;

        // Assemble via shared expanded-view scaffold (owns header padding, 50/50 split, footer)
        tracing::info!(
            surface = "file_search",
            layout_mode = "expanded",
            custom_footer_removed = true,
            custom_divider_removed = true,
            "file_search_chrome_checkpoint: migrated to render_expanded_view_scaffold"
        );
        crate::components::render_expanded_view_scaffold(header_element, list_pane, preview_pane)
            .key_context("FileSearchView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .into_any_element()
