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

                if confirm::consume_main_window_key_while_confirm_open(
                    key,
                    &event.keystroke.modifiers,
                    cx,
                ) {
                    cx.stop_propagation();
                    return;
                }

                // Check for Tab key (no cmd/alt/ctrl modifiers, but shift is allowed)
                if is_tab_key
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // File search: plain Tab captures focused context into ACP,
                            // Shift+Tab still navigates up one directory level.
                            if matches!(this.current_view, AppView::FileSearchView { .. }) {
                                cx.stop_propagation();

                                if this.show_actions_popup {
                                    return;
                                }

                                if has_shift {
                                    let current_query = match &this.current_view {
                                        AppView::FileSearchView { query, .. } => query.clone(),
                                        _ => String::new(),
                                    };

                                    let next_path = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(&current_query)
                                    {
                                        if parsed.filter.is_some() {
                                            Some(parsed.directory)
                                        } else {
                                            crate::file_search::parent_dir_display(&parsed.directory)
                                        }
                                    } else {
                                        None
                                    };

                                    if let Some(parent_path) = next_path {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Navigating up from '{}' to '{}'",
                                                current_query, parent_path
                                            ),
                                        );
                                        this.gpui_input_state.update(cx, |state, cx| {
                                            state.set_value(parent_path.clone(), window, cx);
                                            let len = parent_path.len();
                                            state.set_selection(len, len, window, cx);
                                        });
                                        cx.notify();
                                    } else {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Already at root '{}', no-op",
                                                current_query
                                            ),
                                        );
                                    }
                                } else {
                                    let _ = this.try_route_plain_tab_to_acp_context_capture(cx);
                                }
                                return;
                            }

                            // ChatPrompt: plain Tab opens ACP with focused/input context,
                            // Shift+Tab keeps local setup back-tab behavior.
                            if matches!(this.current_view, AppView::ChatPrompt { .. }) {
                                if has_shift {
                                    if let AppView::ChatPrompt { entity, .. } = &this.current_view {
                                        let handled = entity.update(cx, |chat, cx| {
                                            chat.handle_setup_key("tab", true, cx)
                                        });
                                        if handled {
                                            cx.stop_propagation();
                                            return;
                                        }
                                    }
                                } else if this.try_route_plain_tab_to_acp_context_capture(cx) {
                                    cx.stop_propagation();
                                    return;
                                }
                            }

                            // Shift+Tab in ScriptList: route typed query through the
                            // quick-submit planner so the harness gets intelligent
                            // classification, synthesized intent, and the right
                            // capture kind — not just a raw string paste.
                            if has_shift
                                && matches!(this.current_view, AppView::ScriptList)
                                && !this.filter_text.is_empty()
                                && !this.show_actions_popup
                            {
                                let query = this.filter_text.clone();
                                this.submit_to_current_or_new_tab_ai_harness_from_text(
                                    query,
                                    crate::ai::TabAiQuickSubmitSource::ShiftTab,
                                    cx,
                                );
                                cx.stop_propagation();
                                return;
                            }

                            // Consume Tab/Shift+Tab while the ACP chat is
                            // open so the universal Tab AI fallback does not
                            // spawn a second ACP session on top of this one.
                            if let AppView::AcpChatView { entity, .. } = &this.current_view {
                                let handled = entity.update(cx, |chat, cx| {
                                    chat.handle_tab_key(has_shift, cx)
                                });
                                if handled {
                                    cx.stop_propagation();
                                    return;
                                }
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

                            // Universal Tab AI: route any remaining non-AI surface into ACP
                            if !has_shift && this.try_route_plain_tab_to_acp_context_capture(cx) {
                                cx.stop_propagation();
                                return;
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(tab_interceptor);

        // Prewarm the ACP agent config on a background thread so Tab presses
        // never block on bun transpile of ~/.scriptkit/kit/config.ts.
        crate::ai::acp::prewarm_agent_config();

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
