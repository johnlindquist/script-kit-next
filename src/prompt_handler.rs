// Prompt message handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    /// Handle a prompt message from the script
    fn handle_prompt_message(&mut self, msg: PromptMessage, cx: &mut Context<Self>) {
        match msg {
            PromptMessage::ShowArg {
                id,
                placeholder,
                choices,
                actions,
            } => {
                logging::log(
                    "UI",
                    &format!(
                        "Showing arg prompt: {} with {} choices, {} actions",
                        id,
                        choices.len(),
                        actions.as_ref().map(|a| a.len()).unwrap_or(0)
                    ),
                );
                let choice_count = choices.len();

                // If actions were provided, store them in the SDK actions system
                // so they can be triggered via shortcuts and Cmd+K
                if let Some(ref action_list) = actions {
                    // Store SDK actions for trigger_action_by_name lookup
                    self.sdk_actions = Some(action_list.clone());

                    // Register keyboard shortcuts for SDK actions
                    self.action_shortcuts.clear();
                    for action in action_list {
                        if let Some(shortcut) = &action.shortcut {
                            self.action_shortcuts.insert(
                                shortcuts::normalize_shortcut(shortcut),
                                action.name.clone(),
                            );
                        }
                    }
                } else {
                    // Clear any previous SDK actions
                    self.sdk_actions = None;
                    self.action_shortcuts.clear();
                }

                self.current_view = AppView::ArgPrompt {
                    id,
                    placeholder,
                    choices,
                    actions,
                };
                self.arg_input.clear();
                self.arg_selected_index = 0;
                self.focused_input = FocusedInput::ArgPrompt;
                // Request focus via pending_focus mechanism (will be applied on next render)
                self.pending_focus = Some(FocusTarget::AppRoot); // ArgPrompt uses parent focus
                                                                 // Resize window based on number of choices
                let view_type = if choice_count == 0 {
                    ViewType::ArgPromptNoChoices
                } else {
                    ViewType::ArgPromptWithChoices
                };
                resize_to_view_sync(view_type, choice_count);
                cx.notify();
            }
            PromptMessage::ShowDiv {
                id,
                html,
                container_classes,
                actions,
                placeholder: _placeholder, // TODO: render in header
                hint: _hint,               // TODO: render hint
                footer: _footer,           // TODO: render footer
                container_bg,
                container_padding,
                opacity,
            } => {
                logging::log("UI", &format!("Showing div prompt: {}", id));
                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                // Create submit callback for div prompt
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - div response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Create focus handle for div prompt
                let div_focus_handle = cx.focus_handle();

                // Build container options from protocol message
                let container_options = ContainerOptions {
                    background: container_bg,
                    padding: container_padding.and_then(|v| {
                        if v.is_string() && v.as_str() == Some("none") {
                            Some(ContainerPadding::None)
                        } else if let Some(n) = v.as_f64() {
                            Some(ContainerPadding::Pixels(n as f32))
                        } else {
                            v.as_i64().map(|n| ContainerPadding::Pixels(n as f32))
                        }
                    }),
                    opacity,
                    container_classes,
                };

                // Create DivPrompt entity with proper HTML rendering
                let div_prompt = DivPrompt::with_options(
                    id.clone(),
                    html,
                    None, // tailwind param deprecated - use container_classes in options
                    div_focus_handle,
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                    crate::designs::DesignVariant::Default,
                    container_options,
                );

                let entity = cx.new(|_| div_prompt);
                self.current_view = AppView::DivPrompt { id, entity };
                self.focused_input = FocusedInput::None; // DivPrompt has no text input
                self.pending_focus = Some(FocusTarget::AppRoot); // DivPrompt uses parent focus
                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
            }
            PromptMessage::ShowForm { id, html, actions } => {
                logging::log("UI", &format!("Showing form prompt: {}", id));

                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                // Create form field colors from theme
                let colors = FormFieldColors::from_theme(&self.theme);

                // Create FormPromptState entity with parsed fields
                let form_state = FormPromptState::new(id.clone(), html, colors, cx);
                let field_count = form_state.fields.len();
                let entity = cx.new(|_| form_state);

                self.current_view = AppView::FormPrompt { id, entity };
                self.focused_input = FocusedInput::None; // FormPrompt has its own focus handling
                self.pending_focus = Some(FocusTarget::FormPrompt);

                // Resize based on field count (more fields = taller window)
                let view_type = if field_count > 0 {
                    ViewType::ArgPromptWithChoices
                } else {
                    ViewType::DivPrompt
                };
                resize_to_view_sync(view_type, field_count);
                cx.notify();
            }
            PromptMessage::ShowTerm {
                id,
                command,
                actions,
            } => {
                logging::log(
                    "UI",
                    &format!("Showing term prompt: {} (command: {:?})", id, command),
                );

                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                // Create submit callback for terminal
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - terminal response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Get the target height for terminal view (subtract footer height)
                let term_height =
                    window_resize::layout::MAX_HEIGHT - px(window_resize::layout::FOOTER_HEIGHT);

                // Create terminal with explicit height - GPUI entities don't inherit parent flex sizing
                match term_prompt::TermPrompt::with_height(
                    id.clone(),
                    command,
                    self.focus_handle.clone(),
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                    std::sync::Arc::new(self.config.clone()),
                    Some(term_height),
                ) {
                    Ok(term_prompt) => {
                        let entity = cx.new(|_| term_prompt);
                        self.current_view = AppView::TermPrompt { id, entity };
                        self.focused_input = FocusedInput::None; // Terminal handles its own cursor
                        self.pending_focus = Some(FocusTarget::TermPrompt);
                        // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                        // to after the current GPUI update cycle completes. Synchronous Cocoa
                        // setFrame: calls during render can trigger events that re-borrow GPUI state.
                        cx.spawn(async move |_this, _cx| {
                            resize_to_view_sync(ViewType::TermPrompt, 0);
                        })
                        .detach();
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to create terminal");
                        logging::log("ERROR", &format!("Failed to create terminal: {}", e));
                    }
                }
            }
            PromptMessage::ShowEditor {
                id,
                content,
                language,
                template,
                actions,
            } => {
                logging::log(
                    "UI",
                    &format!(
                        "Showing editor prompt: {} (language: {:?}, template: {})",
                        id,
                        language,
                        template.is_some()
                    ),
                );

                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                // Create submit callback for editor
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - editor response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // CRITICAL: Create a SEPARATE focus handle for the editor.
                // Using the parent's focus handle causes keyboard event routing issues
                // because the parent checks is_focused() in its render and both parent
                // and child would be tracking the same handle.
                let editor_focus_handle = cx.focus_handle();

                // Get the target height for editor view (subtract footer height for unified footer)
                let editor_height = px(700.0 - window_resize::layout::FOOTER_HEIGHT);

                // Create editor v2 (gpui-component based with Find/Replace)
                // Default to markdown for all editor content
                let resolved_language = language.unwrap_or_else(|| "markdown".to_string());

                // Use with_template if template provided, or if content contains tabstop patterns
                // This auto-detects VSCode-style templates like ${1:name} or $1
                let content_str = content.unwrap_or_default();
                let has_tabstops = content_str.contains("${")
                    || regex::Regex::new(r"\$\d")
                        .map(|re| re.is_match(&content_str))
                        .unwrap_or(false);

                let editor_prompt = if let Some(template_str) = template {
                    EditorPrompt::with_template(
                        id.clone(),
                        template_str,
                        resolved_language.clone(),
                        editor_focus_handle.clone(),
                        submit_callback,
                        std::sync::Arc::clone(&self.theme),
                        std::sync::Arc::new(self.config.clone()),
                        Some(editor_height),
                    )
                } else if has_tabstops {
                    // Auto-detect template in content
                    logging::log(
                        "UI",
                        &format!("Auto-detected template in content: {}", content_str),
                    );
                    EditorPrompt::with_template(
                        id.clone(),
                        content_str,
                        resolved_language.clone(),
                        editor_focus_handle.clone(),
                        submit_callback,
                        std::sync::Arc::clone(&self.theme),
                        std::sync::Arc::new(self.config.clone()),
                        Some(editor_height),
                    )
                } else {
                    EditorPrompt::with_height(
                        id.clone(),
                        content_str,
                        resolved_language.clone(),
                        editor_focus_handle.clone(),
                        submit_callback,
                        std::sync::Arc::clone(&self.theme),
                        std::sync::Arc::new(self.config.clone()),
                        Some(editor_height),
                    )
                };

                let entity = cx.new(|_| editor_prompt);
                self.current_view = AppView::EditorPrompt {
                    id,
                    entity,
                    focus_handle: editor_focus_handle,
                };
                self.focused_input = FocusedInput::None; // Editor handles its own focus
                self.pending_focus = Some(FocusTarget::EditorPrompt);

                // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                // to after the current GPUI update cycle completes.
                cx.spawn(async move |_this, _cx| {
                    resize_to_view_sync(ViewType::EditorPrompt, 0);
                })
                .detach();
                cx.notify();
            }
            PromptMessage::ScriptExit => {
                logging::log("VISIBILITY", "=== ScriptExit message received ===");
                let was_visible = script_kit_gpui::is_main_window_visible();
                let script_hid_window = script_kit_gpui::script_requested_hide();
                logging::log(
                    "VISIBILITY",
                    &format!(
                        "WINDOW_VISIBLE was: {}, SCRIPT_REQUESTED_HIDE was: {}",
                        was_visible, script_hid_window
                    ),
                );

                // Reset the script-requested-hide flag
                script_kit_gpui::set_script_requested_hide(false);
                logging::log("VISIBILITY", "SCRIPT_REQUESTED_HIDE reset to: false");

                // Set flag so next hotkey show will reset to script list
                NEEDS_RESET.store(true, Ordering::SeqCst);
                logging::log("VISIBILITY", "NEEDS_RESET set to: true");

                self.reset_to_script_list(cx);
                logging::log("VISIBILITY", "reset_to_script_list() called");

                // If the script had hidden the window (e.g., for getSelectedText),
                // request showing the main window so the menu comes back
                if script_hid_window {
                    logging::log(
                        "VISIBILITY",
                        "Script had hidden window - requesting show main window",
                    );
                    script_kit_gpui::request_show_main_window();
                } else {
                    // Script didn't hide window, so it was user-initiated hide or already visible
                    // Restore window height to main menu size in case a prompt (like EnvPrompt)
                    // had shrunk the window
                    resize_to_view_sync(ViewType::ScriptList, 0);
                    script_kit_gpui::set_main_window_visible(false);
                    logging::log(
                        "VISIBILITY",
                        "Script didn't hide window - restored height and keeping visibility state",
                    );
                }
            }
            PromptMessage::HideWindow => {
                logging::log("VISIBILITY", "=== HideWindow message received ===");
                let was_visible = script_kit_gpui::is_main_window_visible();
                logging::log(
                    "VISIBILITY",
                    &format!("WINDOW_VISIBLE was: {}", was_visible),
                );

                // CRITICAL: Update visibility state so hotkey toggle works correctly
                script_kit_gpui::set_main_window_visible(false);
                logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

                // Mark that script requested hide - so ScriptExit knows to show window again
                script_kit_gpui::set_script_requested_hide(true);
                logging::log("VISIBILITY", "SCRIPT_REQUESTED_HIDE set to: true");

                // Set flag so next hotkey show will reset to script list
                NEEDS_RESET.store(true, Ordering::SeqCst);
                logging::log("VISIBILITY", "NEEDS_RESET set to: true");

                cx.hide();
                logging::log(
                    "VISIBILITY",
                    "cx.hide() called - window should now be hidden",
                );
            }
            PromptMessage::OpenBrowser { url } => {
                logging::log("UI", &format!("Opening browser: {}", url));
                #[cfg(target_os = "macos")]
                {
                    match std::process::Command::new("open").arg(&url).spawn() {
                        Ok(_) => logging::log(
                            "UI",
                            &format!("Successfully opened URL in browser: {}", url),
                        ),
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open URL '{}': {}", url, e))
                        }
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    match std::process::Command::new("xdg-open").arg(&url).spawn() {
                        Ok(_) => logging::log(
                            "UI",
                            &format!("Successfully opened URL in browser: {}", url),
                        ),
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open URL '{}': {}", url, e))
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                {
                    match std::process::Command::new("cmd")
                        .args(["/C", "start", &url])
                        .spawn()
                    {
                        Ok(_) => logging::log(
                            "UI",
                            &format!("Successfully opened URL in browser: {}", url),
                        ),
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open URL '{}': {}", url, e))
                        }
                    }
                }
            }
            PromptMessage::RunScript { path } => {
                logging::log("EXEC", &format!("RunScript command received: {}", path));

                // Create a Script struct from the path
                let script_path = std::path::PathBuf::from(&path);
                let script_name = script_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let extension = script_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("ts")
                    .to_string();

                let script = scripts::Script {
                    name: script_name.clone(),
                    description: Some(format!("External script: {}", path)),
                    path: script_path,
                    extension,
                    icon: None,
                    alias: None,
                    shortcut: None,
                    typed_metadata: None,
                    schema: None,
                    kit_name: None,
                };

                logging::log("EXEC", &format!("Executing script: {}", script_name));
                self.execute_interactive(&script, cx);
            }
            PromptMessage::ScriptError {
                error_message,
                stderr_output,
                exit_code,
                stack_trace,
                script_path,
                suggestions,
            } => {
                logging::log(
                    "ERROR",
                    &format!(
                        "Script error received: {} (exit code: {:?})",
                        error_message, exit_code
                    ),
                );

                // CRITICAL: Show error via HUD (highly visible floating window)
                // This ensures the user sees the error even if the main window is hidden/dismissed
                // HUD appears at bottom-center of screen for 5 seconds
                let hud_message = if error_message.chars().count() > 60 {
                    // Use chars().take() to safely handle multi-byte UTF-8 characters
                    let truncated: String = error_message.chars().take(57).collect();
                    format!("Script Error: {}...", truncated)
                } else {
                    format!("Script Error: {}", error_message)
                };
                self.show_hud(hud_message, Some(5000), cx);

                // Also create in-app toast with expandable details (for when window is visible)
                // Use stderr_output if available, otherwise use stack_trace
                let details_text = stderr_output.clone().or_else(|| stack_trace.clone());
                let toast = Toast::error(error_message.clone(), &self.theme)
                    .details_opt(details_text.clone())
                    .duration_ms(Some(10000)); // 10 seconds for errors

                // Add copy button action if we have stderr/stack trace
                let toast = if let Some(ref trace) = details_text {
                    let trace_clone = trace.clone();
                    toast.action(ToastAction::new(
                        "Copy Error",
                        Box::new(move |_, _, _| {
                            // Copy to clipboard
                            use arboard::Clipboard;
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(trace_clone.clone());
                                logging::log("UI", "Error copied to clipboard");
                            }
                        }),
                    ))
                } else {
                    toast
                };

                // Log suggestions if present
                if !suggestions.is_empty() {
                    logging::log("ERROR", &format!("Suggestions: {:?}", suggestions));
                }

                // Push toast to manager
                let toast_id = self.toast_manager.push(toast);
                logging::log(
                    "UI",
                    &format!(
                        "Toast created for script error: {} (id: {})",
                        script_path, toast_id
                    ),
                );

                cx.notify();
            }
            PromptMessage::ProtocolError {
                correlation_id,
                summary,
                details,
                severity,
                script_path,
            } => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    script_path = %script_path,
                    summary = %summary,
                    "Protocol parse issue received"
                );

                let mut toast = Toast::from_severity(summary.clone(), severity, &self.theme)
                    .details_opt(details.clone())
                    .duration_ms(Some(8000));

                if let Some(ref detail_text) = details {
                    let detail_clone = detail_text.clone();
                    toast = toast.action(ToastAction::new(
                        "Copy Details",
                        Box::new(move |_, _, _| {
                            use arboard::Clipboard;
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(detail_clone.clone());
                            }
                        }),
                    ));
                }

                self.toast_manager.push(toast);
                cx.notify();
            }
            PromptMessage::UnhandledMessage { message_type } => {
                logging::log(
                    "WARN",
                    &format!("Displaying unhandled message warning: {}", message_type),
                );

                let toast = Toast::warning(
                    format!("'{}' is not yet implemented", message_type),
                    &self.theme,
                )
                .duration_ms(Some(5000));

                self.toast_manager.push(toast);
                cx.notify();
            }
            PromptMessage::GetState { request_id } => {
                logging::log(
                    "UI",
                    &format!("Collecting state for request: {}", request_id),
                );

                // Collect current UI state
                let (
                    prompt_type,
                    prompt_id,
                    placeholder,
                    input_value,
                    choice_count,
                    visible_choice_count,
                    selected_index,
                    selected_value,
                ) = match &self.current_view {
                    AppView::ScriptList => {
                        let filtered_len = self.filtered_results().len();
                        let selected_value = if self.selected_index < filtered_len {
                            self.filtered_results()
                                .get(self.selected_index)
                                .map(|r| match r {
                                    scripts::SearchResult::Script(m) => m.script.name.clone(),
                                    scripts::SearchResult::Scriptlet(m) => m.scriptlet.name.clone(),
                                    scripts::SearchResult::BuiltIn(m) => m.entry.name.clone(),
                                    scripts::SearchResult::App(m) => m.app.name.clone(),
                                    scripts::SearchResult::Window(m) => m.window.title.clone(),
                                    scripts::SearchResult::Agent(m) => m.agent.name.clone(),
                                    scripts::SearchResult::Fallback(m) => {
                                        m.fallback.name().to_string()
                                    }
                                })
                        } else {
                            None
                        };
                        (
                            "none".to_string(),
                            None,
                            None,
                            self.filter_text.clone(),
                            self.scripts.len()
                                + self.scriptlets.len()
                                + self.builtin_entries.len()
                                + self.apps.len(),
                            filtered_len,
                            self.selected_index as i32,
                            selected_value,
                        )
                    }
                    AppView::ArgPrompt {
                        id,
                        placeholder,
                        choices,
                        actions: _,
                    } => {
                        let filtered = self.get_filtered_arg_choices(choices);
                        let selected_value = if self.arg_selected_index < filtered.len() {
                            filtered
                                .get(self.arg_selected_index)
                                .map(|c| c.value.clone())
                        } else {
                            None
                        };
                        (
                            "arg".to_string(),
                            Some(id.clone()),
                            Some(placeholder.clone()),
                            self.arg_input.text().to_string(),
                            choices.len(),
                            filtered.len(),
                            self.arg_selected_index as i32,
                            selected_value,
                        )
                    }
                    AppView::DivPrompt { id, .. } => (
                        "div".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::FormPrompt { id, .. } => (
                        "form".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::TermPrompt { id, .. } => (
                        "term".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::EditorPrompt { id, .. } => (
                        "editor".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::SelectPrompt { id, .. } => (
                        "select".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::PathPrompt { id, .. } => (
                        "path".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::EnvPrompt { id, .. } => (
                        "env".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::DropPrompt { id, .. } => (
                        "drop".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::TemplatePrompt { id, .. } => (
                        "template".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::ChatPrompt { id, .. } => (
                        "chat".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::ActionsDialog => (
                        "actions".to_string(),
                        None,
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    // P0 FIX: View state only - data comes from self.cached_clipboard_entries
                    AppView::ClipboardHistoryView {
                        filter,
                        selected_index,
                    } => {
                        let entries = &self.cached_clipboard_entries;
                        let filtered_count = if filter.is_empty() {
                            entries.len()
                        } else {
                            let filter_lower = filter.to_lowercase();
                            entries
                                .iter()
                                .filter(|e| e.text_preview.to_lowercase().contains(&filter_lower))
                                .count()
                        };
                        (
                            "clipboardHistory".to_string(),
                            None,
                            None,
                            filter.clone(),
                            entries.len(),
                            filtered_count,
                            *selected_index as i32,
                            None,
                        )
                    }
                    // P0 FIX: View state only - data comes from self.apps
                    AppView::AppLauncherView {
                        filter,
                        selected_index,
                    } => {
                        let apps = &self.apps;
                        let filtered_count = if filter.is_empty() {
                            apps.len()
                        } else {
                            let filter_lower = filter.to_lowercase();
                            apps.iter()
                                .filter(|a| a.name.to_lowercase().contains(&filter_lower))
                                .count()
                        };
                        (
                            "appLauncher".to_string(),
                            None,
                            None,
                            filter.clone(),
                            apps.len(),
                            filtered_count,
                            *selected_index as i32,
                            None,
                        )
                    }
                    // P0 FIX: View state only - data comes from self.cached_windows
                    AppView::WindowSwitcherView {
                        filter,
                        selected_index,
                    } => {
                        let windows = &self.cached_windows;
                        let filtered_count = if filter.is_empty() {
                            windows.len()
                        } else {
                            let filter_lower = filter.to_lowercase();
                            windows
                                .iter()
                                .filter(|w| {
                                    w.title.to_lowercase().contains(&filter_lower)
                                        || w.app.to_lowercase().contains(&filter_lower)
                                })
                                .count()
                        };
                        (
                            "windowSwitcher".to_string(),
                            None,
                            None,
                            filter.clone(),
                            windows.len(),
                            filtered_count,
                            *selected_index as i32,
                            None,
                        )
                    }
                    AppView::DesignGalleryView {
                        filter,
                        selected_index,
                    } => {
                        let total_items = designs::separator_variations::SeparatorStyle::count()
                            + designs::icon_variations::total_icon_count()
                            + 8
                            + 6; // headers
                        (
                            "designGallery".to_string(),
                            None,
                            None,
                            filter.clone(),
                            total_items,
                            total_items,
                            *selected_index as i32,
                            None,
                        )
                    }
                    AppView::ScratchPadView { .. } => (
                        "scratchPad".to_string(),
                        Some("scratch-pad".to_string()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::QuickTerminalView { .. } => (
                        "quickTerminal".to_string(),
                        Some("quick-terminal".to_string()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::FileSearchView {
                        ref query,
                        selected_index,
                    } => (
                        "fileSearch".to_string(),
                        Some("file-search".to_string()),
                        None,
                        query.clone(),
                        self.cached_file_results.len(),
                        self.cached_file_results.len(),
                        *selected_index as i32,
                        self.cached_file_results
                            .get(*selected_index)
                            .map(|f| f.name.clone()),
                    ),
                };

                // Focus state: we use focused_input as a proxy since we don't have Window access here.
                // When window is visible and we're tracking an input, we're focused.
                let window_visible = script_kit_gpui::is_main_window_visible();
                let is_focused = window_visible && self.focused_input != FocusedInput::None;

                // Create the response
                let response = Message::state_result(
                    request_id.clone(),
                    prompt_type,
                    prompt_id,
                    placeholder,
                    input_value,
                    choice_count,
                    visible_choice_count,
                    selected_index,
                    selected_value,
                    is_focused,
                    window_visible,
                );

                logging::log(
                    "UI",
                    &format!("Sending state result for request: {}", request_id),
                );

                // Send the response - use try_send to avoid blocking UI
                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log("WARN", "Response channel full - state result dropped");
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                } else {
                    logging::log("ERROR", "No response sender available for state result");
                }
            }
            PromptMessage::GetLayoutInfo { request_id } => {
                logging::log(
                    "UI",
                    &format!("Collecting layout info for request: {}", request_id),
                );

                // Build layout info from current window state
                let layout_info = self.build_layout_info(cx);

                // Create the response
                let response = Message::layout_info_result(request_id.clone(), layout_info);

                logging::log(
                    "UI",
                    &format!("Sending layout info result for request: {}", request_id),
                );

                // Send the response - use try_send to avoid blocking UI
                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log("WARN", "Response channel full - layout info dropped");
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                } else {
                    logging::log(
                        "ERROR",
                        "No response sender available for layout info result",
                    );
                }
            }
            PromptMessage::ForceSubmit { value } => {
                logging::log(
                    "UI",
                    &format!("ForceSubmit received with value: {:?}", value),
                );

                // Get the current prompt ID and submit the value
                let prompt_id = match &self.current_view {
                    AppView::ArgPrompt { id, .. } => Some(id.clone()),
                    AppView::DivPrompt { id, .. } => Some(id.clone()),
                    AppView::FormPrompt { id, .. } => Some(id.clone()),
                    AppView::TermPrompt { id, .. } => Some(id.clone()),
                    AppView::EditorPrompt { id, .. } => Some(id.clone()),
                    _ => None,
                };

                if let Some(id) = prompt_id {
                    // Convert serde_json::Value to String for submission
                    let value_str = match &value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        other => other.to_string(),
                    };

                    logging::log(
                        "UI",
                        &format!(
                            "ForceSubmit: submitting '{}' for prompt '{}'",
                            value_str, id
                        ),
                    );
                    self.submit_prompt_response(id, Some(value_str), cx);
                } else {
                    logging::log(
                        "WARN",
                        "ForceSubmit received but no active prompt to submit to",
                    );
                }
            }
            // ============================================================
            // NEW PROMPT TYPES (scaffolding - TODO: implement full UI)
            // ============================================================
            PromptMessage::ShowPath {
                id,
                start_path,
                hint,
            } => {
                logging::log(
                    "UI",
                    &format!(
                        "Showing path prompt: {} (start: {:?}, hint: {:?})",
                        id, start_path, hint
                    ),
                );

                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        logging::log(
                            "UI",
                            &format!(
                                "PathPrompt submit_callback called: id={}, value={:?}",
                                id, value
                            ),
                        );
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - path response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Clone the path_actions_showing and search_text Arcs for header display
                let path_actions_showing = self.path_actions_showing.clone();
                let path_actions_search_text = self.path_actions_search_text.clone();

                let focus_handle = cx.focus_handle();
                let path_prompt = PathPrompt::new(
                    id.clone(),
                    start_path,
                    hint,
                    focus_handle.clone(),
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                )
                // Note: Legacy callbacks are no longer needed - we use events now
                // But we still pass the shared state for header display
                .with_actions_showing(path_actions_showing)
                .with_actions_search_text(path_actions_search_text);

                let entity = cx.new(|_| path_prompt);

                // Subscribe to PathPrompt events for actions dialog control
                // This replaces the mutex-polling pattern with event-driven handling
                cx.subscribe(
                    &entity,
                    |this, _entity, event: &PathPromptEvent, cx| match event {
                        PathPromptEvent::ShowActions(path_info) => {
                            logging::log(
                                "UI",
                                &format!(
                                    "PathPromptEvent::ShowActions received for: {}",
                                    path_info.path
                                ),
                            );
                            this.handle_show_path_actions(path_info.clone(), cx);
                        }
                        PathPromptEvent::CloseActions => {
                            logging::log("UI", "PathPromptEvent::CloseActions received");
                            this.handle_close_path_actions(cx);
                        }
                    },
                )
                .detach();

                self.current_view = AppView::PathPrompt {
                    id,
                    entity,
                    focus_handle,
                };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::PathPrompt);

                // Reset showing state (no more mutex polling needed)
                if let Ok(mut guard) = self.path_actions_showing.lock() {
                    *guard = false;
                }

                resize_to_view_sync(ViewType::ScriptList, 20);
                cx.notify();
            }
            PromptMessage::ShowEnv {
                id,
                key,
                prompt,
                secret,
            } => {
                tracing::info!(id, key, ?prompt, secret, "ShowEnv received");
                logging::log(
                    "UI",
                    &format!(
                        "ShowEnv prompt received: {} (key: {}, secret: {})",
                        id, key, secret
                    ),
                );

                // Create submit callback for env prompt
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - env response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Check if key already exists in secrets (for UX messaging)
                // Empty values don't count as "existing" - must have actual content
                let exists_in_keyring = secrets::get_secret(&key)
                    .map(|v: String| !v.is_empty())
                    .unwrap_or(false);

                // Create EnvPrompt entity
                let focus_handle = self.focus_handle.clone();
                let mut env_prompt = prompts::EnvPrompt::new(
                    id.clone(),
                    key,
                    prompt,
                    secret,
                    focus_handle,
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                    exists_in_keyring,
                );

                // Check keyring first - if value exists, auto-submit without showing UI
                if env_prompt.check_keyring_and_auto_submit() {
                    logging::log("UI", "EnvPrompt: value found in keyring, auto-submitted");
                    // Don't switch view, the callback already submitted
                    cx.notify();
                    return;
                }

                let entity = cx.new(|_| env_prompt);
                self.current_view = AppView::EnvPrompt { id, entity };
                self.focused_input = FocusedInput::None; // EnvPrompt has its own focus handling
                self.pending_focus = Some(FocusTarget::EnvPrompt);

                // Resize to compact height: header + footer only
                resize_to_view_sync(ViewType::ArgPromptNoChoices, 0);
                cx.notify();
            }
            PromptMessage::ShowDrop {
                id,
                placeholder,
                hint,
            } => {
                tracing::info!(id, ?placeholder, ?hint, "ShowDrop received");
                logging::log(
                    "UI",
                    &format!(
                        "ShowDrop prompt received: {} (placeholder: {:?})",
                        id, placeholder
                    ),
                );

                // Create submit callback for drop prompt
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - drop response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Create DropPrompt entity
                let focus_handle = self.focus_handle.clone();
                let drop_prompt = prompts::DropPrompt::new(
                    id.clone(),
                    placeholder,
                    hint,
                    focus_handle,
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                );

                let entity = cx.new(|_| drop_prompt);
                self.current_view = AppView::DropPrompt { id, entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::DropPrompt);

                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
            }
            PromptMessage::ShowTemplate { id, template } => {
                tracing::info!(id, template, "ShowTemplate received");
                logging::log(
                    "UI",
                    &format!(
                        "ShowTemplate prompt received: {} (template: {})",
                        id, template
                    ),
                );

                // Create submit callback for template prompt
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - template response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Create TemplatePrompt entity
                let focus_handle = self.focus_handle.clone();
                let template_prompt = prompts::TemplatePrompt::new(
                    id.clone(),
                    template,
                    focus_handle,
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                );

                let entity = cx.new(|_| template_prompt);
                self.current_view = AppView::TemplatePrompt { id, entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::TemplatePrompt);

                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
            }
            PromptMessage::ShowSelect {
                id,
                placeholder,
                choices,
                multiple,
            } => {
                tracing::info!(
                    id,
                    ?placeholder,
                    choice_count = choices.len(),
                    multiple,
                    "ShowSelect received"
                );
                logging::log(
                    "UI",
                    &format!(
                        "ShowSelect prompt received: {} ({} choices, multiple: {})",
                        id,
                        choices.len(),
                        multiple
                    ),
                );

                // Create submit callback for select prompt
                let response_sender = self.response_sender.clone();
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        if let Some(ref sender) = response_sender {
                            let response = Message::Submit { id, value };
                            // Use try_send to avoid blocking UI thread
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - select response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Create SelectPrompt entity
                let choice_count = choices.len();
                let select_prompt = prompts::SelectPrompt::new(
                    id.clone(),
                    placeholder,
                    choices,
                    multiple,
                    self.focus_handle.clone(),
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                );
                let entity = cx.new(|_| select_prompt);
                self.current_view = AppView::SelectPrompt { id, entity };
                self.focused_input = FocusedInput::None; // SelectPrompt has its own focus handling
                self.pending_focus = Some(FocusTarget::SelectPrompt);

                // Resize window based on number of choices
                let view_type = if choice_count == 0 {
                    ViewType::ArgPromptNoChoices
                } else {
                    ViewType::ArgPromptWithChoices
                };
                resize_to_view_sync(view_type, choice_count);
                cx.notify();
            }
            PromptMessage::ShowConfirm {
                id,
                message,
                confirm_text,
                cancel_text,
            } => {
                logging::log(
                    "CONFIRM",
                    &format!("ShowConfirm prompt: id={}, message={:?}", id, message),
                );

                // Create callback to send response and close the confirm window
                let response_sender = self.response_sender.clone();
                let prompt_id = id.clone();
                let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed: bool| {
                    logging::log(
                        "CONFIRM",
                        &format!(
                            "User choice: {} (id={})",
                            if confirmed { "confirmed" } else { "cancelled" },
                            prompt_id
                        ),
                    );
                    if let Some(ref sender) = response_sender {
                        let value = if confirmed {
                            Some("true".to_string())
                        } else {
                            Some("false".to_string())
                        };
                        let response = Message::Submit {
                            id: prompt_id.clone(),
                            value,
                        };
                        match sender.try_send(response) {
                            Ok(()) => {
                                logging::log("CONFIRM", "Submit message sent");
                            }
                            Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                logging::log(
                                    "WARN",
                                    "Response channel full - confirm response dropped",
                                );
                            }
                            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                logging::log("UI", "Response channel disconnected - script exited");
                            }
                        }
                    }
                });

                // Get main window bounds from native API for positioning
                let main_bounds = if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
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
                    // Fallback to centered on primary display
                    gpui::Bounds {
                        origin: gpui::Point {
                            x: gpui::px(200.0),
                            y: gpui::px(200.0),
                        },
                        size: gpui::Size {
                            width: gpui::px(750.0),
                            height: gpui::px(500.0),
                        },
                    }
                };
                let display_id: Option<gpui::DisplayId> = None; // Use primary display

                // Clone callback for the close handler
                let on_choice_for_close = on_choice.clone();

                // Open confirm window via spawn
                cx.spawn(async move |_this, cx| {
                    cx.update(|cx| {
                        match open_confirm_window(
                            cx,
                            main_bounds,
                            display_id,
                            message,
                            confirm_text,
                            cancel_text,
                            on_choice_for_close,
                        ) {
                            Ok((_handle, _dialog)) => {
                                logging::log("CONFIRM", "Confirm popup window opened");
                            }
                            Err(e) => {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to open confirm window: {}", e),
                                );
                            }
                        }
                    })
                    .ok();
                })
                .detach();

                cx.notify();
            }
            PromptMessage::ShowChat {
                id,
                placeholder,
                messages,
                hint,
                footer,
                actions,
                model,
                models,
                save_history,
            } => {
                tracing::info!(
                    id,
                    ?placeholder,
                    message_count = messages.len(),
                    ?model,
                    model_count = models.len(),
                    save_history,
                    "ShowChat received"
                );
                logging::log(
                    "UI",
                    &format!(
                        "ShowChat prompt received: {} ({} messages, {} models, save={})",
                        id,
                        messages.len(),
                        models.len(),
                        save_history
                    ),
                );

                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                // Create submit callback for chat prompt
                let response_sender = self.response_sender.clone();
                let chat_submit_callback: prompts::ChatSubmitCallback =
                    std::sync::Arc::new(move |id, text| {
                        if let Some(ref sender) = response_sender {
                            // Send ChatSubmit message back to SDK
                            let response = Message::ChatSubmit { id, text };
                            match sender.try_send(response) {
                                Ok(()) => {}
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    logging::log(
                                        "WARN",
                                        "Response channel full - chat response dropped",
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    logging::log(
                                        "UI",
                                        "Response channel disconnected - script exited",
                                    );
                                }
                            }
                        }
                    });

                // Create ChatPrompt entity with configured models
                let focus_handle = self.focus_handle.clone();
                let mut chat_prompt = prompts::ChatPrompt::new(
                    id.clone(),
                    placeholder,
                    messages,
                    hint,
                    footer,
                    focus_handle,
                    chat_submit_callback,
                    std::sync::Arc::clone(&self.theme),
                );

                // Apply model configuration from SDK
                if !models.is_empty() {
                    chat_prompt = chat_prompt.with_model_names(models);
                }
                if let Some(default_model) = model {
                    chat_prompt = chat_prompt.with_default_model(default_model);
                }

                // Configure history saving
                chat_prompt = chat_prompt.with_save_history(save_history);

                let entity = cx.new(|_| chat_prompt);
                self.current_view = AppView::ChatPrompt { id, entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::ChatPrompt);

                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
            }
            PromptMessage::ChatAddMessage { id, message } => {
                logging::log("CHAT", &format!("ChatAddMessage for {}", id));
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.add_message(message, cx);
                        });
                    }
                }
            }
            PromptMessage::ChatStreamStart {
                id,
                message_id,
                position,
            } => {
                logging::log(
                    "CHAT",
                    &format!("ChatStreamStart for {} msg={}", id, message_id),
                );
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.start_streaming(message_id, position, cx);
                        });
                    }
                }
            }
            PromptMessage::ChatStreamChunk {
                id,
                message_id,
                chunk,
            } => {
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.append_chunk(&message_id, &chunk, cx);
                        });
                    }
                }
            }
            PromptMessage::ChatStreamComplete { id, message_id } => {
                logging::log(
                    "CHAT",
                    &format!("ChatStreamComplete for {} msg={}", id, message_id),
                );
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.complete_streaming(&message_id, cx);
                        });
                    }
                }
            }
            PromptMessage::ChatClear { id } => {
                logging::log("CHAT", &format!("ChatClear for {}", id));
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.clear_messages(cx);
                        });
                    }
                }
            }
            PromptMessage::ChatSetError {
                id,
                message_id,
                error,
            } => {
                logging::log(
                    "CHAT",
                    &format!("ChatSetError for {} msg={}: {}", id, message_id, error),
                );
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.set_message_error(&message_id, error.clone(), cx);
                        });
                    }
                }
            }
            PromptMessage::ChatClearError { id, message_id } => {
                logging::log(
                    "CHAT",
                    &format!("ChatClearError for {} msg={}", id, message_id),
                );
                if let AppView::ChatPrompt {
                    id: view_id,
                    entity,
                } = &self.current_view
                {
                    if view_id == &id {
                        entity.update(cx, |chat, cx| {
                            chat.clear_message_error(&message_id, cx);
                        });
                    }
                }
            }
            PromptMessage::ShowHud { text, duration_ms } => {
                self.show_hud(text, duration_ms, cx);
            }
            PromptMessage::SetInput { text } => {
                self.set_prompt_input(text, cx);
            }
            PromptMessage::SetActions { actions } => {
                logging::log(
                    "ACTIONS",
                    &format!("Received setActions with {} actions", actions.len()),
                );

                // Store SDK actions for trigger_action_by_name lookup
                self.sdk_actions = Some(actions.clone());

                // Build action shortcuts map for keyboard handling
                self.action_shortcuts.clear();
                for action in &actions {
                    if let Some(ref shortcut) = action.shortcut {
                        let normalized = shortcuts::normalize_shortcut(shortcut);
                        logging::log(
                            "ACTIONS",
                            &format!(
                                "Registering action shortcut: '{}' -> '{}' (normalized: '{}')",
                                shortcut, action.name, normalized
                            ),
                        );
                        self.action_shortcuts
                            .insert(normalized, action.name.clone());
                    }
                }

                // Update ActionsDialog if it exists and is open
                if let Some(ref dialog) = self.actions_dialog {
                    dialog.update(cx, |d, _cx| {
                        d.set_sdk_actions(actions);
                    });
                }

                cx.notify();
            }
            PromptMessage::AiStartChat {
                request_id,
                message,
                system_prompt,
                image,
                model_id,
                no_response,
            } => {
                logging::log(
                    "AI",
                    &format!(
                        "AiStartChat request: {} (message: {} chars, system_prompt: {}, image: {}, model: {:?}, no_response: {})",
                        request_id,
                        message.len(),
                        system_prompt.is_some(),
                        image.is_some(),
                        model_id,
                        no_response
                    ),
                );

                // Open the AI window (creates new if not open, brings to front if open)
                if let Err(e) = crate::ai::open_ai_window(cx) {
                    tracing::error!(error = %e, "Failed to open AI window for AiStartChat");
                    logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                    // Still send response so SDK doesn't hang
                    if let Some(ref sender) = self.response_sender {
                        let _ = sender.try_send(Message::AiChatCreated {
                            request_id,
                            chat_id: String::new(),
                            title: String::new(),
                            model_id: model_id.unwrap_or_default(),
                            provider: String::new(),
                            streaming_started: false,
                        });
                    }
                    return;
                }

                // Set the input and optionally submit
                // If no_response is false (default), we submit to trigger AI response
                let should_submit = !no_response;

                // Set input with image if provided, otherwise just set text
                if let Some(ref img_base64) = image {
                    crate::ai::set_ai_input_with_image(cx, &message, img_base64, should_submit);
                } else {
                    crate::ai::set_ai_input(cx, &message, should_submit);
                }

                // Generate a chat ID (the AI window will create the actual chat)
                // For now, use a placeholder - the real chat ID is managed by AiApp
                let generated_chat_id = format!("chat-{}", uuid::Uuid::new_v4());
                let title = if message.len() > 30 {
                    format!("{}...", &message[..30])
                } else {
                    message.clone()
                };

                // Send AiChatCreated response back to SDK
                if let Some(ref sender) = self.response_sender {
                    let response = Message::AiChatCreated {
                        request_id: request_id.clone(),
                        chat_id: generated_chat_id,
                        title,
                        model_id: model_id
                            .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
                        provider: "anthropic".to_string(),
                        streaming_started: should_submit,
                    };
                    match sender.try_send(response) {
                        Ok(()) => {
                            logging::log(
                                "AI",
                                &format!("AiChatCreated response sent for {}", request_id),
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log("WARN", "Response channel full - AiChatCreated dropped");
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                }

                cx.notify();
            }
            PromptMessage::AiFocus { request_id } => {
                logging::log("AI", &format!("AiFocus request: {}", request_id));

                // Check if window was already open before we open/focus it
                let was_open = crate::ai::is_ai_window_open();

                // Open the AI window (creates new if not open, brings to front if open)
                let success = match crate::ai::open_ai_window(cx) {
                    Ok(()) => {
                        logging::log("AI", "AI window focused successfully");
                        true
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to focus AI window");
                        logging::log("ERROR", &format!("Failed to focus AI window: {}", e));
                        false
                    }
                };

                // Send AiFocusResult response back to SDK
                if let Some(ref sender) = self.response_sender {
                    let response = Message::AiFocusResult {
                        request_id: request_id.clone(),
                        success,
                        was_open,
                    };
                    match sender.try_send(response) {
                        Ok(()) => {
                            logging::log("AI", &format!("AiFocusResult sent for {}", request_id));
                        }
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log("WARN", "Response channel full - AiFocusResult dropped");
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                }

                cx.notify();
            }
            PromptMessage::ShowGrid { options } => {
                logging::log(
                    "DEBUG_GRID",
                    &format!(
                        "ShowGrid from script: size={}, bounds={}, box_model={}, guides={}",
                        options.grid_size,
                        options.show_bounds,
                        options.show_box_model,
                        options.show_alignment_guides
                    ),
                );
                self.show_grid(options, cx);
            }
            PromptMessage::HideGrid => {
                logging::log("DEBUG_GRID", "HideGrid from script");
                self.hide_grid(cx);
            }
        }
    }
}
