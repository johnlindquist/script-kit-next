        // Add interceptor for actions popup in FileSearchView and ScriptList
        // This handles Cmd+K (toggle), Escape (close), Enter (submit), and typing
        let app_entity_for_actions = cx.entity().downgrade();
        let actions_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_actions;
            move |event, window, cx| {
                // CRITICAL: Skip processing if this keystroke is from Notes or AI window
                // intercept_keystrokes is GLOBAL and fires for ALL windows in the app
                // We only want to handle keystrokes for the main window
                if crate::notes::is_notes_window(window) || crate::ai::is_ai_window(window) {
                    return; // Let the secondary window handle its own keystrokes
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;
                let key_char = event.keystroke.key_char.as_deref();

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // FIRST: If confirm dialog is open, route Enter/Escape to it
                        // NOTE: Tab is handled by the dedicated Tab interceptor above, so
                        // we exclude it here to avoid double-dispatching toggle_focus()
                        if !key.eq_ignore_ascii_case("tab")
                            && crate::confirm::is_confirm_window_open()
                            && crate::confirm::dispatch_confirm_key(key, cx)
                        {
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Cmd+K to toggle actions popup (works in ScriptList, FileSearchView, ArgPrompt)
                        // This MUST be intercepted here because the Input component has focus and
                        // normal on_key_down handlers won't receive the event
                        if has_cmd && key.eq_ignore_ascii_case("k") && !has_shift {
                            match &mut this.current_view {
                                AppView::ScriptList => {
                                    // Toggle actions for the main script list
                                    if this.has_actions() {
                                        logging::log(
                                            "KEY",
                                            "Interceptor: Cmd+K -> toggle_actions (ScriptList)",
                                        );
                                        this.toggle_actions(cx, window);
                                    }
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                } => {
                                    // Get the filter pattern for directory path parsing
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    let filtered_results: Vec<_> =
                                        if let Some(ref pattern) = filter_pattern {
                                            crate::file_search::filter_results_nucleo_simple(
                                                &this.cached_file_results,
                                                pattern,
                                            )
                                        } else {
                                            this.cached_file_results.iter().enumerate().collect()
                                        };

                                    // Defensive bounds check: clamp selected_index if out of bounds
                                    let filtered_len = filtered_results.len();
                                    if filtered_len > 0 && *selected_index >= filtered_len {
                                        *selected_index = filtered_len - 1;
                                    }

                                    if let Some((_, file)) = filtered_results.get(*selected_index) {
                                        let file_clone = (*file).clone();
                                        this.toggle_file_search_actions(&file_clone, window, cx);
                                    }
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::ArgPrompt { .. } => {
                                    // Toggle actions for arg prompts (SDK setActions)
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_arg_actions (ArgPrompt)");
                                    this.toggle_arg_actions(cx, window);
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::ChatPrompt { .. } => {
                                    // Toggle actions for chat prompts
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_chat_actions (ChatPrompt)");
                                    this.toggle_chat_actions(cx, window);
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::WebcamView { .. } => {
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_webcam_actions (WebcamView)");
                                    this.toggle_webcam_actions(cx, window);
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::ClipboardHistoryView { .. } => {
                                    // Toggle actions for selected clipboard entry
                                    if let Some(entry) = this.selected_clipboard_entry() {
                                        logging::log(
                                            "KEY",
                                            "Interceptor: Cmd+K -> toggle_clipboard_actions (ClipboardHistoryView)",
                                        );
                                        this.toggle_clipboard_actions(entry, window, cx);
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                                _ => {
                                    // Other views don't support Cmd+K actions
                                }
                            }
                        }

                        // Handle Cmd+Shift+K for add_shortcut in ScriptList
                        if has_cmd && key.eq_ignore_ascii_case("k") && has_shift
                            && matches!(this.current_view, AppView::ScriptList)
                        {
                            logging::log("KEY", "Interceptor: Cmd+Shift+K -> add_shortcut (ScriptList)");
                            this.handle_action("add_shortcut".to_string(), cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Window tweaker shortcuts (only enabled with SCRIPT_KIT_WINDOW_TWEAKER=1)
                        let window_tweaker_enabled = std::env::var("SCRIPT_KIT_WINDOW_TWEAKER")
                            .map(|v| v == "1")
                            .unwrap_or(false);

                        if window_tweaker_enabled {
                            // Handle Cmd+- to decrease light theme opacity
                            if has_cmd
                                && !has_shift
                                && (key == "-" || key.eq_ignore_ascii_case("minus"))
                            {
                                logging::log("KEY", &format!("Interceptor: Cmd+- (key={}) -> decrease light opacity", key));
                                this.adjust_light_opacity(-0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+= (or Cmd+Shift+=) to increase light theme opacity
                            if has_cmd
                                && (key == "="
                                    || key.eq_ignore_ascii_case("equal")
                                    || key.eq_ignore_ascii_case("plus"))
                            {
                                logging::log("KEY", &format!("Interceptor: Cmd+= (key={}) -> increase light opacity", key));
                                this.adjust_light_opacity(0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+M to cycle vibrancy material (blur effect)
                            if has_cmd && !has_shift && key.eq_ignore_ascii_case("m") {
                                logging::log("KEY", "Interceptor: Cmd+M -> cycle vibrancy material");
                                let description = platform::cycle_vibrancy_material();
                                this.toast_manager.push(components::toast::Toast::info(
                                    description,
                                    &this.theme,
                                ));
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+Shift+A to cycle vibrancy appearance (VibrantLight, VibrantDark, etc.)
                            if has_cmd && has_shift && key.eq_ignore_ascii_case("a") {
                                logging::log("KEY", "Interceptor: Cmd+Shift+A -> cycle vibrancy appearance");
                                let description = platform::cycle_appearance();
                                this.toast_manager.push(components::toast::Toast::info(
                                    description,
                                    &this.theme,
                                ));
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }
                        }

                        // Only handle remaining keys if in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            // Arrow keys are handled by arrow_interceptor to avoid double-processing
                            // (which can skip 2 items per keypress when both interceptors handle arrows).
                            if crate::ui_foundation::is_key_up(key)
                                || crate::ui_foundation::is_key_down(key)
                            {
                                return;
                            }

                            // Route modal actions keys for all views that support actions dialogs.
                            // This ensures enter, escape, backspace, and character keys are
                            // routed to the actions dialog when it's open, regardless of view type.
                            let host = match &this.current_view {
                                AppView::ScriptList => Some(ActionsDialogHost::MainList),
                                AppView::ClipboardHistoryView { .. } => Some(ActionsDialogHost::ClipboardHistory),
                                AppView::ChatPrompt { .. } => Some(ActionsDialogHost::ChatPrompt),
                                AppView::ArgPrompt { .. } => Some(ActionsDialogHost::ArgPrompt),
                                AppView::DivPrompt { .. } => Some(ActionsDialogHost::DivPrompt),
                                AppView::EditorPrompt { .. } => Some(ActionsDialogHost::EditorPrompt),
                                AppView::TermPrompt { .. } => Some(ActionsDialogHost::TermPrompt),
                                AppView::FormPrompt { .. } => Some(ActionsDialogHost::FormPrompt),
                                AppView::WebcamView { .. } => Some(ActionsDialogHost::WebcamPrompt),
                                _ => None,
                            };

                            if let Some(host) = host {
                                match this.route_key_to_actions_dialog(
                                    key,
                                    key_char,
                                    &event.keystroke.modifiers,
                                    host,
                                    window,
                                    cx,
                                ) {
                                    ActionsRoute::NotHandled => {}
                                    ActionsRoute::Handled => {
                                        cx.stop_propagation();
                                        return;
                                    }
                                    ActionsRoute::Execute { action_id } => {
                                        match host {
                                            ActionsDialogHost::ChatPrompt => {
                                                this.execute_chat_action(&action_id, cx);
                                            }
                                            ActionsDialogHost::ArgPrompt => {
                                                this.trigger_action_by_name(&action_id, cx);
                                            }
                                            ActionsDialogHost::WebcamPrompt => {
                                                this.execute_webcam_action(&action_id, cx);
                                            }
                                            _ => {
                                                this.handle_action(action_id, cx);
                                            }
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                            }
                            return;
                        }

                        // Only handle remaining keys if actions popup is open (FileSearchView)
                        if !this.show_actions_popup {
                            return;
                        }

                        // Handle Escape to close actions popup
                        if crate::ui_foundation::is_key_escape(key) {
                            this.close_actions_popup(ActionsDialogHost::FileSearch, window, cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Enter to submit selected action
                        if crate::ui_foundation::is_key_enter(key) {
                            if let Some(ref dialog) = this.actions_dialog {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();

                                if let Some(action_id) = action_id {
                                    crate::logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "FileSearch actions executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );

                                    if should_close {
                                        this.close_actions_popup(
                                            ActionsDialogHost::FileSearch,
                                            window,
                                            cx,
                                        );
                                    }

                                    // Use handle_action instead of trigger_action_by_name
                                    // handle_action supports both built-in actions (open_file, quick_look, etc.)
                                    // and SDK actions
                                    this.handle_action(action_id, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Backspace for actions search
                        if key.eq_ignore_ascii_case("backspace") {
                            if let Some(ref dialog) = this.actions_dialog {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                crate::actions::notify_actions_window(cx);
                                crate::actions::resize_actions_window(cx, dialog);
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle printable character input for actions search
                        if let Some(chars) = key_char {
                            if let Some(ch) = chars.chars().next() {
                                if ch.is_ascii_graphic() || ch == ' ' {
                                    if let Some(ref dialog) = this.actions_dialog {
                                        dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        crate::actions::notify_actions_window(cx);
                                        crate::actions::resize_actions_window(cx, dialog);
                                    }
                                    cx.stop_propagation();
                                }
                            }
                        }
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(actions_interceptor);

        // CRITICAL FIX: Sync list state on initialization
        // This was removed when state mutations were moved out of render(),
        // but we still need to sync once during initialization so the list
        // knows about the scripts that were loaded.
        // Without this, the first render shows "No scripts or snippets found"
        // because main_list_state starts with 0 items.
        app.sync_list_state();
        app.validate_selection_bounds(cx);

        app
