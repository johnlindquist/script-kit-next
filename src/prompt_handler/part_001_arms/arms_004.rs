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
                use_builtin_ai,
            } => {
                logging::bench_log("ShowChat_received");

                // Clear NEEDS_RESET when receiving a UI prompt from an active script
                // This prevents the window from resetting when shown (script wants to use UI)
                if NEEDS_RESET.swap(false, Ordering::SeqCst) {
                    logging::log("CHAT", "Cleared NEEDS_RESET - script is showing chat UI");
                }

                // Show window if hidden (script may have called hide() for getSelectedText)
                if !script_kit_gpui::is_main_window_visible() {
                    logging::bench_log("window_show_requested");
                    logging::log("CHAT", "Window hidden - requesting show for chat UI");
                    script_kit_gpui::set_main_window_visible(true);
                    script_kit_gpui::request_show_main_window();
                }

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
                logging::log(
                    "UI",
                    &format!(
                        "ShowChat prompt received: {} ({} messages, {} models, save={}, builtin_ai={})",
                        id,
                        messages.len(),
                        models.len(),
                        save_history,
                        use_builtin_ai
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

                // If SDK requested built-in AI mode, enable it with the app's AI providers
                if use_builtin_ai {
                    use crate::ai::ProviderRegistry;

                    let registry =
                        ProviderRegistry::from_environment_with_config(Some(&self.config));
                    if registry.has_any_provider() {
                        logging::log(
                            "CHAT",
                            &format!(
                                "Enabling built-in AI with {} providers",
                                registry.provider_ids().len()
                            ),
                        );
                        chat_prompt = chat_prompt.with_builtin_ai(registry, true);
                        // Auto-respond if there are initial user messages (scriptlets with pre-populated messages)
                        if chat_prompt
                            .messages
                            .iter()
                            .any(|m| m.role == Some(crate::protocol::ChatMessageRole::User))
                        {
                            logging::log(
                                "CHAT",
                                "Found user messages - enabling needs_initial_response",
                            );
                            chat_prompt = chat_prompt.with_needs_initial_response(true);
                        }
                    } else {
                        logging::log("CHAT", "Built-in AI requested but no providers configured");

                        // Create configure callback that signals via channel
                        let configure_sender = self.inline_chat_configure_sender.clone();
                        let configure_callback: crate::prompts::ChatConfigureCallback =
                            std::sync::Arc::new(move || {
                                crate::logging::log(
                                    "CHAT",
                                    "Configure callback triggered - sending signal",
                                );
                                let _ = configure_sender.try_send(());
                            });

                        // Create Claude Code callback that signals via channel
                        let claude_code_sender = self.inline_chat_claude_code_sender.clone();
                        let claude_code_callback: crate::prompts::ChatClaudeCodeCallback =
                            std::sync::Arc::new(move || {
                                crate::logging::log(
                                    "CHAT",
                                    "Claude Code callback triggered - sending signal",
                                );
                                let _ = claude_code_sender.try_send(());
                            });

                        chat_prompt = chat_prompt
                            .with_needs_setup(true)
                            .with_configure_callback(configure_callback)
                            .with_claude_code_callback(claude_code_callback);
                    }
                }

                // Note: ⌘K for actions is handled at the main app level in handle_key_event
                // The ChatPrompt's on_show_actions callback is not needed when main app handles it

                logging::bench_log("ChatPrompt_creating");
                let entity = cx.new(|_| chat_prompt);
                self.current_view = AppView::ChatPrompt { id, entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::ChatPrompt);
                logging::bench_log("ChatPrompt_created");

                resize_to_view_sync(ViewType::DivPrompt, 0);
                logging::bench_log("resize_queued");
                cx.notify();
                logging::bench_end("hotkey_to_chat_visible");
            }
