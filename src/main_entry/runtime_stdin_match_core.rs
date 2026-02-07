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
