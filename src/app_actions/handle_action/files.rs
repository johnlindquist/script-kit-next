// File-related action handlers for handle_action dispatch.
//
// Contains: reveal_in_finder, copy_path, copy_deeplink, file search actions
// (open_file, open_directory, quick_look, open_with, show_info, attach_to_ai),
// copy_filename, __cancel__.

impl ScriptListApp {
    /// Handle file-related actions. Returns `true` if handled.
    fn handle_file_action(
        &mut self,
        action_id: &str,
        trace_id: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        match action_id {
            "reveal_in_finder" => {
                tracing::info!(category = "UI", "reveal in Finder action");
                // First check if we have a file search path (takes priority)
                let path_opt = if let Some(path) = self.file_search_actions_path.take() {
                    tracing::info!(category = "UI", path = %path, "reveal in Finder (file search)");
                    Some(std::path::PathBuf::from(path))
                } else if let Some(result) = self.get_selected_result() {
                    // Fall back to main menu selected result
                    match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::App(m) => Some(m.app.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => None,
                        scripts::SearchResult::BuiltIn(_) => None,
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(_) => None,
                    }
                } else {
                    None
                };

                if let Some(path) = path_opt {
                    let reveal_result_rx = self.reveal_in_finder_with_feedback_async(&path);
                    let trace_id = trace_id.to_string();
                    let start = std::time::Instant::now();
                    cx.spawn(async move |this, cx| {
                        let Ok(reveal_result) = reveal_result_rx.recv().await else {
                            return;
                        };

                        let _ = this.update(cx, |this, cx| match reveal_result {
                            Ok(()) => {
                                tracing::info!(
                                    trace_id = %trace_id,
                                    status = "completed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    "Async action completed: reveal_in_finder"
                                );
                                this.show_hud(
                                    "Opened in Finder".to_string(),
                                    Some(HUD_SHORT_MS),
                                    cx,
                                );
                                this.hide_main_and_reset(cx);
                            }
                            Err(message) => {
                                tracing::error!(
                                    trace_id = %trace_id,
                                    status = "failed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    error = %message,
                                    "Async action failed: reveal_in_finder"
                                );
                                this.show_error_toast_with_code(
                                    message,
                                    Some(crate::action_helpers::ERROR_REVEAL_FAILED),
                                    cx,
                                );
                            }
                        });
                    })
                    .detach();
                } else {
                    self.show_error_toast(
                        "Cannot reveal this item type in Finder",
                        cx,
                    );
                }
                true
            }
            "copy_path" => {
                tracing::info!(category = "UI", "copy path action");
                // First check if we have a file search path (takes priority)
                let path_str = if let Some(path) = self.file_search_actions_path.take() {
                    tracing::info!(category = "UI", path = %path, "copy path (file search)");
                    Some(path)
                } else if let Some(result) = self.get_selected_result() {
                    // Fall back to main menu selected result
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(m.script.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::App(m) => {
                            Some(m.app.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Agent(m) => {
                            Some(m.agent.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Scriptlet(_) => None,
                        scripts::SearchResult::BuiltIn(_) => None,
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(_) => None,
                    };
                    if path_opt.is_none() {
                        self.show_error_toast(
                            "Cannot copy path for this item type",
                            cx,
                        );
                    }
                    path_opt
                } else {
                    self.show_error_toast(
                        selection_required_message_for_action(action_id),
                        cx,
                    );
                    None
                };

                if let Some(path_str) = path_str {
                    tracing::info!(category = "UI", path = %path_str, "copying path to clipboard");
                    self.copy_to_clipboard_with_feedback(
                        &path_str,
                        format!("Copied: {}", path_str),
                        true,
                        cx,
                    );
                }
                true
            }
            "copy_deeplink" => {
                tracing::info!(category = "UI", "copy deeplink action");
                if let Some(result) = self.get_selected_result() {
                    let name = result.name();
                    let deeplink_name = crate::actions::to_deeplink_name(name);
                    let deeplink_url = format!("scriptkit://run/{}", deeplink_name);

                    tracing::info!(category = "UI", deeplink = %deeplink_url, "copying deeplink to clipboard");
                    self.copy_to_clipboard_with_feedback(
                        &deeplink_url,
                        format!("Copied: {}", deeplink_url),
                        true,
                        cx,
                    );
                } else {
                    self.show_error_toast(
                        selection_required_message_for_action(action_id),
                        cx,
                    );
                }
                true
            }
            "__cancel__" => {
                tracing::info!(category = "UI", "actions dialog cancelled");
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
                true
            }
            // File search specific actions
            "open_file" | "open_directory" | "quick_look" | "open_with" | "show_info"
            | "attach_to_ai" => {
                if let Some(path) = self.file_search_actions_path.clone() {
                    tracing::info!(category = "UI", action = action_id, path = %path, "file action");

                    let result: Result<(), String> = match action_id {
                        "open_file" | "open_directory" => crate::file_search::open_file(&path),
                        "quick_look" => crate::file_search::quick_look(&path),
                        "open_with" => crate::file_search::open_with(&path),
                        "show_info" => crate::file_search::show_info(&path),
                        "attach_to_ai" => {
                            self.open_ai_window_after_main_hide(
                                DeferredAiWindowAction::AddAttachment {
                                    path: path.clone(),
                                },
                                "Attached to AI",
                                cx,
                            );

                            Ok(())
                        }
                        _ => Ok(()),
                    };

                    match result {
                        Ok(()) => {
                            if action_id != "attach_to_ai" {
                                if let Some(message) =
                                    file_search_action_success_hud(action_id)
                                {
                                    self.show_hud(
                                        message.to_string(),
                                        Some(HUD_SHORT_MS),
                                        cx,
                                    );
                                }
                            }
                            self.file_search_actions_path = None;
                            if action_id == "open_file" || action_id == "open_directory" {
                                self.hide_main_and_reset(cx);
                            }
                        }
                        Err(e) => {
                            tracing::error!(action = action_id, path = %path, error = %e, "file search action failed");
                            let prefix = if action_id == "attach_to_ai" {
                                "Failed to attach"
                            } else {
                                file_search_action_error_hud_prefix(action_id)
                                    .unwrap_or("Failed to complete action")
                            };
                            self.show_error_toast(format!("{}: {}", prefix, e), cx);
                            self.file_search_actions_path = None;
                        }
                    }
                }
                true
            }
            "copy_filename" => {
                if let Some(path) = self.file_search_actions_path.clone() {
                    let Some(filename) = std::path::Path::new(&path)
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                    else {
                        tracing::error!(
                            event = "copy_filename_missing_filename",
                            attempted = "copy_filename",
                            path = %path,
                            "No filename found for path"
                        );
                        self.show_error_toast("No filename found for selected path", cx);
                        self.file_search_actions_path = None;
                        return true;
                    };
                    tracing::info!(category = "UI", filename = %filename, "copy filename");
                    self.file_search_actions_path = None;
                    self.copy_to_clipboard_with_feedback(
                        &filename,
                        format!("Copied: {}", filename),
                        true,
                        cx,
                    );
                }
                true
            }
            _ => false,
        }
    }
}
