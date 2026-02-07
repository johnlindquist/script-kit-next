impl ScriptListApp {
    /// Render app launcher view
    /// P0 FIX: Data comes from self.apps, view passes only state
    fn render_app_launcher(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for spacing/typography/visual, theme for colors
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // P0 FIX: Filter apps from self.apps instead of taking ownership
        let filtered_apps: Vec<_> = if filter.is_empty() {
            self.apps.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.apps
                .iter()
                .enumerate()
                .filter(|(_, a)| a.name.to_lowercase().contains(&filter_lower))
                .collect()
        };
        let filtered_len = filtered_apps.len();

        // Key handler for app launcher
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

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // ESC: Clear filter first if present, otherwise go back/close
                if key_str == "escape" && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                logging::log("KEY", &format!("AppLauncher key: '{}'", key_str));

                // P0 FIX: View state only - data comes from this.apps
                if let AppView::AppLauncherView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // P0 FIX: Reference apps from self
                    let filtered_apps: Vec<_> = if filter.is_empty() {
                        this.apps.iter().enumerate().collect()
                    } else {
                        let filter_lower = filter.to_lowercase();
                        this.apps
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| a.name.to_lowercase().contains(&filter_lower))
                            .collect()
                    };
                    let filtered_len = filtered_apps.len();

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            // Launch selected app and hide window
                            if let Some((_, app)) = filtered_apps.get(*selected_index) {
                                logging::log("EXEC", &format!("Launching app: {}", app.name));
                                if let Err(e) = app_launcher::launch_application(app) {
                                    logging::log("ERROR", &format!("Failed to launch app: {}", e));
                                } else {
                                    logging::log("EXEC", &format!("Launched: {}", app.name));
                                    // Hide main window only (not entire app) to keep HUD visible
                                    script_kit_gpui::set_main_window_visible(false);
                                    platform::hide_main_window();
                                    NEEDS_RESET.store(true, Ordering::SeqCst);
                                }
                            }
                        }
                        // Note: "escape" is handled by handle_global_shortcut_with_options above
                        "backspace" => {
                            if !filter.is_empty() {
                                filter.pop();
                                *selected_index = 0;
                                cx.notify();
                            }
                        }
                        _ => {
                            if let Some(ref key_char) = event.keystroke.key_char {
                                if let Some(ch) = key_char.chars().next() {
                                    if !ch.is_control() {
                                        filter.push(ch);
                                        *selected_index = 0;
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        let input_display = if filter.is_empty() {
            SharedString::from("Search applications...")
        } else {
            SharedString::from(filter.clone())
        };
        let input_is_empty = filter.is_empty();

        // Pre-compute colors
        let list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
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
                    "No applications found"
                } else {
                    "No apps match your filter"
                })
                .into_any_element()
        } else {
            // Clone data for the closure
            let apps_for_closure: Vec<_> = filtered_apps
                .iter()
                .map(|(i, a)| (*i, (*a).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let current_input_mode = self.input_mode;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

            uniform_list(
                "app-launcher",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, app)) = apps_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered = hovered == Some(ix) && current_input_mode == InputMode::Mouse;

                                // Format app path for description
                                let path_str = app.path.to_string_lossy();
                                let description = if path_str.starts_with("/Applications") {
                                    None // No need to show path for standard apps
                                } else {
                                    Some(path_str.to_string())
                                };

                                // Use pre-decoded icon if available, fallback to emoji
                                let icon = match &app.icon {
                                    Some(img) => list_item::IconKind::Image(img.clone()),
                                    None => list_item::IconKind::Emoji("📱".to_string()),
                                };

                                // Click handler: select on click, launch on double-click
                                let click_entity = click_entity_handle.clone();
                                let app_info = app.clone();
                                let click_handler = move |event: &gpui::ClickEvent,
                                                           _window: &mut Window,
                                                           cx: &mut gpui::App| {
                                    if let Some(app_entity) = click_entity.upgrade() {
                                        let app_info = app_info.clone();
                                        app_entity.update(cx, |this, cx| {
                                            if let AppView::AppLauncherView {
                                                selected_index, ..
                                            } = &mut this.current_view
                                            {
                                                *selected_index = ix;
                                            }
                                            cx.notify();

                                            // Double-click: launch app
                                            if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                                if mouse_event.down.click_count == 2 {
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Double-click launching app: {}",
                                                            app_info.name
                                                        ),
                                                    );
                                                    if app_launcher::launch_application(&app_info)
                                                        .is_ok()
                                                    {
                                                        script_kit_gpui::set_main_window_visible(
                                                            false,
                                                        );
                                                        platform::hide_main_window();
                                                        NEEDS_RESET
                                                            .store(true, Ordering::SeqCst);
                                                    }
                                                }
                                            }
                                        });
                                    }
                                };

                                // Hover handler for mouse tracking
                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler = move |is_hovered: &bool, _window: &mut Window, cx: &mut gpui::App| {
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
                                        ListItem::new(app.name.clone(), list_colors)
                                            .icon_kind(icon)
                                            .description_opt(description)
                                            .selected(is_selected)
                                            .hovered(is_hovered)
                                            .with_hover_effect(current_input_mode == InputMode::Mouse)
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
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("app_launcher")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Title
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("🚀 Apps"),
                    )
                    // Search input with blinking cursor
                    // ALIGNMENT FIX: Uses canonical cursor constants and negative margin for placeholder
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} apps", self.apps.len())),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // App list
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Launch")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }
}
