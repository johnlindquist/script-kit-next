impl ScriptListApp {
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

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        // Filter entries from cached data
        let (filtered_entries, _) =
            builtins::filter_menu_bar_entries(&self.cached_current_app_entries, &filter);
        let filtered_len = filtered_entries.len();

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
                let has_cmd = event.keystroke.modifiers.platform;

                // ESC: Clear filter first if present, otherwise go back/close
                if is_key_escape(key) && !this.show_actions_popup {
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

                // Compute filtered list
                let (filtered, _) =
                    builtins::filter_menu_bar_entries(&this.cached_current_app_entries, &current_filter);
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
                        let entry = this.cached_current_app_entries[*orig_idx].clone();
                        tracing::info!(
                            entry_id = %entry.id,
                            entry_name = %entry.name,
                            "current_app_commands.execute_selected"
                        );
                        this.execute_builtin(&entry, cx);
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
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(self.theme.colors.text.muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No menu bar commands available"
                } else {
                    "No commands match your filter"
                })
                .into_any_element()
        } else {
            let entries_for_closure: Vec<(usize, builtins::BuiltInEntry)> = filtered_entries
                .iter()
                .map(|(i, e)| (*i, (*e).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let current_input_mode = self.input_mode;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();
            uniform_list(
                "current-app-commands",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, entry)) = entries_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered =
                                    hovered == Some(ix) && current_input_mode == InputMode::Mouse;

                                let name = entry.name.clone();
                                let description = entry.description.clone();

                                let click_entity = click_entity_handle.clone();
                                let click_handler = move |_event: &gpui::ClickEvent,
                                                          _window: &mut Window,
                                                          cx: &mut gpui::App| {
                                    if let Some(app) = click_entity.upgrade() {
                                        app.update(cx, |this, cx| {
                                            if let AppView::CurrentAppCommandsView {
                                                selected_index,
                                                ..
                                            } = &mut this.current_view
                                            {
                                                *selected_index = ix;
                                            }
                                            cx.notify();
                                        });
                                    }
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
                                            .with_hover_effect(
                                                current_input_mode == InputMode::Mouse,
                                            )
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

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("current_app_commands")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(crate::ui::chrome::HEADER_PADDING_X))
                    .py(px(crate::ui::chrome::HEADER_PADDING_Y))
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
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!(
                                "{} command{}",
                                total_count,
                                if total_count == 1 { "" } else { "s" }
                            )),
                    ),
            )
            // Divider
            .child(crate::components::SectionDivider::new())
            // Command list
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            // Footer — minimal hint strip
            .child(crate::components::render_simple_hint_strip(
                vec![
                    gpui::SharedString::from("↵ Run"),
                    gpui::SharedString::from("Esc Back"),
                ],
                None,
            ))
            .into_any_element()
    }
}
