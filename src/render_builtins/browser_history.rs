impl ScriptListApp {
    fn browser_history_attachment_part(
        &self,
        index: usize,
        entry: &crate::browser_history::BrowserHistoryEntry,
    ) -> crate::ai::message_parts::AiContextPart {
        let title = entry.display_title().to_string();
        let target = crate::ai::TabAiTargetContext {
            source: "BrowserHistory".to_string(),
            kind: "browser_history_entry".to_string(),
            semantic_id: crate::protocol::generate_semantic_id(
                "browser-history",
                index,
                &entry.history_key(),
            ),
            label: title.clone(),
            metadata: Some(serde_json::json!({
                "browserName": entry.browser_name,
                "browserBundleId": entry.browser_bundle_id,
                "title": entry.title,
                "url": entry.url,
                "host": entry.host,
                "lastVisitedAtMs": entry.last_visited_at_ms,
                "lastVisitedAt": crate::browser_history::format_history_timestamp(entry.last_visited_at_ms),
                "visitCount": entry.visit_count,
                "profile": entry.profile,
            })),
        };
        let label = crate::ai::format_explicit_target_chip_label(&target);
        crate::ai::message_parts::AiContextPart::FocusedTarget { target, label }
    }

    fn browser_history_meta(entry: &crate::browser_history::BrowserHistoryEntry) -> String {
        format!(
            "{} · {} visits · {}",
            entry.browser_name,
            entry.visit_count,
            crate::browser_history::format_history_timestamp(entry.last_visited_at_ms)
        )
    }

    fn render_browser_history(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use gpui_component::scroll::ScrollableElement as _;

        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::expanded("browser_history", false),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();

        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;

        let filtered_entries: Vec<crate::browser_history::BrowserHistoryEntry> =
            crate::browser_history::fuzzy_search_browser_history(
                &self.cached_browser_history,
                &filter,
            )
            .into_iter()
            .map(|hit| hit.entry)
            .collect();
        let filtered_len = filtered_entries.len();
        let selected_index = if let Some(reanchored) =
            Self::builtin_reanchor_selection_from_scroll_handle(
                selected_index,
                &self.browser_history_scroll_handle,
                filtered_len,
            )
        {
            tracing::info!(
                target: "script_kit::scroll",
                event = "builtin_selection_resynced_from_scrollbar",
                view = "browser_history",
                reason = "render",
                selected_before = selected_index,
                selected_after = reanchored,
            );
            if let AppView::BrowserHistoryView { selected_index, .. } = &mut self.current_view {
                *selected_index = reanchored;
            }
            reanchored
        } else {
            selected_index
        };
        let selected_entry = filtered_entries.get(selected_index).cloned();
        let in_portal = self.is_in_attachment_portal();

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if crate::ui_foundation::is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        if this.is_in_attachment_portal() {
                            this.close_attachment_portal_cancel(cx);
                        } else {
                            this.go_back_or_close(window, cx);
                        }
                    }
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                let Some((current_filter, current_selected)) = (match &this.current_view {
                    AppView::BrowserHistoryView {
                        filter,
                        selected_index,
                    } => Some((filter.clone(), *selected_index)),
                    _ => None,
                }) else {
                    return;
                };

                let filtered_entries: Vec<crate::browser_history::BrowserHistoryEntry> =
                    crate::browser_history::fuzzy_search_browser_history(
                        &this.cached_browser_history,
                        &current_filter,
                    )
                    .into_iter()
                    .map(|hit| hit.entry)
                    .collect();
                let filtered_len = filtered_entries.len();

                if crate::ui_foundation::is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::BrowserHistoryView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.browser_history_scroll_handle
                                .scroll_to_item(*selected_index);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if crate::ui_foundation::is_key_down(key) {
                    if current_selected < filtered_len.saturating_sub(1) {
                        if let AppView::BrowserHistoryView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.browser_history_scroll_handle
                                .scroll_to_item(*selected_index);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if crate::ui_foundation::is_key_enter(key) {
                    if this.is_in_attachment_portal() {
                        if let Some(entry) = filtered_entries.get(current_selected) {
                            let part =
                                this.browser_history_attachment_part(current_selected, entry);
                            this.close_attachment_portal_with_part(part, cx);
                        }
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        let list_colors = ListItemColors::from_theme(&self.theme);
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No browser history found"
                } else {
                    "No browser history entries match your filter"
                })
                .into_any_element()
        } else {
            let selected = selected_index;
            let entity = cx.entity().downgrade();

            div()
                .id("browser-history-list")
                .w_full()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .track_scroll(&self.browser_history_scroll_handle)
                .overflow_y_scrollbar()
                .children(filtered_entries.iter().enumerate().map(move |(display_ix, entry)| {
                    let item = ListItem::new(entry.display_title().to_string(), list_colors)
                        .description_opt(Some(Self::browser_history_meta(entry)))
                        .selected(display_ix == selected)
                        .with_accent_bar(true);

                    let entity = entity.clone();
                    div()
                        .id(gpui::ElementId::Integer(display_ix as u64))
                        .cursor_pointer()
                        .on_click(move |event, _window, cx| {
                            if let Some(app) = entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    if let AppView::BrowserHistoryView { selected_index, .. } =
                                        &mut this.current_view
                                    {
                                        *selected_index = display_ix;
                                    }
                                    if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                        if mouse_event.down.click_count == 2
                                            && this.is_in_attachment_portal()
                                        {
                                            let filtered_entries: Vec<
                                                crate::browser_history::BrowserHistoryEntry,
                                            > = crate::browser_history::fuzzy_search_browser_history(
                                                &this.cached_browser_history,
                                                this.filter_text(),
                                            )
                                            .into_iter()
                                            .map(|hit| hit.entry)
                                            .collect();
                                            if let Some(entry) =
                                                filtered_entries.get(display_ix)
                                            {
                                                let part = this.browser_history_attachment_part(
                                                    display_ix,
                                                    entry,
                                                );
                                                this.close_attachment_portal_with_part(part, cx);
                                            }
                                        }
                                    }
                                    cx.notify();
                                });
                            }
                        })
                        .child(item)
                }))
                .into_any_element()
        };

        let preview_panel: AnyElement = match selected_entry {
            Some(entry) => div()
                .w_full()
                .h_full()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .px(px(design_spacing.padding_lg))
                .py(px(design_spacing.padding_md))
                .font_family(design_typography.font_family)
                .child(
                    div()
                        .w_full()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .child(Self::browser_history_meta(&entry)),
                )
                .child(
                    div()
                        .w_full()
                        .pt(px(design_spacing.padding_md))
                        .text_sm()
                        .text_color(rgb(text_primary))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(entry.display_title().to_string()),
                )
                .child(
                    div()
                        .w_full()
                        .pt(px(design_spacing.padding_sm))
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .child(entry.url),
                )
                .into_any_element(),
            None => div()
                .w_full()
                .h_full()
                .min_h(px(0.))
                .px(px(design_spacing.padding_lg))
                .py(px(design_spacing.padding_md))
                .font_family(design_typography.font_family)
                .text_xs()
                .text_color(rgb(text_muted))
                .child("Select a browser history entry to preview it")
                .into_any_element(),
        };

        let header_element = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .child(
                div().flex_1().flex().flex_row().items_center().child(
                    Input::new(&self.gpui_input_state)
                        .w_full()
                        .h(px(28.))
                        .px(px(0.))
                        .py(px(0.))
                        .with_size(Size::Size(px(design_typography.font_size_xl)))
                        .appearance(false)
                        .bordered(false)
                        .focus_bordered(false),
                ),
            )
            .child(div().text_sm().text_color(rgb(text_muted)).child(format!(
                "{} entr{}",
                self.cached_browser_history.len(),
                if self.cached_browser_history.len() == 1 {
                    "y"
                } else {
                    "ies"
                }
            )));

        let list_pane = div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .py(px(design_spacing.padding_xs))
            .on_scroll_wheel(cx.listener(
                move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                    let view_state = if let AppView::BrowserHistoryView {
                        filter,
                        selected_index,
                    } = &this.current_view
                    {
                        Some((filter.clone(), *selected_index))
                    } else {
                        None
                    };

                    let Some((current_filter, current_selected)) = view_state else {
                        return;
                    };

                    let filtered_entries: Vec<crate::browser_history::BrowserHistoryEntry> =
                        crate::browser_history::fuzzy_search_browser_history(
                            &this.cached_browser_history,
                            &current_filter,
                        )
                        .into_iter()
                        .map(|hit| hit.entry)
                        .collect();
                    let filtered_len = filtered_entries.len();

                    let Some(new_selected) = this.builtin_scroll_target_from_wheel(
                        event,
                        current_selected,
                        filtered_len,
                    ) else {
                        if filtered_len > 0 {
                            cx.stop_propagation();
                        }
                        return;
                    };

                    if let AppView::BrowserHistoryView { selected_index, .. } =
                        &mut this.current_view
                    {
                        *selected_index = new_selected;
                    }

                    this.browser_history_scroll_handle
                        .scroll_to_item(new_selected);
                    Self::log_builtin_scroll_event(
                        "browser_history",
                        "scroll_to_item",
                        "wheel",
                        filtered_len,
                        Some(new_selected),
                        Some(new_selected),
                        Some(&current_filter),
                        "mouse",
                    );
                    cx.notify();
                    cx.stop_propagation();
                },
            ))
            .child(list_element);

        let hints = if in_portal {
            vec!["↵ Attach".into(), "Esc Cancel".into()]
        } else {
            vec!["Esc Back".into()]
        };
        crate::components::emit_prompt_hint_audit("browser_history", &hints);

        crate::components::render_expanded_view_scaffold_with_hints(
            header_element,
            list_pane,
            preview_panel,
            hints,
            None,
        )
        .text_color(rgb(text_primary))
        .font_family(design_typography.font_family)
        .key_context("browser_history")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}

#[cfg(test)]
mod browser_history_scroll_contract {
    const SOURCE: &str = include_str!("browser_history.rs");

    #[test]
    fn browser_history_intercepts_wheel_scrolling_with_builtin_helpers() {
        assert!(
            SOURCE.contains(".track_scroll(&self.browser_history_scroll_handle)"),
            "browser history should track its dedicated scroll handle"
        );
        assert!(
            SOURCE.contains(".on_scroll_wheel(cx.listener("),
            "browser history should intercept wheel events on the list pane"
        );
        assert!(
            SOURCE.contains("builtin_scroll_target_from_wheel"),
            "browser history wheel scrolling should use the shared builtin helper"
        );
        assert!(
            SOURCE.contains("cx.stop_propagation();"),
            "browser history wheel scrolling must stop propagation so GPUI native scrolling cannot fight selection"
        );
        assert!(
            SOURCE.contains("builtin_reanchor_selection_from_scroll_handle"),
            "browser history should reanchor selection after ScrollHandle movement"
        );
    }
}
