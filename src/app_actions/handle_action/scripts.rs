// Script management action handlers for handle_action dispatch.
//
// Contains: create_script, run_script, view_logs, edit_script,
// remove_script/delete_script, reload_scripts, copy_content,
// reset_ranking, settings, quit.

impl ScriptListApp {
    /// Handle script management actions. Returns `DispatchOutcome` indicating if handled.
    fn handle_script_action(
        &mut self,
        action_id: &str,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let trace_id = &dctx.trace_id;
        match action_id {
            "create_script" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "create script action - opening scripts folder");
                let scripts_dir = shellexpand::tilde("~/.scriptkit/scripts").to_string();
                let trace_id = trace_id.to_string();
                let start = std::time::Instant::now();
                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move {
                            use std::process::Command;
                            Command::new("open").arg(&scripts_dir).spawn()
                        })
                        .await;
                    let _ = this.update(cx, |this, cx| match result {
                        Ok(_) => {
                            tracing::info!(
                                category = "UI",
                                trace_id = %trace_id,
                                status = "completed",
                                duration_ms = start.elapsed().as_millis() as u64,
                                "Async action completed: create_script"
                            );
                            this.show_hud(
                                "Opened scripts folder".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            this.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            tracing::error!(
                                trace_id = %trace_id,
                                status = "failed",
                                duration_ms = start.elapsed().as_millis() as u64,
                                error = %e,
                                "Async action failed: create_script"
                            );
                            this.show_error_toast(
                                format!("Failed to open scripts folder: {}", e),
                                cx,
                            );
                        }
                    });
                })
                .detach();
                DispatchOutcome::success()
            }
            "run_script" => {
                tracing::info!(category = "UI", "run script action");
                self.execute_selected(cx);
                DispatchOutcome::success()
            }
            "view_logs" => {
                tracing::info!(category = "UI", "view logs action");
                self.toggle_logs(cx);
                DispatchOutcome::success()
            }
            "edit_script" => {
                tracing::info!(category = "UI", "edit script action");
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => None,
                        scripts::SearchResult::BuiltIn(_) => None,
                        scripts::SearchResult::App(_) => None,
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(_) => None,
                    };

                    if let Some(path) = path_opt {
                        let editor_launch_rx = self.launch_editor_with_feedback_async(&path, trace_id);
                        let trace_id = trace_id.to_string();
                        let start = std::time::Instant::now();
                        cx.spawn(async move |this, cx| {
                            let Ok(launch_result) = editor_launch_rx.recv().await else {
                                return;
                            };

                            let _ = this.update(cx, |this, cx| match launch_result {
                                Ok(()) => {
                                    tracing::info!(
                                        trace_id = %trace_id,
                                        status = "completed",
                                        duration_ms = start.elapsed().as_millis() as u64,
                                        "Async action completed: edit_script"
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                                Err(message) => {
                                    tracing::error!(
                                        trace_id = %trace_id,
                                        status = "failed",
                                        duration_ms = start.elapsed().as_millis() as u64,
                                        error = %message,
                                        "Async action failed: edit_script"
                                    );
                                    this.show_error_toast_with_code(
                                        message,
                                        Some(crate::action_helpers::ERROR_LAUNCH_FAILED),
                                        cx,
                                    );
                                }
                            });
                        })
                        .detach();
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Cannot edit this item type",
                        );
                    }
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            "remove_script" | "delete_script" => {
                tracing::info!(category = "UI", action = action_id, "action triggered");

                let Some(result) = self.get_selected_result() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                };

                let Some(target) = script_removal_target_from_result(&result) else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "Cannot remove this item type",
                    );
                };

                if !target.path.exists() {
                    self.refresh_scripts(cx);
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        format!("{} no longer exists", target.name),
                    );
                }

                let message = format!(
                    "Move this {} to Trash?\n\n{}",
                    target.item_kind, target.name
                );

                let trace_id = trace_id.to_string();
                let start = std::time::Instant::now();
                cx.spawn(async move |this, cx| {
                    match confirm_with_modal(cx, message, "Move to Trash", "Cancel", &trace_id).await {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::info!(
                                trace_id = %trace_id,
                                status = "cancelled",
                                duration_ms = start.elapsed().as_millis() as u64,
                                "Async action cancelled: remove_script"
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

                    let _ =
                        this.update(cx, move |this, cx| match move_path_to_trash(&target.path) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "UI",
                                    trace_id = %trace_id,
                                    status = "completed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    item_kind = target.item_kind,
                                    name = %target.name,
                                    path = %target.path.display(),
                                    "Async action completed: remove_script"
                                );
                                this.refresh_scripts(cx);
                                this.show_hud(
                                    format!("Moved '{}' to Trash", target.name),
                                    Some(HUD_2200_MS),
                                    cx,
                                );
                                this.hide_main_and_reset(cx);
                                cx.notify();
                            }
                            Err(e) => {
                                tracing::error!(
                                    trace_id = %trace_id,
                                    status = "failed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    item_kind = target.item_kind,
                                    name = %target.name,
                                    path = %target.path.display(),
                                    error = %e,
                                    "Async action failed: remove_script"
                                );
                                this.show_error_toast_with_code(
                                    format!("Failed to remove: {}", e),
                                    Some(crate::action_helpers::ERROR_TRASH_FAILED),
                                    cx,
                                );
                            }
                        });
                })
                .detach();
                DispatchOutcome::success()
            }
            "reload_scripts" => {
                tracing::info!(category = "UI", "reload scripts action");
                self.refresh_scripts(cx);
                self.show_hud("Scripts reloaded".to_string(), Some(HUD_SHORT_MS), cx);
                DispatchOutcome::success()
            }
            "settings" => {
                tracing::info!(category = "UI", "settings action - opening config.ts");

                // Get editor from config
                let editor = self.config.get_editor();
                let config_dir = shellexpand::tilde("~/.scriptkit/kit").to_string();
                let config_file = format!("{}/config.ts", config_dir);

                let editor_for_hud = editor.clone();
                let trace_id = trace_id.to_string();
                let start = std::time::Instant::now();

                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move {
                            use std::process::Command;

                            // Editor-specific arguments for opening folder with file focused
                            match editor.as_str() {
                                // VS Code and Cursor: -r (reuse window) + folder + file
                                "code" | "cursor" => Command::new(&editor)
                                    .arg("-r")
                                    .arg(&config_dir)
                                    .arg(&config_file)
                                    .spawn(),
                                // Zed: just the file (doesn't support folder context the same way)
                                "zed" => Command::new("zed").arg(&config_file).spawn(),
                                // Sublime: -a (add to current window) + folder + file
                                "subl" => Command::new("subl")
                                    .arg("-a")
                                    .arg(&config_dir)
                                    .arg(&config_file)
                                    .spawn(),
                                // Generic fallback: just open the file
                                _ => Command::new(&editor).arg(&config_file).spawn(),
                            }
                        })
                        .await;
                    let _ = this.update(cx, |this, cx| match result {
                        Ok(_) => {
                            tracing::info!(
                                category = "UI",
                                trace_id = %trace_id,
                                status = "completed",
                                duration_ms = start.elapsed().as_millis() as u64,
                                editor = %editor_for_hud,
                                "Async action completed: settings"
                            );
                            this.show_hud(
                                format!("Opening config.ts in {}", editor_for_hud),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            this.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            tracing::error!(
                                trace_id = %trace_id,
                                status = "failed",
                                duration_ms = start.elapsed().as_millis() as u64,
                                editor = %editor_for_hud,
                                error = %e,
                                "Async action failed: settings"
                            );
                            this.show_error_toast(
                                format!(
                                    "Failed to open {} for settings: {}",
                                    editor_for_hud, e
                                ),
                                cx,
                            );
                        }
                    });
                })
                .detach();
                DispatchOutcome::success()
            }
            "quit" => {
                tracing::info!(category = "UI", "quit action");
                PROCESS_MANAGER.kill_all_processes();
                PROCESS_MANAGER.remove_main_pid();
                cx.quit();
                DispatchOutcome::success()
            }
            "copy_content" => {
                tracing::info!(category = "UI", "copy content action");
                if let Some(result) = self.get_selected_result() {
                    // Get the file path based on the result type
                    let file_path_opt: Option<String> = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(m.script.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Agent(m) => {
                            Some(m.agent.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            // Extract just the path without the anchor
                            m.scriptlet
                                .file_path
                                .as_ref()
                                .map(|p| p.split('#').next().unwrap_or(p).to_string())
                        }
                        _ => None,
                    };

                    if let Some(file_path) = file_path_opt {
                        // Read the file content
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => {
                                tracing::info!(category = "UI", path = %file_path, "copying content to clipboard");
                                self.copy_to_clipboard_with_feedback(
                                    &content,
                                    "Content copied to clipboard".to_string(),
                                    true,
                                    cx,
                                );
                            }
                            Err(e) => {
                                tracing::error!(path = %file_path, error = %e, "failed to read file");
                                return DispatchOutcome::error(
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    format!("Failed to read file: {}", e),
                                );
                            }
                        }
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Cannot copy content for this item type",
                        );
                    }
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            "reset_ranking" => {
                tracing::info!(category = "UI", "reset ranking action");
                // Get the frecency path from the focused script info
                if let Some(script_info) = self.get_focused_script_info() {
                    if let Some(ref frecency_path) = script_info.frecency_path {
                        // Remove the frecency entry for this item
                        if self.frecency_store.remove(frecency_path).is_some() {
                            // Save the updated frecency store
                            if let Err(e) = self.frecency_store.save() {
                                tracing::error!(
                                    error = %e,
                                    "failed to save frecency after reset"
                                );
                            }
                            // Invalidate the grouped cache AND refresh scripts to rebuild the list
                            self.invalidate_grouped_cache();
                            self.refresh_scripts(cx);
                            tracing::info!(category = "UI", name = %script_info.name, "reset ranking");
                            self.show_hud(
                                format!("Ranking reset for \"{}\"", script_info.name),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        } else {
                            tracing::info!(category = "UI", frecency_path = %frecency_path, "no frecency entry found");
                            self.show_hud(
                                "Item has no ranking to reset".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Item has no ranking to reset",
                        );
                    }
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                // Don't hide main window - stay in the main menu so user can see the change
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
