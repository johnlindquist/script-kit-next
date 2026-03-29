        // Add Tab key interceptor for "Ask AI" feature and file search directory navigation
        // This fires BEFORE normal key handling, allowing us to intercept Tab
        // even when the Input component has focus
        let app_entity_for_tab = cx.entity().downgrade();
        let tab_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_tab;
            move |event, window, cx| {
                // When the main window is hidden (e.g. Notes/AI open), main-menu
                // key interceptors must not consume keystrokes from secondary windows.
                if !script_kit_gpui::is_main_window_visible() {
                    return;
                }

                // Skip keystrokes from secondary windows
                if crate::actions::is_actions_window(window) {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let is_tab_key = key.eq_ignore_ascii_case("tab");
                let has_shift = event.keystroke.modifiers.shift;

                // Check for Tab key (no cmd/alt/ctrl modifiers, but shift is allowed)
                if is_tab_key
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Handle Tab/Shift+Tab in FileSearchView for directory/file navigation
                            // CRITICAL: ALWAYS consume Tab/Shift+Tab to prevent focus traversal
                            if let AppView::FileSearchView {
                                query,
                                selected_index,
                            } = &mut this.current_view
                            {
                                // ALWAYS stop propagation for Tab/Shift+Tab in FileSearchView
                                // This prevents Tab from falling through to focus traversal
                                cx.stop_propagation();

                                if has_shift {
                                    // Shift+Tab: Go up one directory level using parent_dir_display helper
                                    // This handles ~/, /, ./, ../ and regular paths correctly
                                    if let Some(parent_path) =
                                        crate::file_search::parent_dir_display(query)
                                    {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Navigating up from '{}' to '{}'",
                                                query, parent_path
                                            ),
                                        );

                                        // Update the input - handle_filter_input_change will:
                                        // - Update query
                                        // - Reset selected_index to 0
                                        // - Detect directory change
                                        // - Trigger async directory load
                                        this.gpui_input_state.update(cx, |state, cx| {
                                            state.set_value(parent_path.clone(), window, cx);
                                            // Ensure cursor is at end with no selection after programmatic set_value
                                            // This prevents issues where GPUI might leave caret at wrong position
                                            let len = parent_path.len();
                                            state.set_selection(len, len, window, cx);
                                        });

                                        cx.notify();
                                    } else {
                                        // At root (/ or ~/) - no parent to navigate to
                                        // Key is consumed (stop_propagation called above) but no action taken
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Already at root '{}', no-op",
                                                query
                                            ),
                                        );
                                    }
                                } else {
                                    // Tab: Enter directory OR autocomplete file name
                                    // Get filtered results to find selected item
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
                                        if file.file_type == crate::file_search::FileType::Directory
                                        {
                                            // Directory: Enter it (append /)
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            let new_path = format!("{}/", shortened);
                                            crate::logging::log(
                                                "KEY",
                                                &format!("Tab: Entering directory: {}", new_path),
                                            );

                                            // Update the input - handle_filter_input_change handles the rest
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(new_path.clone(), window, cx);
                                                // Ensure cursor is at end with no selection after programmatic set_value
                                                let len = new_path.len();
                                                state.set_selection(len, len, window, cx);
                                            });

                                            cx.notify();
                                        } else {
                                            // File: Autocomplete the full path (terminal-style tab completion)
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            crate::logging::log(
                                                "KEY",
                                                &format!(
                                                    "Tab: Autocompleting file path: {}",
                                                    shortened
                                                ),
                                            );

                                            // Set the input to the file's full path
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(shortened.clone(), window, cx);
                                                // Ensure cursor is at end with no selection after programmatic set_value
                                                let len = shortened.len();
                                                state.set_selection(len, len, window, cx);
                                            });

                                            cx.notify();
                                        }
                                    } else {
                                        // No selection - just consume the key
                                        crate::logging::log(
                                            "KEY",
                                            "Tab: No selection to autocomplete, no-op",
                                        );
                                    }
                                }
                                return;
                            }

                            // Handle Tab/Shift+Tab in ChatPrompt setup mode
                            // Must intercept here to prevent GPUI focus traversal from consuming Tab
                            if let AppView::ChatPrompt { entity, .. } = &this.current_view {
                                let handled = entity.update(cx, |chat, cx| {
                                    chat.handle_setup_key("tab", has_shift, cx)
                                });
                                if handled {
                                    cx.stop_propagation();
                                    return;
                                }
                            }

                            // Shift+Tab in ScriptList: route typed query through the
                            // harness terminal so Tab always means "send me to my
                            // running harness with context."
                            if has_shift
                                && matches!(this.current_view, AppView::ScriptList)
                                && !this.filter_text.is_empty()
                                && !this.show_actions_popup
                            {
                                let query = this.filter_text.clone();
                                this.open_tab_ai_chat_with_entry_intent(Some(query), cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Forward Tab/Shift+Tab directly to the harness
                            // terminal PTY.  We must NOT call cx.propagate()
                            // here because GPUI's built-in focus-traversal
                            // would consume the Tab keystroke before it reaches
                            // the TermPrompt key handler.  Instead, write the
                            // raw byte to the PTY and stop propagation.
                            if let AppView::QuickTerminalView { entity, .. } = &this.current_view {
                                entity.update(cx, |term, _cx| {
                                    let running = term.terminal.is_running();
                                    let bytes: &[u8] = if has_shift {
                                        b"\x1b[Z" // Shift+Tab (backtab)
                                    } else {
                                        b"\t" // Tab
                                    };
                                    if !running {
                                        tracing::warn!(
                                            event = "quick_terminal_tab_pty_dead",
                                            has_shift,
                                            "Tab intercepted but PTY is not running"
                                        );
                                        return;
                                    }
                                    match term.terminal.input(bytes) {
                                        Ok(()) => tracing::debug!(
                                            event = "quick_terminal_tab_sent",
                                            has_shift,
                                            "Tab byte written to PTY"
                                        ),
                                        Err(e) => tracing::warn!(
                                            event = "quick_terminal_tab_write_failed",
                                            error = %e,
                                            has_shift,
                                            "Failed to write Tab to PTY"
                                        ),
                                    }
                                });
                                cx.stop_propagation();
                                return;
                            }

                            // Block Tab while the save-offer overlay is visible
                            if this.tab_ai_save_offer_state.is_some() {
                                cx.stop_propagation();
                                return;
                            }

                            // Universal Tab AI: open the harness terminal surface
                            // from any non-special surface (not FileSearch, not ChatPrompt setup)
                            if !has_shift && !this.show_actions_popup {
                                this.open_tab_ai_chat(cx);
                                cx.stop_propagation();
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(tab_interceptor);

        // Prewarm the Tab AI harness asynchronously so the first Tab press
        // reuses a live PTY instead of paying spawn cost.  Runs once, silently.
        let app_entity_for_tab_ai_warm = cx.entity().downgrade();
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;
            let _ = cx.update(|cx| {
                let Some(app) = app_entity_for_tab_ai_warm.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| {
                    this.warm_tab_ai_harness_on_startup(cx);
                });
            });
        })
        .detach();
