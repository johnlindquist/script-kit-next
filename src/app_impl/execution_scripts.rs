use super::*;

const NO_MAIN_WINDOW_BUILTINS: &[&str] = &[
    "builtin-ai-chat",
    "builtin-open-ai",
    "builtin-notes",
    "builtin-open-notes",
    "builtin-new-note",
    "builtin-search-notes",
    "builtin-quick-capture",
    "builtin-new-conversation",
];

fn builtin_needs_main_window_for_command_id(identifier: &str) -> bool {
    !NO_MAIN_WINDOW_BUILTINS.contains(&identifier)
}

#[cfg(test)]
mod builtin_command_window_visibility_tests {
    use super::builtin_needs_main_window_for_command_id;

    #[test]
    fn test_builtin_needs_main_window_false_for_open_ai_and_open_notes() {
        assert!(!builtin_needs_main_window_for_command_id("builtin-open-ai"));
        assert!(!builtin_needs_main_window_for_command_id(
            "builtin-open-notes"
        ));
    }

    #[test]
    fn test_builtin_needs_main_window_true_for_unlisted_builtin() {
        assert!(builtin_needs_main_window_for_command_id(
            "builtin-refresh-scripts"
        ));
    }
}

impl ScriptListApp {
    pub(crate) fn execute_scriptlet(&mut self, scriptlet: &scripts::Scriptlet, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!(
                "Executing scriptlet: {} (tool: {})",
                scriptlet.name, scriptlet.tool
            ),
        );

        let tool = scriptlet.tool.to_lowercase();

        // TypeScript/Kit scriptlets need to run interactively (they may use SDK prompts)
        // These should be spawned like regular scripts, not run synchronously
        if matches!(tool.as_str(), "kit" | "ts" | "bun" | "deno" | "js") {
            logging::log(
                "EXEC",
                &format!(
                    "TypeScript scriptlet '{}' - running interactively",
                    scriptlet.name
                ),
            );

            // Write scriptlet content to a temp file
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!(
                "scriptlet-{}-{}.ts",
                scriptlet.name.to_lowercase().replace(' ', "-"),
                std::process::id()
            ));

            if let Err(e) = std::fs::write(&temp_file, &scriptlet.code) {
                logging::log(
                    "ERROR",
                    &format!("Failed to write temp scriptlet file: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to write scriptlet: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }

            // Create a Script struct and run it interactively
            let script = scripts::Script {
                name: scriptlet.name.clone(),
                description: scriptlet.description.clone(),
                path: temp_file,
                extension: "ts".to_string(),
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
                kit_name: None,
            };

            self.execute_interactive(&script, cx);
            return;
        }

        // Shell tools (bash, zsh, sh, fish, etc.) run in the built-in terminal
        // so users can see output interactively
        if scriptlets::SHELL_TOOLS.contains(&tool.as_str()) {
            logging::log(
                "EXEC",
                &format!(
                    "Shell scriptlet '{}' (tool: {}) - running in terminal",
                    scriptlet.name, tool
                ),
            );

            // Write scriptlet code to a temp file and execute it
            let temp_dir = std::env::temp_dir();
            let extension = match tool.as_str() {
                "bash" | "zsh" | "sh" => "sh",
                "fish" => "fish",
                "powershell" | "pwsh" => "ps1",
                "cmd" => "bat",
                _ => "sh",
            };
            let temp_file = temp_dir.join(format!(
                "extension-{}-{}.{}",
                scriptlet.name.to_lowercase().replace(' ', "-"),
                std::process::id(),
                extension
            ));

            if let Err(e) = std::fs::write(&temp_file, &scriptlet.code) {
                logging::log(
                    "ERROR",
                    &format!("Failed to write temp extension file: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to write extension: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }

            // Build the command to execute the script file
            let shell_command = format!("{} {}", tool, temp_file.display());

            self.open_terminal_with_command(shell_command, cx);
            return;
        }

        // For other tools (python, ruby, template, etc.), run synchronously
        // These don't use the SDK and won't block waiting for input

        // Convert scripts::Scriptlet to scriptlets::Scriptlet for executor
        let exec_scriptlet = scriptlets::Scriptlet {
            name: scriptlet.name.clone(),
            command: scriptlet.command.clone().unwrap_or_else(|| {
                // Generate command slug from name if not present
                scriptlet.name.to_lowercase().replace(' ', "-")
            }),
            tool: scriptlet.tool.clone(),
            scriptlet_content: scriptlet.code.clone(),
            inputs: vec![], // TODO: Parse inputs from code if needed
            group: scriptlet.group.clone().unwrap_or_default(),
            preview: None,
            metadata: scriptlets::ScriptletMetadata {
                shortcut: scriptlet.shortcut.clone(),
                keyword: scriptlet.keyword.clone(),
                description: scriptlet.description.clone(),
                ..Default::default()
            },
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: scriptlet.file_path.clone(),
            actions: vec![], // Scriptlet actions parsed from H3 headers
        };

        // Execute with default options (no inputs for now)
        let options = executor::ScriptletExecOptions::default();

        match executor::run_scriptlet(&exec_scriptlet, options) {
            Ok(result) => {
                if result.success {
                    logging::log(
                        "EXEC",
                        &format!(
                            "Scriptlet '{}' succeeded: exit={}",
                            scriptlet.name, result.exit_code
                        ),
                    );

                    // Handle special tool types that need interactive prompts
                    if tool == "template" && !result.stdout.is_empty() {
                        // Template tool: show template prompt with the content
                        let id = format!("scriptlet-template-{}", uuid::Uuid::new_v4());
                        logging::log(
                            "EXEC",
                            &format!(
                                "Template scriptlet '{}' - showing template prompt",
                                scriptlet.name
                            ),
                        );
                        self.handle_prompt_message(
                            PromptMessage::ShowTemplate {
                                id,
                                template: result.stdout.clone(),
                            },
                            cx,
                        );
                        return;
                    }

                    // Store output if any
                    if !result.stdout.is_empty() {
                        self.last_output = Some(SharedString::from(result.stdout.clone()));
                    }

                    // Hide window after successful execution
                    script_kit_gpui::set_main_window_visible(false);
                    cx.hide();
                } else {
                    // Execution failed (non-zero exit code)
                    let error_msg = if !result.stderr.is_empty() {
                        result.stderr.clone()
                    } else {
                        format!("Exit code: {}", result.exit_code)
                    };

                    logging::log(
                        "ERROR",
                        &format!("Scriptlet '{}' failed: {}", scriptlet.name, error_msg),
                    );

                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Scriptlet failed: {}", error_msg),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            Err(e) => {
                logging::log(
                    "ERROR",
                    &format!("Failed to execute scriptlet '{}': {}", scriptlet.name, e),
                );

                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to execute: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    /// Execute a script or scriptlet by its file path
    /// Used by global shortcuts to directly invoke scripts
    #[allow(dead_code)]
    pub(crate) fn execute_script_by_path(&mut self, path: &str, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Executing script by path: {}", path));

        // Check if it's a scriptlet (contains #)
        if path.contains('#') {
            // It's a scriptlet path like "/path/to/file.md#command"
            if let Some(scriptlet) = self
                .scriptlets
                .iter()
                .find(|s| s.file_path.as_ref().map(|p| p == path).unwrap_or(false))
            {
                let scriptlet_clone = scriptlet.clone();
                self.execute_scriptlet(&scriptlet_clone, cx);
                return;
            }
            logging::log("ERROR", &format!("Scriptlet not found: {}", path));
            return;
        }

        // It's a regular script - find by path
        if let Some(script) = self
            .scripts
            .iter()
            .find(|s| s.path.to_string_lossy() == path)
        {
            let script_clone = script.clone();
            self.execute_interactive(&script_clone, cx);
            return;
        }

        // Not found in loaded scripts - try to execute directly as a file
        let script_path = std::path::PathBuf::from(path);
        if script_path.exists() {
            let name = script_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("script")
                .to_string();

            let script = scripts::Script {
                name,
                path: script_path.clone(),
                extension: script_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("ts")
                    .to_string(),
                description: None,
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
                kit_name: None,
            };

            self.execute_interactive(&script, cx);
        } else {
            logging::log("ERROR", &format!("Script file not found: {}", path));
        }
    }

    /// Execute by command ID or legacy file path.
    ///
    /// Command IDs have formats like:
    /// - "scriptlet/my-scriptlet" - execute a scriptlet
    /// - "builtin/ai-chat" - execute a builtin
    /// - "app/com.apple.Finder" - launch an app
    /// - Otherwise: treated as a file path (legacy behavior)
    ///
    /// Returns `true` if the main window should be shown, `false` if not.
    /// Apps and certain builtins (AI Chat, Notes) open their own windows
    /// and don't need the main window.
    pub fn execute_by_command_id_or_path(
        &mut self,
        command_id: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        logging::log(
            "EXEC",
            &format!("Executing by command ID or path: {}", command_id),
        );

        // Parse command ID format: "type/identifier"
        if let Some((cmd_type, identifier)) = command_id.split_once('/') {
            match cmd_type {
                "scriptlet" => {
                    // Find scriptlet by name
                    logging::bench_log("scriptlet_lookup_start");
                    if let Some(scriptlet) = self.scriptlets.iter().find(|s| s.name == identifier) {
                        logging::bench_log("scriptlet_found");
                        let scriptlet_clone = scriptlet.clone();
                        logging::log("EXEC", &format!("Executing scriptlet: {}", identifier));
                        self.execute_scriptlet(&scriptlet_clone, cx);
                        // Don't show window immediately - scriptlets that need it (like getSelectedText)
                        // will call hide() first, then their prompts (chat, arg, etc.) will show the window.
                        // This prevents the flash of main menu before the scriptlet UI appears.
                        return false;
                    }
                    logging::log("ERROR", &format!("Scriptlet not found: {}", identifier));
                    return false;
                }
                "builtin" => {
                    // Execute builtin by ID
                    let config = crate::config::BuiltInConfig::default();
                    if let Some(entry) = builtins::get_builtin_entries(&config)
                        .iter()
                        .find(|e| e.id == identifier)
                    {
                        logging::log("EXEC", &format!("Executing builtin: {}", identifier));
                        self.execute_builtin(entry, cx);
                        // Check if this builtin opens its own window
                        let needs_main_window =
                            builtin_needs_main_window_for_command_id(identifier);
                        logging::log(
                            "EXEC",
                            &format!(
                                "Builtin {} needs_main_window: {}",
                                identifier, needs_main_window
                            ),
                        );
                        return needs_main_window;
                    }
                    logging::log("ERROR", &format!("Builtin not found: {}", identifier));
                    return false;
                }
                "app" => {
                    // Launch app by bundle ID - find app in cached apps and launch
                    // Apps NEVER need the main window - they open externally
                    logging::log(
                        "EXEC",
                        &format!("Launching app by bundle ID: {}", identifier),
                    );
                    let apps = crate::app_launcher::get_cached_apps();
                    if let Some(app) = apps
                        .iter()
                        .find(|a| a.bundle_id.as_deref() == Some(identifier))
                    {
                        if let Err(e) = crate::app_launcher::launch_application(app) {
                            logging::log("ERROR", &format!("Failed to launch app: {}", e));
                        }
                    } else {
                        logging::log("ERROR", &format!("App not found: {}", identifier));
                    }
                    return false; // Apps never need main window
                }
                _ => {
                    // Unknown type - fall through to path-based execution
                    logging::log(
                        "EXEC",
                        &format!("Unknown command type '{}', trying as path", cmd_type),
                    );
                }
            }
        }

        // Check if command_id matches a scriptlet by name or file_path
        // Scriptlets don't need immediate window show - they control their own visibility
        if let Some(scriptlet) = self.scriptlets.iter().find(|s| {
            s.name == command_id
                || s.file_path
                    .as_ref()
                    .map(|p| p == command_id)
                    .unwrap_or(false)
        }) {
            logging::log(
                "EXEC",
                &format!("Found scriptlet by name/path: {}", scriptlet.name),
            );
            let scriptlet_clone = scriptlet.clone();
            self.execute_scriptlet(&scriptlet_clone, cx);
            return false; // Scriptlets don't need immediate window show
        }

        // Fall back to path-based execution (legacy behavior)
        // Scripts typically need the main window for prompts
        self.execute_script_by_path(command_id, cx);
        true
    }

}
