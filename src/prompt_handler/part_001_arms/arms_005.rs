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
                // IMPORTANT: Only register shortcuts for visible actions
                // Hidden actions should not be triggerable via keyboard shortcuts
                self.action_shortcuts.clear();
                for action in &actions {
                    if action.is_visible() {
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
