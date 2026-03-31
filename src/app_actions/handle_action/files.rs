// File-related action handlers for handle_action dispatch.
//
// Contains: reveal_in_finder, copy_path, copy_deeplink, file search actions
// (open_file, open_directory, quick_look, open_with, show_info, attach_to_ai),
// copy_filename, __cancel__.

impl ScriptListApp {
    /// Resolve the target path for a file action.
    ///
    /// Priority: `file_search_actions_path` (consumed) > selected SearchResult.
    /// The `extractor` callback is used for SearchResult-based path extraction so
    /// callers can choose `extract_path_for_reveal` vs `extract_path_for_copy`.
    fn resolve_file_action_path<F>(
        &mut self,
        extractor: F,
    ) -> Result<std::path::PathBuf, Option<gpui::SharedString>>
    where
        F: FnOnce(
            Option<&scripts::SearchResult>,
        ) -> Result<std::path::PathBuf, crate::action_helpers::PathExtractionError>,
    {
        // file_search_actions_path takes priority (consumed on use)
        if let Some(path) = self.file_search_actions_path.take() {
            return Ok(std::path::PathBuf::from(path));
        }
        // Fall back to main menu selected result via the shared extractor
        let selected = self.get_selected_result();
        extractor(selected.as_ref()).map_err(|e| Some(e.message()))
    }

    /// Extract (path, is_dir, name) from the actions-path or the selected file search result.
    fn resolve_file_search_path_info(&self) -> Option<(String, bool, String)> {
        if let Some(ref path) = self.file_search_actions_path {
            let p = std::path::Path::new(path);
            let is_dir = p.is_dir();
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            return Some((path.clone(), is_dir, name));
        }
        let AppView::FileSearchView { selected_index, .. } = &self.current_view else {
            return None;
        };
        let (_, entry) = self.selected_file_search_result(*selected_index)?;
        let is_dir = matches!(entry.file_type, crate::file_search::FileType::Directory);
        Some((entry.path.clone(), is_dir, entry.name.clone()))
    }

    /// After a mutation (trash, etc.), re-resolve directory contents and refresh the display.
    fn refresh_file_search_after_mutation(&mut self, cx: &mut Context<Self>) {
        let AppView::FileSearchView {
            query, presentation, ..
        } = &self.current_view
        else {
            return;
        };
        let query_value = query.clone();
        let presentation_value = *presentation;
        let results = Self::resolve_file_search_results(&query_value);
        self.update_file_search_results(results);

        // Reset selection to top
        if let AppView::FileSearchView {
            ref mut selected_index,
            ..
        } = self.current_view
        {
            *selected_index = 0;
        }

        Self::resize_file_search_window_for_presentation(
            presentation_value,
            self.file_search_display_indices.len(),
        );
        self.file_search_scroll_handle
            .scroll_to_item(0, gpui::ScrollStrategy::Top);
        cx.notify();
    }

    /// Handle file-related actions. Returns `true` if handled.
    fn handle_file_action(
        &mut self,
        action_id: &str,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let trace_id = &dctx.trace_id;
        match action_id {
            "reveal_in_finder" => {
                tracing::info!(category = "UI", "reveal in Finder action");
                let path_result =
                    self.resolve_file_action_path(crate::action_helpers::extract_path_for_reveal);

                if let Ok(path) = path_result {
                    let reveal_result_rx = self.reveal_in_finder_with_feedback_async(&path, trace_id);
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
                    let msg = path_result
                        .err()
                        .flatten()
                        .unwrap_or_else(|| gpui::SharedString::from("Cannot reveal this item type in Finder"));
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        msg.to_string(),
                    );
                }
                DispatchOutcome::success()
            }
            "copy_path" => {
                tracing::info!(category = "UI", "copy path action");
                let path_result =
                    self.resolve_file_action_path(crate::action_helpers::extract_path_for_copy);

                match path_result {
                    Ok(path) => {
                        let path_str = path.to_string_lossy().to_string();
                        tracing::info!(category = "UI", path = %path_str, "copying path to clipboard");
                        self.copy_to_clipboard_with_feedback(
                            &path_str,
                            format!("Copied: {}", path_str),
                            true,
                            cx,
                        );
                    }
                    Err(msg) => {
                        let error_msg = msg
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                selection_required_message_for_action(action_id).to_string()
                            });
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            error_msg,
                        );
                    }
                }
                DispatchOutcome::success()
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
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            "__cancel__" => {
                tracing::info!(category = "UI", "actions dialog cancelled");
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
                DispatchOutcome::success()
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
                                action_id,
                                &dctx.trace_id,
                                DeferredAiWindowAction::AddAttachment {
                                    path: path.clone(),
                                },
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
                            self.file_search_actions_path = None;
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                format!("{}: {}", prefix, e),
                            );
                        }
                    }
                }
                DispatchOutcome::success()
            }
            "open_in_editor" => {
                let Some((path, _, _)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                let path_buf = std::path::PathBuf::from(&path);
                match crate::script_creation::open_in_editor(&path_buf, &self.config) {
                    Ok(()) => {
                        self.file_search_actions_path = None;
                        self.show_hud(
                            "Opened in Editor".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        self.file_search_actions_path = None;
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            format!("Failed to open in editor: {}", e),
                        );
                    }
                }
                DispatchOutcome::success()
            }
            "open_in_terminal" => {
                let Some((path, is_dir, _)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                match crate::file_search::open_in_terminal(&path, is_dir) {
                    Ok(_) => {
                        self.file_search_actions_path = None;
                        self.show_hud(
                            "Opened in Terminal".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        self.file_search_actions_path = None;
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            format!("Failed to open in terminal: {}", e),
                        );
                    }
                }
                DispatchOutcome::success()
            }
            "move_to_trash" => {
                let Some((path, _, name)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                let trace_id = trace_id.to_string();
                let start = std::time::Instant::now();

                cx.spawn(async move |this, cx| {
                    let confirm_options = crate::confirm::ParentConfirmOptions::destructive(
                        "Move to Trash",
                        format!("Move \"{}\" to Trash?", name),
                        "Move to Trash",
                    );

                    match crate::confirm::confirm_with_parent_dialog(
                        cx,
                        confirm_options,
                        &trace_id,
                    )
                    .await
                    {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::info!(
                                trace_id = %trace_id,
                                status = "cancelled",
                                duration_ms = start.elapsed().as_millis() as u64,
                                "Async action cancelled: move_to_trash"
                            );
                            return;
                        }
                        Err(e) => {
                            let _ = this.update(cx, |this, cx| {
                                tracing::error!(
                                    trace_id = %trace_id,
                                    status = "failed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    error = %e,
                                    "failed to open confirmation modal"
                                );
                                this.show_error_toast_with_code(
                                    "Failed to open confirmation dialog",
                                    Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                    cx,
                                );
                            });
                            return;
                        }
                    }

                    let _ = this.update(cx, |this, cx| {
                        match crate::file_search::move_to_trash(&path) {
                            Ok(()) => {
                                tracing::info!(
                                    trace_id = %trace_id,
                                    status = "completed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    path = %path,
                                    "file moved to trash"
                                );
                                this.file_search_actions_path = None;
                                this.show_hud(
                                    format!("Moved to Trash: {}", name),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                this.refresh_file_search_after_mutation(cx);
                            }
                            Err(e) => {
                                tracing::error!(
                                    trace_id = %trace_id,
                                    status = "failed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    error = %e,
                                    path = %path,
                                    "failed to move to trash"
                                );
                                this.file_search_actions_path = None;
                                this.show_error_toast(
                                    format!("Failed to move to Trash: {}", e),
                                    cx,
                                );
                            }
                        }
                    });
                })
                .detach();

                DispatchOutcome::success()
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
                        self.file_search_actions_path = None;
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "No filename found for selected path",
                        );
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
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
