// External command listener - receives commands via stdin (event-driven, no polling)
let stdin_rx = start_stdin_listener();
let window_for_stdin = window;
let app_entity_for_stdin = app_entity.clone();

// Track if we've received any stdin commands (for timeout warning)
static STDIN_RECEIVED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// Spawn a timeout warning task - helps AI agents detect when they forgot to use stdin protocol
cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
    Timer::after(std::time::Duration::from_secs(2)).await;
    if !STDIN_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
        logging::log("STDIN", "");
        logging::log(
            "STDIN",
            "╔════════════════════════════════════════════════════════════════════════════╗",
        );
        logging::log(
            "STDIN",
            "║  WARNING: No stdin JSON received after 2 seconds                          ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  If you're testing, use the stdin JSON protocol:                          ║",
        );
        logging::log(
            "STDIN",
            "║  echo '{\"type\":\"run\",\"path\":\"...\"}' | ./target/debug/script-kit-gpui     ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  Command line args do NOT work:                                           ║",
        );
        logging::log(
            "STDIN",
            "║  ./target/debug/script-kit-gpui test.ts  # WRONG - does nothing!          ║",
        );
        logging::log(
            "STDIN",
            "╚════════════════════════════════════════════════════════════════════════════╝",
        );
        logging::log("STDIN", "");
    }
})
.detach();

cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    logging::log("STDIN", "Async stdin command handler started");

    // Event-driven: recv().await yields until a command arrives
    while let Ok(ExternalCommandEnvelope {
        command: cmd,
        correlation_id,
    }) = stdin_rx.recv().await
    {
        let _guard = logging::set_correlation_id(correlation_id);
        // Mark that we've received stdin (clears the timeout warning)
        STDIN_RECEIVED.store(true, std::sync::atomic::Ordering::SeqCst);
        logging::log(
            "STDIN",
            &format!("Processing external command type={}", cmd.command_type()),
        );

        let app_entity_inner = app_entity_for_stdin.clone();
        let _ = cx.update(|cx| {
            // Use the Root window to get Window reference, then update the app entity
            let _ = window_for_stdin.update(cx, |_root, window, root_cx| {
                app_entity_inner.update(root_cx, |view, ctx| {
                    // Note: We have both `window` from Root and `view` from entity here
                    // ctx is Context<ScriptListApp>, window is &mut Window
                    match cmd {
                            ExternalCommand::Run { ref path, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Executing script: {}", rid, path));

                                // NOTE: This is a simplified show path for script execution.
                                // We show the window, then immediately run the script.
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown(); // Focus grace period
                                platform::ensure_move_to_active_space();

                                // Use Window::defer via window_ops to coalesce and defer window move.
                                // This avoids RefCell borrow conflicts from synchronous macOS window operations.
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                window_ops::queue_move(bounds, window, ctx);

                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    platform::swizzle_gpui_blurred_view();
                                    // Configure vibrancy based on actual theme colors
                                    let theme = theme::load_theme();
                                    let is_dark = theme.should_use_dark_vibrancy();
                                    platform::configure_window_vibrancy_material_for_appearance(is_dark);
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                // Show window WITHOUT activating (floating panel behavior)
                                platform::show_main_window_without_activation();
                                window.activate_window();

                                // Send AI window to back so it doesn't come forward with main menu
                                platform::send_ai_window_to_back();

                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);

                                // Send RunScript message to be handled
                                view.handle_prompt_message(PromptMessage::RunScript { path: path.clone() }, ctx);
                            }
                            ExternalCommand::Show { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Showing window", rid));

                                // NOTE: This is a simplified show path for explicit stdin commands.
                                // Unlike the hotkey handler, we don't need NEEDS_RESET handling
                                // because this is an explicit show (not a toggle).
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown(); // Focus grace period
                                platform::ensure_move_to_active_space();

                                // Position window - try per-display saved position first, then fall back to eye-line
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let displays = platform::get_macos_displays();
                                let bounds = if let Some((mouse_x, mouse_y)) = platform::get_global_mouse_position() {
                                    // Try to restore saved position for the mouse display
                                    if let Some((saved, display)) =
                                        window_state::get_main_position_for_mouse_display(mouse_x, mouse_y, &displays)
                                    {
                                        // Validate the saved position is still visible
                                        if window_state::is_bounds_visible(&saved, &displays) {
                                            logging::log(
                                                "STDIN",
                                                &format!(
                                                    "Restoring saved position for display {}: ({:.0}, {:.0})",
                                                    window_state::display_key(&display),
                                                    saved.x,
                                                    saved.y
                                                ),
                                            );
                                            // Use saved position but with current window height (may have changed)
                                            gpui::Bounds {
                                                origin: gpui::point(px(saved.x as f32), px(saved.y as f32)),
                                                size: window_size,
                                            }
                                        } else {
                                            logging::log("STDIN", "Saved position no longer visible, using eye-line");
                                            platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                        }
                                    } else {
                                        logging::log("STDIN", "No saved position for this display, using eye-line");
                                        platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                    }
                                } else {
                                    logging::log("STDIN", "Could not get mouse position, using eye-line");
                                    platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                };
                                window_ops::queue_move(bounds, window, ctx);

                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    platform::swizzle_gpui_blurred_view();
                                    // Configure vibrancy based on actual theme colors
                                    let theme = theme::load_theme();
                                    let is_dark = theme.should_use_dark_vibrancy();
                                    platform::configure_window_vibrancy_material_for_appearance(is_dark);
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                // Show window WITHOUT activating (floating panel behavior)
                                platform::show_main_window_without_activation();
                                window.activate_window();

                                // Send AI window to back so it doesn't come forward with main menu
                                platform::send_ai_window_to_back();

                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);
                            }
                            ExternalCommand::Hide { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Hiding main window", rid));

                                // Save window position for the current display BEFORE hiding
                                if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                    let displays = platform::get_macos_displays();
                                    let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                    if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                        logging::log(
                                            "STDIN",
                                            &format!(
                                                "Saving position for display {}: ({:.0}, {:.0})",
                                                window_state::display_key(display),
                                                x,
                                                y
                                            ),
                                        );
                                        window_state::save_main_position_for_display(display, bounds);
                                    }
                                }

                                script_kit_gpui::set_main_window_visible(false);

                                // Check if Notes or AI windows are open
                                let notes_open = notes::is_notes_window_open();
                                let ai_open = ai::is_ai_window_open();

                                // CRITICAL: Only hide main window if Notes/AI are open
                                // ctx.hide() hides the ENTIRE app (all windows)
                                if notes_open || ai_open {
                                    logging::log("STDIN", "Using hide_main_window() - secondary windows are open");
                                    platform::hide_main_window();
                                } else {
                                    ctx.hide();
                                }
                            }
                            ExternalCommand::SetFilter { ref text, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Setting filter to: '{}'", rid, text));
                                view.set_filter_text_immediate(text.clone(), window, ctx);
                                let _ = view.get_filtered_results_cached(); // Update cache
                            }
                            ExternalCommand::TriggerBuiltin { ref name } => {
                                logging::log("STDIN", &format!("Triggering built-in: '{}'", name));
                                // Opened via protocol command - ESC should close window (not return to main menu)
                                view.opened_from_main_menu = false;
                                // Match built-in name and trigger the corresponding feature
                                match name.to_lowercase().as_str() {
                                    "design-gallery" | "designgallery" | "design gallery" => {
                                        view.current_view = AppView::DesignGalleryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    // P0 FIX: Store data in self, view holds only state
                                    "clipboard" | "clipboard-history" | "clipboardhistory" => {
                                        view.cached_clipboard_entries =
                                            clipboard_history::get_cached_entries(100);
                                        view.focused_clipboard_entry_id = view
                                            .cached_clipboard_entries
                                            .first()
                                            .map(|entry| entry.id.clone());
                                        view.current_view = AppView::ClipboardHistoryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    // P0 FIX: Use existing self.apps, view holds only state
                                    "apps" | "app-launcher" | "applauncher" => {
                                        view.current_view = AppView::AppLauncherView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    "file-search" | "filesearch" | "files" | "searchfiles" => {
                                        view.open_file_search(String::new(), ctx);
                                    }
                                    _ => {
                                        logging::log("ERROR", &format!("Unknown built-in: '{}'", name));
                                    }
                                }
                            }

                            ExternalCommand::SimulateKey { ref key, ref modifiers } => {
                                logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

                                // Parse modifiers
                                let has_cmd = modifiers.contains(&KeyModifier::Cmd);
                                let has_shift = modifiers.contains(&KeyModifier::Shift);
                                let _has_alt = modifiers.contains(&KeyModifier::Alt);
                                let _has_ctrl = modifiers.contains(&KeyModifier::Ctrl);

                                // Handle key based on current view
                                let key_lower = key.to_lowercase();

                                match &view.current_view {
                                    AppView::ScriptList => {
                                        // Main script list key handling
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle actions");
                                            view.toggle_actions(ctx, window);
                                        } else if view.fallback_mode && !view.cached_fallbacks.is_empty() {
                                            // Handle keys in fallback mode
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    // Tab with filter text opens inline AI chat (even in fallback mode)
                                                    if !view.filter_text.is_empty() && !view.show_actions_popup {
                                                        let query = view.filter_text.clone();
                                                        view.filter_text.clear();
                                                        view.show_inline_ai_chat(Some(query), ctx);
                                                    }
                                                }
                                                "up" | "arrowup" => {
                                                    if view.fallback_selected_index > 0 {
                                                        view.fallback_selected_index -= 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    if view.fallback_selected_index < view.cached_fallbacks.len().saturating_sub(1) {
                                                        view.fallback_selected_index += 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute fallback");
                                                    view.execute_selected_fallback(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter (exit fallback mode)");
                                                    view.clear_filter(window, ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in fallback mode", key_lower));
                                                }
                                            }
                                        } else {
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    // Tab with filter text opens inline AI chat
                                                    if !view.filter_text.is_empty() && !view.show_actions_popup {
                                                        let query = view.filter_text.clone();
                                                        view.filter_text.clear();
                                                        view.show_inline_ai_chat(Some(query), ctx);
                                                    }
                                                }
                                                "up" | "arrowup" => {
                                                    // Use move_selection_up to properly skip section headers
                                                    view.move_selection_up(ctx);
                                                }
                                                "down" | "arrowdown" => {
                                                    // Use move_selection_down to properly skip section headers
                                                    view.move_selection_down(ctx);
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute selected");
                                                    view.execute_selected(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter or hide");
                                                    if !view.filter_text.is_empty() {
                                                        view.clear_filter(window, ctx);
                                                    } else {
                                                        // Save window position for the current display BEFORE hiding
                                                        if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                                            let displays = platform::get_macos_displays();
                                                            let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                                            if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                                                window_state::save_main_position_for_display(display, bounds);
                                                            }
                                                        }
                                                        script_kit_gpui::set_main_window_visible(false);
                                                        ctx.hide();
                                                    }
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ScriptList", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::PathPrompt { entity, .. } => {
                                        // Path prompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to PathPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        entity_clone.update(ctx, |path_prompt: &mut PathPrompt, path_cx| {
                                            if has_cmd && key_lower == "k" {
                                                path_prompt.toggle_actions(path_cx);
                                            } else {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => path_prompt.move_up(path_cx),
                                                    "down" | "arrowdown" => path_prompt.move_down(path_cx),
                                                    "enter" => path_prompt.handle_enter(path_cx),
                                                    "escape" => path_prompt.submit_cancel(),
                                                    "left" | "arrowleft" => path_prompt.navigate_to_parent(path_cx),
                                                    "right" | "arrowright" => path_prompt.navigate_into_selected(path_cx),
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in PathPrompt", key_lower));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    AppView::ArgPrompt { id, .. } => {
                                        // Arg prompt key handling via SimulateKey
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ArgPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        // Check for Cmd+K to toggle actions popup
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
                                            view.toggle_arg_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.show_actions_popup = false;
                                                                view.actions_dialog = None;
                                                                view.focused_input = FocusedInput::ArgPrompt;
                                                                window.focus(&view.focus_handle, ctx);
                                                            }
                                                            view.trigger_action_by_name(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
                                                        view.show_actions_popup = false;
                                                        view.actions_dialog = None;
                                                        view.focused_input = FocusedInput::ArgPrompt;
                                                        window.focus(&view.focus_handle, ctx);
                                                    }
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
                                                    }
                                                }
                                            }
                                        } else {
                                            // Normal arg prompt key handling
                                            let prompt_id = id.clone();
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    if view.arg_selected_index > 0 {
                                                        view.arg_selected_index -= 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    let filtered = view.filtered_arg_choices();
                                                    if view.arg_selected_index < filtered.len().saturating_sub(1) {
                                                        view.arg_selected_index += 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - submit selection");
                                                    let filtered = view.filtered_arg_choices();
                                                    if let Some((_, choice)) = filtered.get(view.arg_selected_index) {
                                                        let value = choice.value.clone();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    } else if !view.arg_input.is_empty() {
                                                        let value = view.arg_input.text().to_string();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    }
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - cancel script");
                                                    view.submit_prompt_response(prompt_id, None, ctx);
                                                    view.cancel_script_execution(ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::EditorPrompt { entity, id, .. } => {
                                        // Editor prompt key handling for template/snippet navigation and choice popup
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to EditorPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        let prompt_id_clone = id.clone();

                                        // Check if choice popup is visible
                                        let has_choice_popup = entity_clone.update(ctx, |editor: &mut EditorPrompt, _| {
                                            editor.is_choice_popup_visible()
                                        });

                                        if has_choice_popup {
                                            // Handle choice popup navigation
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    logging::log("STDIN", "SimulateKey: Up in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_up_public(cx);
                                                    });
                                                }
                                                "down" | "arrowdown" => {
                                                    logging::log("STDIN", "SimulateKey: Down in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_down_public(cx);
                                                    });
                                                }
                                                "enter" if !has_cmd => {
                                                    logging::log("STDIN", "SimulateKey: Enter in choice popup - confirming");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                    });
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape in choice popup - cancelling");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_cancel_public(cx);
                                                    });
                                                }
                                                "tab" if !has_shift => {
                                                    logging::log("STDIN", "SimulateKey: Tab in choice popup - confirm and next");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                        editor.next_tabstop_public(window, cx);
                                                    });
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in choice popup", key_lower));
                                                }
                                            }
                                        } else if key_lower == "tab" && !has_cmd {
                                            // Handle Tab key for snippet navigation
                                            entity_clone.update(ctx, |editor: &mut EditorPrompt, editor_cx| {
                                                logging::log("STDIN", "SimulateKey: Tab in EditorPrompt - calling next_tabstop");
                                                if editor.in_snippet_mode() {
                                                    editor.next_tabstop_public(window, editor_cx);
                                                } else {
                                                    logging::log("STDIN", "SimulateKey: Tab - not in snippet mode");
                                                }
                                            });
                                        } else if key_lower == "enter" && has_cmd {
                                            // Cmd+Enter submits - get content from editor
                                            logging::log("STDIN", "SimulateKey: Cmd+Enter in EditorPrompt - submitting");
                                            let content = entity_clone.update(ctx, |editor, editor_cx| {
                                                editor.content(editor_cx)
                                            });
                                            view.submit_prompt_response(prompt_id_clone.clone(), Some(content), ctx);
                                        } else if key_lower == "escape" && !has_cmd {
                                            logging::log("STDIN", "SimulateKey: Escape in EditorPrompt - cancelling");
                                            view.submit_prompt_response(prompt_id_clone.clone(), None, ctx);
                                            view.cancel_script_execution(ctx);
                                        } else {
                                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EditorPrompt", key_lower));
                                        }
                                    }
                                    AppView::ChatPrompt { entity, .. } => {
                                        // ChatPrompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ChatPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle chat actions");
                                            view.toggle_chat_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in chat actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in chat actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in chat actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing chat action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                                            }
                                                            view.execute_chat_action(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close chat actions dialog");
                                                        view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                                    }
                                                    _ => {
                                                        // Handle printable characters for search
                                                        if let Some(ch) = key_lower.chars().next() {
                                                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                                                                logging::log("STDIN", &format!("SimulateKey: Char '{}' in chat actions dialog", ch));
                                                                dialog.update(ctx, |d, cx| d.handle_char(ch, cx));
                                                            } else {
                                                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ChatPrompt actions dialog", key_lower));
                                                            }
                                                        }
                                                    }
                                                }
                                                // Notify the actions window to re-render
                                                crate::actions::notify_actions_window(ctx);
                                            }
                                        } else {
                                            // Route setup keys (tab, arrows, enter, escape) to ChatPrompt
                                            entity.update(ctx, |chat, cx| {
                                                if chat.handle_setup_key(&key_lower, has_shift, cx) {
                                                    logging::log("STDIN", &format!("SimulateKey: Setup handled '{}'", key_lower));
                                                } else {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled '{}' in ChatPrompt", key_lower));
                                                }
                                            });
                                        }
                                    }
                                    _ => {
                                        logging::log("STDIN", &format!("SimulateKey: View {:?} not supported for key simulation", std::mem::discriminant(&view.current_view)));
                                    }
                                }
                            }

                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening AI window via stdin command");
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log("STDIN", "Opening AI window with mock data via stdin command");
                                // First insert mock data
                                if let Err(e) = ai::insert_mock_data() {
                                    logging::log("STDIN", &format!("Failed to insert mock data: {}", e));
                                } else {
                                    logging::log("STDIN", "Mock data inserted successfully");
                                }
                                // Then open the window
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::ShowAiCommandBar => {
                                logging::log("STDIN", "Showing AI command bar via stdin command");
                                ai::show_ai_command_bar(ctx);
                            }
                            ExternalCommand::SimulateAiKey { key, modifiers } => {
                                logging::log(
                                    "STDIN",
                                    &format!("Simulating AI key: '{}' with modifiers: {:?}", key, modifiers),
                                );
                                ai::simulate_ai_key(&key, modifiers);
                            }
                            ExternalCommand::CaptureWindow { title, path } => {
                                logging::log("STDIN", &format!("Capturing window with title '{}' to '{}'", title, path));
                                match validate_capture_window_output_path(&path) {
                                    Ok(validated_path) => {
                                        match capture_window_by_title(&title, false) {
                                            Ok((png_data, width, height)) => {
                                                let mut can_write = true;
                                                if let Some(parent) = validated_path.parent() {
                                                    if let Err(e) = std::fs::create_dir_all(parent) {
                                                        can_write = false;
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Failed to create screenshot directory '{}': {}",
                                                                parent.display(),
                                                                e
                                                            ),
                                                        );
                                                    }
                                                }

                                                if can_write {
                                                    if let Err(e) = std::fs::write(&validated_path, &png_data) {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!("Failed to write screenshot: {}", e),
                                                        );
                                                    } else {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Screenshot saved: {} ({}x{})",
                                                                validated_path.display(),
                                                                width,
                                                                height
                                                            ),
                                                        );
                                                    }
                                                } else {
                                                    tracing::warn!(
                                                        category = "STDIN",
                                                        event_type = "stdin_capture_window_dir_create_failed",
                                                        requested_path = %path,
                                                        resolved_path = %validated_path.display(),
                                                        correlation_id = %logging::current_correlation_id(),
                                                        "Skipping screenshot write due to directory creation failure"
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                logging::log("STDIN", &format!("Failed to capture window: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let correlation_id = logging::current_correlation_id();
                                        tracing::warn!(
                                            category = "STDIN",
                                            event_type = "stdin_capture_window_path_rejected",
                                            requested_path = %path,
                                            reason = %e,
                                            correlation_id = %correlation_id,
                                            "Rejected captureWindow output path"
                                        );
                                        logging::log(
                                            "STDIN",
                                            &format!("Rejected captureWindow path '{}': {}", path, e),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAiSearch { text } => {
                                logging::log("STDIN", &format!("Setting AI search filter to: {}", text));
                                ai::set_ai_search(ctx, &text);
                            }
                            ExternalCommand::SetAiInput { text, submit } => {
                                logging::log("STDIN", &format!("Setting AI input to: {} (submit={})", text, submit));
                                ai::set_ai_input(ctx, &text, submit);
                            }
                            ExternalCommand::ShowGrid { grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, ref depth } => {
                                logging::log("STDIN", &format!(
                                    "ShowGrid: size={}, bounds={}, box_model={}, guides={}, dimensions={}, depth={:?}",
                                    grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, depth
                                ));
                                let options = protocol::GridOptions {
                                    grid_size,
                                    show_bounds,
                                    show_box_model,
                                    show_alignment_guides,
                                    show_dimensions,
                                    depth: depth.clone(),
                                    color_scheme: None,
                                };
                                view.show_grid(options, ctx);
                            }
                            ExternalCommand::HideGrid => {
                                logging::log("STDIN", "HideGrid: hiding debug grid overlay");
                                view.hide_grid(ctx);
                            }
                            ExternalCommand::ExecuteFallback { ref fallback_id, ref input } => {
                                logging::log("STDIN", &format!("ExecuteFallback: id='{}', input='{}'", fallback_id, input));
                                execute_fallback_action(view, fallback_id, input, window, ctx);
                            }
                            ExternalCommand::ShowShortcutRecorder { ref command_id, ref command_name } => {
                                logging::log("STDIN", &format!("ShowShortcutRecorder: command_id='{}', command_name='{}'", command_id, command_name));
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), ctx);
                            }
                        }

                    }
                    ctx.notify();
                }); // close app_entity_inner.update
            }); // close window_for_stdin.update
        }); // close cx.update
    }

    logging::log("STDIN", "Async stdin command handler exiting");
})
.detach();
