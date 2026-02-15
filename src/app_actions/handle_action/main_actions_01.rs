            "create_script" => {
                logging::log("UI", "Create script action - opening scripts folder");
                let scripts_dir = shellexpand::tilde("~/.scriptkit/scripts").to_string();
                std::thread::spawn(move || {
                    use std::process::Command;
                    match Command::new("open").arg(&scripts_dir).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened scripts folder: {}", scripts_dir))
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open scripts folder: {}", e))
                        }
                    }
                });
                self.show_hud("Opened scripts folder".to_string(), Some(HUD_SHORT_MS), cx);
                self.hide_main_and_reset(cx);
            }
            "run_script" => {
                logging::log("UI", "Run script action");
                self.execute_selected(cx);
            }
            "view_logs" => {
                logging::log("UI", "View logs action");
                self.toggle_logs(cx);
            }
            "reveal_in_finder" => {
                logging::log("UI", "Reveal in Finder action");
                // First check if we have a file search path (takes priority)
                let path_opt = if let Some(path) = self.file_search_actions_path.take() {
                    logging::log("UI", &format!("Reveal in Finder (file search): {}", path));
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
                    self.reveal_in_finder(&path);
                    self.show_hud("Opened in Finder".to_string(), Some(HUD_SHORT_MS), cx);
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        "Cannot reveal this item type in Finder".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "copy_path" => {
                logging::log("UI", "Copy path action");
                // First check if we have a file search path (takes priority)
                let path_str = if let Some(path) = self.file_search_actions_path.take() {
                    logging::log("UI", &format!("Copy path (file search): {}", path));
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
                        self.show_hud(
                            "Cannot copy path for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    path_opt
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    None
                };

                if let Some(path_str) = path_str {
                    #[cfg(target_os = "macos")]
                    {
                        match self.pbcopy(&path_str) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_str),
                                );
                                self.show_hud(format!("Copied: {}", path_str), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                self.show_hud("Failed to copy path".to_string(), Some(HUD_LONG_MS), cx);
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        match Clipboard::new().and_then(|mut c| c.set_text(&path_str)) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_str),
                                );
                                self.show_hud(format!("Copied: {}", path_str), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                self.show_hud("Failed to copy path".to_string(), Some(HUD_LONG_MS), cx);
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                }
            }
            "copy_deeplink" => {
                logging::log("UI", "Copy deeplink action");
                if let Some(result) = self.get_selected_result() {
                    let name = result.name();
                    let deeplink_name = crate::actions::to_deeplink_name(name);
                    let deeplink_url = format!("scriptkit://run/{}", deeplink_name);

                    #[cfg(target_os = "macos")]
                    {
                        match self.pbcopy(&deeplink_url) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied deeplink to clipboard: {}", deeplink_url),
                                );
                                self.show_hud(format!("Copied: {}", deeplink_url), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                self.show_hud(
                                    "Failed to copy deeplink".to_string(),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        match Clipboard::new().and_then(|mut c| c.set_text(&deeplink_url)) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied deeplink to clipboard: {}", deeplink_url),
                                );
                                self.show_hud(format!("Copied: {}", deeplink_url), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy deeplink: {}", e));
                                self.show_hud(
                                    "Failed to copy deeplink".to_string(),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // Handle both legacy "configure_shortcut" and new dynamic actions
            // "add_shortcut" and "update_shortcut" open the shortcut recorder
            "configure_shortcut" | "add_shortcut" | "update_shortcut" => {
                logging::log("UI", &format!("{} action", action_id));
                if let Some(result) = self.get_selected_result() {
                    match result {
                        // Scripts: open the script file to edit // Shortcut: comment
                        scripts::SearchResult::Script(m) => {
                            self.edit_script(&m.script.path);
                            self.hide_main_and_reset(cx);
                        }
                        scripts::SearchResult::Agent(m) => {
                            self.edit_script(&m.agent.path);
                            self.hide_main_and_reset(cx);
                        }
                        // Non-scripts: show inline shortcut recorder
                        scripts::SearchResult::Scriptlet(m) => {
                            let command_id = format!("scriptlet/{}", m.scriptlet.name);
                            let command_name = m.scriptlet.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            let command_id = format!("builtin/{}", m.entry.id);
                            let command_name = m.entry.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::App(m) => {
                            // Use bundle ID if available, otherwise use name
                            let command_id = if let Some(ref bundle_id) = m.app.bundle_id {
                                format!("app/{}", bundle_id)
                            } else {
                                format!("app/{}", m.app.name.to_lowercase().replace(' ', "-"))
                            };
                            let command_name = m.app.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::Window(_) => {
                            self.show_hud(
                                "Window shortcuts not supported - windows are transient"
                                    .to_string(),
                                Some(HUD_2500_MS),
                                cx,
                            );
                        }
                        scripts::SearchResult::Fallback(m) => {
                            match &m.fallback {
                                crate::fallbacks::collector::FallbackItem::Builtin(b) => {
                                    let command_id = format!("fallback/{}", m.fallback.name());
                                    let command_name = b.name.to_string();
                                    self.show_shortcut_recorder(command_id, command_name, cx);
                                }
                                crate::fallbacks::collector::FallbackItem::Script(s) => {
                                    // Script-based fallback - open the script
                                    self.edit_script(&s.script.path);
                                    self.hide_main_and_reset(cx);
                                }
                            }
                        }
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // "remove_shortcut" removes the existing shortcut from the registry
            "remove_shortcut" => {
                logging::log("UI", "Remove shortcut action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(format!("script/{}", m.script.name))
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            Some(format!("scriptlet/{}", m.scriptlet.name))
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            Some(format!("builtin/{}", m.entry.id))
                        }
                        scripts::SearchResult::App(m) => {
                            if let Some(ref bundle_id) = m.app.bundle_id {
                                Some(format!("app/{}", bundle_id))
                            } else {
                                Some(format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                ))
                            }
                        }
                        scripts::SearchResult::Agent(m) => Some(format!("agent/{}", m.agent.name)),
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(m) => {
                            Some(format!("fallback/{}", m.fallback.name()))
                        }
                    };

                    if let Some(command_id) = command_id_opt {
                        // Remove the shortcut override from persistence
                        match crate::shortcuts::remove_shortcut_override(&command_id) {
                            Ok(()) => {
                                logging::log(
                                    "SHORTCUT",
                                    &format!("Removed shortcut for: {}", command_id),
                                );
                                self.show_hud("Shortcut removed".to_string(), Some(HUD_MEDIUM_MS), cx);
                                // Refresh scripts to update shortcut display
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to remove shortcut: {}", e));
                                self.show_hud(
                                    format!("Failed to remove shortcut: {}", e),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot remove shortcut for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // Alias actions: add_alias, update_alias open the alias input
