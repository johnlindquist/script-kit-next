impl ScriptListApp {
    // @lat: [[lat.md/protocol#Protocol#Query and introspection]]
    fn current_app_commands_filtered_entries<'a>(
        entries: &'a [builtins::BuiltInEntry],
        filter: &str,
    ) -> Vec<(usize, &'a builtins::BuiltInEntry)> {
        let (filtered, _) = builtins::filter_menu_bar_entries(entries, filter);
        filtered
    }

    fn current_app_commands_visible_row_names(&self, filter: &str) -> Vec<String> {
        Self::current_app_commands_filtered_entries(&self.cached_current_app_entries, filter)
            .into_iter()
            .map(|(_, entry)| entry.name.clone())
            .collect()
    }

    fn current_app_commands_dataset_and_visible_counts(&self, filter: &str) -> (usize, usize) {
        (
            self.cached_current_app_entries.len(),
            Self::current_app_commands_filtered_entries(&self.cached_current_app_entries, filter)
                .len(),
        )
    }

    fn current_app_commands_selected_visible_row_name(
        &self,
        filter: &str,
        selected_index: usize,
    ) -> Option<String> {
        Self::current_app_commands_filtered_entries(&self.cached_current_app_entries, filter)
            .get(selected_index)
            .map(|(_, entry)| entry.name.clone())
    }

    /// Render the current app commands view showing menu bar commands from the frontmost app.
    fn render_current_app_commands(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let chrome = theme::AppChromeColors::from_theme(&self.theme);

        let filtered_entries =
            Self::current_app_commands_filtered_entries(&self.cached_current_app_entries, &filter);
        let filtered_len = filtered_entries.len();
        let selected_index = if let Some(reanchored) = Self::builtin_reanchor_selection_from_scroll(
            selected_index,
            &self.current_app_commands_scroll_handle,
            filtered_len,
            8,
        ) {
            tracing::info!(
                target: "script_kit::scroll",
                event = "builtin_selection_resynced_from_scrollbar",
                view = "current_app_commands",
                reason = "render",
                selected_before = selected_index,
                selected_after = reanchored,
            );
            if let AppView::CurrentAppCommandsView { selected_index, .. } = &mut self.current_view {
                *selected_index = reanchored;
            }
            reanchored
        } else {
            selected_index
        };

        // Key handler
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
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::BuiltinList,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {}
                    ActionsRoute::Handled => {
                        tracing::debug!(
                            target: "script_kit::actions",
                            event = "builtin_view_actions_key_routed",
                            surface = "current_app_commands",
                            key = %key,
                        );
                        cx.stop_propagation();
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        this.execute_action_for_actions_host(
                            ActionsDialogHost::BuiltinList,
                            action_id,
                            window,
                            cx,
                        );
                        cx.stop_propagation();
                        return;
                    }
                }

                // ESC: Clear filter first if present, otherwise go back/close
                if is_key_escape(key) {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                // Extract current view state
                let view_state = if let AppView::CurrentAppCommandsView {
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

                let filtered = Self::current_app_commands_filtered_entries(
                    &this.cached_current_app_entries,
                    &current_filter,
                );
                let current_filtered_len = filtered.len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::CurrentAppCommandsView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.current_app_commands_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_down(key) {
                    if current_selected < current_filtered_len.saturating_sub(1) {
                        if let AppView::CurrentAppCommandsView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.current_app_commands_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_enter(key) {
                    // Execute selected menu bar action
                    if let Some((orig_idx, _)) = filtered.get(current_selected) {
                        this.execute_selected_current_app_command(*orig_idx, cx);
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        // Pre-compute colors
        let list_colors = ListItemColors::from_theme(&self.theme);

        // Build virtualized list
        let list_element: AnyElement = if filtered_len == 0 {
            let empty_title = if filter.trim().is_empty() {
                "No commands ready yet"
            } else {
                "No matching commands"
            };
            let empty_detail = if filter.trim().is_empty() {
                "Switch back to the app you want to control, then open Current App Commands again."
                    .to_string()
            } else {
                format!(
                    "Nothing matched \"{}\". Press Esc to clear the filter.",
                    filter.trim()
                )
            };

            tracing::info!(
                filter = %filter,
                total_entries = self.cached_current_app_entries.len(),
                "current_app_commands.render_empty_state"
            );

            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .font_family(design_typography.font_family)
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(chrome.text_primary_hex))
                        .child(empty_title),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgba(chrome.text_hint_rgba))
                        .child(empty_detail),
                )
                .into_any_element()
        } else {
            let entries_for_closure: Vec<(usize, builtins::BuiltInEntry)> = filtered_entries
                .iter()
                .map(|(i, e)| (*i, (*e).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();
            uniform_list(
                "current-app-commands",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((orig_idx, entry)) = entries_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered = hovered == Some(ix);

                                let name = entry.name.clone();
                                let description = entry.description.clone();
                                let original_entry_index = *orig_idx;

                                let click_entity = click_entity_handle.clone();
                                let click_handler =
                                    move |event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app) = click_entity.upgrade() {
                                            app.update(cx, |this, cx| {
                                                let should_submit = if let AppView::CurrentAppCommandsView {
                                                    selected_index, ..
                                                } = &mut this.current_view {
                                                    let was_selected = *selected_index == ix;
                                                    *selected_index = ix;
                                                    crate::ui_foundation::should_submit_selected_row_click(
                                                        was_selected,
                                                        event.click_count(),
                                                    )
                                                } else {
                                                    false
                                                };
                                                if should_submit {
                                                    this.execute_selected_current_app_command(
                                                        original_entry_index,
                                                        cx,
                                                    );
                                                }
                                                cx.notify();
                                            });
                                        }
                                        cx.stop_propagation();
                                    };

                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler =
                                    move |is_hovered: &bool,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app) = hover_entity.upgrade() {
                                            app.update(cx, |this, cx| {
                                                if *is_hovered {
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
                                    .cursor_pointer()
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(
                                        ListItem::new(name, list_colors)
                                            .description_opt(Some(description))
                                            .selected(is_selected)
                                            .hovered(is_hovered)
                                            .with_accent_bar(true),
                                    )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.current_app_commands_scroll_handle)
            .into_any_element()
        };

        let total_count = self.cached_current_app_entries.len();

        let header = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
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
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .text_sm()
                    .text_color(rgba(chrome.text_hint_rgba))
                    .child(format!(
                        "{} command{}",
                        total_count,
                        if total_count == 1 { "" } else { "s" }
                    )),
            );

        let content = div()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .overflow_hidden()
            .py(px(design_spacing.padding_xs))
            .relative()
            .on_scroll_wheel(cx.listener(
                move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                    let view_state = if let AppView::CurrentAppCommandsView {
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

                    let filtered = Self::current_app_commands_filtered_entries(
                        &this.cached_current_app_entries,
                        &current_filter,
                    );
                    let filtered_len = filtered.len();

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

                    if let AppView::CurrentAppCommandsView { selected_index, .. } =
                        &mut this.current_view
                    {
                        *selected_index = new_selected;
                    }

                    this.current_app_commands_scroll_handle
                        .scroll_to_item(new_selected, ScrollStrategy::Nearest);

                    if let Some(reanchored) = Self::builtin_reanchor_selection_from_scroll(
                        new_selected,
                        &this.current_app_commands_scroll_handle,
                        filtered_len,
                        8,
                    ) {
                        if let AppView::CurrentAppCommandsView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = reanchored;
                        }
                        tracing::info!(
                            target: "script_kit::scroll",
                            event = "builtin_selection_resynced_from_scrollbar",
                            view = "current_app_commands",
                            reason = "wheel",
                            selected_before = new_selected,
                            selected_after = reanchored,
                        );
                    }

                    Self::log_builtin_scroll_event(
                        "current_app_commands",
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
            .child(list_element)
            .child(self.builtin_uniform_list_scrollbar(
                &self.current_app_commands_scroll_handle,
                filtered_len,
                8,
            ));

        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            vec![
                gpui::SharedString::from("↵ Run"),
                gpui::SharedString::from("Esc Back"),
            ],
            None,
        ));

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .px(px(crate::ui::chrome::HEADER_PADDING_X))
                    .py(px(crate::ui::chrome::HEADER_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(header),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    .child(content),
            )
            .when_some(footer, |d, footer| d.child(footer))
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(chrome.text_primary_hex))
            .font_family(design_typography.font_family)
            .key_context("current_app_commands")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .into_any_element()
    }
}

#[cfg(test)]
mod current_app_commands_chrome_audit {
    fn production_source() -> &'static str {
        include_str!("current_app_commands.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source should exist")
    }

    #[test]
    fn current_app_commands_uses_minimal_chrome_footer() {
        let source = production_source();
        assert!(
            source.contains("render_simple_hint_strip(")
                && source.contains("main_window_footer_slot("),
            "current_app_commands should route minimal hint chrome through the shared footer slot"
        );
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert_eq!(
            source.matches(&legacy).count(),
            0,
            "current_app_commands should not use PromptFooter"
        );
    }

    #[test]
    fn current_app_commands_use_wheel_contract_and_vendor_scrollbar() {
        let source = production_source();
        assert!(
            source.contains(".on_scroll_wheel(cx.listener("),
            "current_app_commands should intercept wheel scrolling on the list pane"
        );
        assert!(
            source.contains("builtin_scroll_target_from_wheel("),
            "current_app_commands should use shared wheel delta conversion"
        );
        assert!(
            source.contains("builtin_reanchor_selection_from_scroll("),
            "current_app_commands should reanchor selection after handle movement"
        );
        assert!(
            source.contains("builtin_uniform_list_scrollbar("),
            "current_app_commands should attach the shared vendor scrollbar helper"
        );
    }
}
