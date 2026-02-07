            PromptMessage::ShowArg {
                id,
                placeholder,
                choices,
                actions,
            } => {
                // Clear NEEDS_RESET when receiving a UI prompt from an active script
                // This prevents the window from resetting when shown (script wants to use UI)
                if NEEDS_RESET.swap(false, Ordering::SeqCst) {
                    logging::log("UI", "Cleared NEEDS_RESET - script is showing arg UI");
                }

                // Show window if hidden (script may have called hide() for getSelectedText)
                if !script_kit_gpui::is_main_window_visible() {
                    logging::log("UI", "Window hidden - requesting show for arg UI");
                    script_kit_gpui::set_main_window_visible(true);
                    script_kit_gpui::request_show_main_window();
                }

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
                    // IMPORTANT: Only register shortcuts for visible actions
                    // Hidden actions should not be triggerable via keyboard shortcuts
                    self.action_shortcuts.clear();
                    for action in action_list {
                        if action.is_visible() {
                            if let Some(shortcut) = &action.shortcut {
                                self.action_shortcuts.insert(
                                    shortcuts::normalize_shortcut(shortcut),
                                    action.name.clone(),
                                );
                            }
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
                // Clear NEEDS_RESET when receiving a UI prompt from an active script
                if NEEDS_RESET.swap(false, Ordering::SeqCst) {
                    logging::log("UI", "Cleared NEEDS_RESET - script is showing div UI");
                }

                // Show window if hidden
                if !script_kit_gpui::is_main_window_visible() {
                    logging::log("UI", "Window hidden - requesting show for div UI");
                    script_kit_gpui::set_main_window_visible(true);
                    script_kit_gpui::request_show_main_window();
                }

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
