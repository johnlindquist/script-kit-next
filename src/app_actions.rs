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
                // First check if we have a file search path (takes priority)
                let path_opt = if let Some(path) = self.file_search_actions_path.take() {
                    logging::log(
                        "UI",
                        &format!("Reveal in Finder (file search): {}", path),
                    );
                    Some(std::path::PathBuf::from(path))
                } else if let Some(result) = self.get_selected_result() {
                    // Fall back to main menu selected result
                    match result {
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
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                    None
                };

                if let Some(path) = path_opt {
                    self.reveal_in_finder(&path);
                    self.last_output = Some(SharedString::from("Revealed in Finder"));
                    self.hide_main_and_reset(cx);
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
                    path_opt
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
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
                                self.last_output =
                                    Some(SharedString::from(format!("Copied: {}", deeplink_url)));
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                self.last_output =
                                    Some(SharedString::from("Failed to copy deeplink"));
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
                                self.last_output =
                                    Some(SharedString::from(format!("Copied: {}", deeplink_url)));
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy deeplink: {}", e));
                                self.last_output =
                                    Some(SharedString::from("Failed to copy deeplink"));
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
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
                            self.last_output = Some(SharedString::from(
                                "Window shortcuts not supported - windows are transient",
                            ));
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
                    self.last_output = Some(SharedString::from("No item selected"));
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
                        scripts::SearchResult::Window(_) => {
                            self.last_output =
                                Some(SharedString::from("Window shortcuts not supported"));
                            None
                        }
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
                                self.last_output = Some(SharedString::from("Shortcut removed"));
                                // Refresh scripts to update shortcut display
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to remove shortcut: {}", e));
                                self.last_output = Some(SharedString::from(format!(
                                    "Failed to remove shortcut: {}",
                                    e
                                )));
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            // Alias actions: add_alias, update_alias open the alias input
            "add_alias" | "update_alias" => {
                logging::log("UI", &format!("{} action", action_id));
                if let Some(result) = self.get_selected_result() {
                    let (command_id, command_name) = match result {
                        scripts::SearchResult::Script(m) => {
                            (format!("script/{}", m.script.name), m.script.name.clone())
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            (format!("scriptlet/{}", m.scriptlet.name), m.scriptlet.name.clone())
                        }
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
                            self.last_output =
                                Some(SharedString::from("Window aliases not supported - windows are transient"));
                            return;
                        }
                        scripts::SearchResult::Fallback(m) => {
                            (format!("fallback/{}", m.fallback.name()), m.fallback.name().to_string())
                        }
                    };
                    self.show_alias_input(command_id, command_name, cx);
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
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
                        scripts::SearchResult::Window(_) => {
                            self.last_output =
                                Some(SharedString::from("Window aliases not supported"));
                            None
                        }
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
                                self.last_output = Some(SharedString::from("Alias removed"));
                                // Refresh scripts to update alias display and registry
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to remove alias: {}", e));
                                self.last_output = Some(SharedString::from(format!(
                                    "Failed to remove alias: {}",
                                    e
                                )));
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
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
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
            }
            // File search specific actions
            "open_file" | "open_directory" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Opening file: {}", path));
                    let _ = crate::file_search::open_file(path);
                    self.file_search_actions_path = None;
                    self.close_and_reset_window(cx);
                }
            }
            "quick_look" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Quick Look: {}", path));
                    let _ = crate::file_search::quick_look(path);
                    self.file_search_actions_path = None;
                    // Don't close window for Quick Look - user may want to continue
                }
            }
            "open_with" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Open With: {}", path));
                    let _ = crate::file_search::open_with(path);
                    self.file_search_actions_path = None;
                }
            }
            "show_info" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Show Info: {}", path));
                    let _ = crate::file_search::show_info(path);
                    self.file_search_actions_path = None;
                }
            }
            "copy_filename" => {
                if let Some(ref path) = self.file_search_actions_path {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    logging::log("UI", &format!("Copy filename: {}", filename));
                    #[cfg(target_os = "macos")]
                    {
                        let _ = self.pbcopy(filename);
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        let _ = Clipboard::new().and_then(|mut c| c.set_text(filename));
                    }
                    self.last_output = Some(SharedString::from(format!("Copied: {}", filename)));
                    self.file_search_actions_path = None;
                }
            }
            // Scriptlet-specific actions
            "edit_scriptlet" => {
                logging::log("UI", "Edit scriptlet action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor (e.g., "/path/to/file.md#slug" -> "/path/to/file.md")
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::PathBuf::from(path_str);
                            self.edit_script(&path);
                            self.hide_main_and_reset(cx);
                        } else {
                            self.last_output =
                                Some(SharedString::from("Scriptlet has no source file path"));
                        }
                    } else {
                        self.last_output =
                            Some(SharedString::from("Selected item is not a scriptlet"));
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "reveal_scriptlet_in_finder" => {
                logging::log("UI", "Reveal scriptlet in Finder action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::Path::new(path_str);
                            self.reveal_in_finder(path);
                            self.last_output = Some(SharedString::from("Revealed in Finder"));
                            self.hide_main_and_reset(cx);
                        } else {
                            self.last_output =
                                Some(SharedString::from("Scriptlet has no source file path"));
                        }
                    } else {
                        self.last_output =
                            Some(SharedString::from("Selected item is not a scriptlet"));
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "copy_scriptlet_path" => {
                logging::log("UI", "Copy scriptlet path action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);

                            #[cfg(target_os = "macos")]
                            {
                                match self.pbcopy(path_str) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.last_output = Some(SharedString::from(format!(
                                            "Copied: {}",
                                            path_str
                                        )));
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
                                match Clipboard::new().and_then(|mut c| c.set_text(path_str)) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.last_output = Some(SharedString::from(format!(
                                            "Copied: {}",
                                            path_str
                                        )));
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "ERROR",
                                            &format!("Failed to copy path: {}", e),
                                        );
                                        self.last_output =
                                            Some(SharedString::from("Failed to copy path"));
                                    }
                                }
                            }
                        } else {
                            self.last_output =
                                Some(SharedString::from("Scriptlet has no source file path"));
                        }
                    } else {
                        self.last_output =
                            Some(SharedString::from("Selected item is not a scriptlet"));
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
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
