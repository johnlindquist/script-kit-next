// Scriptlet-specific action handlers for handle_action dispatch.
//
// Contains: edit_scriptlet, reveal_scriptlet_in_finder, copy_scriptlet_path,
// and dynamic scriptlet_action:* handlers.

impl ScriptListApp {
    /// Handle scriptlet-specific actions. Returns `DispatchOutcome` indicating if handled.
    fn handle_scriptlet_action(
        &mut self,
        action_id: &str,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let trace_id = &dctx.trace_id;
        match action_id {
            "edit_scriptlet" => {
                tracing::info!(category = "UI", "edit scriptlet action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str =
                                file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::PathBuf::from(path_str);
                            let editor_launch_rx =
                                self.launch_editor_with_feedback_async(&path, trace_id);
                            let trace_id = trace_id.to_string();
                            let start = std::time::Instant::now();
                            cx.spawn(async move |this, cx| {
                                let Ok(launch_result) = editor_launch_rx.recv().await
                                else {
                                    return;
                                };

                                let _ =
                                    this.update(cx, |this, cx| match launch_result {
                                        Ok(()) => {
                                            tracing::info!(
                                                trace_id = %trace_id,
                                                status = "completed",
                                                duration_ms = start.elapsed().as_millis() as u64,
                                                "Async action completed: edit_scriptlet"
                                            );
                                            this.hide_main_and_reset(cx);
                                        }
                                        Err(message) => {
                                            tracing::error!(
                                                trace_id = %trace_id,
                                                status = "failed",
                                                duration_ms = start.elapsed().as_millis() as u64,
                                                error = %message,
                                                "Async action failed: edit_scriptlet"
                                            );
                                            this.show_error_toast(message, cx);
                                        }
                                    });
                            })
                            .detach();
                        } else {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Scriptlet has no source file path",
                            );
                        }
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Selected item is not a scriptlet",
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
            "reveal_scriptlet_in_finder" => {
                tracing::info!(category = "UI", "reveal scriptlet in Finder action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str =
                                file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::Path::new(path_str);
                            let reveal_result_rx =
                                self.reveal_in_finder_with_feedback_async(path, trace_id);
                            let trace_id = trace_id.to_string();
                            let start = std::time::Instant::now();
                            cx.spawn(async move |this, cx| {
                                let Ok(reveal_result) = reveal_result_rx.recv().await
                                else {
                                    return;
                                };

                                let _ =
                                    this.update(cx, |this, cx| match reveal_result {
                                        Ok(()) => {
                                            tracing::info!(
                                                trace_id = %trace_id,
                                                status = "completed",
                                                duration_ms = start.elapsed().as_millis() as u64,
                                                "Async action completed: reveal_scriptlet_in_finder"
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
                                                "Async action failed: reveal_scriptlet_in_finder"
                                            );
                                            this.show_error_toast(message, cx);
                                        }
                                    });
                            })
                            .detach();
                        } else {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Scriptlet has no source file path",
                            );
                        }
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Selected item is not a scriptlet",
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
            "copy_scriptlet_path" => {
                tracing::info!(category = "UI", "copy scriptlet path action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str =
                                file_path.split('#').next().unwrap_or(file_path);

                            tracing::info!(category = "UI", path = %path_str, "copying scriptlet path to clipboard");
                            self.copy_to_clipboard_with_feedback(
                                path_str,
                                format!("Copied: {}", path_str),
                                true,
                                cx,
                            );
                        } else {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Scriptlet has no source file path",
                            );
                        }
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Selected item is not a scriptlet",
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
            // Handle scriptlet actions defined via H3 headers
            action_id if action_id.starts_with("scriptlet_action:") => {
                let action_command =
                    action_id.strip_prefix("scriptlet_action:").unwrap_or("");
                tracing::info!(category = "UI", action = %action_command, "scriptlet action triggered");

                // Find the scriptlet and execute its action
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(scriptlet_match) = result {
                        // Get the file path from the UI scriptlet type
                        let file_path = scriptlet_match.scriptlet.file_path.clone();
                        let scriptlet_command =
                            scriptlet_match.scriptlet.command.clone();

                        // We need to re-parse the markdown file to get the full scriptlet with actions
                        let action_found = if let Some(ref path_with_anchor) = file_path
                        {
                            // Extract just the file path (before #anchor)
                            let file_only = path_with_anchor
                                .split('#')
                                .next()
                                .unwrap_or(path_with_anchor);

                            // Read and parse the markdown file
                            if let Ok(content) = std::fs::read_to_string(file_only) {
                                let parsed_scriptlets =
                                    scriptlets::parse_markdown_as_scriptlets(
                                        &content,
                                        Some(file_only),
                                    );

                                // Find the matching scriptlet by command
                                let target_command =
                                    scriptlet_command.clone().unwrap_or_default();
                                if let Some(full_scriptlet) = parsed_scriptlets
                                    .iter()
                                    .find(|s| s.command == target_command)
                                {
                                    // Find the action in the scriptlet
                                    if let Some(action) = full_scriptlet
                                        .actions
                                        .iter()
                                        .find(|a| a.command == action_command)
                                    {
                                        // Create a scriptlet for executing the action
                                        let action_scriptlet = scriptlets::Scriptlet {
                                            name: action.name.clone(),
                                            command: action.command.clone(),
                                            tool: action.tool.clone(),
                                            scriptlet_content: action.code.clone(),
                                            inputs: action.inputs.clone(),
                                            group: full_scriptlet.group.clone(),
                                            preview: None,
                                            metadata: scriptlets::ScriptletMetadata {
                                                shortcut: action.shortcut.clone(),
                                                description: action
                                                    .description
                                                    .clone(),
                                                ..Default::default()
                                            },
                                            typed_metadata: None,
                                            schema: None,
                                            kit: full_scriptlet.kit.clone(),
                                            source_path: full_scriptlet
                                                .source_path
                                                .clone(),
                                            actions: vec![],
                                        };

                                        // Pass the parent scriptlet's content to the action
                                        let mut inputs =
                                            std::collections::HashMap::new();
                                        inputs.insert(
                                            "content".to_string(),
                                            full_scriptlet
                                                .scriptlet_content
                                                .trim()
                                                .to_string(),
                                        );
                                        let options =
                                            executor::ScriptletExecOptions {
                                                inputs,
                                                ..Default::default()
                                            };
                                        match executor::run_scriptlet(
                                            &action_scriptlet,
                                            options,
                                        ) {
                                            Ok(exec_result) => {
                                                if exec_result.success {
                                                    tracing::info!(category = "UI", action = %action.name, "scriptlet action executed successfully");
                                                    self.show_hud(
                                                        format!(
                                                            "Executed: {}",
                                                            action.name
                                                        ),
                                                        Some(HUD_MEDIUM_MS),
                                                        cx,
                                                    );
                                                } else {
                                                    let error_msg =
                                                        if exec_result.stderr.is_empty()
                                                        {
                                                            "Unknown error".to_string()
                                                        } else {
                                                            exec_result.stderr.clone()
                                                        };
                                                    tracing::error!(action = %action.name, error = %error_msg, "scriptlet action failed");
                                                    self.show_error_toast(
                                                        format!(
                                                            "Failed to execute action: {}",
                                                            error_msg
                                                        ),
                                                        cx,
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!(action = %action.name, error = %e, "failed to execute scriptlet action");
                                                self.show_error_toast(
                                                    format!("Failed to execute action: {}", e),
                                                    cx,
                                                );
                                            }
                                        }
                                        self.hide_main_and_reset(cx);
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                tracing::error!(path = %file_only, "failed to read scriptlet file");
                                false
                            }
                        } else {
                            false
                        };

                        if !action_found {
                            tracing::error!(action = %action_command, "scriptlet action not found");
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Scriptlet action not found",
                            );
                        }
                    } else {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Selected item is not a scriptlet",
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
            _ => DispatchOutcome::not_handled(),
        }
    }
}
