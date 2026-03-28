impl ScriptListApp {
    /// Render window switcher view with 50/50 split layout
    /// P0 FIX: Data comes from self.cached_windows, view passes only state
    fn render_window_switcher(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // P0 FIX: Filter windows from self.cached_windows instead of taking ownership
        let filtered_windows: Vec<_> = if filter.is_empty() {
            self.cached_windows.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.cached_windows
                .iter()
                .enumerate()
                .filter(|(_, w)| {
                    w.title.to_lowercase().contains(&filter_lower)
                        || w.app.to_lowercase().contains(&filter_lower)
                })
                .collect()
        };
        let filtered_len = filtered_windows.len();

        // Key handler for window switcher
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = is_platform_modifier(&event.keystroke.modifiers);

                // ESC: Clear filter first if present, otherwise go back/close
                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                logging::log("KEY", &format!("WindowSwitcher key: '{}'", key));

                // P0 FIX: View state only - data comes from this.cached_windows
                if let AppView::WindowSwitcherView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // P0 FIX: Reference cached_windows from self
                    let filtered_windows: Vec<_> = if filter.is_empty() {
                        this.cached_windows.iter().enumerate().collect()
                    } else {
                        let filter_lower = filter.to_lowercase();
                        this.cached_windows
                            .iter()
                            .enumerate()
                            .filter(|(_, w)| {
                                w.title.to_lowercase().contains(&filter_lower)
                                    || w.app.to_lowercase().contains(&filter_lower)
                            })
                            .collect()
                    };
                    let filtered_len = filtered_windows.len();

                    match key {
                        _ if is_key_up(key) => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.window_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        _ if is_key_down(key) => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.window_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        _ if is_key_enter(key) => {
                            // Focus selected window and hide Script Kit
                            if let Some((_, window_info)) = filtered_windows.get(*selected_index) {
                                logging::log(
                                    "EXEC",
                                    &format!("Focusing window: {}", window_info.title),
                                );
                                if let Err(e) = window_control::focus_window(window_info.id) {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to focus window: {}", e),
                                    );
                                    this.toast_manager.push(
                                        components::toast::Toast::error(
                                            format!("Failed to focus window: {}", e),
                                            &this.theme,
                                        )
                                        .duration_ms(Some(TOAST_ERROR_MS)),
                                    );
                                    cx.notify();
                                } else {
                                    logging::log(
                                        "EXEC",
                                        &format!("Focused window: {}", window_info.title),
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                            }
                        }
                        // Note: "escape" is handled by handle_global_shortcut_with_options above
                        // Text input (backspace, characters) is handled by the shared Input component
                        // which syncs via handle_filter_input_change()
                        _ => {}
                    }
                }
            },
        );

        // Pre-compute colors
        let list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
        #[allow(unused_variables)]
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;

        // Build virtualized list
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(self.theme.colors.text.muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No windows found"
                } else {
                    "No windows match your filter"
                })
                .into_any_element()
        } else {
            // Clone data for the closure
            let windows_for_closure: Vec<_> = filtered_windows
                .iter()
                .map(|(i, w)| (*i, (*w).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let current_input_mode = self.input_mode;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

            uniform_list(
                "window-switcher",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, window_info)) = windows_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered =
                                    hovered == Some(ix) && current_input_mode == InputMode::Mouse;

                                // Format: "AppName: Window Title"
                                let name = format!("{}: {}", window_info.app, window_info.title);

                                // Format bounds as description
                                let description = format!(
                                    "{}×{} at ({}, {})",
                                    window_info.bounds.width,
                                    window_info.bounds.height,
                                    window_info.bounds.x,
                                    window_info.bounds.y
                                );

                                // Click handler: select on click, focus window on double-click
                                let click_entity = click_entity_handle.clone();
                                let win_id = window_info.id;
                                let click_handler =
                                    move |event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app) = click_entity.upgrade() {
                                            app.update(cx, |this, cx| {
                                                if let AppView::WindowSwitcherView {
                                                    selected_index,
                                                    ..
                                                } = &mut this.current_view
                                                {
                                                    *selected_index = ix;
                                                }
                                                cx.notify();

                                                // Double-click: focus window
                                                if let gpui::ClickEvent::Mouse(mouse_event) = event
                                                {
                                                    if mouse_event.down.click_count == 2 {
                                                        logging::log(
                                                            "UI",
                                                            &format!(
                                                                "Double-click focusing window {}",
                                                                win_id
                                                            ),
                                                        );
                                                        if window_control::focus_window(win_id)
                                                            .is_ok()
                                                        {
                                                            this.hide_main_and_reset(cx);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    };

                                // Hover handler for mouse tracking
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
                                    .tooltip(|window, cx| {
                                        gpui_component::tooltip::Tooltip::new(
                                            "Switch to selected window",
                                        )
                                        .key_binding(
                                            gpui::Keystroke::parse("enter")
                                                .ok()
                                                .map(gpui_component::kbd::Kbd::new),
                                        )
                                        .build(window, cx)
                                    })
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
            .track_scroll(&self.window_list_scroll_handle)
            .into_any_element()
        };

        // Build actions panel for selected window
        let selected_window = filtered_windows
            .get(selected_index)
            .map(|(_, w)| (*w).clone());
        let actions_panel = self.render_window_actions_panel(
            &selected_window,
            &design_colors,
            &design_spacing,
            &design_typography,
            &design_visual,
            cx,
        );

        let header = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            // Search input - uses shared gpui_input_state for consistent cursor/selection
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
                    .child(format!("{} windows", self.cached_windows.len())),
            );

        // Main content area - 50/50 split: Window list on left, Actions on right
        let content = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .h_full()
            // Left side: Window list (50% width)
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .min_h(px(0.))
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            // Right side: Actions panel (50% width)
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .child(actions_panel),
            );

        crate::components::render_minimal_list_prompt_scaffold(
            header,
            content,
            vec![
                gpui::SharedString::from("↵ Switch"),
                gpui::SharedString::from("Esc Back"),
            ],
            None,
        )
        .rounded(px(design_visual.radius_lg))
        .text_color(rgb(text_primary))
        .font_family(design_typography.font_family)
        .key_context("window_switcher")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}

#[cfg(test)]
mod window_switcher_chrome_audit {
    #[test]
    fn window_switcher_uses_minimal_chrome_footer() {
        let source = include_str!("window_switcher.rs");
        assert!(
            source.contains("render_minimal_list_prompt_scaffold("),
            "window_switcher should use render_minimal_list_prompt_scaffold"
        );
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert_eq!(
            source.matches(&legacy).count(),
            0,
            "window_switcher should not use PromptFooter"
        );
    }
}
