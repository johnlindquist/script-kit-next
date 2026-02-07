            "clipboard_pin" | "clipboard_unpin" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let result = if action_id == "clipboard_pin" {
                    clipboard_history::pin_entry(&entry.id)
                } else {
                    clipboard_history::unpin_entry(&entry.id)
                };

                match result {
                    Ok(()) => {
                        // Refresh cached entries (pin/unpin updates cache ordering)
                        self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);

                        // Keep selection on the same entry when possible
                        if let AppView::ClipboardHistoryView {
                            filter,
                            selected_index,
                        } = &mut self.current_view
                        {
                            let filtered_entries: Vec<_> = if filter.is_empty() {
                                self.cached_clipboard_entries.iter().enumerate().collect()
                            } else {
                                let filter_lower = filter.to_lowercase();
                                self.cached_clipboard_entries
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, e)| {
                                        e.text_preview.to_lowercase().contains(&filter_lower)
                                    })
                                    .collect()
                            };

                            if let Some(new_index) =
                                filtered_entries.iter().position(|(_, e)| e.id == entry.id)
                            {
                                *selected_index = new_index;
                            } else if !filtered_entries.is_empty() {
                                *selected_index =
                                    (*selected_index).min(filtered_entries.len().saturating_sub(1));
                            } else {
                                *selected_index = 0;
                            }

                            if !filtered_entries.is_empty() {
                                self.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            }
                            self.focused_clipboard_entry_id = filtered_entries
                                .get(*selected_index)
                                .map(|(_, entry)| entry.id.clone());
                        }

                        if let Some(message) = clipboard_pin_action_success_hud(&action_id) {
                            self.show_hud(message.to_string(), Some(1500), cx);
                        }
                        cx.notify();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to toggle clipboard pin: {}", e));
                        self.show_hud(format!("Failed to update pin: {}", e), Some(3000), cx);
                    }
                }
                return;
            }
            "clipboard_share" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Clipboard entry content unavailable".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                };

                logging::log(
                    "UI",
                    &format!(
                        "Opening share sheet for clipboard entry {} ({:?})",
                        entry.id, entry.content_type
                    ),
                );

                let share_result = match entry.content_type {
                    clipboard_history::ContentType::Text => {
                        crate::platform::show_share_sheet(crate::platform::ShareSheetItem::Text(
                            content,
                        ));
                        Ok(())
                    }
                    clipboard_history::ContentType::Image => {
                        if let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content) {
                            crate::platform::show_share_sheet(
                                crate::platform::ShareSheetItem::ImagePng(png_bytes),
                            );
                            Ok(())
                        } else {
                            Err("Failed to decode clipboard image".to_string())
                        }
                    }
                };

                match share_result {
                    Ok(()) => self.show_hud("Share sheet opened".to_string(), Some(1500), cx),
                    Err(message) => self.show_hud(message, Some(2000), cx),
                }
                return;
            }
            // Paste to active app and close window (Enter)
            "clipboard_paste" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                logging::log("CLIPBOARD", &format!("Paste entry: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        logging::log("CLIPBOARD", "Entry copied, simulating paste");
                        cx.spawn(async move |_this, _cx| {
                            Timer::after(std::time::Duration::from_millis(50)).await;
                            if let Err(e) = selected_text::simulate_paste_with_cg() {
                                logging::log("ERROR", &format!("Failed to simulate paste: {}", e));
                            } else {
                                logging::log("CLIPBOARD", "Simulated Cmd+V paste");
                            }
                        })
                        .detach();
                        self.show_hud("Pasted".to_string(), Some(1000), cx);
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to paste entry: {}", e));
                        self.show_hud(format!("Failed to paste: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            "clipboard_attach_to_ai" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Clipboard entry content unavailable".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                };

                logging::log(
                    "AI",
                    &format!(
                        "Attaching clipboard entry {} ({:?}) to AI chat",
                        entry.id, entry.content_type
                    ),
                );

                match entry.content_type {
                    clipboard_history::ContentType::Text => {
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            self.show_hud("Failed to open AI window".to_string(), Some(2000), cx);
                            return;
                        }
                        ai::set_ai_input(cx, &content, false);
                    }
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                        else {
                            self.show_hud(
                                "Failed to decode clipboard image".to_string(),
                                Some(2000),
                                cx,
                            );
                            return;
                        };

                        use base64::Engine;
                        let base64_data =
                            base64::engine::general_purpose::STANDARD.encode(&png_bytes);

                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            self.show_hud("Failed to open AI window".to_string(), Some(2000), cx);
                            return;
                        }
                        ai::set_ai_input_with_image(cx, "", &base64_data, false);
                    }
                }

                self.show_hud("Attached to AI".to_string(), Some(1500), cx);
                self.hide_main_and_reset(cx);
                return;
            }
            // Copy to clipboard without pasting (Cmd+Enter)
            "clipboard_copy" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                logging::log(
                    "CLIPBOARD",
                    &format!("Copying entry to clipboard: {}", entry.id),
                );
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        logging::log("CLIPBOARD", "Entry copied to clipboard");
                        self.show_hud("Copied to clipboard".to_string(), Some(1500), cx);
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                        self.show_hud(format!("Failed to copy: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            // Paste and keep window open (Opt+Enter)
            "clipboard_paste_keep_open" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                logging::log("CLIPBOARD", &format!("Paste and keep open: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        logging::log("CLIPBOARD", "Entry copied, simulating paste");
                        // Simulate Cmd+V paste after a brief delay
                        cx.spawn(async move |_this, _cx| {
                            Timer::after(std::time::Duration::from_millis(50)).await;
                            if let Err(e) = selected_text::simulate_paste_with_cg() {
                                logging::log("ERROR", &format!("Failed to simulate paste: {}", e));
                            } else {
                                logging::log("CLIPBOARD", "Simulated Cmd+V paste");
                            }
                        })
                        .detach();
                        self.show_hud("Pasted".to_string(), Some(1000), cx);
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                        self.show_hud(format!("Failed to paste: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            "clipboard_quick_look" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                if let Err(e) = clipboard_history::quick_look_entry(&entry) {
                    logging::log("ERROR", &format!("Quick Look failed: {}", e));
                    self.show_hud(format!("Quick Look failed: {}", e), Some(2500), cx);
                }
                return;
            }
