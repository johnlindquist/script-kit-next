        // Clone data for the uniform_list closure
        // Use display_indices to get files in the correct order (filtered + sorted)
        // Include the original result index for animation timestamp lookup
        let files_for_closure: Vec<(usize, file_search::FileResult)> = display_indices
            .iter()
            .filter_map(|&idx| self.cached_file_results.get(idx).map(|f| (idx, f.clone())))
            .collect();
        let current_selected = selected_index;
        let file_hovered = self.hovered_index;
        let file_input_mode = self.input_mode;
        let is_loading = self.file_search_loading;
        let click_entity_handle = cx.entity().downgrade();
        let hover_entity_handle = cx.entity().downgrade();

        // Use uniform_list for virtualized scrolling
        // Skeleton loading: show placeholder rows while loading and no results yet
        let list_element = if is_loading && filtered_len == 0 {
            // Loading with no results yet - show static skeleton rows
            let skeleton_bg = rgba((ui_border << 8) | 0x30); // ~18% opacity

            // Render 6 skeleton rows
            div()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .children((0..6).map(|ix| {
                    div()
                        .id(ix)
                        .w_full()
                        .h(px(52.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .px(px(12.))
                        .gap(px(12.))
                        // Icon placeholder
                        .child(div().w(px(24.)).h(px(24.)).rounded(px(6.)).bg(skeleton_bg))
                        // Text placeholders
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(6.))
                                .child(div().w(px(160.)).h(px(12.)).rounded(px(4.)).bg(skeleton_bg))
                                .child(
                                    div().w(px(240.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg),
                                ),
                        )
                        // Right side placeholders (size/time)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_end()
                                .gap(px(6.))
                                .child(div().w(px(56.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg))
                                .child(div().w(px(72.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg)),
                        )
                }))
                .into_any_element()
        } else if filtered_len == 0 {
            // No results and not loading - show empty state message
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_dimmed))
                .child(if query.is_empty() {
                    "Type to search files"
                } else {
                    "No files found"
                })
                .into_any_element()
        } else {
            uniform_list(
                "file-search-list",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_result_idx, file)) = files_for_closure.get(ix) {
                                let is_selected = ix == current_selected;
                                let is_hovered = !is_selected && file_hovered == Some(ix) && file_input_mode == InputMode::Mouse;

                                // Use theme opacity for vibrancy-compatible selection
                                let bg = if is_selected {
                                    rgba((list_selected << 8) | selected_alpha)
                                } else if is_hovered {
                                    rgba((list_hover << 8) | hover_alpha)
                                } else {
                                    gpui::transparent_black().into()
                                };
                                let hover_bg = rgba((list_hover << 8) | hover_alpha);
                                let is_mouse_mode = file_input_mode == InputMode::Mouse;

                                // Click handler: select on click, open file on double-click
                                let click_entity = click_entity_handle.clone();
                                let file_path = file.path.clone();
                                let click_handler = move |event: &gpui::ClickEvent,
                                                           _window: &mut Window,
                                                           cx: &mut gpui::App| {
                                    if let Some(app) = click_entity.upgrade() {
                                        let file_path = file_path.clone();
                                        app.update(cx, |this, cx| {
                                            if let AppView::FileSearchView {
                                                selected_index, ..
                                            } = &mut this.current_view
                                            {
                                                *selected_index = ix;
                                            }
                                            cx.notify();

                                            // Double-click: open file
                                            if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                                if mouse_event.down.click_count == 2 {
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Double-click opening file: {}",
                                                            file_path
                                                        ),
                                                    );
                                                    let _ = file_search::open_file(&file_path);
                                                    this.close_and_reset_window(cx);
                                                }
                                            }
                                        });
                                    }
                                };

                                // Hover handler for mouse tracking
                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler = move |hov: &bool, _window: &mut Window, cx: &mut gpui::App| {
                                    if let Some(app) = hover_entity.upgrade() {
                                        app.update(cx, |this, cx| {
                                            if *hov {
                                                this.input_mode = InputMode::Mouse;
                                                if this.hovered_index != Some(ix) {
                                                    this.hovered_index = Some(ix);
                                                    cx.notify();
                                                }
                                            } else if this.hovered_index == Some(ix) {
                                                this.hovered_index = None;
                                                cx.notify();
                                            }
                                        });
                                    }
                                };

                                div()
                                    .id(ix)
                                    .w_full()
                                    .h(px(52.))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .px(px(12.))
                                    .gap(px(12.))
                                    .bg(bg)
                                    .cursor_pointer()
                                    .when(is_mouse_mode, |d| d.hover(move |s| s.bg(hover_bg)))
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_color(rgb(text_muted))
                                            .child(file_search::file_type_icon(file.file_type)),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(text_primary))
                                                    .child(file.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_dimmed))
                                                    .child(file_search::shorten_path(&file.path)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_end()
                                            .gap(px(2.))
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_file_size(file.size),
                                                ),
                                            )
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_relative_time(
                                                        file.modified,
                                                    ),
                                                ),
                                            ),
                                    )
                            } else {
                                div().id(ix).h(px(52.))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.file_search_scroll_handle)
            .into_any_element()
        };
