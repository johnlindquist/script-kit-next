            "clipboard_delete_multiple" => {
                let filter_text = match &self.current_view {
                    AppView::ClipboardHistoryView { filter, .. } => filter.trim().to_string(),
                    _ => String::new(),
                };

                if filter_text.is_empty() {
                    self.show_hud(
                        "Type in search first, then use Delete Entries...".to_string(),
                        Some(2500),
                        cx,
                    );
                    return;
                }

                let filter_lower = filter_text.to_lowercase();
                let ids_to_delete: Vec<String> = self
                    .cached_clipboard_entries
                    .iter()
                    .filter(|entry| entry.text_preview.to_lowercase().contains(&filter_lower))
                    .map(|entry| entry.id.clone())
                    .collect();

                if ids_to_delete.is_empty() {
                    self.show_hud("No matching entries to delete".to_string(), Some(2000), cx);
                    return;
                }

                let delete_count = ids_to_delete.len();
                let message = format!(
                    "Are you sure you want to delete these {} matching clipboard entries?",
                    delete_count
                );

                cx.spawn(async move |this, cx| {
                    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);
                    let open_result = cx.update(|cx| {
                        let main_bounds =
                            if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(x as f32),
                                        y: gpui::px(y as f32),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(w as f32),
                                        height: gpui::px(h as f32),
                                    },
                                }
                            } else {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(100.0),
                                        y: gpui::px(100.0),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(600.0),
                                        height: gpui::px(400.0),
                                    },
                                }
                            };

                        let sender = confirm_tx.clone();
                        let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                            let _ = sender.try_send(confirmed);
                        });

                        open_confirm_window(
                            cx,
                            main_bounds,
                            None,
                            message,
                            Some("Yes".to_string()),
                            Some("Cancel".to_string()),
                            on_choice,
                        )
                    });

                    match open_result {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            this.update(cx, |this, cx| {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to open confirmation modal: {}", e),
                                );
                                this.show_hud(
                                    "Failed to open confirmation dialog".to_string(),
                                    Some(2500),
                                    cx,
                                );
                            })
                            .ok();
                            return;
                        }
                        Err(_) => return,
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    this.update(cx, move |this, cx| {
                        let mut deleted = 0usize;
                        let mut failed = 0usize;
                        for id in ids_to_delete {
                            match clipboard_history::remove_entry(&id) {
                                Ok(()) => deleted += 1,
                                Err(e) => {
                                    failed += 1;
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to delete clipboard entry {}: {}", id, e),
                                    );
                                }
                            }
                        }

                        this.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                        if let AppView::ClipboardHistoryView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = 0;
                            if let Some(first) = this.cached_clipboard_entries.first() {
                                this.focused_clipboard_entry_id = Some(first.id.clone());
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(0, ScrollStrategy::Top);
                            } else {
                                this.focused_clipboard_entry_id = None;
                            }
                        }
                        cx.notify();

                        if failed == 0 {
                            this.show_hud(format!("Deleted {} entries", deleted), Some(2500), cx);
                        } else {
                            this.show_hud(
                                format!("Deleted {}, failed {}", deleted, failed),
                                Some(3000),
                                cx,
                            );
                        }
                    })
                    .ok();
                })
                .detach();
                return;
            }
            "clipboard_delete" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                match clipboard_history::remove_entry(&entry.id) {
                    Ok(()) => {
                        logging::log("UI", &format!("Deleted clipboard entry: {}", entry.id));
                        // Refresh cached entries
                        self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);

                        // Update selection in ClipboardHistoryView
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

                            // Keep selection in bounds after deletion
                            if !filtered_entries.is_empty() {
                                *selected_index =
                                    (*selected_index).min(filtered_entries.len().saturating_sub(1));
                                self.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                self.focused_clipboard_entry_id = filtered_entries
                                    .get(*selected_index)
                                    .map(|(_, entry)| entry.id.clone());
                            } else {
                                *selected_index = 0;
                                self.focused_clipboard_entry_id = None;
                            }
                        }

                        self.show_hud("Entry deleted".to_string(), Some(1500), cx);
                        cx.notify();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to delete clipboard entry: {}", e));
                        self.show_hud(format!("Delete failed: {}", e), Some(3000), cx);
                    }
                }
                return;
            }
            "clipboard_delete_all" => {
                // Delete all unpinned entries
                let unpinned_count = self
                    .cached_clipboard_entries
                    .iter()
                    .filter(|e| !e.pinned)
                    .count();

                if unpinned_count == 0 {
                    self.show_hud("No unpinned entries to delete".to_string(), Some(2000), cx);
                    return;
                }

                let message = format!(
                    "Are you sure you want to delete all {} unpinned clipboard entries?",
                    unpinned_count
                );

                cx.spawn(async move |this, cx| {
                    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);
                    let open_result = cx.update(|cx| {
                        let main_bounds =
                            if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(x as f32),
                                        y: gpui::px(y as f32),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(w as f32),
                                        height: gpui::px(h as f32),
                                    },
                                }
                            } else {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(100.0),
                                        y: gpui::px(100.0),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(600.0),
                                        height: gpui::px(400.0),
                                    },
                                }
                            };

                        let sender = confirm_tx.clone();
                        let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                            let _ = sender.try_send(confirmed);
                        });

                        open_confirm_window(
                            cx,
                            main_bounds,
                            None,
                            message,
                            Some("Yes".to_string()),
                            Some("Cancel".to_string()),
                            on_choice,
                        )
                    });

                    match open_result {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            this.update(cx, |this, cx| {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to open confirmation modal: {}", e),
                                );
                                this.show_hud(
                                    "Failed to open confirmation dialog".to_string(),
                                    Some(2500),
                                    cx,
                                );
                            })
                            .ok();
                            return;
                        }
                        Err(_) => return,
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    this.update(cx, move |this, cx| {
                        match clipboard_history::clear_unpinned_history() {
                            Ok(()) => {
                                logging::log(
                                    "UI",
                                    &format!(
                                        "Deleted {} unpinned clipboard entries",
                                        unpinned_count
                                    ),
                                );
                                this.cached_clipboard_entries =
                                    clipboard_history::get_cached_entries(100);

                                // Reset selection
                                if let AppView::ClipboardHistoryView { selected_index, .. } =
                                    &mut this.current_view
                                {
                                    *selected_index = 0;
                                    if let Some(first) = this.cached_clipboard_entries.first() {
                                        this.focused_clipboard_entry_id = Some(first.id.clone());
                                    } else {
                                        this.focused_clipboard_entry_id = None;
                                    }
                                }

                                this.show_hud(
                                    format!(
                                        "Deleted {} entries (pinned preserved)",
                                        unpinned_count
                                    ),
                                    Some(2500),
                                    cx,
                                );
                                cx.notify();
                            }
                            Err(e) => {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to clear unpinned history: {}", e),
                                );
                                this.show_hud(format!("Delete failed: {}", e), Some(3000), cx);
                            }
                        }
                    })
                    .ok();
                })
                .detach();
                return;
            }
