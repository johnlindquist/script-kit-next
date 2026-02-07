        // Add arrow key interceptor for builtin views with Input components
        // This fires BEFORE Input component handles arrow keys, allowing list navigation
        let app_entity_for_arrows = cx.entity().downgrade();
        let arrow_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_arrows;
            move |event, _window, cx| {
                let key = event.keystroke.key.to_lowercase();
                // Check for Up/Down arrow keys (no modifiers except shift for selection)
                if (key == "up" || key == "arrowup" || key == "down" || key == "arrowdown")
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // FIRST: If confirm dialog is open, route all arrow keys to it
                            if crate::confirm::is_confirm_window_open()
                                && crate::confirm::dispatch_confirm_key(&key, cx)
                            {
                                cx.stop_propagation();
                                return;
                            }

                            // Universal: Route arrow keys to actions dialog when popup is open
                            // This ensures ALL views (ChatPrompt, ArgPrompt, etc.) route
                            // arrows to the dialog, not just the few views with explicit cases below.
                            if this.show_actions_popup {
                                if let Some(ref dialog) = this.actions_dialog {
                                    if key == "up" || key == "arrowup" {
                                        dialog.update(cx, |d, cx| d.move_up(cx));
                                    } else if key == "down" || key == "arrowdown" {
                                        dialog.update(cx, |d, cx| d.move_down(cx));
                                    }
                                    crate::actions::notify_actions_window(cx);
                                }
                                cx.stop_propagation();
                                return;
                            }

                            // Only intercept in views that use Input + list navigation
                            match &mut this.current_view {
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Compute filtered length using same logic as render
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    // Use Nucleo fuzzy matching for consistent filtering with render
                                    let filtered_len = if let Some(ref pattern) = filter_pattern {
                                        crate::file_search::filter_results_nucleo_simple(
                                            &this.cached_file_results,
                                            pattern,
                                        )
                                        .len()
                                    } else {
                                        this.cached_file_results.len()
                                    };

                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    // Stop propagation so Input doesn't handle it
                                    cx.stop_propagation();
                                }
                                AppView::ClipboardHistoryView {
                                    selected_index,
                                    filter,
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    let filtered_entries: Vec<_> = if filter.is_empty() {
                                        this.cached_clipboard_entries.iter().enumerate().collect()
                                    } else {
                                        let filter_lower = filter.to_lowercase();
                                        this.cached_clipboard_entries
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, e)| {
                                                e.text_preview
                                                    .to_lowercase()
                                                    .contains(&filter_lower)
                                            })
                                            .collect()
                                    };
                                    let filtered_len = filtered_entries.len();
                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.clipboard_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.clipboard_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                    }
                                    this.focused_clipboard_entry_id = filtered_entries
                                        .get(*selected_index)
                                        .map(|(_, entry)| entry.id.clone());
                                    cx.notify();
                                    cx.stop_propagation();
                                }
                                AppView::AppLauncherView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    // Filter apps to get correct count
                                    let filtered_len = this.apps.len();
                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::WindowSwitcherView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    let filtered_len = this.cached_windows.len();
                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.window_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.window_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::ScriptList => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Main menu: handle list navigation + input history
                                    if key == "up" || key == "arrowup" {
                                        // Input history: only when filter empty AND at top of list
                                        if this.filter_text.is_empty() && this.selected_index == 0 {
                                            if let Some(text) = this.input_history.navigate_up() {
                                                logging::log(
                                                    "HISTORY",
                                                    &format!("Recalled: {}", text),
                                                );
                                                this.filter_text = text.clone();
                                                let text_len = text.len();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            text.clone(),
                                                            _window,
                                                            input_cx,
                                                        );
                                                        state.set_selection(
                                                            text_len, text_len, _window, input_cx,
                                                        );
                                                    },
                                                );
                                                this.queue_filter_compute(text, cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                                return;
                                            }
                                        }
                                        // Normal up navigation - use move_selection_up to skip section headers
                                        this.move_selection_up(cx);
                                    } else if key == "down" || key == "arrowdown" {
                                        // Down during history navigation returns to newer entries
                                        if this.input_history.current_index().is_some() {
                                            if let Some(text) = this.input_history.navigate_down() {
                                                logging::log(
                                                    "HISTORY",
                                                    &format!("Recalled: {}", text),
                                                );
                                                this.filter_text = text.clone();
                                                let text_len = text.len();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            text.clone(),
                                                            _window,
                                                            input_cx,
                                                        );
                                                        state.set_selection(
                                                            text_len, text_len, _window, input_cx,
                                                        );
                                                    },
                                                );
                                                this.queue_filter_compute(text, cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                                return;
                                            } else {
                                                // Past newest - clear to empty
                                                this.input_history.reset_navigation();
                                                this.filter_text.clear();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            String::new(),
                                                            _window,
                                                            input_cx,
                                                        );
                                                        state
                                                            .set_selection(0, 0, _window, input_cx);
                                                    },
                                                );
                                                this.queue_filter_compute(String::new(), cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                                return;
                                            }
                                        }
                                        // Normal down navigation - use move_selection_down to skip section headers
                                        this.move_selection_down(cx);
                                    }
                                    cx.stop_propagation();
                                }
                                _ => {
                                    // Don't intercept arrows for other views (let normal handling work)
                                }
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(arrow_interceptor);

        // Add Home/End/PageUp/PageDown key interceptor for jump navigation
        let app_entity_for_home_end = cx.entity().downgrade();
        let home_end_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_home_end;
            move |event, window, cx| {
                // Skip processing if this keystroke is from Notes or AI window
                if crate::notes::is_notes_window(window) || crate::ai::is_ai_window(window) {
                    return;
                }

                let key = event.keystroke.key.to_lowercase();
                let has_platform_mod = event.keystroke.modifiers.platform; // Cmd on macOS

                // Home key or Cmd+Up → jump to first item
                // End key or Cmd+Down → jump to last item
                let is_home =
                    key == "home" || (has_platform_mod && (key == "up" || key == "arrowup"));
                let is_end =
                    key == "end" || (has_platform_mod && (key == "down" || key == "arrowdown"));
                // Page Up/Down → move by ~10 selectable items
                let is_page_up = key == "pageup";
                let is_page_down = key == "pagedown";

                if !is_home && !is_end && !is_page_up && !is_page_down {
                    return;
                }

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Only handle in ScriptList view
                        if !matches!(this.current_view, AppView::ScriptList) {
                            return;
                        }

                        // Don't handle if actions popup is open
                        if this.show_actions_popup {
                            return;
                        }

                        if is_home {
                            this.move_selection_to_first(cx);
                        } else if is_end {
                            this.move_selection_to_last(cx);
                        } else if is_page_up {
                            this.move_selection_page_up(cx);
                        } else if is_page_down {
                            this.move_selection_page_down(cx);
                        }

                        cx.stop_propagation();
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(home_end_interceptor);

