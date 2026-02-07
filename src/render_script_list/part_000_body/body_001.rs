        // Log panel - uses pre-extracted theme values to avoid borrow conflicts
        let log_panel = if self.show_logs {
            let logs = logging::get_last_logs(10);
            let mut log_container = div()
                .flex()
                .flex_col()
                .w_full()
                .bg(rgb(log_panel_bg))
                .border_t_1()
                .border_color(rgb(log_panel_border))
                .p(px(design_spacing.padding_md))
                .max_h(px(LOG_PANEL_MAX_HEIGHT))
                .font_family(FONT_MONO);

            for log_line in logs.iter().rev() {
                log_container = log_container.child(
                    div()
                        .text_color(rgb(log_panel_success))
                        .text_xs()
                        .child(log_line.clone()),
                );
            }
            Some(log_container)
        } else {
            None
        };

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

                // Global shortcuts (Cmd+W only - ScriptList has special ESC handling below)
                if this.handle_global_shortcut_with_options(event, false, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check SDK action shortcuts FIRST (before built-in shortcuts)
                // This allows scripts to override default shortcuts via setActions()
                if !this.action_shortcuts.is_empty() {
                    let key_combo =
                        shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                    if let Some(action_name) = this.action_shortcuts.get(&key_combo).cloned() {
                        logging::log(
                            "ACTIONS",
                            &format!(
                                "SDK action shortcut matched: '{}' -> '{}'",
                                key_combo, action_name
                            ),
                        );
                        if this.trigger_action_by_name(&action_name, cx) {
                            return;
                        }
                    }
                }

                if has_cmd {
                    let has_shift = event.keystroke.modifiers.shift;

                    match key_str.as_str() {
                        "l" => {
                            logging::log("KEY", "Shortcut Cmd+L -> toggle_logs");
                            this.toggle_logs(cx);
                            return;
                        }
                        // Cmd+1 cycles through all designs
                        "1" => {
                            logging::log("KEY", "Shortcut Cmd+1 -> cycle_design");
                            this.cycle_design(cx);
                            return;
                        }
                        // Script context shortcuts (require a selected script)
                        // Note: More specific patterns (with shift) must come BEFORE less specific ones
                        "k" if has_shift => {
                            // Cmd+Shift+K - Add/Update Keyboard Shortcut
                            logging::log("KEY", "Shortcut Cmd+Shift+K -> add_shortcut");
                            this.handle_action("add_shortcut".to_string(), cx);
                            return;
                        }
                        "k" => {
                            // Cmd+K - Toggle actions menu
                            if this.has_actions() {
                                logging::log("KEY", "Shortcut Cmd+K -> toggle_actions");
                                this.toggle_actions(cx, window);
                            }
                            return;
                        }
                        "e" => {
                            // Cmd+E - Edit Script
                            logging::log("KEY", "Shortcut Cmd+E -> edit_script");
                            this.handle_action("edit_script".to_string(), cx);
                            return;
                        }
                        "f" if has_shift => {
                            // Cmd+Shift+F - Reveal in Finder
                            logging::log("KEY", "Shortcut Cmd+Shift+F -> reveal_in_finder");
                            this.handle_action("reveal_in_finder".to_string(), cx);
                            return;
                        }
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            logging::log("KEY", "Shortcut Cmd+Shift+C -> copy_path");
                            this.handle_action("copy_path".to_string(), cx);
                            return;
                        }
                        "d" if has_shift => {
                            // Cmd+Shift+D - Copy Deeplink
                            logging::log("KEY", "Shortcut Cmd+Shift+D -> copy_deeplink");
                            this.handle_action("copy_deeplink".to_string(), cx);
                            return;
                        }
                        "a" if has_shift => {
                            // Cmd+Shift+A - Add/Update Alias
                            logging::log("KEY", "Shortcut Cmd+Shift+A -> add_alias");
                            this.handle_action("add_alias".to_string(), cx);
                            return;
                        }
                        // Global shortcuts
                        "n" => {
                            // Cmd+N - Create Script
                            logging::log("KEY", "Shortcut Cmd+N -> create_script");
                            this.handle_action("create_script".to_string(), cx);
                            return;
                        }
                        "r" => {
                            // Cmd+R - Reload Scripts
                            logging::log("KEY", "Shortcut Cmd+R -> reload_scripts");
                            this.handle_action("reload_scripts".to_string(), cx);
                            return;
                        }
                        "," => {
                            // Cmd+, - Settings
                            logging::log("KEY", "Shortcut Cmd+, -> settings");
                            this.handle_action("settings".to_string(), cx);
                            return;
                        }
                        "q" => {
                            // Cmd+Q - Quit
                            logging::log("KEY", "Shortcut Cmd+Q -> quit");
                            this.handle_action("quit".to_string(), cx);
                            return;
                        }
                        _ => {}
                    }
                }

                // If confirm dialog is open, just return - key routing is handled by
                // the dedicated interceptors in app_impl.rs (Tab at line 462-478,
                // arrows at line 645-659, all others at line 920-928)
                // We must NOT dispatch here or it will double-fire toggle_focus!
                if crate::confirm::is_confirm_window_open() {
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                // Notify actions window to re-render
                                cx.spawn(async move |_this, cx| {
                                    cx.update(notify_actions_window).ok();
                                })
                                .detach();
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                // Notify actions window to re-render
                                cx.spawn(async move |_this, cx| {
                                    cx.update(notify_actions_window).ok();
                                })
                                .detach();
                                return;
                            }
                            "enter" | "return" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    // Only close if action has close: true (default)
                                    if should_close {
                                        this.close_actions_popup(
                                            ActionsDialogHost::MainList,
                                            window,
                                            cx,
                                        );
                                    }
                                    this.handle_action(action_id, cx);
                                }
                                // Notify to update UI state after closing popup
                                cx.notify();
                                return;
                            }
                            "escape" | "esc" => {
                                this.close_actions_popup(ActionsDialogHost::MainList, window, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                // Resize and notify actions window to re-render
                                let dialog_for_resize = dialog.clone();
                                cx.spawn(async move |_this, cx| {
                                    cx.update(|cx| {
                                        resize_actions_window(cx, &dialog_for_resize);
                                    })
                                    .ok();
                                })
                                .detach();
                                return;
                            }
                            _ => {
                                let modifiers = &event.keystroke.modifiers;

                                // Check for printable character input (only when no modifiers are held)
                                // This prevents Cmd+E from being treated as typing 'e' into the search
                                if !modifiers.platform && !modifiers.control && !modifiers.alt {
                                    if let Some(ref key_char) = event.keystroke.key_char {
                                        if let Some(ch) = key_char.chars().next() {
                                            if !ch.is_control() {
                                                dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                                // Resize and notify actions window to re-render
                                                let dialog_for_resize = dialog.clone();
                                                cx.spawn(async move |_this, cx| {
                                                    cx.update(|cx| {
                                                        resize_actions_window(
                                                            cx,
                                                            &dialog_for_resize,
                                                        );
                                                    })
                                                    .ok();
                                                })
                                                .detach();
                                                return;
                                            }
                                        }
                                    }
                                }

                                // Check if keystroke matches any action shortcut in the dialog
                                // This allows Cmd+E, Cmd+L, etc. to execute the corresponding action
                                let key_lower = key_str.to_lowercase();
                                let keystroke_shortcut =
                                    shortcuts::keystroke_to_shortcut(&key_lower, modifiers);

                                // Read dialog actions and look for matching shortcut
                                let dialog_ref = dialog.read(cx);
                                let mut matched_action: Option<String> = None;
                                for action in &dialog_ref.actions {
                                    if let Some(ref display_shortcut) = action.shortcut {
                                        let normalized =
                                            Self::normalize_display_shortcut(display_shortcut);
                                        if normalized == keystroke_shortcut {
                                            matched_action = Some(action.id.clone());
                                            break;
                                        }
                                    }
                                }
                                let _ = dialog_ref;

                                if let Some(action_id) = matched_action {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Actions dialog shortcut matched: {} -> {}",
                                            keystroke_shortcut, action_id
                                        ),
                                    );
                                    // Close the dialog using centralized helper
                                    this.close_actions_popup(
                                        ActionsDialogHost::MainList,
                                        window,
                                        cx,
                                    );
                                    // Execute the action
                                    this.handle_action(action_id, cx);
                                    cx.notify();
                                }
                                return;
                            }
                        }
                    }
                }

                // LEGACY: Check if we're in fallback mode (no script matches, showing fallback commands)
                // Note: This is legacy code that handled a separate fallback rendering path.
                // Now fallbacks flow through GroupedListItem from grouping.rs, so this
                // branch should rarely (if ever) be triggered. The normal navigation below
                // handles fallback items in the unified list.
                if this.fallback_mode && !this.cached_fallbacks.is_empty() {
                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if this.fallback_selected_index > 0 {
                                this.fallback_selected_index -= 1;
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if this.fallback_selected_index
                                < this.cached_fallbacks.len().saturating_sub(1)
                            {
                                this.fallback_selected_index += 1;
                                cx.notify();
                            }
                        }
                        "enter" => {
                            if !this.gpui_input_focused {
                                this.execute_selected_fallback(cx);
                            }
                        }
                        "escape" => {
                            // Clear filter to exit fallback mode
                            this.clear_filter(window, cx);
                        }
                        _ => {}
                    }
                    return;
                }

                // Normal script list navigation
                // NOTE: Arrow keys are now handled by the arrow_interceptor in app_impl.rs
                // which fires before the Input component can consume them. This allows
                // input history navigation + list navigation to work correctly.
                match key_str.as_str() {
                    "enter" => {
                        if !this.gpui_input_focused {
                            this.execute_selected(cx);
                        }
                    }
                    "escape" => {
                        // Clear filter first if there's text, otherwise close window
                        if !this.filter_text.is_empty() {
                            this.clear_filter(window, cx);
                        } else {
                            // Filter is empty - close window
                            this.close_and_reset_window(cx);
                        }
                    }
                    // Tab key: Send query to AI chat if filter has text
                    // Note: This is a fallback - primary Tab handling is in app_impl.rs via intercept_keystrokes
                    "tab" | "Tab" => {
                        if !this.filter_text.is_empty() {
                            let query = this.filter_text.clone();

                            // Open AI window first
                            if let Err(e) = ai::open_ai_window(cx) {
                                logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            } else {
                                // Set input in AI chat (don't auto-submit - let user review first)
                                ai::set_ai_input(cx, &query, false);
                            }

                            // Clear filter and close main window
                            this.clear_filter(window, cx);
                            this.close_and_reset_window(cx);
                        }
                    }
                    _ => {}
                }
            },
        );

        // Main container with system font and transparency
        // NOTE: Shadow disabled for vibrancy - shadows on transparent elements cause gray fill

        // Use unified color resolver for text and fonts
        let text_primary = color_resolver.primary_text_color();
        let font_family = typography_resolver.primary_font();

        // Extract footer colors BEFORE render_preview_panel (borrow checker)
        let footer_accent = color_resolver.primary_accent();
        let footer_text_muted = color_resolver.empty_text_color();
        let footer_border = color_resolver.border_color();
        let footer_background = color_resolver.selection_background();

        // NOTE: No .bg() here - Root provides vibrancy background for ALL content
        // This ensures main menu, AI chat, and all prompts have consistent styling

