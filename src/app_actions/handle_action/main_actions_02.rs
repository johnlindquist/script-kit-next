            "add_alias" | "update_alias" => {
                logging::log("UI", &format!("{} action", action_id));
                if let Some(result) = self.get_selected_result() {
                    let (command_id, command_name) = match result {
                        scripts::SearchResult::Script(m) => {
                            (format!("script/{}", m.script.name), m.script.name.clone())
                        }
                        scripts::SearchResult::Scriptlet(m) => (
                            format!("scriptlet/{}", m.scriptlet.name),
                            m.scriptlet.name.clone(),
                        ),
                        scripts::SearchResult::BuiltIn(m) => {
                            (format!("builtin/{}", m.entry.id), m.entry.name.clone())
                        }
                        scripts::SearchResult::App(m) => {
                            let id = if let Some(ref bundle_id) = m.app.bundle_id {
                                format!("app/{}", bundle_id)
                            } else {
                                format!("app/{}", m.app.name.to_lowercase().replace(' ', "-"))
                            };
                            (id, m.app.name.clone())
                        }
                        scripts::SearchResult::Agent(m) => {
                            (format!("agent/{}", m.agent.name), m.agent.name.clone())
                        }
                        scripts::SearchResult::Window(_) => {
                            self.show_hud(
                                "Window aliases not supported - windows are transient".to_string(),
                                Some(2500),
                                cx,
                            );
                            return;
                        }
                        scripts::SearchResult::Fallback(m) => (
                            format!("fallback/{}", m.fallback.name()),
                            m.fallback.name().to_string(),
                        ),
                    };
                    self.show_alias_input(command_id, command_name, cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(2000),
                        cx,
                    );
                }
            }
            // "remove_alias" removes the existing alias from persistence
            "remove_alias" => {
                logging::log("UI", "Remove alias action");
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
                        // Remove the alias override from persistence
                        match crate::aliases::remove_alias_override(&command_id) {
                            Ok(()) => {
                                logging::log(
                                    "ALIAS",
                                    &format!("Removed alias for: {}", command_id),
                                );
                                self.show_hud("Alias removed".to_string(), Some(2000), cx);
                                // Refresh scripts to update alias display and registry
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to remove alias: {}", e));
                                self.show_hud(
                                    format!("Failed to remove alias: {}", e),
                                    Some(3000),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot remove alias for this item type".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(2000),
                        cx,
                    );
                }
            }
            "edit_script" => {
                logging::log("UI", "Edit script action");
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
                        self.edit_script(&path);
                        self.hide_main_and_reset(cx);
                    } else {
                        self.show_hud("Cannot edit this item type".to_string(), Some(2000), cx);
                    }
                } else {
                    self.show_hud("No script selected".to_string(), Some(2000), cx);
                }
            }
            "remove_script" | "delete_script" => {
                logging::log("UI", &format!("{} action", action_id));

                let Some(result) = self.get_selected_result() else {
                    self.show_hud("No script selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(target) = script_removal_target_from_result(&result) else {
                    self.show_hud("Cannot remove this item type".to_string(), Some(2500), cx);
                    return;
                };

                if !target.path.exists() {
                    self.show_hud(format!("{} no longer exists", target.name), Some(2500), cx);
                    self.refresh_scripts(cx);
                    return;
                }

                let message = format!(
                    "Move this {} to Trash?\n\n{}",
                    target.item_kind, target.name
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
                            Some("Move to Trash".to_string()),
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

                    this.update(cx, move |this, cx| match move_path_to_trash(&target.path) {
                        Ok(()) => {
                            logging::log(
                                "UI",
                                &format!(
                                    "Moved {} '{}' to trash: {}",
                                    target.item_kind,
                                    target.name,
                                    target.path.display()
                                ),
                            );
                            this.refresh_scripts(cx);
                            this.show_hud(
                                format!("Moved '{}' to Trash", target.name),
                                Some(2200),
                                cx,
                            );
                            this.hide_main_and_reset(cx);
                            cx.notify();
                        }
                        Err(e) => {
                            logging::log(
                                "ERROR",
                                &format!(
                                    "Failed to move {} '{}' to trash ({}): {}",
                                    target.item_kind,
                                    target.name,
                                    target.path.display(),
                                    e
                                ),
                            );
                            this.show_hud(format!("Failed to remove: {}", e), Some(3200), cx);
                        }
                    })
                    .ok();
                })
                .detach();
                return;
            }
            "reload_scripts" => {
                logging::log("UI", "Reload scripts action");
                self.refresh_scripts(cx);
                self.show_hud("Scripts reloaded".to_string(), Some(1500), cx);
            }
            "settings" => {
                logging::log("UI", "Settings action - opening config.ts");

                // Get editor from config
                let editor = self.config.get_editor();
                let config_dir = shellexpand::tilde("~/.scriptkit/kit").to_string();
                let config_file = format!("{}/config.ts", config_dir);

                // Clone editor for HUD message before moving into thread
                let editor_for_hud = editor.clone();

                // Spawn editor in background thread
                std::thread::spawn(move || {
                    use std::process::Command;

                    // Editor-specific arguments for opening folder with file focused
                    let result = match editor.as_str() {
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
                    };

                    match result {
                        Ok(_) => logging::log("UI", &format!("Opened config.ts in {}", editor)),
                        Err(e) => logging::log(
                            "ERROR",
                            &format!("Failed to open editor '{}': {}", editor, e),
                        ),
                    }
                });

                self.show_hud(
                    format!("Opening config.ts in {}", editor_for_hud),
                    Some(1500),
                    cx,
                );
                self.hide_main_and_reset(cx);
            }
            "quit" => {
                logging::log("UI", "Quit action");
                PROCESS_MANAGER.kill_all_processes();
                PROCESS_MANAGER.remove_main_pid();
                cx.quit();
                return; // Early return after quit - no notify needed
            }
            "__cancel__" => {
                logging::log("UI", "Actions dialog cancelled");
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
            }
            // File search specific actions
