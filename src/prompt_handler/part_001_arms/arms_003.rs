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
                    None, // title - SDK scripts don't provide one yet
                    secret,
                    focus_handle,
                    submit_callback,
                    std::sync::Arc::clone(&self.theme),
                    exists_in_keyring,
                    modified_at,
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

                // Resize to standard height for full-window centered layout
                resize_to_view_sync(ViewType::DivPrompt, 0);
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
