// Prompt message handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs

// --- merged from part_000.rs ---
fn unhandled_message_warning(message_type: &str) -> String {
    format!(
        "'{}' is not supported yet. Update the script to a supported message type or update Script Kit GPUI.",
        message_type
    )
}

fn prompt_coming_soon_warning(prompt_name: &str) -> String {
    format!("{prompt_name} prompt coming soon.")
}

fn resolve_ai_start_chat_provider(
    registry: &crate::ai::ProviderRegistry,
    model_id: &str,
) -> Option<String> {
    registry
        .find_provider_for_model(model_id)
        .map(|provider| provider.provider_id().to_string())
}

#[cfg(any(test, target_os = "windows"))]
fn escape_windows_cmd_open_target(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '^' | '&' | '|' | '<' | '>' | '(' | ')' | '%' | '!' | '"' => {
                escaped.push('^');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptMessageRoute {
    ConfirmDialog,
    UnhandledWarning,
    Other,
}
#[inline]
fn classify_prompt_message_route(message: &PromptMessage) -> PromptMessageRoute {
    match message {
        PromptMessage::ShowConfirm { .. } => PromptMessageRoute::ConfirmDialog,
        PromptMessage::UnhandledMessage { .. } => PromptMessageRoute::UnhandledWarning,
        _ => PromptMessageRoute::Other,
    }
}

// --- merged from part_001.rs ---
impl ScriptListApp {
    pub(crate) fn make_submit_callback(
        &self,
        dropped_label: &'static str,
    ) -> Arc<dyn Fn(String, Option<String>) + Send + Sync> {
        let response_sender = self.response_sender.clone();
        Arc::new(move |id, value| {
            if let Some(ref sender) = response_sender {
                let response = Message::Submit { id, value };
                // Use try_send to avoid blocking UI thread
                match sender.try_send(response) {
                    Ok(()) => {}
                    Err(std::sync::mpsc::TrySendError::Full(_)) => {
                        tracing::warn!(
                            category = "WARN",
                            dropped_label = %dropped_label,
                            "Response channel full - response dropped"
                        );
                    }
                    Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                        tracing::info!(
                            category = "UI",
                            "Response channel disconnected - script exited"
                        );
                    }
                }
            }
        })
    }

    pub(crate) fn prepare_window_for_prompt(
        &self,
        log_target: &str,
        prompt_kind: &str,
        bench_marker: &str,
    ) {
        // Clear NEEDS_RESET when receiving a UI prompt from an active script.
        // This prevents the window from resetting when shown.
        if NEEDS_RESET.swap(false, Ordering::SeqCst) {
            tracing::info!(
                category = log_target,
                prompt_kind = %prompt_kind,
                "Cleared NEEDS_RESET - script is showing prompt UI"
            );
        }

        // Show window if hidden (script may have called hide() for getSelectedText)
        if !script_kit_gpui::is_main_window_visible() {
            if !bench_marker.is_empty() {
                logging::bench_log(bench_marker);
            }
            tracing::info!(
                category = log_target,
                prompt_kind = %prompt_kind,
                "Window hidden - requesting show for prompt UI"
            );
            script_kit_gpui::set_main_window_visible(true);
            script_kit_gpui::request_show_main_window();
        }
    }

    pub(crate) fn set_sdk_actions_and_shortcuts(
        &mut self,
        actions: Vec<ProtocolAction>,
        log_target: &str,
        log_shortcuts: bool,
    ) {
        // Store SDK actions for trigger_action_by_name lookup
        self.sdk_actions = Some(actions.clone());

        // Register keyboard shortcuts for visible SDK actions only
        self.action_shortcuts.clear();
        for action in &actions {
            if action.is_visible() {
                if let Some(shortcut) = &action.shortcut {
                    let normalized = shortcuts::normalize_shortcut(shortcut);
                    if log_shortcuts {
                        tracing::info!(
                            category = log_target,
                            shortcut = %shortcut,
                            action_name = %action.name,
                            normalized = %normalized,
                            "Registering action shortcut"
                        );
                    }
                    self.action_shortcuts
                        .insert(normalized, action.name.clone());
                }
            }
        }
    }

    fn show_prompt_coming_soon_toast(&mut self, prompt_name: &str, cx: &mut Context<Self>) {
        let toast = Toast::warning(prompt_coming_soon_warning(prompt_name), &self.theme)
            .duration_ms(Some(TOAST_WARNING_MS));
        self.toast_manager.push(toast);
        cx.notify();
    }

    /// Handle a prompt message from the script
    #[tracing::instrument(skip(self, cx), fields(msg_type = ?msg))]
    fn handle_prompt_message(&mut self, msg: PromptMessage, cx: &mut Context<Self>) {
        let route = classify_prompt_message_route(&msg);
        tracing::debug!(target: "prompt_handler", ?route, "Routing prompt message");

        match msg {
            PromptMessage::ShowArg {
                id,
                placeholder,
                choices,
                actions,
            } => {
                self.prepare_window_for_prompt("UI", "arg", "");

                tracing::info!(
                    category = "UI",
                    id = %id,
                    choice_count = choices.len(),
                    action_count = actions.as_ref().map(|a| a.len()).unwrap_or(0),
                    "Showing arg prompt"
                );
                let choice_count = choices.len();

                // If actions were provided, store them in the SDK actions system
                // so they can be triggered via shortcuts and Cmd+K
                if let Some(ref action_list) = actions {
                    self.set_sdk_actions_and_shortcuts(action_list.clone(), "UI", false);
                } else {
                    // Clear any previous SDK actions
                    self.sdk_actions = None;
                    self.action_shortcuts.clear();
                }

                let pending_placeholder = placeholder.clone();
                self.current_view = AppView::ArgPrompt {
                    id,
                    placeholder,
                    choices,
                    actions,
                };
                self.arg_input.clear();
                self.filter_text.clear();
                self.arg_selected_index = 0;
                self.focused_input = FocusedInput::ArgPrompt;
                self.pending_filter_sync = true;
                self.pending_placeholder = Some(pending_placeholder);
                self.pending_focus = Some(FocusTarget::MainFilter);
                // Resize window based on number of choices
                let view_type = if choice_count == 0 {
                    ViewType::ArgPromptNoChoices
                } else {
                    ViewType::ArgPromptWithChoices
                };
                resize_to_view_sync(view_type, choice_count);
                cx.notify();
            }
            PromptMessage::ShowMini {
                id,
                placeholder,
                choices,
            } => {
                self.prepare_window_for_prompt("UI", "mini", "");

                tracing::info!(
                    category = "UI",
                    id = %id,
                    choice_count = choices.len(),
                    "Showing mini prompt"
                );
                let choice_count = choices.len();

                // Clear any previous SDK actions (mini has no actions)
                self.sdk_actions = None;
                self.action_shortcuts.clear();

                let pending_placeholder = placeholder.clone();
                self.current_view = AppView::MiniPrompt {
                    id,
                    placeholder,
                    choices,
                };
                self.arg_input.clear();
                self.filter_text.clear();
                self.arg_selected_index = 0;
                self.focused_input = FocusedInput::ArgPrompt;
                self.pending_filter_sync = true;
                self.pending_placeholder = Some(pending_placeholder);
                self.pending_focus = Some(FocusTarget::MainFilter);
                let view_type = if choice_count == 0 {
                    ViewType::ArgPromptNoChoices
                } else {
                    ViewType::ArgPromptWithChoices
                };
                resize_to_view_sync(view_type, choice_count);
                cx.notify();
            }
            PromptMessage::ShowMicro {
                id,
                placeholder,
                choices,
            } => {
                self.prepare_window_for_prompt("UI", "micro", "");

                tracing::info!(
                    category = "UI",
                    id = %id,
                    choice_count = choices.len(),
                    "Showing micro prompt"
                );

                // Clear any previous SDK actions (micro has no actions)
                self.sdk_actions = None;
                self.action_shortcuts.clear();

                self.current_view = AppView::MicroPrompt {
                    id,
                    placeholder,
                    choices,
                };
                self.arg_input.clear();
                self.arg_selected_index = 0;
                self.focused_input = FocusedInput::ArgPrompt;
                self.pending_focus = Some(FocusTarget::AppRoot);
                // Micro always uses compact (no-choices) height
                resize_to_view_sync(ViewType::ArgPromptNoChoices, 0);
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
                self.prepare_window_for_prompt("UI", "div", "");

                tracing::info!(category = "UI", id = %id, "Showing div prompt");
                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                let submit_callback = self.make_submit_callback("div");

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
                self.prepare_window_for_prompt("UI", "form", "");

                tracing::info!(category = "UI", id = %id, "Showing form prompt");

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
                self.prepare_window_for_prompt("UI", "term", "");

                tracing::info!(
                    category = "UI",
                    id = %id,
                    command = ?command,
                    "Showing term prompt"
                );

                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                let submit_callback = self.make_submit_callback("terminal");

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
                        tracing::error!(category = "ERROR", error = %e, "Failed to create terminal");
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
                self.prepare_window_for_prompt("UI", "editor", "");

                tracing::info!(
                    category = "UI",
                    id = %id,
                    language = ?language,
                    has_template = template.is_some(),
                    "Showing editor prompt"
                );

                // Store SDK actions for the actions panel (Cmd+K)
                self.sdk_actions = actions;

                let submit_callback = self.make_submit_callback("editor");

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
                let has_tabstops =
                    crate::snippet::analysis::contains_explicit_tabstops(&content_str);

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
                    tracing::info!(
                        category = "UI",
                        content = %content_str,
                        "Auto-detected template in content"
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
                tracing::info!(
                    category = "VISIBILITY",
                    "=== ScriptExit message received ==="
                );

                // Complete pending Tab AI execution on clean exit.
                // If ScriptError already consumed the record, this is a no-op.
                self.complete_tab_ai_execution(true, None, cx);

                let was_visible = script_kit_gpui::is_main_window_visible();
                let script_hid_window = script_kit_gpui::script_requested_hide();
                tracing::info!(
                    category = "VISIBILITY",
                    was_visible,
                    script_hid_window,
                    "Window visibility state before script exit reset"
                );

                // Reset the script-requested-hide flag
                script_kit_gpui::set_script_requested_hide(false);
                tracing::info!(
                    category = "VISIBILITY",
                    "SCRIPT_REQUESTED_HIDE reset to: false"
                );

                let keep_tab_ai_save_offer_open = self.tab_ai_save_offer_state.is_some();

                if keep_tab_ai_save_offer_open {
                    tracing::info!(
                        category = "VISIBILITY",
                        keep_tab_ai_save_offer_open,
                        "Tab AI active after script exit - preserving view"
                    );

                    if script_hid_window {
                        tracing::info!(
                            category = "VISIBILITY",
                            "Script had hidden window - requesting show for Tab AI"
                        );
                        script_kit_gpui::request_show_main_window();
                    }

                    return;
                }

                // Set flag so next hotkey show will reset to script list
                NEEDS_RESET.store(true, Ordering::SeqCst);
                tracing::info!(category = "VISIBILITY", "NEEDS_RESET set to: true");

                self.reset_to_script_list(cx);
                tracing::info!(category = "VISIBILITY", "reset_to_script_list() called");

                // If the script had hidden the window (e.g., for getSelectedText),
                // request showing the main window so the menu comes back
                if script_hid_window {
                    tracing::info!(
                        category = "VISIBILITY",
                        "Script had hidden window - requesting show main window"
                    );
                    script_kit_gpui::request_show_main_window();
                } else {
                    // Script didn't hide window, so it was user-initiated hide or already visible
                    // Restore window height to main menu size in case a prompt (like EnvPrompt)
                    // had shrunk the window
                    resize_to_view_sync(ViewType::ScriptList, 0);
                    self.hide_main_and_reset(cx);
                    tracing::info!(
                        category = "VISIBILITY",
                        "Script didn't hide window - restored height and hid/reset main window"
                    );
                }
            }
            PromptMessage::HideWindow => {
                tracing::info!(
                    category = "VISIBILITY",
                    "=== HideWindow message received ==="
                );
                let was_visible = script_kit_gpui::is_main_window_visible();
                tracing::info!(
                    category = "VISIBILITY",
                    was_visible,
                    "Window visibility state before hide request"
                );

                // Mark that script requested hide - so ScriptExit knows to show window again
                script_kit_gpui::set_script_requested_hide(true);
                tracing::info!(
                    category = "VISIBILITY",
                    "SCRIPT_REQUESTED_HIDE set to: true"
                );

                self.hide_main_and_reset(cx);
                tracing::info!(
                    category = "VISIBILITY",
                    "hide_main_and_reset() called - main window hidden and reset requested"
                );
            }
            PromptMessage::OpenBrowser { url } => {
                tracing::info!(category = "UI", url = %url, "Opening browser");
                #[cfg(target_os = "macos")]
                {
                    match std::process::Command::new("open").arg(&url).spawn() {
                        Ok(_) => tracing::info!(
                            category = "UI",
                            url = %url,
                            "Successfully opened URL in browser"
                        ),
                        Err(e) => {
                            tracing::error!(
                                category = "ERROR",
                                url = %url,
                                error = %e,
                                "Failed to open URL"
                            )
                        }
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    match std::process::Command::new("xdg-open").arg(&url).spawn() {
                        Ok(_) => tracing::info!(
                            category = "UI",
                            url = %url,
                            "Successfully opened URL in browser"
                        ),
                        Err(e) => {
                            tracing::error!(
                                category = "ERROR",
                                url = %url,
                                error = %e,
                                "Failed to open URL"
                            )
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                {
                    let escaped_url = escape_windows_cmd_open_target(&url);
                    match std::process::Command::new("cmd")
                        .args(["/C", "start", ""])
                        .arg(&escaped_url)
                        .spawn()
                    {
                        Ok(_) => tracing::info!(
                            category = "UI",
                            url = %url,
                            "Successfully opened URL in browser"
                        ),
                        Err(e) => {
                            tracing::error!(
                                category = "ERROR",
                                url = %url,
                                error = %e,
                                "Failed to open URL"
                            )
                        }
                    }
                }
            }
            PromptMessage::RunScript { path } => {
                tracing::info!(category = "EXEC", path = %path, "RunScript command received");

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

                tracing::info!(
                    category = "EXEC",
                    script_name = %script_name,
                    "Executing script"
                );
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
                tracing::error!(
                    category = "ERROR",
                    error_message = %error_message,
                    exit_code = ?exit_code,
                    script_path = %script_path,
                    "Script error received"
                );
                if let Some(ref stderr) = stderr_output {
                    tracing::error!(
                        category = "ERROR",
                        script_path = %script_path,
                        stderr = %stderr,
                        "Script stderr output"
                    );
                }
                if let Some(ref trace) = stack_trace {
                    tracing::error!(
                        category = "ERROR",
                        script_path = %script_path,
                        stack_trace = %trace,
                        "Script stack trace"
                    );
                }

                // CRITICAL: Show error via HUD (highly visible floating window)
                // This ensures the user sees the error even if the main window is hidden/dismissed
                // HUD appears at bottom-center of screen for 5 seconds
                let hud_message = if error_message.chars().count() > 140 {
                    // Use chars().take() to safely handle multi-byte UTF-8 characters
                    let truncated: String = error_message.chars().take(137).collect();
                    format!("Script Error: {}...", truncated)
                } else {
                    format!("Script Error: {}", error_message)
                };
                self.show_hud(hud_message, Some(HUD_SLOW_MS), cx);

                // Also create in-app toast with expandable details (for when window is visible)
                // Use stderr_output if available, otherwise use stack_trace
                let details_text = stderr_output.clone().or_else(|| stack_trace.clone());
                let toast = Toast::error(error_message.clone(), &self.theme)
                    .details_opt(details_text.clone())
                    .duration_ms(Some(TOAST_CRITICAL_MS)); // 10 seconds for errors

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
                                tracing::info!(category = "UI", "Error copied to clipboard");
                            }
                        }),
                    ))
                } else {
                    toast
                };

                // Log suggestions if present
                if !suggestions.is_empty() {
                    tracing::error!(
                        category = "ERROR",
                        suggestions = ?suggestions,
                        "Script error suggestions"
                    );
                }

                // Push toast to manager
                let toast_id = self.toast_manager.push(toast);
                tracing::info!(
                    category = "UI",
                    script_path = %script_path,
                    toast_id = %toast_id,
                    "Toast created for script error"
                );

                // Complete pending Tab AI execution on failure.
                // Consumes the record so the subsequent ScriptExit is a no-op.
                let tab_ai_error_msg = format!(
                    "Tab AI script exited with code {:?}: {}",
                    exit_code, error_message
                );
                self.complete_tab_ai_execution(false, Some(tab_ai_error_msg), cx);

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
                    .duration_ms(Some(TOAST_ERROR_DETAILED_MS));

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
                tracing::warn!(
                    category = "WARN",
                    message_type = %message_type,
                    "Displaying unhandled message warning"
                );

                let toast = Toast::warning(unhandled_message_warning(&message_type), &self.theme)
                    .duration_ms(Some(TOAST_WARNING_MS));

                self.toast_manager.push(toast);
                cx.notify();
            }

            PromptMessage::GetState { request_id } => {
                tracing::info!(
                    category = "UI",
                    request_id = %request_id,
                    "Collecting state for request"
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
                    AppView::MiniPrompt {
                        id,
                        placeholder,
                        choices,
                    } => {
                        let filtered = self.get_filtered_arg_choices(choices);
                        let selected_value = filtered
                            .get(self.arg_selected_index)
                            .map(|c| c.value.clone());
                        (
                            "mini".to_string(),
                            Some(id.clone()),
                            Some(placeholder.clone()),
                            self.arg_input.text().to_string(),
                            choices.len(),
                            filtered.len(),
                            self.arg_selected_index as i32,
                            selected_value,
                        )
                    }
                    AppView::MicroPrompt {
                        id,
                        placeholder,
                        choices,
                    } => {
                        let filtered = self.get_filtered_arg_choices(choices);
                        let selected_value = filtered
                            .get(self.arg_selected_index)
                            .map(|c| c.value.clone());
                        (
                            "micro".to_string(),
                            Some(id.clone()),
                            Some(placeholder.clone()),
                            self.arg_input.text().to_string(),
                            choices.len(),
                            filtered.len(),
                            self.arg_selected_index as i32,
                            selected_value,
                        )
                    }
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
                    #[cfg(feature = "storybook")]
                    AppView::DesignExplorerView { .. } => (
                        "designExplorer".to_string(),
                        None,
                        None,
                        String::new(),
                        0,
                        0,
                        0,
                        None,
                    ),
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
                        ..
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
                    AppView::ThemeChooserView { selected_index, .. } => (
                        "themeChooser".to_string(),
                        Some("theme-chooser".to_string()),
                        None,
                        String::new(),
                        0,
                        0,
                        *selected_index as i32,
                        None,
                    ),
                    AppView::EmojiPickerView {
                        filter,
                        selected_index,
                        selected_category,
                    } => {
                        let filtered_count = crate::emoji::search_emojis(filter)
                            .into_iter()
                            .filter(|emoji| {
                                selected_category
                                    .map(|category| emoji.category == category)
                                    .unwrap_or(true)
                            })
                            .count();
                        (
                            "emojiPicker".to_string(),
                            Some("emoji-picker".to_string()),
                            None,
                            filter.clone(),
                            filtered_count,
                            filtered_count,
                            *selected_index as i32,
                            None,
                        )
                    }
                    AppView::WebcamView { .. } => (
                        "webcam".to_string(),
                        Some("webcam".to_string()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::CreationFeedback { .. } => (
                        "creationFeedback".to_string(),
                        None,
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::NamingPrompt { id, .. } => (
                        "namingPrompt".to_string(),
                        Some(id.clone()),
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
                    ),
                    AppView::BrowseKitsView {
                        query,
                        selected_index,
                        results,
                    } => (
                        "browseKits".to_string(),
                        None,
                        None,
                        query.clone(),
                        results.len(),
                        results.len(),
                        *selected_index as i32,
                        None,
                    ),
                    AppView::InstalledKitsView {
                        selected_index,
                        kits,
                    } => (
                        "installedKits".to_string(),
                        None,
                        None,
                        String::new(),
                        kits.len(),
                        kits.len(),
                        *selected_index as i32,
                        None,
                    ),
                    AppView::ProcessManagerView {
                        filter,
                        selected_index,
                    } => {
                        let total = self.cached_processes.len();
                        let filtered_count = if filter.is_empty() {
                            total
                        } else {
                            let filter_lower = filter.to_lowercase();
                            self.cached_processes
                                .iter()
                                .filter(|p| p.script_path.to_lowercase().contains(&filter_lower))
                                .count()
                        };
                        (
                            "processManager".to_string(),
                            None,
                            None,
                            filter.clone(),
                            total,
                            filtered_count,
                            *selected_index as i32,
                            None,
                        )
                    }
                    AppView::CurrentAppCommandsView {
                        filter,
                        selected_index,
                    } => {
                        let total = self.cached_current_app_entries.len();
                        let filtered_count = if filter.is_empty() {
                            total
                        } else {
                            let filter_lower = filter.to_lowercase();
                            self.cached_current_app_entries
                                .iter()
                                .filter(|e| {
                                    e.name.to_lowercase().contains(&filter_lower)
                                        || e.keywords.iter().any(|k| k.contains(&filter_lower))
                                })
                                .count()
                        };
                        (
                            "currentAppCommands".to_string(),
                            None,
                            None,
                            filter.clone(),
                            total,
                            filtered_count,
                            *selected_index as i32,
                            None,
                        )
                    }
                    AppView::SearchAiPresetsView {
                        filter,
                        selected_index,
                    } => (
                        "searchAiPresets".to_string(),
                        None,
                        None,
                        filter.clone(),
                        0,
                        0,
                        *selected_index as i32,
                        None,
                    ),
                    AppView::CreateAiPresetView { .. } => (
                        "createAiPreset".to_string(),
                        None,
                        None,
                        String::new(),
                        0,
                        0,
                        0,
                        None,
                    ),
                    AppView::SettingsView { selected_index } => (
                        "settings".to_string(),
                        None,
                        None,
                        String::new(),
                        0,
                        0,
                        *selected_index as i32,
                        None,
                    ),
                    AppView::FavoritesBrowseView {
                        filter,
                        selected_index,
                    } => (
                        "favorites".to_string(),
                        None,
                        None,
                        filter.clone(),
                        0,
                        0,
                        *selected_index as i32,
                        None,
                    ),
                    AppView::AcpChatView { .. } => (
                        "acpChat".to_string(),
                        None,
                        None,
                        String::new(),
                        0,
                        0,
                        -1,
                        None,
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

                tracing::info!(
                    category = "UI",
                    request_id = %request_id,
                    "Sending state result for request"
                );

                // Send the response - use try_send to avoid blocking UI
                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "WARN",
                                "Response channel full - state result dropped"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "UI",
                                "Response channel disconnected - script exited"
                            );
                        }
                    }
                } else {
                    tracing::error!(
                        category = "ERROR",
                        "No response sender available for state result"
                    );
                }
            }

            PromptMessage::GetAcpState { request_id } => {
                tracing::info!(
                    category = "ACP_STATE",
                    request_id = %request_id,
                    "acp_state.request"
                );

                let state = self.collect_acp_state(cx);

                tracing::info!(
                    target: "script_kit::acp_telemetry",
                    category = "ACP_STATE",
                    request_id = %request_id,
                    status = %state.status,
                    cursor_index = state.cursor_index,
                    picker_open = state.picker.as_ref().map_or(false, |p| p.open),
                    message_count = state.message_count,
                    context_ready = state.context_ready,
                    "acp_state.result"
                );

                let response = Message::acp_state_result(request_id.clone(), state);

                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "ACP_STATE",
                                request_id = %request_id,
                                "acp_state.response_channel_full"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "ACP_STATE",
                                request_id = %request_id,
                                "acp_state.response_channel_disconnected"
                            );
                        }
                    }
                } else {
                    tracing::error!(
                        category = "ACP_STATE",
                        request_id = %request_id,
                        "acp_state.no_response_sender"
                    );
                }
            }

            PromptMessage::ResetAcpTestProbe { request_id } => {
                tracing::info!(
                    category = "ACP_PROBE",
                    request_id = %request_id,
                    "acp_test_probe.reset"
                );

                self.reset_acp_test_probe(cx);

                // Respond with the current (now-empty) probe snapshot.
                let probe = self.collect_acp_test_probe(protocol::ACP_TEST_PROBE_MAX_EVENTS, cx);
                let response = Message::acp_test_probe_result(request_id.clone(), probe);

                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "ACP_PROBE",
                                request_id = %request_id,
                                "acp_test_probe.response_channel_full"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "ACP_PROBE",
                                request_id = %request_id,
                                "acp_test_probe.response_channel_disconnected"
                            );
                        }
                    }
                }
            }

            PromptMessage::GetAcpTestProbe { request_id, tail } => {
                let tail = tail.unwrap_or(protocol::ACP_TEST_PROBE_MAX_EVENTS).clamp(1, protocol::ACP_TEST_PROBE_MAX_EVENTS);
                tracing::info!(
                    category = "ACP_PROBE",
                    request_id = %request_id,
                    tail,
                    "acp_test_probe.request"
                );

                let probe = self.collect_acp_test_probe(tail, cx);
                let response = Message::acp_test_probe_result(request_id.clone(), probe);

                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "ACP_PROBE",
                                request_id = %request_id,
                                "acp_test_probe.response_channel_full"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "ACP_PROBE",
                                request_id = %request_id,
                                "acp_test_probe.response_channel_disconnected"
                            );
                        }
                    }
                }
            }

            PromptMessage::GetElements { request_id, limit } => {
                let max_elements = limit.unwrap_or(50).clamp(1, 1000);

                tracing::info!(
                    category = "UI_ELEMENTS",
                    request_id = %request_id,
                    limit = max_elements,
                    "ui.elements.request"
                );

                let outcome = self.collect_visible_elements(max_elements, cx);
                let returned_count = outcome.elements.len();
                let focused_semantic_id = outcome.focused_semantic_id();
                let selected_semantic_id = outcome.selected_semantic_id();
                let truncated = outcome.total_count > returned_count;
                let warnings = outcome.warnings.clone();

                tracing::info!(
                    category = "UI_ELEMENTS",
                    request_id = %request_id,
                    limit = max_elements,
                    returned_count = returned_count,
                    total_count = outcome.total_count,
                    truncated = truncated,
                    focused_semantic_id = focused_semantic_id.as_deref().unwrap_or(""),
                    selected_semantic_id = selected_semantic_id.as_deref().unwrap_or(""),
                    warnings = ?warnings,
                    "ui.elements.result"
                );

                let response = Message::elements_result(
                    request_id.clone(),
                    outcome.elements,
                    outcome.total_count,
                    focused_semantic_id,
                    selected_semantic_id,
                    warnings,
                );

                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "UI_ELEMENTS",
                                request_id = %request_id,
                                "ui.elements.response_channel_full"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "UI_ELEMENTS",
                                request_id = %request_id,
                                "ui.elements.response_channel_disconnected"
                            );
                        }
                    }
                } else {
                    tracing::error!(
                        category = "UI_ELEMENTS",
                        request_id = %request_id,
                        "ui.elements.no_response_sender"
                    );
                }
            }

            PromptMessage::GetLayoutInfo { request_id } => {
                tracing::info!(
                    category = "UI",
                    request_id = %request_id,
                    "Collecting layout info for request"
                );

                // Build layout info from current window state
                let layout_info = self.build_layout_info(cx);

                // Create the response
                let response = Message::layout_info_result(request_id.clone(), layout_info);

                tracing::info!(
                    category = "UI",
                    request_id = %request_id,
                    "Sending layout info result for request"
                );

                // Send the response - use try_send to avoid blocking UI
                if let Some(ref sender) = self.response_sender {
                    match sender.try_send(response) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "WARN",
                                "Response channel full - layout info dropped"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "UI",
                                "Response channel disconnected - script exited"
                            );
                        }
                    }
                } else {
                    tracing::error!(
                        category = "ERROR",
                        "No response sender available for layout info result"
                    );
                }
            }
            PromptMessage::WaitFor {
                request_id,
                condition,
                timeout,
                poll_interval,
            } => {
                let timeout_ms = timeout.unwrap_or(5_000);
                let poll_ms = poll_interval.unwrap_or(25);
                let rid = request_id.clone();

                tracing::info!(
                    category = "WAIT",
                    request_id = %rid,
                    timeout_ms = timeout_ms,
                    poll_ms = poll_ms,
                    "wait_for.start"
                );

                // Check if condition is already satisfied
                if self.wait_condition_satisfied(&condition, cx) {
                    tracing::info!(
                        category = "AUTOMATION",
                        request_id = %rid,
                        success = true,
                        elapsed_ms = 0_u64,
                        error_code = "",
                        "automation.wait_for.completed"
                    );
                    let response = Message::wait_for_result(
                        request_id.clone(),
                        true,
                        0,
                        None::<crate::protocol::TransactionError>,
                    );
                    if let Some(ref sender) = self.response_sender {
                        let _ = sender.try_send(response);
                    }
                } else {
                    // Poll asynchronously
                    let sender = self.response_sender.clone();
                    let condition = condition.clone();
                    cx.spawn(async move |this, cx| {
                        let start = std::time::Instant::now();
                        let timeout_dur = std::time::Duration::from_millis(timeout_ms);
                        let poll_dur = std::time::Duration::from_millis(poll_ms);
                        loop {
                            cx.background_executor().timer(poll_dur).await;
                            if start.elapsed() >= timeout_dur {
                                let elapsed_ms = start.elapsed().as_millis() as u64;
                                tracing::info!(
                                    category = "AUTOMATION",
                                    request_id = %rid,
                                    success = false,
                                    elapsed_ms = elapsed_ms,
                                    error_code = "wait_condition_timeout",
                                    "automation.wait_for.completed"
                                );
                                if let Some(ref s) = sender {
                                    let _ = s.try_send(Message::wait_for_result(
                                        rid.clone(),
                                        false,
                                        start.elapsed().as_millis() as u64,
                                        Some(crate::protocol::TransactionError {
                                            code: crate::protocol::TransactionErrorCode::WaitConditionTimeout,
                                            message: format!("Timeout after {}ms", timeout_ms),
                                            suggestion: None,
                                        }),
                                    ));
                                }
                                break;
                            }
                            match this.update(cx, |this, cx| {
                                this.wait_condition_satisfied(&condition, cx)
                            }) {
                                Ok(true) => {
                                    let elapsed_ms = start.elapsed().as_millis() as u64;
                                    tracing::info!(
                                        category = "AUTOMATION",
                                        request_id = %rid,
                                        success = true,
                                        elapsed_ms = elapsed_ms,
                                        error_code = "",
                                        "automation.wait_for.completed"
                                    );
                                    if let Some(ref s) = sender {
                                        let _ = s.try_send(Message::wait_for_result(
                                            rid.clone(),
                                            true,
                                            start.elapsed().as_millis() as u64,
                                            None::<crate::protocol::TransactionError>,
                                        ));
                                    }
                                    break;
                                }
                                Ok(false) => continue,
                                Err(_) => {
                                    tracing::info!(
                                        category = "AUTOMATION",
                                        request_id = %rid,
                                        success = false,
                                        elapsed_ms = start.elapsed().as_millis() as u64,
                                        error_code = "action_failed",
                                        "automation.wait_for.completed"
                                    );
                                    if let Some(ref s) = sender {
                                        let _ = s.try_send(Message::wait_for_result(
                                            rid.clone(),
                                            false,
                                            start.elapsed().as_millis() as u64,
                                            Some(crate::protocol::TransactionError {
                                                code: crate::protocol::TransactionErrorCode::ActionFailed,
                                                message: "Entity dropped during WaitFor".to_string(),
                                                suggestion: None,
                                            }),
                                        ));
                                    }
                                    break;
                                }
                            }
                        }
                    })
                    .detach();
                }
            }

            PromptMessage::Batch {
                request_id,
                commands,
                options,
            } => {
                let opts = options.unwrap_or(protocol::BatchOptions {
                    stop_on_error: true,
                    rollback_on_error: false,
                    timeout: 5_000,
                });
                let rid = request_id.clone();
                let sender = self.response_sender.clone();

                tracing::info!(
                    category = "BATCH",
                    request_id = %rid,
                    command_count = commands.len(),
                    "batch.start"
                );

                cx.spawn(async move |this, cx| {
                    let batch_start = std::time::Instant::now();
                    let batch_timeout = std::time::Duration::from_millis(opts.timeout);
                    let mut results: Vec<protocol::BatchResultEntry> = Vec::new();
                    let mut failed = false;

                    for (index, cmd) in commands.iter().enumerate() {
                        // Check batch timeout
                        if batch_start.elapsed() >= batch_timeout {
                            let entry = protocol::BatchResultEntry {
                                index,
                                success: false,
                                command: batch_command_name(cmd),
                                elapsed: Some(0),
                                value: None,
                                error: Some(protocol::TransactionError::wait_timeout("Batch timeout exceeded")),
                            };
                            results.push(entry);
                            failed = true;
                            break;
                        }

                        let cmd_start = std::time::Instant::now();
                        match cmd {
                            protocol::BatchCommand::SetInput { text } => {
                                match this.update(cx, |this, cx| {
                                    this.set_input_text(text, cx);
                                }) {
                                    Ok(()) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "setInput", "batch.step.ok");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "setInput".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: None,
                                        });
                                    }
                                    Err(e) => {
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "setInput".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                            protocol::BatchCommand::SelectByValue { value, submit } => {
                                let submit = *submit;
                                let value = value.clone();
                                match this.update(cx, |this, cx| {
                                    this.select_choice_by_value(&value, submit, cx)
                                }) {
                                    Ok(Ok(v)) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "selectByValue", value = %v, "batch.step.ok");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "selectByValue".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: Some(v),
                                            error: None,
                                        });
                                    }
                                    Ok(Err(e)) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "selectByValue", error = %e, "batch.step.error");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "selectByValue".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                    Err(e) => {
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "selectByValue".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                            protocol::BatchCommand::SelectBySemanticId { semantic_id, submit } => {
                                let submit = *submit;
                                let semantic_id = semantic_id.clone();
                                match this.update(cx, |this, cx| {
                                    this.select_choice_by_semantic_id(&semantic_id, submit, cx)
                                }) {
                                    Ok(Ok(v)) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "selectBySemanticId", value = %v, "batch.step.ok");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "selectBySemanticId".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: Some(v),
                                            error: None,
                                        });
                                    }
                                    Ok(Err(e)) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "selectBySemanticId", error = %e, "batch.step.error");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "selectBySemanticId".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::selection_not_found(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                    Err(e) => {
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "selectBySemanticId".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                            protocol::BatchCommand::FilterAndSelect { filter, select_first, submit } => {
                                let filter = filter.clone();
                                let select_first = *select_first;
                                let submit = *submit;
                                match this.update(cx, |this, cx| {
                                    this.set_input_text(&filter, cx);
                                    if select_first {
                                        this.select_first_choice(submit, cx)
                                    } else {
                                        Ok(None)
                                    }
                                }) {
                                    Ok(Ok(selected_value)) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "filterAndSelect", filter = %filter, selected = ?selected_value, "batch.step.ok");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "filterAndSelect".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: selected_value,
                                            error: None,
                                        });
                                    }
                                    Ok(Err(e)) | Err(e) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "filterAndSelect", error = %e, "batch.step.error");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "filterAndSelect".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                            protocol::BatchCommand::TypeAndSubmit { text } => {
                                let text = text.clone();
                                match this.update(cx, |this, cx| {
                                    this.set_input_text(&text, cx);
                                    this.submit_current_value(cx);
                                }) {
                                    Ok(()) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "typeAndSubmit", "batch.step.ok");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "typeAndSubmit".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: None,
                                        });
                                    }
                                    Err(e) => {
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "typeAndSubmit".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                            protocol::BatchCommand::ForceSubmit { value } => {
                                let value = value.clone();
                                match this.update(cx, |this, cx| {
                                    let prompt_id = match &this.current_view {
                                        AppView::ArgPrompt { id, .. } => Some(id.clone()),
                                        AppView::DivPrompt { id, .. } => Some(id.clone()),
                                        AppView::FormPrompt { id, .. } => Some(id.clone()),
                                        AppView::TermPrompt { id, .. } => Some(id.clone()),
                                        AppView::EditorPrompt { id, .. } => Some(id.clone()),
                                        _ => None,
                                    };
                                    if let Some(id) = prompt_id {
                                        let value_str = match &value {
                                            serde_json::Value::String(s) => s.clone(),
                                            serde_json::Value::Null => String::new(),
                                            other => other.to_string(),
                                        };
                                        this.submit_prompt_response(id, Some(value_str.clone()), cx);
                                        Ok(value_str)
                                    } else {
                                        Err(anyhow::anyhow!("No active prompt to submit to"))
                                    }
                                }) {
                                    Ok(Ok(v)) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "forceSubmit", "batch.step.ok");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "forceSubmit".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: Some(v),
                                            error: None,
                                        });
                                    }
                                    Ok(Err(e)) | Err(e) => {
                                        tracing::info!(category = "BATCH", request_id = %rid, index = index, command = "forceSubmit", error = %e, "batch.step.error");
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "forceSubmit".to_string(),
                                            elapsed: Some(cmd_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed(format!("{e}"))),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                            protocol::BatchCommand::WaitFor { condition, timeout, poll_interval } => {
                                let wait_timeout = std::time::Duration::from_millis(timeout.unwrap_or(5_000));
                                let wait_poll = std::time::Duration::from_millis(poll_interval.unwrap_or(25));
                                let wait_start = std::time::Instant::now();

                                // Check if already satisfied
                                let already = this.update(cx, |this, cx| {
                                    this.wait_condition_satisfied(condition, cx)
                                });
                                match already {
                                    Ok(true) => {
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: true,
                                            command: "waitFor".to_string(),
                                            elapsed: Some(0),
                                            value: None,
                                            error: None,
                                        });
                                    }
                                    Ok(false) => {
                                        // Poll loop
                                        let mut wait_result: Result<Option<String>, protocol::TransactionError> = Err(protocol::TransactionError::wait_timeout(format!("WaitFor timeout after {}ms", wait_timeout.as_millis())));
                                        loop {
                                            cx.background_executor().timer(wait_poll).await;
                                            if wait_start.elapsed() >= wait_timeout {
                                                break;
                                            }
                                            match this.update(cx, |this, cx| {
                                                this.wait_condition_satisfied(condition, cx)
                                            }) {
                                                Ok(true) => { wait_result = Ok(None); break; }
                                                Ok(false) => continue,
                                                _ => { wait_result = Err(protocol::TransactionError::action_failed("Entity dropped during WaitFor")); break; }
                                            }
                                        }
                                        match wait_result {
                                            Ok(_) => {
                                                results.push(protocol::BatchResultEntry {
                                                    index,
                                                    success: true,
                                                    command: "waitFor".to_string(),
                                                    elapsed: Some(wait_start.elapsed().as_millis() as u64),
                                                    value: None,
                                                    error: None,
                                                });
                                            }
                                            Err(e) => {
                                                tracing::info!(
                                                    category = "BATCH",
                                                    request_id = %rid,
                                                    index = index,
                                                    command = %batch_command_name(cmd),
                                                    error = %e.message,
                                                    "batch.step.error"
                                                );
                                                results.push(protocol::BatchResultEntry {
                                                    index,
                                                    success: false,
                                                    command: "waitFor".to_string(),
                                                    elapsed: Some(wait_start.elapsed().as_millis() as u64),
                                                    value: None,
                                                    error: Some(e),
                                                });
                                                failed = true;
                                                if opts.stop_on_error { break; }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        results.push(protocol::BatchResultEntry {
                                            index,
                                            success: false,
                                            command: "waitFor".to_string(),
                                            elapsed: Some(wait_start.elapsed().as_millis() as u64),
                                            value: None,
                                            error: Some(protocol::TransactionError::action_failed("Entity dropped")),
                                        });
                                        failed = true;
                                        if opts.stop_on_error { break; }
                                    }
                                }
                            }
                        }
                    }

                    let total_elapsed = batch_start.elapsed().as_millis() as u64;
                    let failed_at = if failed {
                        results.iter().position(|r| !r.success)
                    } else {
                        None
                    };

                    tracing::info!(
                        category = "AUTOMATION",
                        request_id = %rid,
                        success = !failed,
                        total_elapsed_ms = total_elapsed,
                        failed_at = ?failed_at,
                        "automation.batch.completed"
                    );

                    if let Some(ref s) = sender {
                        let _ = s.try_send(Message::batch_result(
                            rid.clone(),
                            !failed,
                            results,
                            failed_at,
                            total_elapsed,
                        ));
                    }
                })
                .detach();
            }

            PromptMessage::ForceSubmit { value } => {
                // Get the current prompt ID and submit the value
                let prompt_id = match &self.current_view {
                    AppView::ArgPrompt { id, .. } => Some(id.clone()),
                    AppView::DivPrompt { id, .. } => Some(id.clone()),
                    AppView::FormPrompt { id, .. } => Some(id.clone()),
                    AppView::TermPrompt { id, .. } => Some(id.clone()),
                    AppView::EditorPrompt { id, .. } => Some(id.clone()),
                    AppView::EmojiPickerView { .. } => None,
                    _ => None,
                };

                if let Some(id) = prompt_id {
                    // Convert serde_json::Value to String for submission
                    let value_str = match &value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        other => other.to_string(),
                    };

                    self.submit_prompt_response(id, Some(value_str), cx);
                } else {
                    tracing::warn!(
                        category = "WARN",
                        "ForceSubmit received but no active prompt to submit to"
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
                self.prepare_window_for_prompt("UI", "path", "");

                tracing::info!(
                    category = "UI",
                    id = %id,
                    start_path = ?start_path,
                    hint = ?hint,
                    "Showing path prompt"
                );

                let path_submit_callback = self.make_submit_callback("path");
                let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
                    std::sync::Arc::new(move |id, value| {
                        tracing::info!(
                            category = "UI",
                            id = %id,
                            value = ?value,
                            "PathPrompt submit callback called"
                        );
                        path_submit_callback(id, value);
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
                            tracing::info!(
                                category = "UI",
                                path = %path_info.path,
                                "PathPromptEvent::ShowActions received"
                            );
                            this.handle_show_path_actions(path_info.clone(), cx);
                        }
                        PathPromptEvent::CloseActions => {
                            tracing::info!(
                                category = "UI",
                                "PathPromptEvent::CloseActions received"
                            );
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
                title,
                secret,
            } => {
                self.prepare_window_for_prompt("UI", "env", "");

                tracing::info!(id, key, ?prompt, ?title, secret, "ShowEnv received");
                tracing::info!(
                    category = "UI",
                    id = %id,
                    key = %key,
                    secret,
                    "ShowEnv prompt received"
                );

                let submit_callback = self.make_submit_callback("env");

                // Check if key already exists in secrets (for UX messaging)
                // Empty values don't count as "existing" - must have actual content
                // Use get_secret_info to get both existence and modification timestamp
                let secret_info = secrets::get_secret_info(&key);
                let exists_in_keyring = secret_info
                    .as_ref()
                    .map(|info| !info.value.is_empty())
                    .unwrap_or(false);
                let modified_at = secret_info.map(|info| info.modified_at);

                // Create EnvPrompt entity
                let focus_handle = self.focus_handle.clone();
                let mut env_prompt = prompts::EnvPrompt::new(
                    id.clone(),
                    key,
                    prompt,
                    title,
                    secret,
                    focus_handle,
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                    exists_in_keyring,
                    modified_at,
                );

                // Check keyring first - if value exists and no contextual prompt/title
                // was provided, auto-submit without showing UI. When prompt or title
                // are set, the script wants the user to see the setup context.
                let has_contextual_text = env_prompt.has_prompt_or_title();
                if !has_contextual_text && env_prompt.check_keyring_and_auto_submit() {
                    tracing::info!(
                        category = "UI",
                        "EnvPrompt value found in keyring, auto-submitted"
                    );
                    // Don't switch view, the callback already submitted
                    cx.notify();
                    return;
                }

                let entity = cx.new(|_| env_prompt);
                self.current_view = AppView::EnvPrompt { id, entity };
                self.focused_input = FocusedInput::None; // EnvPrompt has its own focus handling
                self.pending_focus = Some(FocusTarget::EnvPrompt);

                // Resize to standard height for full-window centered layout
                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
            }
            PromptMessage::ShowDrop {
                id,
                placeholder,
                hint,
            } => {
                self.prepare_window_for_prompt("UI", "drop", "");

                tracing::info!(id, ?placeholder, ?hint, "ShowDrop received");
                tracing::info!(
                    category = "UI",
                    id = %id,
                    placeholder = ?placeholder,
                    "ShowDrop prompt received"
                );

                let submit_callback = self.make_submit_callback("drop");

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
                self.prepare_window_for_prompt("UI", "template", "");

                tracing::info!(id, template, "ShowTemplate received");
                tracing::info!(
                    category = "UI",
                    id = %id,
                    template = %template,
                    "ShowTemplate prompt received"
                );

                let submit_callback = self.make_submit_callback("template");

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
                self.prepare_window_for_prompt("UI", "select", "");

                tracing::info!(
                    id,
                    ?placeholder,
                    choice_count = choices.len(),
                    multiple,
                    "ShowSelect received"
                );
                tracing::info!(
                    category = "UI",
                    id = %id,
                    choice_count = choices.len(),
                    multiple,
                    "ShowSelect prompt received"
                );

                let submit_callback = self.make_submit_callback("select");

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
                tracing::info!(
                    category = "CONFIRM",
                    id = %id,
                    message = ?message,
                    "ShowConfirm prompt"
                );

                // Build response callback that sends submit message back to the script
                let response_sender = self.response_sender.clone();
                let prompt_id = id.clone();
                let send_response = {
                    let response_sender = response_sender.clone();
                    let prompt_id = prompt_id.clone();
                    move |confirmed: bool| {
                        tracing::info!(
                            category = "CONFIRM",
                            prompt_id = %prompt_id,
                            confirmed,
                            "User choice received"
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
                                    tracing::info!(category = "CONFIRM", "Submit message sent");
                                }
                                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                                    tracing::warn!(
                                        category = "WARN",
                                        "Response channel full - confirm response dropped"
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    tracing::info!(
                                        category = "UI",
                                        "Response channel disconnected - script exited"
                                    );
                                }
                            }
                        }
                    }
                };

                let send_confirm = send_response.clone();
                let send_cancel = send_response.clone();
                let send_fallback = send_response;

                // Open parent confirm dialog via shared async helper
                cx.spawn(async move |_this, cx| {
                    match crate::confirm::confirm_with_parent_dialog(
                        cx,
                        crate::confirm::ParentConfirmOptions {
                            title: "Confirm".into(),
                            body: gpui::SharedString::from(message),
                            confirm_text: confirm_text
                                .map(gpui::SharedString::from)
                                .unwrap_or("OK".into()),
                            cancel_text: cancel_text
                                .map(gpui::SharedString::from)
                                .unwrap_or("Cancel".into()),
                            ..Default::default()
                        },
                        "prompt_handler_confirm",
                    )
                    .await
                    {
                        Ok(true) => send_confirm(true),
                        Ok(false) => send_cancel(false),
                        Err(error) => {
                            tracing::error!(
                                category = "ERROR",
                                error = %error,
                                "Failed to open confirm dialog window — failing closed"
                            );
                            send_fallback(false);
                        }
                    }
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
                use_builtin_ai,
            } => {
                logging::bench_log("ShowChat_received");

                self.prepare_window_for_prompt("CHAT", "chat", "window_show_requested");

                tracing::info!(
                    id,
                    ?placeholder,
                    message_count = messages.len(),
                    ?model,
                    model_count = models.len(),
                    save_history,
                    use_builtin_ai,
                    "ShowChat received"
                );
                tracing::info!(
                    category = "UI",
                    id = %id,
                    message_count = messages.len(),
                    model_count = models.len(),
                    save_history,
                    use_builtin_ai,
                    "ShowChat prompt received"
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
                                    tracing::warn!(
                                        category = "WARN",
                                        "Response channel full - chat response dropped"
                                    );
                                }
                                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                                    tracing::info!(
                                        category = "UI",
                                        "Response channel disconnected - script exited"
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

                // If SDK requested built-in AI mode, enable it with the app's AI providers
                if use_builtin_ai {
                    use crate::ai::ProviderRegistry;

                    let registry =
                        ProviderRegistry::from_environment_with_config(Some(&self.config));
                    if registry.has_any_provider() {
                        tracing::info!(
                            category = "CHAT",
                            provider_count = registry.provider_ids().len(),
                            "Enabling built-in AI"
                        );
                        chat_prompt = chat_prompt.with_builtin_ai(registry, true);
                        // Auto-respond if there are initial user messages (scriptlets with pre-populated messages)
                        if chat_prompt
                            .messages
                            .iter()
                            .any(|m| m.role == Some(crate::protocol::ChatMessageRole::User))
                        {
                            tracing::info!(
                                category = "CHAT",
                                "Found user messages - enabling needs_initial_response"
                            );
                            chat_prompt = chat_prompt.with_needs_initial_response(true);
                        }
                    } else {
                        tracing::info!(
                            category = "CHAT",
                            "Built-in AI requested but no providers configured"
                        );

                        // Create configure callback that signals via channel
                        let configure_sender = self.inline_chat_configure_sender.clone();
                        let configure_callback: crate::prompts::ChatConfigureCallback =
                            std::sync::Arc::new(move || {
                                tracing::info!(
                                    category = "CHAT",
                                    "Configure callback triggered - sending signal"
                                );
                                let _ = configure_sender.try_send(());
                            });

                        // Create Claude Code callback that signals via channel
                        let claude_code_sender = self.inline_chat_claude_code_sender.clone();
                        let claude_code_callback: crate::prompts::ChatClaudeCodeCallback =
                            std::sync::Arc::new(move || {
                                tracing::info!(
                                    category = "CHAT",
                                    "Claude Code callback triggered - sending signal"
                                );
                                let _ = claude_code_sender.try_send(());
                            });

                        chat_prompt = chat_prompt
                            .with_needs_setup(true)
                            .with_configure_callback(configure_callback)
                            .with_claude_code_callback(claude_code_callback);
                    }
                }

                // Wire on_show_actions so ChatPrompt's internal toggle_actions_menu
                // has a live callback. ⌘K is also intercepted at the parent level.
                logging::bench_log("ChatPrompt_creating");
                let entity = cx.new(|_| chat_prompt);
                let app_weak = cx.entity().downgrade();
                entity.update(cx, |chat, _cx| {
                    chat.set_on_show_actions(std::sync::Arc::new(move |_prompt_id| {
                        tracing::info!(
                            event = "on_show_actions.triggered",
                            source = "sdk-chat",
                            "ChatPrompt requested actions dialog via callback"
                        );
                        let _ = &app_weak;
                    }));
                });
                self.current_view = AppView::ChatPrompt { id, entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::ChatPrompt);
                logging::bench_log("ChatPrompt_created");

                resize_to_view_sync(ViewType::DivPrompt, 0);
                logging::bench_log("resize_queued");
                cx.notify();
                logging::bench_end("hotkey_to_chat_visible");
            }

            PromptMessage::ChatAddMessage { id, message } => {
                tracing::info!(category = "CHAT", id = %id, "ChatAddMessage");
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
                tracing::info!(
                    category = "CHAT",
                    id = %id,
                    message_id = %message_id,
                    "ChatStreamStart"
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
                tracing::info!(
                    category = "CHAT",
                    id = %id,
                    message_id = %message_id,
                    "ChatStreamComplete"
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
                tracing::info!(category = "CHAT", id = %id, "ChatClear");
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
                tracing::info!(
                    category = "CHAT",
                    id = %id,
                    message_id = %message_id,
                    error = %error,
                    "ChatSetError"
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
                tracing::info!(
                    category = "CHAT",
                    id = %id,
                    message_id = %message_id,
                    "ChatClearError"
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
            PromptMessage::SetStatus { status, message } => {
                tracing::info!(
                    category = "STATUS",
                    state = "received",
                    status = %status,
                    has_message = message.is_some(),
                    message = %message.as_deref().unwrap_or(""),
                    "Received setStatus() protocol message"
                );
            }
            PromptMessage::SetInput { text } => {
                self.set_prompt_input(text, cx);
            }
            PromptMessage::SetActions { actions } => {
                tracing::info!(
                    category = "ACTIONS",
                    action_count = actions.len(),
                    "Received setActions"
                );

                self.set_sdk_actions_and_shortcuts(actions.clone(), "ACTIONS", true);

                // Update ActionsDialog if it exists and is open
                if let Some(ref dialog) = self.actions_dialog {
                    dialog.update(cx, |d, _cx| {
                        d.set_sdk_actions(actions);
                    });
                }

                cx.notify();
            }
            PromptMessage::FieldsComingSoon { id, field_count } => {
                tracing::warn!(
                    category = "WARN",
                    prompt = "fields()",
                    id = %id,
                    field_count = field_count,
                    state = "stubbed",
                    "Received unsupported prompt message"
                );
                self.show_prompt_coming_soon_toast("fields()", cx);
            }
            PromptMessage::HotkeyComingSoon { id, placeholder } => {
                tracing::warn!(
                    category = "WARN",
                    prompt = "hotkey()",
                    id = %id,
                    has_placeholder = placeholder.is_some(),
                    state = "stubbed",
                    "Received unsupported prompt message"
                );
                self.show_prompt_coming_soon_toast("hotkey()", cx);
            }
            PromptMessage::WidgetComingSoon { id } => {
                tracing::warn!(
                    category = "WARN",
                    prompt = "widget()",
                    id = %id,
                    state = "stubbed",
                    "Received unsupported prompt message"
                );
                self.show_prompt_coming_soon_toast("widget()", cx);
            }
            PromptMessage::WebcamComingSoon { id } => {
                tracing::warn!(
                    category = "WARN",
                    prompt = "webcam()",
                    id = %id,
                    state = "stubbed",
                    "Received unsupported prompt message"
                );
                self.show_prompt_coming_soon_toast("webcam()", cx);
            }
            PromptMessage::MicComingSoon { id } => {
                tracing::warn!(
                    category = "WARN",
                    prompt = "mic()",
                    id = %id,
                    state = "stubbed",
                    "Received unsupported prompt message"
                );
                self.show_prompt_coming_soon_toast("mic()", cx);
            }
            PromptMessage::AiStartChat {
                request_id,
                message,
                system_prompt,
                image,
                model_id,
                no_response,
                parts,
            } => {
                tracing::info!(
                    category = "AI",
                    request_id = %request_id,
                    message_len = message.len(),
                    has_system_prompt = system_prompt.is_some(),
                    has_image = image.is_some(),
                    model_id = ?model_id,
                    no_response,
                    "AiStartChat request"
                );

                // Open the AI window (creates new if not open, brings to front if open)
                if let Err(e) = crate::ai::open_ai_window(cx) {
                    tracing::error!(
                        category = "ERROR",
                        error = %e,
                        "Failed to open AI window for AiStartChat"
                    );
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

                // Pre-generate a real ChatId so the SDK gets an actual persistent ID
                let chat_id = crate::ai::ChatId::new();
                let should_submit = !no_response;
                let provider = model_id.as_deref().and_then(|selected_model_id| {
                    let registry =
                        crate::ai::ProviderRegistry::from_environment_with_config(Some(&self.config));
                    resolve_ai_start_chat_provider(&registry, selected_model_id)
                });
                let context_parts = parts
                    .into_iter()
                    .map(|part| match part {
                        crate::protocol::AiContextPartInput::ResourceUri { uri, label } => {
                            crate::ai::AiContextPart::ResourceUri { uri, label }
                        }
                        crate::protocol::AiContextPartInput::FilePath { path, label } => {
                            crate::ai::AiContextPart::FilePath { path, label }
                        }
                    })
                    .collect();

                // Queue the StartChat command — the AI window will create the chat,
                // save the user message (with optional image), and optionally stream.
                crate::ai::start_ai_chat(
                    cx,
                    chat_id,
                    &message,
                    context_parts,
                    image.as_deref(),
                    system_prompt.as_deref(),
                    model_id.as_deref(),
                    provider.as_deref(),
                    None,
                    should_submit,
                );

                // Build title from message content
                let title = if message.trim().is_empty() && image.is_some() {
                    "Image attachment".to_string()
                } else {
                    crate::ai::Chat::generate_title_from_content(&message)
                };

                // Send AiChatCreated response with the real chat ID
                if let Some(ref sender) = self.response_sender {
                    let response = Message::AiChatCreated {
                        request_id: request_id.clone(),
                        chat_id: chat_id.as_str(),
                        title,
                        model_id: model_id
                            .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
                        provider: provider.unwrap_or_else(|| "anthropic".to_string()),
                        streaming_started: should_submit,
                    };
                    match sender.try_send(response) {
                        Ok(()) => {
                            tracing::info!(
                                category = "AI",
                                request_id = %request_id,
                                chat_id = %chat_id,
                                "AiChatCreated response sent"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "WARN",
                                "Response channel full - AiChatCreated dropped"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "UI",
                                "Response channel disconnected - script exited"
                            );
                        }
                    }
                }

                cx.notify();
            }
            PromptMessage::AiFocus { request_id } => {
                tracing::info!(category = "AI", request_id = %request_id, "AiFocus request");

                // Check if window was already open before we open/focus it
                let was_open = crate::ai::is_ai_window_open();

                // Open the AI window (creates new if not open, brings to front if open)
                let success = match crate::ai::open_ai_window(cx) {
                    Ok(()) => {
                        tracing::info!(category = "AI", "AI window focused successfully");
                        true
                    }
                    Err(e) => {
                        tracing::error!(
                            category = "ERROR",
                            error = %e,
                            "Failed to focus AI window"
                        );
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
                            tracing::info!(
                                category = "AI",
                                request_id = %request_id,
                                "AiFocusResult sent"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            tracing::warn!(
                                category = "WARN",
                                "Response channel full - AiFocusResult dropped"
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            tracing::info!(
                                category = "UI",
                                "Response channel disconnected - script exited"
                            );
                        }
                    }
                }

                cx.notify();
            }
            PromptMessage::ShowGrid { options } => {
                tracing::info!(
                    category = "DEBUG_GRID",
                    grid_size = options.grid_size,
                    show_bounds = options.show_bounds,
                    show_box_model = options.show_box_model,
                    show_alignment_guides = options.show_alignment_guides,
                    "ShowGrid from script"
                );
                self.show_grid(options, cx);
            }
            PromptMessage::HideGrid => {
                tracing::info!(category = "DEBUG_GRID", "HideGrid from script");
                self.hide_grid(cx);
            }
        }
    }

    /// Check if a wait condition is currently satisfied.
    fn wait_condition_satisfied(
        &self,
        condition: &protocol::WaitCondition,
        cx: &Context<Self>,
    ) -> bool {
        match condition {
            protocol::WaitCondition::Named(named) => match named {
                protocol::WaitNamedCondition::ChoicesRendered => {
                    let elements = self.collect_visible_elements(100, cx);
                    elements
                        .elements
                        .iter()
                        .any(|el| el.element_type == protocol::ElementType::Choice)
                }
                protocol::WaitNamedCondition::InputEmpty => {
                    let input = self.current_input_value();
                    input.is_empty()
                }
                protocol::WaitNamedCondition::WindowVisible => {
                    script_kit_gpui::is_main_window_visible()
                }
                protocol::WaitNamedCondition::WindowFocused => {
                    let visible = script_kit_gpui::is_main_window_visible();
                    visible && self.focused_input != FocusedInput::None
                }
            },
            protocol::WaitCondition::Detailed(detailed) => match detailed {
                protocol::WaitDetailedCondition::ElementExists { semantic_id }
                | protocol::WaitDetailedCondition::ElementVisible { semantic_id } => {
                    let elements = self.collect_visible_elements(1000, cx);
                    elements
                        .elements
                        .iter()
                        .any(|el| el.semantic_id == *semantic_id)
                }
                protocol::WaitDetailedCondition::ElementFocused { semantic_id } => {
                    let elements = self.collect_visible_elements(1000, cx);
                    elements
                        .elements
                        .iter()
                        .any(|el| el.semantic_id == *semantic_id && el.focused == Some(true))
                }
                protocol::WaitDetailedCondition::StateMatch { state: expected } => {
                    let prompt_type = self.current_prompt_type();
                    let input_value = self.current_input_value();
                    let selected_value = self.current_selected_value();
                    let window_visible = script_kit_gpui::is_main_window_visible();

                    expected
                        .prompt_type
                        .as_deref()
                        .is_none_or(|v| v == prompt_type)
                        && expected
                            .input_value
                            .as_deref()
                            .is_none_or(|v| v == input_value)
                        && expected
                            .selected_value
                            .as_deref()
                            .is_none_or(|v| selected_value.as_deref() == Some(v))
                        && expected
                            .window_visible
                            .is_none_or(|v| v == window_visible)
                }
                // ── ACP-specific wait conditions ────────────────────
                protocol::WaitDetailedCondition::AcpReady => {
                    let state = self.collect_acp_state(cx);
                    state.context_ready && state.status == "idle"
                }
                protocol::WaitDetailedCondition::AcpPickerOpen => {
                    let state = self.collect_acp_state(cx);
                    state.picker.as_ref().is_some_and(|p| p.open)
                }
                protocol::WaitDetailedCondition::AcpPickerClosed => {
                    let state = self.collect_acp_state(cx);
                    state.picker.is_none() || state.picker.as_ref().is_some_and(|p| !p.open)
                }
                protocol::WaitDetailedCondition::AcpItemAccepted => {
                    let state = self.collect_acp_state(cx);
                    state.last_accepted_item.is_some()
                }
                protocol::WaitDetailedCondition::AcpCursorAt { index } => {
                    let state = self.collect_acp_state(cx);
                    state.cursor_index == *index
                }
                protocol::WaitDetailedCondition::AcpStatus { status } => {
                    let state = self.collect_acp_state(cx);
                    state.status == *status
                }
                protocol::WaitDetailedCondition::AcpInputMatch { text } => {
                    let state = self.collect_acp_state(cx);
                    state.input_text == *text
                }
                protocol::WaitDetailedCondition::AcpInputContains { substring } => {
                    let state = self.collect_acp_state(cx);
                    state.input_text.contains(substring.as_str())
                }
                // ── ACP proof wait conditions (test probe) ─────────
                protocol::WaitDetailedCondition::AcpAcceptedViaKey { key } => {
                    let probe = self.collect_acp_test_probe(1, cx);
                    probe
                        .accepted_items
                        .last()
                        .is_some_and(|item| item.accepted_via_key == *key)
                }
                protocol::WaitDetailedCondition::AcpAcceptedLabel { label } => {
                    let probe = self.collect_acp_test_probe(1, cx);
                    probe
                        .accepted_items
                        .last()
                        .is_some_and(|item| item.item_label == *label)
                }
                protocol::WaitDetailedCondition::AcpAcceptedCursorAt { index } => {
                    let probe = self.collect_acp_test_probe(1, cx);
                    probe
                        .accepted_items
                        .last()
                        .is_some_and(|item| item.cursor_after == *index)
                }
                protocol::WaitDetailedCondition::AcpInputLayoutMatch {
                    visible_start,
                    visible_end,
                    cursor_in_window,
                } => {
                    let probe = self.collect_acp_test_probe(1, cx);
                    probe.input_layout.as_ref().is_some_and(|layout| {
                        layout.visible_start == *visible_start
                            && layout.visible_end == *visible_end
                            && layout.cursor_in_window == *cursor_in_window
                    })
                }
            },
        }
    }

    /// Get the current prompt type as a string.
    fn current_prompt_type(&self) -> String {
        match &self.current_view {
            AppView::ScriptList => "none".to_string(),
            AppView::ArgPrompt { .. } => "arg".to_string(),
            AppView::DivPrompt { .. } => "div".to_string(),
            AppView::FormPrompt { .. } => "form".to_string(),
            AppView::EditorPrompt { .. } => "editor".to_string(),
            AppView::TermPrompt { .. } => "term".to_string(),
            AppView::ChatPrompt { .. } => "chat".to_string(),
            AppView::MiniPrompt { .. } => "mini".to_string(),
            AppView::MicroPrompt { .. } => "micro".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Get the current input/filter value.
    fn current_input_value(&self) -> String {
        match &self.current_view {
            AppView::ScriptList => self.filter_text.clone(),
            AppView::ArgPrompt { .. } => self.arg_input.text().to_string(),
            AppView::MiniPrompt { .. } => self.arg_input.text().to_string(),
            AppView::MicroPrompt { .. } => self.arg_input.text().to_string(),
            _ => String::new(),
        }
    }

    /// Get the currently selected value if any.
    fn current_selected_value(&self) -> Option<String> {
        match &self.current_view {
            AppView::ArgPrompt { choices, .. }
            | AppView::MiniPrompt { choices, .. }
            | AppView::MicroPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                filtered
                    .get(self.arg_selected_index)
                    .map(|c| c.value.clone())
            }
            _ => None,
        }
    }

    /// Collect a machine-readable ACP state snapshot.
    ///
    /// Returns a default (idle, empty) snapshot when the current view is not
    /// `AcpChatView` — callers should check `status == "notAcp"` to detect this.
    fn collect_acp_state(&self, cx: &Context<Self>) -> protocol::AcpStateSnapshot {
        let entity = match &self.current_view {
            AppView::AcpChatView { entity } => entity,
            _ => {
                return protocol::AcpStateSnapshot {
                    status: "notAcp".to_string(),
                    ..Default::default()
                };
            }
        };

        let view = entity.read(cx);

        // Extract state from the ACP view's public API.
        view.collect_acp_state_snapshot(cx)
    }

    /// Reset the ACP test probe ring buffer.
    fn reset_acp_test_probe(&mut self, cx: &mut Context<Self>) {
        if let AppView::AcpChatView { entity } = &self.current_view {
            entity.update(cx, |view, _cx| {
                view.reset_test_probe();
            });
        }
    }

    /// Collect a bounded ACP test probe snapshot.
    fn collect_acp_test_probe(
        &self,
        tail: usize,
        cx: &Context<Self>,
    ) -> protocol::AcpTestProbeSnapshot {
        let entity = match &self.current_view {
            AppView::AcpChatView { entity } => entity,
            _ => {
                return protocol::AcpTestProbeSnapshot {
                    state: protocol::AcpStateSnapshot {
                        status: "notAcp".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
            }
        };

        let view = entity.read(cx);
        view.test_probe_snapshot(tail, cx)
    }

    /// Set the input text for the current prompt.
    fn set_input_text(&mut self, text: &str, cx: &mut Context<Self>) {
        match &self.current_view {
            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                cx.notify();
            }
            AppView::ScriptList => {
                self.filter_text = text.to_string();
                self.selected_index = 0;
                cx.notify();
            }
            _ => {
                tracing::warn!(
                    category = "BATCH",
                    "setInput not supported for current view"
                );
            }
        }
    }

    /// Select a choice by its value from the filtered list.
    fn select_choice_by_value(
        &mut self,
        value: &str,
        submit: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<String> {
        let choices = match &self.current_view {
            AppView::ArgPrompt { choices, .. }
            | AppView::MiniPrompt { choices, .. }
            | AppView::MicroPrompt { choices, .. } => choices.clone(),
            _ => anyhow::bail!("selectByValue only supports choice-backed prompts"),
        };

        let filtered = self.get_filtered_arg_choices(&choices);
        let Some(index) = filtered.iter().position(|choice| choice.value == value) else {
            anyhow::bail!("No visible choice matched value '{value}'");
        };

        self.arg_selected_index = index;
        cx.notify();

        let selected = filtered[index].value.clone();

        if submit {
            self.submit_current_value(cx);
        }

        Ok(selected)
    }

    /// Select a choice by semantic ID, optionally submitting.
    fn select_choice_by_semantic_id(
        &mut self,
        semantic_id: &str,
        submit: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<String> {
        let choices = match &self.current_view {
            AppView::ArgPrompt { choices, .. }
            | AppView::MiniPrompt { choices, .. }
            | AppView::MicroPrompt { choices, .. } => choices.clone(),
            _ => anyhow::bail!("selectBySemanticId only supports choice-backed prompts"),
        };

        let filtered = self.get_filtered_arg_choices(&choices);
        let Some(index) = filtered
            .iter()
            .enumerate()
            .position(|(i, choice)| choice.generate_id(i) == semantic_id)
        else {
            anyhow::bail!("No visible choice matched semantic ID '{semantic_id}'");
        };

        self.arg_selected_index = index;
        cx.notify();

        let selected = filtered[index].value.clone();

        if submit {
            self.submit_current_value(cx);
        }

        Ok(selected)
    }

    /// Select the first choice in the filtered list.
    fn select_first_choice(
        &mut self,
        submit: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Option<String>> {
        let choices = match &self.current_view {
            AppView::ArgPrompt { choices, .. }
            | AppView::MiniPrompt { choices, .. }
            | AppView::MicroPrompt { choices, .. } => choices.clone(),
            _ => anyhow::bail!("selectFirst only supports choice-backed prompts"),
        };

        let filtered = self.get_filtered_arg_choices(&choices);
        if filtered.is_empty() {
            anyhow::bail!("No visible choices to select");
        }

        self.arg_selected_index = 0;
        cx.notify();

        let selected = filtered[0].value.clone();

        if submit {
            self.submit_current_value(cx);
        }

        Ok(Some(selected))
    }

    /// Submit the currently selected value.
    fn submit_current_value(&mut self, cx: &mut Context<Self>) {
        match &self.current_view {
            AppView::ArgPrompt { id, choices, .. }
            | AppView::MiniPrompt { id, choices, .. }
            | AppView::MicroPrompt { id, choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                let value = if self.arg_selected_index < filtered.len() {
                    filtered[self.arg_selected_index].value.clone()
                } else {
                    self.arg_input.text().to_string()
                };
                if let Some(ref sender) = self.response_sender {
                    let _ = sender.try_send(Message::Submit {
                        id: id.clone(),
                        value: Some(value),
                    });
                }
                cx.notify();
            }
            _ => {
                tracing::warn!(
                    category = "BATCH",
                    "submit not supported for current view"
                );
            }
        }
    }
}

/// Get the wire name for a batch command.
fn batch_command_name(cmd: &protocol::BatchCommand) -> String {
    match cmd {
        protocol::BatchCommand::SetInput { .. } => "setInput".to_string(),
        protocol::BatchCommand::ForceSubmit { .. } => "forceSubmit".to_string(),
        protocol::BatchCommand::WaitFor { .. } => "waitFor".to_string(),
        protocol::BatchCommand::SelectByValue { .. } => "selectByValue".to_string(),
        protocol::BatchCommand::SelectBySemanticId { .. } => "selectBySemanticId".to_string(),
        protocol::BatchCommand::FilterAndSelect { .. } => "filterAndSelect".to_string(),
        protocol::BatchCommand::TypeAndSubmit { .. } => "typeAndSubmit".to_string(),
    }
}

// --- merged from part_002.rs ---
#[cfg(test)]
mod prompt_handler_message_tests {
    use super::{
        classify_prompt_message_route, escape_windows_cmd_open_target,
        prompt_coming_soon_warning, resolve_ai_start_chat_provider, unhandled_message_warning,
        PromptMessageRoute,
    };
    use crate::ai::providers::OpenAiProvider;
    use crate::PromptMessage;

    #[test]
    fn test_handle_prompt_message_routes_confirm_request_to_confirm_window() {
        let message = PromptMessage::ShowConfirm {
            id: "confirm-id".to_string(),
            message: "Continue?".to_string(),
            confirm_text: Some("Yes".to_string()),
            cancel_text: Some("No".to_string()),
        };
        assert_eq!(
            classify_prompt_message_route(&message),
            PromptMessageRoute::ConfirmDialog
        );
    }

    #[test]
    fn test_handle_prompt_message_ignores_unknown_message_without_state_corruption() {
        let message = PromptMessage::UnhandledMessage {
            message_type: "widget".to_string(),
        };
        assert_eq!(
            classify_prompt_message_route(&message),
            PromptMessageRoute::UnhandledWarning
        );

        let warning = unhandled_message_warning("widget");
        assert!(warning.contains("'widget'"));
        assert!(warning.contains("not supported yet"));
    }

    #[test]
    fn test_unhandled_message_warning_includes_recovery_guidance() {
        let message = unhandled_message_warning("widget");
        assert!(message.contains("'widget'"));
        assert!(message.contains("Update the script to a supported message type"));
        assert!(message.contains("update Script Kit GPUI"));
    }

    #[test]
    fn test_prompt_coming_soon_warning_uses_function_style_name() {
        assert_eq!(
            prompt_coming_soon_warning("fields()"),
            "fields() prompt coming soon."
        );
    }

    #[test]
    fn test_truncate_str_chars_returns_valid_utf8_boundary_when_message_is_multibyte() {
        let message = "🙂".repeat(50);
        let truncated = crate::utils::truncate_str_chars(&message, 30);

        assert_eq!(truncated.chars().count(), 30);
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_escape_windows_cmd_open_target_escapes_shell_metacharacters() {
        let escaped = escape_windows_cmd_open_target(r#"https://example.com/?x=1&y=2|3"#);
        assert_eq!(escaped, r#"https://example.com/?x=1^&y=2^|3"#);
    }

    #[test]
    fn test_resolve_ai_start_chat_provider_returns_registered_provider_for_model() {
        let mut registry = crate::ai::ProviderRegistry::new();
        registry.register(std::sync::Arc::new(OpenAiProvider::new("test-key")));

        assert_eq!(
            resolve_ai_start_chat_provider(&registry, "gpt-4o"),
            Some("openai".to_string())
        );
    }

    #[test]
    fn test_resolve_ai_start_chat_provider_returns_none_for_unknown_model() {
        let mut registry = crate::ai::ProviderRegistry::new();
        registry.register(std::sync::Arc::new(OpenAiProvider::new("test-key")));

        assert_eq!(
            resolve_ai_start_chat_provider(&registry, "claude-3-5-sonnet-20241022"),
            None
        );
    }
}
