impl ScriptListApp {
    fn execute_app(&mut self, app: &app_launcher::AppInfo, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Launching app from search: {}", app.name));

        if let Err(e) = app_launcher::launch_application(app) {
            logging::log("ERROR", &format!("Failed to launch {}: {}", app.name, e));
            self.last_output = Some(SharedString::from(format!(
                "Failed to launch: {}",
                app.name
            )));
            cx.notify();
        } else {
            logging::log("EXEC", &format!("Launched app: {}", app.name));
            self.close_and_reset_window(cx);
        }
    }

    /// Focus a window from the main search results
    fn execute_window_focus(
        &mut self,
        window: &window_control::WindowInfo,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!("Focusing window: {} - {}", window.app, window.title),
        );

        if let Err(e) = window_control::focus_window(window.id) {
            logging::log("ERROR", &format!("Failed to focus window: {}", e));
            self.toast_manager.push(
                components::toast::Toast::error(
                    format!("Failed to focus window: {}", e),
                    &self.theme,
                )
                .duration_ms(Some(5000)),
            );
            cx.notify();
        } else {
            logging::log("EXEC", &format!("Focused window: {}", window.title));
            self.close_and_reset_window(cx);
        }
    }

    /// Show an API key configuration prompt.
    ///
    /// This creates an EnvPrompt that stores the key in the system keyring.
    /// Once stored, the key will be available to:
    /// - AI Chat window (via DetectedKeys::from_environment)
    /// - Scripts using `await env("SCRIPT_KIT_*_API_KEY")`
    fn show_api_key_prompt(
        &mut self,
        key_name: &str,
        prompt_text: &str,
        provider_name: &str,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!("Showing API key prompt for: {}", provider_name),
        );

        let id = format!("configure-{}", key_name.to_lowercase());
        let key = key_name.to_string();
        let prompt = Some(prompt_text.to_string());
        let secret = true; // API keys are always secrets

        // Store provider name for success message after completion
        self.pending_api_key_config = Some(provider_name.to_string());

        // Create submit callback that signals completion
        // The actual toast and view reset happens in handle_api_key_completion
        let completion_sender = self.api_key_completion_sender.clone();
        let provider_for_callback = provider_name.to_string();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id, value| {
                // Value being Some means the user submitted a value (key was saved)
                // Value being None means the user cancelled
                let success = value.is_some();
                logging::log(
                    "EXEC",
                    &format!(
                        "API key config callback: provider={}, success={}",
                        provider_for_callback, success
                    ),
                );
                // Signal completion to the app
                let _ = completion_sender.try_send((provider_for_callback.clone(), success));
            });

        // Check if key already exists in secrets (for UX messaging)
        // Use get_secret_info to get both existence and modification timestamp
        let secret_info = secrets::get_secret_info(&key);
        let exists_in_keyring = secret_info
            .as_ref()
            .map(|info| !info.value.is_empty())
            .unwrap_or(false);
        let modified_at = secret_info.map(|info| info.modified_at);

        if exists_in_keyring {
            logging::log(
                "EXEC",
                &format!(
                    "{} API key already configured (modified: {:?}) - showing update prompt",
                    provider_name, modified_at
                ),
            );
        }

        // Create EnvPrompt entity
        let focus_handle = self.focus_handle.clone();
        let env_prompt = prompts::EnvPrompt::new(
            id.clone(),
            key.clone(),
            prompt,
            Some(provider_name.to_string()), // title
            secret,
            focus_handle,
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            exists_in_keyring,
            modified_at,
        );

        let entity = cx.new(|_| env_prompt);
        self.current_view = AppView::EnvPrompt { id, entity };
        self.focused_input = FocusedInput::None; // EnvPrompt has its own focus handling
        self.pending_focus = Some(FocusTarget::EnvPrompt);

        // Resize to standard height for full-window centered layout
        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }

    /// Handle API key configuration completion.
    /// Called when the EnvPrompt callback signals completion.
    ///
    /// NOTE: This is called from render(), so we must use deferred resize via Window::defer
    /// to avoid layout issues where the macOS window resizes but GPUI's layout doesn't update.
    fn handle_api_key_completion(
        &mut self,
        provider: String,
        success: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pending_api_key_config = None;

        if success {
            // Show success toast
            self.toast_manager.push(
                components::toast::Toast::success(
                    format!("{} API key saved successfully", provider),
                    &self.theme,
                )
                .duration_ms(Some(3000)),
            );

            // Rebuild provider registry so new key is available next time chat opens
            self.rebuild_provider_registry_async(cx);
        }

        // Return to main menu
        self.reset_to_script_list(cx);

        // CRITICAL: Use deferred resize because this is called from render().
        // Synchronous resize (resize_to_view_sync) would resize the macOS window
        // but GPUI's layout system wouldn't update until the next frame,
        // causing the content to render at the wrong size (empty list bug).
        let target_height = window_resize::height_for_view(ViewType::ScriptList, 0);
        window.defer(cx, move |_window, _cx| {
            window_resize::resize_first_window_to_height(target_height);
        });

        cx.notify();
    }

    /// Enable Claude Code in config.ts and re-show the inline chat.
    ///
    /// This modifies the user's config.ts to enable Claude Code provider,
    /// reloads the config, and then re-opens the inline chat with
    /// the newly available Claude Code provider.
    pub fn enable_claude_code_in_config(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::config::editor::{self, ConfigWriteError, WriteOutcome};

        logging::log("EXEC", "Enabling Claude Code in config.ts");

        let config_path =
            std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());
        let bun_path = self.config.bun_path.as_deref();

        match editor::enable_claude_code_safely(&config_path, bun_path) {
            Ok(WriteOutcome::Written) => {
                logging::log("EXEC", "Claude Code enabled in config.ts");
            }
            Ok(WriteOutcome::Created) => {
                logging::log("EXEC", "Created new config.ts with Claude Code enabled");
            }
            Ok(WriteOutcome::AlreadySet) => {
                logging::log("EXEC", "Claude Code already enabled in config.ts");
            }
            Err(ConfigWriteError::ValidationFailed(reason)) => {
                logging::log(
                    "EXEC",
                    &format!("Config validation failed: {}", reason),
                );
                // Attempt to recover from backup
                match editor::recover_from_backup(&config_path, bun_path) {
                    Ok(true) => {
                        logging::log("EXEC", "Config restored from backup after validation failure");
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                "Failed to enable Claude Code (invalid config). Backup restored."
                                    .to_string(),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                    Ok(false) => {
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!(
                                    "Failed to enable Claude Code: {}. No backup available.",
                                    reason
                                ),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                    Err(recover_err) => {
                        logging::log(
                            "EXEC",
                            &format!("Backup recovery also failed: {}", recover_err),
                        );
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!(
                                    "Failed to enable Claude Code: {}. Recovery failed: {}",
                                    reason, recover_err
                                ),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                }
                cx.notify();
                return;
            }
            Err(e) => {
                logging::log("EXEC", &format!("Failed to enable Claude Code: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to enable Claude Code: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        }

        // Reload config and rebuild provider registry in background
        self.config = crate::config::load_config();
        self.rebuild_provider_registry_async(cx);

        // Check if Claude CLI is actually installed (this is an explicit user action,
        // so the brief sync check is acceptable for correct toast messaging)
        let claude_path = self
            .config
            .get_claude_code()
            .path
            .unwrap_or_else(|| "claude".to_string());
        let claude_available = std::process::Command::new(&claude_path)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if claude_available {
            self.toast_manager.push(
                components::toast::Toast::success(
                    "Claude Code enabled! Ready to use.".to_string(),
                    &self.theme,
                )
                .duration_ms(Some(3000)),
            );

            // Go back to main menu, then re-show inline chat
            self.go_back_or_close(window, cx);
            self.show_inline_ai_chat(None, cx);
        } else {
            self.toast_manager.push(
                components::toast::Toast::warning(
                    "Config saved! Install Claude CLI: npm install -g @anthropic-ai/claude-code"
                        .to_string(),
                    &self.theme,
                )
                .duration_ms(Some(8000)),
            );
            logging::log(
                "EXEC",
                "Claude Code config saved but CLI not found - user needs to install it",
            );
        }

        cx.notify();
    }

    /// Get the scratch pad file path
    fn get_scratch_pad_path() -> std::path::PathBuf {
        setup::get_kit_path().join("scratch-pad.md")
    }

    /// Open the scratch pad editor with auto-save functionality
    fn open_scratch_pad(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "Opening Scratch Pad");

        // Get or create scratch pad file path
        let scratch_path = Self::get_scratch_pad_path();

        // Ensure parent directory exists
        if let Some(parent) = scratch_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                logging::log(
                    "ERROR",
                    &format!("Failed to create scratch pad directory: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to create directory: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        }

        // Load existing content or create empty file
        let content = match std::fs::read_to_string(&scratch_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Create empty file
                if let Err(write_err) = std::fs::write(&scratch_path, "") {
                    logging::log(
                        "ERROR",
                        &format!("Failed to create scratch pad file: {}", write_err),
                    );
                }
                String::new()
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to read scratch pad: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to read scratch pad: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        };

        logging::log(
            "EXEC",
            &format!("Loaded scratch pad with {} bytes", content.len()),
        );

        // Create editor focus handle
        let editor_focus_handle = cx.focus_handle();

        // Create submit callback that saves and closes
        let scratch_path_clone = scratch_path.clone();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, value: Option<String>| {
                if let Some(content) = value {
                    // Save the content to disk
                    if let Err(e) = std::fs::write(&scratch_path_clone, &content) {
                        tracing::error!(error = %e, "Failed to save scratch pad on submit");
                    } else {
                        tracing::info!(bytes = content.len(), "Scratch pad saved on submit");
                    }
                }
            });

        // Get the target height for editor view (subtract footer height for unified footer)
        let editor_height = px(700.0 - window_resize::layout::FOOTER_HEIGHT);

        // Create the editor prompt
        let editor_prompt = EditorPrompt::with_height(
            "scratch-pad".to_string(),
            content,
            "markdown".to_string(), // Use markdown for nice highlighting
            editor_focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(editor_height),
        );

        let entity = cx.new(|_| editor_prompt);

        // Set up auto-save timer using weak reference
        let scratch_path_for_save = scratch_path;
        let entity_weak = entity.downgrade();
        cx.spawn(async move |_this, cx| {
            loop {
                // Auto-save every 2 seconds
                gpui::Timer::after(std::time::Duration::from_secs(2)).await;

                // Try to save the current content
                let save_result = cx.update(|cx| {
                    if let Some(entity) = entity_weak.upgrade() {
                        // Use update on the entity to get the correct Context<EditorPrompt>
                        let content: String = entity.update(cx, |editor, cx| editor.content(cx));
                        if let Err(e) = std::fs::write(&scratch_path_for_save, &content) {
                            tracing::warn!(error = %e, "Auto-save failed");
                        } else {
                            tracing::debug!(bytes = content.len(), "Auto-saved scratch pad");
                        }
                        true // Entity still exists
                    } else {
                        false // Entity dropped, stop the task
                    }
                });

                match save_result {
                    Ok(true) => continue,
                    Ok(false) | Err(_) => break, // Entity gone or context invalid
                }
            }
        })
        .detach();

        self.current_view = AppView::ScratchPadView {
            entity,
            focus_handle: editor_focus_handle,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::EditorPrompt);

        // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
        // to after the current GPUI update cycle completes.
        cx.spawn(async move |_this, _cx| {
            resize_to_view_sync(ViewType::EditorPrompt, 0);
        })
        .detach();
        cx.notify();
    }

}
