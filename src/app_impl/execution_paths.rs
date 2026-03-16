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
                self.copy_to_clipboard_with_feedback(
                    &path_info.path,
                    format!("Copied path: {}", path_info.path),
                    false,
                    cx,
                );
            }
            "copy_filename" => {
                // Copy just the filename to clipboard
                self.copy_to_clipboard_with_feedback(
                    &path_info.name,
                    format!("Copied filename: {}", path_info.name),
                    false,
                    cx,
                );
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
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log(
                            "ERROR",
                            &format!("Failed to reveal in {}: {}", file_manager, e),
                        );
                        self.show_error_toast(format!("Failed to open in {}: {}", file_manager, e), cx);
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
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                        self.show_error_toast(format!("Failed to open in {}: {}", editor, e), cx);
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
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open terminal: {}", e));
                        self.show_error_toast(format!("Failed to open terminal: {}", e), cx);
                    }
                }
            }
            "move_to_trash" => {
                let path_info = path_info.clone();
                let path_prompt_entity = path_prompt_entity.clone();
                let message = format!("Move \"{}\" to Trash?", path_info.name);

                cx.spawn(async move |this, cx| {
                    let confirmed = match crate::confirm::confirm_with_parent_dialog(
                        cx,
                        crate::confirm::ParentConfirmOptions::destructive(
                            "Move to Trash",
                            message,
                            "Move to Trash",
                        ),
                        "execution_path_move_to_trash",
                    )
                    .await
                    {
                        Ok(confirmed) => confirmed,
                        Err(e) => {
                            let _ = this.update(cx, move |this, cx| {
                                tracing::error!(
                                    event = "execution_path_dialog_open_failed",
                                    error = %e,
                                    "Failed to open execution path move-to-trash dialog"
                                );
                                this.show_error_toast("Failed to open confirmation dialog", cx);
                            });
                            return;
                        }
                    };

                    if !confirmed {
                        return;
                    }

                    let _ = this.update(cx, move |this, cx| {
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
                                this.show_error_toast(format!("Failed to move to Trash: {}", e), cx);
                            }
                        }
                        cx.notify();
                    });
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
