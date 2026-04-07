        // Pre-compute colors - use theme for consistency with main menu
        let list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
        #[allow(unused_variables)]
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        // Build virtualized list
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(self.theme.colors.text.muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No clipboard history"
                } else {
                    "No entries match your filter"
                })
                .into_any_element()
        } else {
            // Clone data for the closure
            let entries_for_closure: Vec<_> = filtered_entries
                .iter()
                .map(|(i, e)| (*i, (*e).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let image_cache_for_list = image_cache.clone();
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

            uniform_list(
                "clipboard-history",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, entry)) = entries_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered = hovered == Some(ix);

                                // Get cached thumbnail for images
                                let cached_image = if entry.content_type
                                    == clipboard_history::ContentType::Image
                                {
                                    image_cache_for_list.get(&entry.id).cloned()
                                } else {
                                    None
                                };

                                // Use display_preview() from ClipboardEntryMeta
                                let display_content = entry.display_preview();

                                // Format relative time (entry.timestamp is in milliseconds)
                                let now_ms = chrono::Utc::now().timestamp_millis();
                                let age_secs = (now_ms - entry.timestamp) / 1000;
                                let relative_time = if age_secs < 60 {
                                    "just now".to_string()
                                } else if age_secs < 3600 {
                                    format!("{}m ago", age_secs / 60)
                                } else if age_secs < 86400 {
                                    format!("{}h ago", age_secs / 3600)
                                } else {
                                    format!("{}d ago", age_secs / 86400)
                                };

                                // Add pin indicator
                                let name = if entry.pinned {
                                    format!("📌 {}", display_content)
                                } else {
                                    display_content
                                };

                                // Build list item with optional thumbnail
                                let mut item = ListItem::new(name, list_colors)
                                    .description_opt(Some(relative_time))
                                    .selected(is_selected)
                                    .hovered(is_hovered)
                                    .with_accent_bar(true);

                                // Add thumbnail for images, text icon for text entries
                                if let Some(render_image) = cached_image {
                                    item = item.icon_image(render_image);
                                } else if entry.content_type == clipboard_history::ContentType::Text
                                {
                                    item = item.icon("📄");
                                }

                                // Click handler via canonical mouse contract
                                let click_entity = click_entity_handle.clone();
                                let entry_id = entry.id.clone();
                                let click_handler = move |event: &gpui::ClickEvent,
                                                           _window: &mut Window,
                                                           cx: &mut gpui::App| {
                                    if let Some(app) = click_entity.upgrade() {
                                        let entry_id = entry_id.clone();
                                        app.update(cx, |this, cx| {
                                            this.enter_mouse_mode_from_row(cx);
                                            if let AppView::ClipboardHistoryView {
                                                selected_index, ..
                                            } = &mut this.current_view
                                            {
                                                *selected_index = ix;
                                            }
                                            this.focused_clipboard_entry_id =
                                                Some(entry_id.clone());
                                            cx.notify();

                                            // Double-click: copy and paste
                                            if Self::mouse_click_count(event) >= 2 {
                                                tracing::debug!(
                                                    target: "script_kit::mouse",
                                                    event = "clipboard_list_row_double_clicked",
                                                    row_index = ix,
                                                    entry_id = %entry_id,
                                                    "Pasting clipboard entry from mouse double-click"
                                                );
                                                if clipboard_history::copy_entry_to_clipboard(
                                                    &entry_id,
                                                )
                                                .is_ok()
                                                {
                                                    script_kit_gpui::set_main_window_visible(
                                                        false,
                                                    );
                                                    platform::defer_hide_main_window(cx);
                                                    NEEDS_RESET
                                                        .store(true, Ordering::SeqCst);
                                                    std::thread::spawn(|| {
                                                        std::thread::sleep(
                                                            std::time::Duration::from_millis(
                                                                100,
                                                            ),
                                                        );
                                                        let _ = selected_text::simulate_paste_with_cg();
                                                    });
                                                }
                                            }
                                        });
                                    }
                                };

                                // Hover handler via canonical mouse contract
                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler = move |is_hovered: &bool, _window: &mut Window, cx: &mut gpui::App| {
                                    if let Some(app) = hover_entity.upgrade() {
                                        app.update(cx, |this, cx| {
                                            this.update_row_hover_from_mouse(ix, *is_hovered, cx);
                                        });
                                    }
                                };

                                div()
                                    .id(ix)
                                    .cursor_pointer()
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(item)
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.clipboard_list_scroll_handle)
            .into_any_element()
        };
