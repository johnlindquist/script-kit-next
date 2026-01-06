// Actions handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: handle_action, trigger_action_by_name

impl ScriptListApp {
    /// Helper to hide main window and set reset flag
    fn hide_main_and_reset(&self, cx: &mut Context<Self>) {
        set_main_window_visible(false);
        NEEDS_RESET.store(true, Ordering::SeqCst);
        cx.hide();
    }

    /// Helper to reveal a path in Finder (macOS)
    fn reveal_in_finder(&self, path: &std::path::Path) {
        let path_str = path.to_string_lossy().to_string();
        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new("open").arg("-R").arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Revealed in Finder: {}", path_str)),
                Err(e) => logging::log("ERROR", &format!("Failed to reveal in Finder: {}", e)),
            }
        });
    }

    /// Copy text to clipboard using pbcopy on macOS.
    /// Critical: This properly closes stdin before waiting to prevent hangs.
    #[cfg(target_os = "macos")]
    fn pbcopy(&self, text: &str) -> Result<(), std::io::Error> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

        // Take ownership of stdin, write, then drop to signal EOF
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
            // stdin is dropped here => EOF delivered to pbcopy
        }

        // Now it's safe to wait - pbcopy has received EOF
        child.wait()?;
        Ok(())
    }

    /// Handle action selection from the actions dialog
    fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
        logging::log("UI", &format!("Action selected: {}", action_id));

        // Close the dialog and return to script list
        self.current_view = AppView::ScriptList;
        self.pending_focus = Some(FocusTarget::MainFilter);

        match action_id.as_str() {
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
                self.last_output = Some(SharedString::from("Opened scripts folder"));
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
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::App(m) => Some(m.app.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal scriptlets in Finder"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal built-in features"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal windows in Finder"));
                            None
                        }
                        scripts::SearchResult::Fallback(_) => {
                            self.last_output = Some(SharedString::from(
                                "Cannot reveal fallback commands in Finder",
                            ));
                            None
                        }
                    };

                    if let Some(path) = path_opt {
                        self.reveal_in_finder(&path);
                        self.last_output = Some(SharedString::from("Revealed in Finder"));
                        self.hide_main_and_reset(cx);
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "copy_path" => {
                logging::log("UI", "Copy path action");
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::App(m) => Some(m.app.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy scriptlet path"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy built-in path"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot copy window path"));
                            None
                        }
                        scripts::SearchResult::Fallback(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy fallback command path"));
                            None
                        }
                    };

                    if let Some(path) = path_opt {
                        let path_str = path.to_string_lossy().to_string();

                        #[cfg(target_os = "macos")]
                        {
                            match self.pbcopy(&path_str) {
                                Ok(_) => {
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_str),
                                    );
                                    self.last_output =
                                        Some(SharedString::from(format!("Copied: {}", path_str)));
                                }
                                Err(e) => {
                                    logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
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
                                    self.last_output =
                                        Some(SharedString::from(format!("Copied: {}", path_str)));
                                }
                                Err(e) => {
                                    logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "edit_script" => {
                logging::log("UI", "Edit script action");
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit scriptlets"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot edit built-in features"));
                            None
                        }
                        scripts::SearchResult::App(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit applications"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit windows"));
                            None
                        }
                        scripts::SearchResult::Fallback(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot edit fallback commands"));
                            None
                        }
                    };

                    if let Some(path) = path_opt {
                        self.edit_script(&path);
                        self.hide_main_and_reset(cx);
                    }
                } else {
                    self.last_output = Some(SharedString::from("No script selected"));
                }
            }
            "reload_scripts" => {
                logging::log("UI", "Reload scripts action");
                self.refresh_scripts(cx);
                self.last_output = Some(SharedString::from("Scripts reloaded"));
            }
            "settings" => {
                logging::log("UI", "Settings action");
                self.last_output = Some(SharedString::from("Settings (TODO)"));
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
            }
            _ => {
                // Handle SDK actions using shared helper
                self.trigger_sdk_action_internal(&action_id);
            }
        }

        cx.notify();
    }

    /// Internal helper for triggering SDK actions - used by both handle_action and trigger_action_by_name
    fn trigger_sdk_action_internal(&mut self, action_name: &str) {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                let send_result = if action.has_action {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action with handler: '{}' (has_action=true), sending ActionTriggered",
                            action_name
                        ),
                    );
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::action_triggered(
                            action_name.to_string(),
                            action.value.clone(),
                            self.arg_input.text().to_string(),
                        );
                        Some(sender.try_send(msg))
                    } else {
                        None
                    }
                } else if let Some(ref value) = action.value {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action without handler: '{}' (has_action=false), submitting value: {:?}",
                            action_name, value
                        ),
                    );
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::Submit {
                            id: "action".to_string(),
                            value: Some(value.clone()),
                        };
                        Some(sender.try_send(msg))
                    } else {
                        None
                    }
                } else {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action '{}' has no value and has_action=false",
                            action_name
                        ),
                    );
                    None
                };

                // Log any send errors
                if let Some(result) = send_result {
                    match result {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log(
                                "WARN",
                                &format!(
                                    "Response channel full - action '{}' dropped",
                                    action_name
                                ),
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                }
            } else {
                logging::log("UI", &format!("Unknown action: {}", action_name));
            }
        } else {
            logging::log("UI", &format!("Unknown action: {}", action_name));
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if actions.iter().any(|a| a.name == action_name) {
                logging::log(
                    "ACTIONS",
                    &format!("Triggering SDK action '{}' via shortcut", action_name),
                );
                self.trigger_sdk_action_internal(action_name);
                cx.notify();
                return true;
            }
        }
        false
    }
}
