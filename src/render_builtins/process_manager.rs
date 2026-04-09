impl ScriptListApp {
    /// Refresh interval for the process manager list (2 seconds).
    const PROCESS_MANAGER_REFRESH_INTERVAL: std::time::Duration =
        std::time::Duration::from_secs(2);

    /// Start the periodic refresh task for the ProcessManagerView.
    ///
    /// Spawns a background timer that polls the process manager every 2 seconds
    /// and updates `cached_processes` only when the list has changed.
    /// The task is stored in `process_manager_refresh_task` and is automatically
    /// cancelled when the field is overwritten or set to None.
    fn start_process_manager_refresh(&mut self, cx: &mut Context<Self>) {
        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(ScriptListApp::PROCESS_MANAGER_REFRESH_INTERVAL)
                    .await;

                let should_continue = cx
                    .update(|cx| {
                        this.update(cx, |app, cx| {
                            // Bail if we've navigated away
                            if !matches!(
                                app.current_view,
                                AppView::ProcessManagerView { .. }
                            ) {
                                tracing::info!(
                                    correlation_id = "process-manager-refresh",
                                    "process_manager.refresh_stopped.view_changed"
                                );
                                app.process_manager_refresh_task = None;
                                return false;
                            }

                            let new_processes = crate::process_manager::PROCESS_MANAGER
                                .get_active_processes_sorted();

                            // Compare by PIDs to avoid unnecessary rerenders
                            let old_pids: Vec<u32> =
                                app.cached_processes.iter().map(|p| p.pid).collect();
                            let new_pids: Vec<u32> =
                                new_processes.iter().map(|p| p.pid).collect();

                            if old_pids != new_pids {
                                tracing::info!(
                                    correlation_id = "process-manager-refresh",
                                    old_count = old_pids.len(),
                                    new_count = new_pids.len(),
                                    "process_manager.refresh.list_changed"
                                );

                                app.cached_processes = new_processes;

                                // Clamp selection index if list shrank
                                if let AppView::ProcessManagerView {
                                    selected_index, ..
                                } = &mut app.current_view
                                {
                                    let len = app.cached_processes.len();
                                    if *selected_index >= len && len > 0 {
                                        *selected_index = len - 1;
                                    }
                                }

                                cx.notify();
                            }

                            true
                        })
                    })
                    .unwrap_or(false);

                if !should_continue {
                    break;
                }
            }
        });

        self.process_manager_refresh_task = Some(task);
        tracing::info!(
            correlation_id = "process-manager-refresh",
            interval_secs = 2,
            "process_manager.refresh_started"
        );
    }

    /// Stop the periodic refresh task for ProcessManagerView.
    fn stop_process_manager_refresh(&mut self) {
        if self.process_manager_refresh_task.take().is_some() {
            tracing::info!(
                correlation_id = "process-manager-refresh",
                "process_manager.refresh_stopped"
            );
        }
    }

    /// Render process manager view showing running background scripts
    fn render_process_manager(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal(
                "process_manager",
                2,     // hints: "↵ Stop", "Esc Back"
                false, // no leading status text
                false, // no actions hint
            ),
        );
        let tokens = get_tokens(self.current_design);
        let _design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;

        // Filter processes from cached data
        let filtered_processes: Vec<_> = if filter.is_empty() {
            self.cached_processes.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.cached_processes
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.script_path.to_lowercase().contains(&filter_lower)
                        || p.pid.to_string().contains(&filter_lower)
                })
                .collect()
        };
        let filtered_len = filtered_processes.len();

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

                // Extract current view state without holding mutable borrow
                let view_state = if let AppView::ProcessManagerView {
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

                // Compute filtered list from cached data
                let filtered: Vec<_> = if current_filter.is_empty() {
                    this.cached_processes.iter().enumerate().collect()
                } else {
                    let filter_lower = current_filter.to_lowercase();
                    this.cached_processes
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            p.script_path.to_lowercase().contains(&filter_lower)
                                || p.pid.to_string().contains(&filter_lower)
                        })
                        .collect()
                };
                let current_filtered_len = filtered.len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::ProcessManagerView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.process_list_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_down(key) {
                    if current_selected < current_filtered_len.saturating_sub(1) {
                        if let AppView::ProcessManagerView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.process_list_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_enter(key) {
                    // Kill selected process
                    if let Some((_, process_info)) = filtered.get(current_selected) {
                        let pid = process_info.pid;
                        let script_path = process_info.script_path.clone();

                        tracing::info!(
                            correlation_id = "process-manager-terminate",
                            pid,
                            script_path = script_path.as_str(),
                            "process_manager.terminate_selected"
                        );

                        match crate::process_manager::PROCESS_MANAGER.terminate_process(pid) {
                            Ok(()) => {
                                // Refresh the cached processes
                                this.cached_processes = crate::process_manager::PROCESS_MANAGER
                                    .get_active_processes_sorted();

                                let script_name = std::path::Path::new(&script_path)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("process");

                                this.show_hud(
                                    format!("Stopped {}", script_name),
                                    Some(HUD_SHORT_MS),
                                    cx,
                                );

                                // Clamp selection if list got shorter
                                let new_len = this.cached_processes.len();
                                if let AppView::ProcessManagerView { selected_index, .. } =
                                    &mut this.current_view
                                {
                                    if *selected_index >= new_len && new_len > 0 {
                                        *selected_index = new_len - 1;
                                    }
                                }

                                if new_len == 0 {
                                    this.go_back_or_close(window, cx);
                                    return;
                                }

                                cx.notify();
                            }
                            Err(err_msg) => {
                                tracing::warn!(
                                    pid,
                                    error = err_msg.as_str(),
                                    "process_manager.terminate_failed"
                                );
                                this.show_error_toast(
                                    format!("Failed to stop PID {}: {}", pid, err_msg),
                                    cx,
                                );
                            }
                        }
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
                    "No running scripts"
                } else {
                    "No processes match your filter"
                })
                .into_any_element()
        } else {
            let processes_for_closure: Vec<_> = filtered_processes
                .iter()
                .map(|(i, p)| (*i, (*p).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

            uniform_list(
                "process-manager",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, process_info)) = processes_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered = hovered == Some(ix);

                                // Show script name as primary, PID and path as description
                                let script_name = std::path::Path::new(&process_info.script_path)
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(&process_info.script_path);
                                let name = script_name.to_string();

                                let elapsed = chrono::Utc::now()
                                    .signed_duration_since(process_info.started_at);
                                let duration_str = if elapsed.num_hours() > 0 {
                                    format!("{}h {}m", elapsed.num_hours(), elapsed.num_minutes() % 60)
                                } else if elapsed.num_minutes() > 0 {
                                    format!("{}m {}s", elapsed.num_minutes(), elapsed.num_seconds() % 60)
                                } else {
                                    format!("{}s", elapsed.num_seconds())
                                };

                                let description =
                                    format!("PID {} • running {}", process_info.pid, duration_str);

                                let click_entity = click_entity_handle.clone();
                                let click_handler = move |_event: &gpui::ClickEvent,
                                                          _window: &mut Window,
                                                          cx: &mut gpui::App| {
                                    if let Some(app) = click_entity.upgrade() {
                                        app.update(cx, |this, cx| {
                                            if let AppView::ProcessManagerView {
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
                                    .tooltip(|window, cx| {
                                        gpui_component::tooltip::Tooltip::new("Stop this process")
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
            .track_scroll(&self.process_list_scroll_handle)
            .into_any_element()
        };

        let total_count = self.cached_processes.len();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("process_manager")
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
                                "{} process{}",
                                total_count,
                                if total_count == 1 { "" } else { "es" }
                            )),
                    ),
            )
            // Divider
            .child(crate::components::SectionDivider::new())
            // Process list
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            .child(if matches!(
                crate::footer_popup::active_main_window_footer_surface(),
                Some("process_manager")
            ) {
                crate::components::prompt_layout_shell::render_native_main_window_footer_spacer()
            } else {
                crate::components::render_simple_hint_strip(
                    vec![
                        gpui::SharedString::from("↵ Stop"),
                        gpui::SharedString::from("Esc Back"),
                    ],
                    None,
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod process_manager_chrome_audit {
    #[test]
    fn process_manager_uses_minimal_chrome_footer() {
        let source = include_str!("process_manager.rs");
        assert!(
            source.contains("render_simple_hint_strip("),
            "process_manager should use render_simple_hint_strip"
        );
        assert!(
            source.contains("SectionDivider::new()"),
            "process_manager should use SectionDivider"
        );
        assert!(
            source.contains("HEADER_PADDING_X") && source.contains("HEADER_PADDING_Y"),
            "process_manager should use shared chrome header padding"
        );
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert_eq!(
            source.matches(&legacy).count(),
            0,
            "process_manager should not use PromptFooter"
        );
    }
}
