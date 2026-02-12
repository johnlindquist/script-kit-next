use super::*;
use std::sync::Arc;

impl ScriptListApp {
    /// Show the naming dialog for creating a new script or extension.
    ///
    /// This creates a NamingPrompt entity and switches to the NamingPrompt view.
    /// The user types a friendly name and sees a live kebab-case filename preview.
    /// On submit, the naming channel receives the payload; on cancel, it receives None.
    pub(crate) fn show_naming_dialog(
        &mut self,
        target: prompts::NamingTarget,
        cx: &mut Context<Self>,
    ) {
        let (target_directory, extension) = match target {
            prompts::NamingTarget::Script => (script_creation::scripts_dir(), "ts"),
            prompts::NamingTarget::Extension => (script_creation::extensions_dir(), "md"),
        };

        let id = format!("naming-{}", target.as_str());
        let sender = self.naming_submit_sender.clone();

        let on_submit: prompts::SubmitCallback =
            Arc::new(move |_id: String, value: Option<String>| {
                let _ = sender.try_send(value);
            });

        let config = prompts::NamingPromptConfig::new(target, target_directory, extension)
            .placeholder(format!("My Cool {}", target.display_name()));

        let theme = self.theme.clone();
        let entity = cx.new(|cx| {
            prompts::NamingPrompt::new(id.clone(), config, cx.focus_handle(), on_submit, theme)
        });

        self.current_view = AppView::NamingPrompt {
            id: id.clone(),
            entity,
        };
        self.opened_from_main_menu = true;
        self.pending_focus = Some(FocusTarget::NamingPrompt);
        cx.notify();

        logging::log(
            "NAMING",
            &format!("Showing naming dialog for {}", target.as_str()),
        );
    }

    /// Handle the result from the naming dialog channel.
    ///
    /// - `None` → user cancelled (Esc) → go back to script list
    /// - `Some(json)` → user submitted → create file, open in editor, show feedback
    pub(crate) fn handle_naming_dialog_completion(
        &mut self,
        payload: Option<String>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(json) = payload else {
            // User cancelled
            logging::log("NAMING", "Naming dialog cancelled - returning to main menu");
            self.current_view = AppView::ScriptList;
            self.request_script_list_main_filter_focus(cx);
            cx.notify();
            return;
        };

        let result: prompts::NamingSubmitResult = match serde_json::from_str(&json) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, payload = %json, "Failed to parse naming payload");
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to parse naming result: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                self.current_view = AppView::ScriptList;
                self.request_script_list_main_filter_focus(cx);
                cx.notify();
                return;
            }
        };

        // Extract the stem (filename without extension) for the creation functions
        let filename_stem = match std::path::Path::new(&result.filename).file_stem() {
            Some(s) => s.to_string_lossy().to_string(),
            None => result.filename.clone(),
        };

        let item_type = result.target.as_str();

        logging::log(
            "NAMING",
            &format!(
                "Creating {} with name '{}' (filename: {})",
                item_type, result.friendly_name, result.filename
            ),
        );

        let create_result = match result.target {
            prompts::NamingTarget::Script => script_creation::create_new_script(&filename_stem),
            prompts::NamingTarget::Extension => {
                script_creation::create_new_extension(&filename_stem)
            }
        };

        match create_result {
            Ok(path) => {
                let created_file_path: std::path::PathBuf = if path.is_absolute() {
                    path.clone()
                } else {
                    match std::env::current_dir() {
                        Ok(cwd) => cwd.join(&path),
                        Err(_) => path.clone(),
                    }
                };

                logging::log(
                    "NAMING",
                    &format!("Created new {}: {:?}", item_type, created_file_path),
                );

                if let Err(e) = script_creation::open_in_editor(&path, &self.config) {
                    logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!(
                                "Created {} but failed to open editor: {}",
                                item_type, e
                            ),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                } else {
                    self.toast_manager.push(
                        components::toast::Toast::success(
                            format!("New {} created and opened in editor", item_type),
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                }

                self.current_view = AppView::CreationFeedback {
                    path: created_file_path,
                };
                self.opened_from_main_menu = true;
                cx.notify();
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to create {}: {}", item_type, e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to create {}: {}", item_type, e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                self.current_view = AppView::ScriptList;
                self.request_script_list_main_filter_focus(cx);
                cx.notify();
            }
        }
    }
}
