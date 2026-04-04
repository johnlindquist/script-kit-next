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

    /// Build a `FileResult` from live filesystem metadata.
    fn build_file_result_from_metadata(path: &str) -> Option<crate::file_search::FileResult> {
        crate::file_search::get_file_metadata(path).map(|meta| crate::file_search::FileResult {
            path: meta.path,
            name: meta.name,
            size: meta.size,
            modified: meta.modified,
            file_type: meta.file_type,
        })
    }

    /// Return the absolute directory path when the current view is a
    /// directory-browse (not a global search).
    fn current_file_search_directory_abs(&self) -> Option<String> {
        let AppView::FileSearchView { query, .. } = &self.current_view else {
            return None;
        };
        let parsed = crate::file_search::parse_directory_path(query)?;
        crate::file_search::expand_path(parsed.directory.trim_end_matches('/'))
            .map(|dir| crate::file_search::ensure_trailing_slash(&dir))
    }

    /// Absolute parent directory of `path`, with trailing slash.
    fn parent_directory_abs(path: &str) -> Option<String> {
        std::path::Path::new(path)
            .parent()
            .and_then(|parent| parent.to_str())
            .map(|s| crate::file_search::ensure_trailing_slash(s))
    }

    /// After a mutation (trash, rename, move, etc.), patch the cached directory
    /// listing in place when possible and fall back to a full refresh for global
    /// search.
    fn refresh_file_search_after_mutation(
        &mut self,
        old_path: &str,
        preferred_path: Option<&str>,
        previous_display_index: usize,
        cx: &mut Context<Self>,
    ) {
        let AppView::FileSearchView { presentation, .. } = &self.current_view else {
            return;
        };
        let presentation_value = *presentation;

        let current_dir = self.current_file_search_directory_abs();
        let old_dir = Self::parent_directory_abs(old_path);
        let new_dir = preferred_path.and_then(Self::parent_directory_abs);

        // We can patch in place when we are browsing a concrete directory and
        // the mutation touches that directory (source or destination).
        let can_patch_in_place = current_dir.is_some()
            && (old_dir.as_ref() == current_dir.as_ref()
                || new_dir.as_ref() == current_dir.as_ref());

        if can_patch_in_place {
            // Remove the old entry from the cache.
            self.cached_file_results.retain(|entry| entry.path != old_path);

            // If the item was renamed/moved into the current directory, add it.
            if let Some(new_path) = preferred_path {
                if new_dir.as_ref() == current_dir.as_ref() {
                    if let Some(updated) = Self::build_file_result_from_metadata(new_path) {
                        self.cached_file_results.push(updated);
                    }
                }
            }

            self.apply_file_search_sort_mode();
            self.recompute_file_search_display_indices();
        } else {
            // Global search or cross-directory — full refresh.
            let AppView::FileSearchView { query, .. } = &self.current_view else {
                return;
            };
            let query_value = query.clone();
            let results = Self::resolve_file_search_results(&query_value);
            self.update_file_search_results(results);
        }

        let next_index = preferred_path
            .and_then(|path| self.file_search_display_index_for_path(path))
            .or_else(|| {
                let len = self.file_search_display_len();
                (len > 0).then_some(previous_display_index.min(len.saturating_sub(1)))
            });

        if let AppView::FileSearchView {
            ref mut selected_index,
            ..
        } = self.current_view
        {
            *selected_index = next_index.unwrap_or(0);
        }

        Self::resize_file_search_window_for_presentation(
            presentation_value,
            self.file_search_display_indices.len(),
        );
        if let Some(index) = next_index {
            self.file_search_scroll_handle
                .scroll_to_item(index, gpui::ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    /// Clear the pending file-search action target so the next verb acts on
    /// the current selection, not a stale path from a cancelled/failed action.
    fn clear_file_search_action_target(&mut self) {
        self.file_search_actions_path = None;
    }

    /// Restore keyboard focus to the file-search input after an async
    /// file verb (rename, move, trash, copy-name) completes.
    ///
    /// Routes through the focus coordinator so popup-close and post-verb
    /// restore follow the same path, then syncs to legacy fields.
    fn restore_file_search_input_focus(&mut self, cx: &mut Context<Self>) {
        if matches!(self.current_view, AppView::FileSearchView { .. }) {
            self.focus_coordinator
                .request(crate::focus_coordinator::FocusRequest::main_filter());
            self.sync_coordinator_to_legacy();
            cx.notify();
        }
    }

    /// After an insertion (duplicate), patch the cached directory listing
    /// in place when possible and fall back to a full refresh for global search.
    fn refresh_file_search_after_insert(
        &mut self,
        preferred_path: &str,
        previous_display_index: usize,
        cx: &mut Context<Self>,
    ) {
        let AppView::FileSearchView { presentation, .. } = &self.current_view else {
            return;
        };
        let presentation_value = *presentation;

        let current_dir = self.current_file_search_directory_abs();
        let new_dir = Self::parent_directory_abs(preferred_path);

        if current_dir.is_some() && new_dir.as_ref() == current_dir.as_ref() {
            if let Some(new_entry) = Self::build_file_result_from_metadata(preferred_path) {
                self.cached_file_results.push(new_entry);
                self.apply_file_search_sort_mode();
                self.recompute_file_search_display_indices();
            }
        } else {
            let AppView::FileSearchView { query, .. } = &self.current_view else {
                return;
            };
            let query_value = query.clone();
            let results = Self::resolve_file_search_results(&query_value);
            self.update_file_search_results(results);
        }

        let next_index = self
            .file_search_display_index_for_path(preferred_path)
            .or_else(|| {
                let len = self.file_search_display_len();
                (len > 0).then_some(previous_display_index.min(len.saturating_sub(1)))
            });

        if let AppView::FileSearchView {
            ref mut selected_index,
            ..
        } = self.current_view
        {
            *selected_index = next_index.unwrap_or(0);
        }

        Self::resize_file_search_window_for_presentation(
            presentation_value,
            self.file_search_display_indices.len(),
        );
        if let Some(index) = next_index {
            self.file_search_scroll_handle
                .scroll_to_item(index, gpui::ScrollStrategy::Nearest);
        }
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
        let action_id = action_id.strip_prefix("file:").unwrap_or(action_id);
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
                self.clear_file_search_action_target();
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
                            self.clear_file_search_action_target();
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
                            self.clear_file_search_action_target();
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
                        self.clear_file_search_action_target();
                        self.show_hud(
                            "Opened in Editor".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        self.clear_file_search_action_target();
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
                        self.clear_file_search_action_target();
                        self.show_hud(
                            "Opened in Terminal".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        self.clear_file_search_action_target();
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            format!("Failed to open in terminal: {}", e),
                        );
                    }
                }
                DispatchOutcome::success()
            }
            "rename_path" => {
                let Some((path, _, _name)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                let previous_display_index = match &self.current_view {
                    AppView::FileSearchView { selected_index, .. } => *selected_index,
                    _ => 0,
                };

                cx.spawn(async move |this, cx| {
                    let new_name = match crate::file_search::prompt_rename_target_name(&path) {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            let _ = this.update(cx, |this, cx| {
                                this.clear_file_search_action_target();
                                this.restore_file_search_input_focus(cx);
                            });
                            return;
                        }
                        Err(e) => {
                            let _ = this.update(cx, |this, cx| {
                                this.clear_file_search_action_target();
                                this.show_error_toast(
                                    format!("Failed to rename: {}", e),
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
                            });
                            return;
                        }
                    };

                    let _ = this.update(cx, |this, cx| {
                        match crate::file_search::rename_path(&path, &new_name) {
                            Ok(new_path) => {
                                this.clear_file_search_action_target();
                                this.show_hud(
                                    format!("Renamed to {}", new_name),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                this.refresh_file_search_after_mutation(
                                    &path,
                                    Some(&new_path),
                                    previous_display_index,
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
                            }
                            Err(e) => {
                                this.file_search_actions_path = None;
                                this.show_error_toast(
                                    format!("Failed to rename: {}", e),
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
                            }
                        }
                    });
                })
                .detach();

                DispatchOutcome::success()
            }
            "move_path" => {
                let Some((path, is_dir, _name)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                let previous_display_index = match &self.current_view {
                    AppView::FileSearchView { selected_index, .. } => *selected_index,
                    _ => 0,
                };

                cx.spawn(async move |this, cx| {
                    let destination_dir =
                        match crate::file_search::prompt_move_destination_dir(&path, is_dir) {
                            Ok(Some(value)) => value,
                            Ok(None) => {
                                let _ = this.update(cx, |this, cx| {
                                    this.clear_file_search_action_target();
                                    this.restore_file_search_input_focus(cx);
                                });
                                return;
                            }
                            Err(e) => {
                                let _ = this.update(cx, |this, cx| {
                                    this.clear_file_search_action_target();
                                    this.show_error_toast(
                                        format!("Failed to move: {}", e),
                                        cx,
                                    );
                                    this.restore_file_search_input_focus(cx);
                                });
                                return;
                            }
                        };

                    let _ = this.update(cx, |this, cx| {
                        match crate::file_search::move_path(&path, &destination_dir) {
                            Ok(new_path) => {
                                this.clear_file_search_action_target();
                                this.show_hud(
                                    format!(
                                        "Moved to {}",
                                        crate::file_search::shorten_path(&destination_dir)
                                    ),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                this.refresh_file_search_after_mutation(
                                    &path,
                                    Some(&new_path),
                                    previous_display_index,
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
                            }
                            Err(e) => {
                                this.clear_file_search_action_target();
                                this.show_error_toast(
                                    format!("Failed to move: {}", e),
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
                            }
                        }
                    });
                })
                .detach();

                DispatchOutcome::success()
            }
            "move_to_trash" => {
                let Some((path, _, name)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                let previous_display_index = match &self.current_view {
                    AppView::FileSearchView { selected_index, .. } => *selected_index,
                    _ => 0,
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
                            let _ = this.update(cx, |this, cx| {
                                this.clear_file_search_action_target();
                                this.restore_file_search_input_focus(cx);
                            });
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
                                this.clear_file_search_action_target();
                                this.show_error_toast_with_code(
                                    "Failed to open confirmation dialog",
                                    Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
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
                                this.clear_file_search_action_target();
                                this.show_hud(
                                    format!("Moved to Trash: {}", name),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                this.refresh_file_search_after_mutation(
                                    &path,
                                    None,
                                    previous_display_index,
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
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
                                this.clear_file_search_action_target();
                                this.show_error_toast(
                                    format!("Failed to move to Trash: {}", e),
                                    cx,
                                );
                                this.restore_file_search_input_focus(cx);
                            }
                        }
                    });
                })
                .detach();

                DispatchOutcome::success()
            }
            "duplicate_path" => {
                let Some((path, _, name)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                let previous_display_index = match &self.current_view {
                    AppView::FileSearchView { selected_index, .. } => *selected_index,
                    _ => 0,
                };
                match crate::file_search::duplicate_path(&path) {
                    Ok(new_path) => {
                        self.clear_file_search_action_target();
                        self.show_hud(
                            format!("Duplicated {}", name),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                        self.refresh_file_search_after_insert(
                            &new_path,
                            previous_display_index,
                            cx,
                        );
                        self.restore_file_search_input_focus(cx);
                    }
                    Err(e) => {
                        self.clear_file_search_action_target();
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            format!("Failed to duplicate: {}", e),
                        );
                    }
                }
                DispatchOutcome::success()
            }
            "copy_filename" => {
                let Some((_path, _is_dir, name)) = self.resolve_file_search_path_info() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No file selected",
                    );
                };
                tracing::info!(category = "UI", filename = %name, "copy filename");
                self.clear_file_search_action_target();
                self.copy_to_clipboard_with_feedback(
                    &name,
                    format!("Copied filename: {}", name),
                    true,
                    cx,
                );
                self.restore_file_search_input_focus(cx);
                DispatchOutcome::success()
            }
            // ── Current-directory actions ────────────────────────────────
            "sort_name_asc" | "sort_name_desc" | "sort_modified_desc" | "sort_modified_asc" => {
                let preferred_selected_path = self.current_file_search_selected_path();
                let mode = match action_id {
                    "sort_name_asc" => crate::actions::FileSearchSortMode::NameAsc,
                    "sort_name_desc" => crate::actions::FileSearchSortMode::NameDesc,
                    "sort_modified_desc" => crate::actions::FileSearchSortMode::ModifiedDesc,
                    "sort_modified_asc" => crate::actions::FileSearchSortMode::ModifiedAsc,
                    _ => unreachable!(),
                };
                tracing::info!(
                    category = "FILE_SEARCH",
                    event = "sort_action_selected",
                    action = action_id,
                    ?mode,
                    selected_path = preferred_selected_path.as_deref().unwrap_or(""),
                    cached_count = self.cached_file_results.len(),
                    "Applying file-search sort action"
                );
                self.file_search_sort_mode = mode;
                self.apply_file_search_sort_mode();
                self.recompute_file_search_display_indices();
                self.restore_file_search_selection_after_results_change(
                    preferred_selected_path.as_deref(),
                );
                // Scroll the preserved selection back into view after resort.
                if let AppView::FileSearchView { selected_index, .. } = &self.current_view {
                    self.file_search_scroll_handle
                        .scroll_to_item(*selected_index, gpui::ScrollStrategy::Nearest);
                }
                let label = match mode {
                    crate::actions::FileSearchSortMode::NameAsc => "Sorted by Name (A\u{2192}Z)",
                    crate::actions::FileSearchSortMode::NameDesc => "Sorted by Name (Z\u{2192}A)",
                    crate::actions::FileSearchSortMode::ModifiedDesc => "Sorted by Modified (Newest)",
                    crate::actions::FileSearchSortMode::ModifiedAsc => "Sorted by Modified (Oldest)",
                };
                self.show_hud(label.to_string(), Some(HUD_SHORT_MS), cx);
                self.restore_file_search_input_focus(cx);
                cx.notify();
                DispatchOutcome::success()
            }
            "refresh_directory" => {
                let Some(dir) = self.current_file_search_directory_abs() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No current directory to refresh",
                    );
                };
                let (query, presentation) = if let AppView::FileSearchView {
                    query, presentation, ..
                } = &self.current_view
                {
                    (query.clone(), *presentation)
                } else {
                    (format!("{dir}/"), FileSearchPresentation::Mini)
                };
                let frozen_filter = crate::file_search::parse_directory_path(&query)
                    .map(|parsed| parsed.filter)
                    .unwrap_or(None);
                self.restart_file_search_stream_for_query(
                    query,
                    presentation,
                    Some(frozen_filter),
                    true,
                    cx,
                );
                self.show_hud("Refreshed Directory".to_string(), Some(HUD_SHORT_MS), cx);
                self.restore_file_search_input_focus(cx);
                DispatchOutcome::success()
            }
            "reveal_current_directory" => {
                let Some(dir) = self.current_file_search_directory_abs() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No current directory to reveal",
                    );
                };
                match crate::file_search::reveal_in_finder(&dir) {
                    Ok(()) => {
                        self.show_hud("Opened in Finder".to_string(), Some(HUD_SHORT_MS), cx);
                        DispatchOutcome::success()
                    }
                    Err(e) => DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        format!("Failed to reveal current directory: {e}"),
                    ),
                }
            }
            "copy_current_directory_path" => {
                let Some(dir) = self.current_file_search_directory_abs() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No current directory to copy",
                    );
                };
                self.copy_to_clipboard_with_feedback(
                    &dir,
                    format!("Copied: {dir}"),
                    true,
                    cx,
                );
                DispatchOutcome::success()
            }
            "open_current_directory_in_terminal" => {
                let Some(dir) = self.current_file_search_directory_abs() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No current directory to open",
                    );
                };
                match crate::file_search::open_in_terminal(&dir, true) {
                    Ok(_) => {
                        self.show_hud(
                            "Opened in Terminal".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                        DispatchOutcome::success()
                    }
                    Err(e) => DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        format!("Failed to open current directory in terminal: {e}"),
                    ),
                }
            }
            _ => DispatchOutcome::not_handled(),
        }
    }

    /// Compare two file-search results according to the given sort mode.
    ///
    /// Directories always sort before files regardless of mode.
    /// This is the single source of truth for file-search ordering.
    fn compare_file_search_results_for_mode(
        mode: crate::actions::FileSearchSortMode,
        a: &crate::file_search::FileResult,
        b: &crate::file_search::FileResult,
    ) -> std::cmp::Ordering {
        let a_is_dir = matches!(a.file_type, crate::file_search::FileType::Directory);
        let b_is_dir = matches!(b.file_type, crate::file_search::FileType::Directory);
        match (a_is_dir, b_is_dir) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }
        match mode {
            crate::actions::FileSearchSortMode::NameAsc => {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
            crate::actions::FileSearchSortMode::NameDesc => {
                b.name.to_lowercase().cmp(&a.name.to_lowercase())
            }
            crate::actions::FileSearchSortMode::ModifiedDesc => {
                b.modified
                    .cmp(&a.modified)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            }
            crate::actions::FileSearchSortMode::ModifiedAsc => {
                a.modified
                    .cmp(&b.modified)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            }
        }
    }

    /// Apply the current sort mode to cached file results.
    fn apply_file_search_sort_mode(&mut self) {
        let mode = self.file_search_sort_mode;
        self.cached_file_results
            .sort_by(|a, b| Self::compare_file_search_results_for_mode(mode, a, b));
        tracing::info!(
            category = "FILE_SEARCH",
            event = "apply_file_search_sort_mode",
            ?mode,
            cached_count = self.cached_file_results.len(),
            "Applied file-search sort mode to cached results"
        );
    }
}
