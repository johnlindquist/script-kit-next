use super::*;

impl ScriptListApp {
    pub(crate) fn execute_path_action(
        &mut self,
        action_id: &str,
        path_info: &PathInfo,
        path_prompt_entity: &Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!(
                "Executing path action '{}' for: {} (is_dir={})",
                action_id, path_info.path, path_info.is_dir
            ),
        );

        let action_id = action_id.strip_prefix("file:").unwrap_or(action_id);

        match action_id {
            "select_file" | "open_directory" => {
                // For select/open, trigger submission through the path prompt
                // We need to trigger the submit callback with this path
                path_prompt_entity.update(cx, |prompt, cx| {
                    // Find the index of this path in filtered_entries and submit it
                    if let Some(idx) = prompt
                        .filtered_entries
                        .iter()
                        .position(|e| e.path == path_info.path)
                    {
                        prompt.selected_index = idx;
                    }
                    // For directories, navigate into them; for files, submit
                    if path_info.is_dir && action_id == "open_directory" {
                        prompt.navigate_to(&path_info.path, cx);
                    } else {
                        // Submit the selected path
                        let id = prompt.id.clone();
                        let path = path_info.path.clone();
                        (prompt.on_submit)(id, Some(path));
                    }
                });
            }
            "copy_path" => {
                // Copy full path to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.path.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_info.path),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.path
                                    )));
                                    self.show_hud(
                                        format!("Copied path: {}", path_info.path),
                                        Some(HUD_MEDIUM_MS),
                                        cx,
                                    );
                                } else {
                                    logging::log("ERROR", "Failed to write to pbcopy stdin");
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                    self.show_hud(
                                        "Failed to copy path".to_string(),
                                        Some(HUD_2500_MS),
                                        cx,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                            self.last_output = Some(SharedString::from("Failed to copy path"));
                            self.show_hud(format!("Failed to copy path: {}", e), Some(HUD_2500_MS), cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    use arboard::Clipboard;
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&path_info.path) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_info.path),
                                );
                                self.last_output =
                                    Some(SharedString::from(format!("Copied: {}", path_info.path)));
                                self.show_hud(
                                    format!("Copied path: {}", path_info.path),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                self.last_output = Some(SharedString::from("Failed to copy path"));
                                self.show_hud(
                                    format!("Failed to copy path: {}", e),
                                    Some(HUD_2500_MS),
                                    cx,
                                );
                            }
                        },
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to access clipboard: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to access clipboard"));
                            self.show_hud(
                                format!("Failed to access clipboard: {}", e),
                                Some(HUD_2500_MS),
                                cx,
                            );
                        }
                    }
                }
            }
            "copy_filename" => {
                // Copy just the filename to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.name.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!(
                                            "Copied filename to clipboard: {}",
                                            path_info.name
                                        ),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.name
                                    )));
                                    self.show_hud(
                                        format!("Copied filename: {}", path_info.name),
                                        Some(HUD_MEDIUM_MS),
                                        cx,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                            self.show_hud(
                                format!("Failed to copy filename: {}", e),
                                Some(HUD_2500_MS),
                                cx,
                            );
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    use arboard::Clipboard;
                    match Clipboard::new().and_then(|mut c| c.set_text(&path_info.name)) {
                        Ok(_) => {
                            self.show_hud(
                                format!("Copied filename: {}", path_info.name),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                        Err(e) => {
                            self.show_hud(
                                format!("Failed to copy filename: {}", e),
                                Some(HUD_2500_MS),
                                cx,
                            );
                        }
                    }
                }
            }
            "open_in_finder" => {
                let file_manager = if cfg!(target_os = "macos") {
                    "Finder"
                } else if cfg!(target_os = "windows") {
                    "Explorer"
                } else {
                    "File Manager"
                };

                match crate::file_search::reveal_in_finder(&path_info.path) {
                    Ok(_) => {
                        logging::log(
                            "UI",
                            &format!("Revealed in {}: {}", file_manager, path_info.path),
                        );
                        self.show_hud(format!("Opened in {}", file_manager), Some(HUD_SHORT_MS), cx);
                        // Hide main window only (not entire app) to keep HUD visible
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        platform::hide_main_window();
                    }
                    Err(e) => {
                        logging::log(
                            "ERROR",
                            &format!("Failed to reveal in {}: {}", file_manager, e),
                        );
                        self.show_hud(
                            format!("Failed to open in {}: {}", file_manager, e),
                            Some(HUD_2500_MS),
                            cx,
                        );
                    }
                }
            }
            "open_in_editor" => {
                // Open in configured editor
                let editor = self.config.get_editor();
                let path_str = path_info.path.clone();
                logging::log(
                    "UI",
                    &format!("Opening in editor '{}': {}", editor, path_str),
                );

                match std::process::Command::new(&editor).arg(&path_str).spawn() {
                    Ok(_) => {
                        logging::log("UI", &format!("Opened in editor: {}", path_str));
                        self.show_hud(format!("Opened in {}", editor), Some(HUD_SHORT_MS), cx);
                        // Hide main window only (not entire app) to keep HUD visible
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        platform::hide_main_window();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                        self.show_hud(
                            format!("Failed to open in {}: {}", editor, e),
                            Some(HUD_2500_MS),
                            cx,
                        );
                    }
                }
            }
            "open_in_terminal" => {
                match crate::file_search::open_in_terminal(&path_info.path, path_info.is_dir) {
                    Ok(terminal_path) => {
                        logging::log("UI", &format!("Opened terminal at: {}", terminal_path));
                        self.show_hud(
                            format!("Opened Terminal at {}", terminal_path),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        // Hide main window only (not entire app) to keep HUD visible
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        platform::hide_main_window();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open terminal: {}", e));
                        self.show_hud(format!("Failed to open terminal: {}", e), Some(HUD_2500_MS), cx);
                    }
                }
            }
            "move_to_trash" => {
                let path_info = path_info.clone();
                let path_prompt_entity = path_prompt_entity.clone();
                let message = format!(
                    "Are you sure you want to move '{}' to Trash?",
                    path_info.name
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
                                    Some(HUD_2500_MS),
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
                        let path_str = path_info.path.clone();
                        let name = path_info.name.clone();

                        match crate::file_search::move_to_trash(&path_str) {
                            Ok(()) => {
                                logging::log("UI", &format!("Moved to trash: {}", path_str));
                                this.last_output =
                                    Some(SharedString::from(format!("Moved to Trash: {}", name)));
                                this.show_hud(format!("Moved to Trash: {}", name), Some(HUD_MEDIUM_MS), cx);
                                // Refresh the path prompt to show the file is gone
                                path_prompt_entity.update(cx, |prompt, cx| {
                                    let current = prompt.current_path.clone();
                                    prompt.navigate_to(&current, cx);
                                });
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to move to trash: {}", e));
                                this.last_output =
                                    Some(SharedString::from("Failed to move to Trash"));
                                this.show_hud(
                                    format!("Failed to move to Trash: {}", e),
                                    Some(HUD_2500_MS),
                                    cx,
                                );
                            }
                        }
                        cx.notify();
                    })
                    .ok();
                })
                .detach();
                return;
            }
            _ => {
                logging::log("UI", &format!("Unknown path action: {}", action_id));
            }
        }

        cx.notify();
    }
}
