            "copy_content" => {
                logging::log("UI", "Copy content action");
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
                            // Extract just the path without the anchor (e.g., "/path/to/file.md#slug" -> "/path/to/file.md")
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
                                #[cfg(target_os = "macos")]
                                {
                                    match self.pbcopy(&content) {
                                        Ok(_) => {
                                            logging::log(
                                                "UI",
                                                &format!(
                                                    "Copied content to clipboard from: {}",
                                                    file_path
                                                ),
                                            );
                                            self.show_hud(
                                                "Content copied to clipboard".to_string(),
                                                Some(HUD_MEDIUM_MS),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                            self.show_hud(
                                                "Failed to copy content".to_string(),
                                                Some(HUD_LONG_MS),
                                                cx,
                                            );
                                        }
                                    }
                                }

                                #[cfg(not(target_os = "macos"))]
                                {
                                    use arboard::Clipboard;
                                    match Clipboard::new().and_then(|mut c| c.set_text(&content)) {
                                        Ok(_) => {
                                            logging::log(
                                                "UI",
                                                &format!(
                                                    "Copied content to clipboard from: {}",
                                                    file_path
                                                ),
                                            );
                                            self.show_hud(
                                                "Content copied to clipboard".to_string(),
                                                Some(HUD_MEDIUM_MS),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            logging::log(
                                                "ERROR",
                                                &format!("Failed to copy content: {}", e),
                                            );
                                            self.show_hud(
                                                "Failed to copy content".to_string(),
                                                Some(HUD_LONG_MS),
                                                cx,
                                            );
                                        }
                                    }
                                }
                                self.hide_main_and_reset(cx);
                            }
                            Err(e) => {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to read file {}: {}", file_path, e),
                                );
                                self.show_hud(
                                    format!("Failed to read file: {}", e),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot copy content for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "reset_ranking" => {
                logging::log("UI", "Reset ranking action");
                // Get the frecency path from the focused script info
                if let Some(script_info) = self.get_focused_script_info() {
                    if let Some(ref frecency_path) = script_info.frecency_path {
                        // Remove the frecency entry for this item
                        if self.frecency_store.remove(frecency_path).is_some() {
                            // Save the updated frecency store
                            if let Err(e) = self.frecency_store.save() {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to save frecency after reset: {}", e),
                                );
                            }
                            // Invalidate the grouped cache AND refresh scripts to rebuild the list
                            // This ensures the item is immediately removed from the Suggested section
                            self.invalidate_grouped_cache();
                            self.refresh_scripts(cx);
                            logging::log("UI", &format!("Reset ranking for: {}", script_info.name));
                            self.show_hud(
                                format!("Ranking reset for \"{}\"", script_info.name),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        } else {
                            logging::log(
                                "UI",
                                &format!("No frecency entry found for: {}", frecency_path),
                            );
                            self.show_hud(
                                "Item has no ranking to reset".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud("Item has no ranking to reset".to_string(), Some(HUD_MEDIUM_MS), cx);
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
                // Don't hide main window - stay in the main menu so user can see the change
                // The actions dialog is already closed by setting current_view = AppView::ScriptList
                // at the start of handle_action()
            }
            // Handle scriptlet actions defined via H3 headers
            action_id if action_id.starts_with("scriptlet_action:") => {
                let action_command = action_id.strip_prefix("scriptlet_action:").unwrap_or("");
                logging::log(
                    "UI",
                    &format!("Scriptlet action triggered: {}", action_command),
                );

                // Find the scriptlet and execute its action
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(scriptlet_match) = result {
                        // Get the file path from the UI scriptlet type
                        // The file_path contains path#slug format
                        let file_path = scriptlet_match.scriptlet.file_path.clone();
                        let scriptlet_command = scriptlet_match.scriptlet.command.clone();

                        // We need to re-parse the markdown file to get the full scriptlet with actions
                        // because scripts::types::Scriptlet is a simplified type without actions
                        let action_found = if let Some(ref path_with_anchor) = file_path {
                            // Extract just the file path (before #anchor)
                            let file_only = path_with_anchor
                                .split('#')
                                .next()
                                .unwrap_or(path_with_anchor);

                            // Read and parse the markdown file
                            if let Ok(content) = std::fs::read_to_string(file_only) {
                                let parsed_scriptlets = scriptlets::parse_markdown_as_scriptlets(
                                    &content,
                                    Some(file_only),
                                );

                                // Find the matching scriptlet by command
                                let target_command = scriptlet_command.clone().unwrap_or_default();
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
                                                description: action.description.clone(),
                                                ..Default::default()
                                            },
                                            typed_metadata: None,
                                            schema: None,
                                            kit: full_scriptlet.kit.clone(),
                                            source_path: full_scriptlet.source_path.clone(),
                                            actions: vec![], // Actions don't have nested actions
                                        };

                                        // Pass the parent scriptlet's content to the action
                                        // This allows actions to use {{content}} to access the
                                        // parent's code (e.g., the URL for `open` tool scriptlets)
                                        let mut inputs = std::collections::HashMap::new();
                                        inputs.insert(
                                            "content".to_string(),
                                            full_scriptlet.scriptlet_content.trim().to_string(),
                                        );
                                        let options = executor::ScriptletExecOptions {
                                            inputs,
                                            ..Default::default()
                                        };
                                        match executor::run_scriptlet(&action_scriptlet, options) {
                                            Ok(exec_result) => {
                                                if exec_result.success {
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Scriptlet action '{}' executed successfully",
                                                            action.name
                                                        ),
                                                    );
                                                    self.show_hud(
                                                        format!("Executed: {}", action.name),
                                                        Some(HUD_MEDIUM_MS),
                                                        cx,
                                                    );
                                                } else {
                                                    let error_msg = if exec_result.stderr.is_empty()
                                                    {
                                                        "Unknown error".to_string()
                                                    } else {
                                                        exec_result.stderr.clone()
                                                    };
                                                    logging::log(
                                                        "ERROR",
                                                        &format!(
                                                            "Scriptlet action '{}' failed: {}",
                                                            action.name, error_msg
                                                        ),
                                                    );
                                                    self.show_hud(
                                                        format!("Error: {}", error_msg),
                                                        Some(HUD_LONG_MS),
                                                        cx,
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                logging::log(
                                                    "ERROR",
                                                    &format!(
                                                        "Failed to execute scriptlet action '{}': {}",
                                                        action.name, e
                                                    ),
                                                );
                                                self.show_hud(
                                                    format!("Error: {}", e),
                                                    Some(HUD_LONG_MS),
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
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to read scriptlet file: {}", file_only),
                                );
                                false
                            }
                        } else {
                            false
                        };

                        if !action_found {
                            logging::log(
                                "ERROR",
                                &format!("Scriptlet action not found: {}", action_command),
                            );
                            self.show_hud("Scriptlet action not found".to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
