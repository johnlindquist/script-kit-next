        // Use theme for all colors - consistent with main menu
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use theme colors for consistency with main menu
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // P0 FIX: Reference data from self instead of taking ownership
        // P1 FIX: NEVER do synchronous SQLite queries or image decoding in render loop!
        // Only copy from global cache (populated async by background prewarm thread).
        // Images not yet cached will show placeholder with dimensions from metadata.
        for entry in &self.cached_clipboard_entries {
            if entry.content_type == clipboard_history::ContentType::Image {
                // Only use already-cached images - NO synchronous fetch/decode
                if !self.clipboard_image_cache.contains_key(&entry.id) {
                    if let Some(cached) = clipboard_history::get_cached_image(&entry.id) {
                        self.clipboard_image_cache.insert(entry.id.clone(), cached);
                    }
                    // If not in global cache yet, background thread will populate it.
                    // We'll show placeholder with dimensions until then.
                }
            }
        }

        // Clone the cache for use in closures
        let image_cache = self.clipboard_image_cache.clone();

        // Filter entries based on current filter
        let filtered_entries: Vec<_> = if filter.is_empty() {
            self.cached_clipboard_entries.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.cached_clipboard_entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    let matches = e.text_preview.to_lowercase().contains(&filter_lower)
                        || e.ocr_text
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&filter_lower);
                    matches
                })
                .collect()
        };
        let filtered_len = filtered_entries.len();

        // Key handler for clipboard history
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
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    modifiers,
                    ActionsDialogHost::ClipboardHistory,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {}
                    ActionsRoute::Handled => {
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

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

                logging::log("KEY", &format!("ClipboardHistory key: '{}'", key_str));

                // P0 FIX: View state only - data comes from this.cached_clipboard_entries
                if let AppView::ClipboardHistoryView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // P0 FIX: Reference cached_clipboard_entries from self
                    let filtered_entries: Vec<_> = if filter.is_empty() {
                        this.cached_clipboard_entries.iter().enumerate().collect()
                    } else {
                        let filter_lower = filter.to_lowercase();
                        this.cached_clipboard_entries
                            .iter()
                            .enumerate()
                            .filter(|(_, e)| {
                                let matches = e.text_preview.to_lowercase().contains(&filter_lower)
                                    || e.ocr_text
                                        .as_deref()
                                        .unwrap_or("")
                                        .to_lowercase()
                                        .contains(&filter_lower);
                                matches
                            })
                            .collect()
                    };
                    let filtered_len = filtered_entries.len();
                    let selected_entry = filtered_entries
                        .get(*selected_index)
                        .map(|(_, entry)| (*entry).clone());
                    this.focused_clipboard_entry_id =
                        selected_entry.as_ref().map(|entry| entry.id.clone());

                    // Cmd+P toggles pin state for selected entry
                    if has_cmd && key_str == "p" {
                        if let Some(entry) = selected_entry {
                            drop(filtered_entries);
                            let action_id = if entry.pinned {
                                "clipboard_unpin"
                            } else {
                                "clipboard_pin"
                            };
                            this.handle_action(action_id.to_string(), cx);
                        }
                        return;
                    }

                    // Cmd+K opens clipboard actions dialog
                    if has_cmd && key_str == "k" {
                        if let Some(entry) = selected_entry {
                            drop(filtered_entries);
                            this.toggle_clipboard_actions(entry, window, cx);
                        }
                        return;
                    }

                    // Ctrl+Cmd+A attaches selected entry to AI chat
                    if modifiers.control && has_cmd && key_str == "a" {
                        if let Some(_entry) = selected_entry {
                            drop(filtered_entries);
                            this.handle_action("clipboard_attach_to_ai".to_string(), cx);
                        }
                        return;
                    }

                    // Space opens Quick Look (macOS Finder behavior)
                    if key_str == "space"
                        && filter.is_empty()
                        && !modifiers.platform
                        && !modifiers.control
                        && !modifiers.alt
                        && !modifiers.shift
                    {
                        if let Some(entry) = selected_entry {
                            if let Err(e) = clipboard_history::quick_look_entry(&entry) {
                                logging::log("ERROR", &format!("Quick Look failed: {}", e));
                                this.show_hud(format!("Quick Look failed: {}", e), Some(HUD_2500_MS), cx);
                            }
                        }
                        return;
                    }

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                // Scroll to keep selection visible
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                this.focused_clipboard_entry_id = filtered_entries
                                    .get(*selected_index)
                                    .map(|(_, entry)| entry.id.clone());
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                // Scroll to keep selection visible
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                this.focused_clipboard_entry_id = filtered_entries
                                    .get(*selected_index)
                                    .map(|(_, entry)| entry.id.clone());
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            // Copy selected entry to clipboard, hide window, then paste
                            if let Some((_, entry)) = filtered_entries.get(*selected_index) {
                                logging::log(
                                    "EXEC",
                                    &format!("Copying clipboard entry: {}", entry.id),
                                );
                                if let Err(e) =
                                    clipboard_history::copy_entry_to_clipboard(&entry.id)
                                {
                                    logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                                } else {
                                    logging::log("EXEC", "Entry copied to clipboard");
                                    this.hide_main_and_reset(cx);

                                    // Simulate Cmd+V paste after a brief delay to let focus return
                                    std::thread::spawn(|| {
                                        std::thread::sleep(std::time::Duration::from_millis(100));
                                        if let Err(e) = selected_text::simulate_paste_with_cg() {
                                            logging::log(
                                                "ERROR",
                                                &format!("Failed to simulate paste: {}", e),
                                            );
                                        } else {
                                            logging::log("EXEC", "Simulated Cmd+V paste");
                                        }
                                    });
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
