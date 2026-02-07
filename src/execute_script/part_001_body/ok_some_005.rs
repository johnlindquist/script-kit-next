                                let prompt_msg = match msg {
                                    Message::Arg {
                                        id,
                                        placeholder,
                                        choices,
                                        actions,
                                    } => Some(PromptMessage::ShowArg {
                                        id,
                                        placeholder,
                                        choices,
                                        actions,
                                    }),
                                    Message::Div {
                                        id,
                                        html,
                                        container_classes,
                                        actions,
                                        placeholder,
                                        hint,
                                        footer,
                                        container_bg,
                                        container_padding,
                                        opacity,
                                    } => Some(PromptMessage::ShowDiv {
                                        id,
                                        html,
                                        container_classes,
                                        actions,
                                        placeholder,
                                        hint,
                                        footer,
                                        container_bg,
                                        container_padding,
                                        opacity,
                                    }),
                                    Message::Form { id, html, actions } => {
                                        Some(PromptMessage::ShowForm { id, html, actions })
                                    }
                                    Message::Term {
                                        id,
                                        command,
                                        actions,
                                    } => Some(PromptMessage::ShowTerm {
                                        id,
                                        command,
                                        actions,
                                    }),
                                    Message::Editor {
                                        id,
                                        content,
                                        language,
                                        template,
                                        actions,
                                        ..
                                    } => Some(PromptMessage::ShowEditor {
                                        id,
                                        content,
                                        language,
                                        template,
                                        actions,
                                    }),
                                    // New prompt types (scaffolding)
                                    Message::Path {
                                        id,
                                        start_path,
                                        hint,
                                    } => Some(PromptMessage::ShowPath {
                                        id,
                                        start_path,
                                        hint,
                                    }),
                                    Message::Env { id, key, secret } => {
                                        Some(PromptMessage::ShowEnv {
                                            id,
                                            key,
                                            prompt: None,
                                            secret: secret.unwrap_or(false),
                                        })
                                    }
                                    Message::Drop { id } => Some(PromptMessage::ShowDrop {
                                        id,
                                        placeholder: None,
                                        hint: None,
                                    }),
                                    Message::Template { id, template } => {
                                        Some(PromptMessage::ShowTemplate { id, template })
                                    }
                                    Message::Select {
                                        id,
                                        placeholder,
                                        choices,
                                        multiple,
                                    } => Some(PromptMessage::ShowSelect {
                                        id,
                                        placeholder: Some(placeholder),
                                        choices,
                                        multiple: multiple.unwrap_or(false),
                                    }),
                                    Message::Confirm {
                                        id,
                                        message,
                                        confirm_text,
                                        cancel_text,
                                    } => Some(PromptMessage::ShowConfirm {
                                        id,
                                        message,
                                        confirm_text,
                                        cancel_text,
                                    }),
                                    Message::Exit { .. } => Some(PromptMessage::ScriptExit),
                                    Message::ForceSubmit { value } => {
                                        Some(PromptMessage::ForceSubmit { value })
                                    }
                                    Message::Hide {} => Some(PromptMessage::HideWindow),
                                    Message::Browse { url } => {
                                        Some(PromptMessage::OpenBrowser { url })
                                    }
                                    Message::Hud { text, duration_ms } => {
                                        Some(PromptMessage::ShowHud { text, duration_ms })
                                    }
                                    Message::SetActions { actions } => {
                                        Some(PromptMessage::SetActions { actions })
                                    }
                                    Message::SetInput { text } => {
                                        Some(PromptMessage::SetInput { text })
                                    }
                                    Message::ShowGrid { options } => {
                                        Some(PromptMessage::ShowGrid { options })
                                    }
                                    Message::HideGrid => Some(PromptMessage::HideGrid),
                                    // Chat prompt messages
                                    Message::Chat {
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
                                        logging::bench_log("Chat_message_parsed");
                                        Some(PromptMessage::ShowChat {
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
                                        })
                                    }
                                    Message::ChatMessage { id, message } => {
                                        Some(PromptMessage::ChatAddMessage { id, message })
                                    }
                                    Message::ChatStreamStart {
                                        id,
                                        message_id,
                                        position,
                                    } => Some(PromptMessage::ChatStreamStart {
                                        id,
                                        message_id,
                                        position,
                                    }),
                                    Message::ChatStreamChunk {
                                        id,
                                        message_id,
                                        chunk,
                                    } => Some(PromptMessage::ChatStreamChunk {
                                        id,
                                        message_id,
                                        chunk,
                                    }),
                                    Message::ChatStreamComplete { id, message_id } => {
                                        Some(PromptMessage::ChatStreamComplete { id, message_id })
                                    }
                                    Message::ChatClear { id } => {
                                        Some(PromptMessage::ChatClear { id })
                                    }
                                    Message::ChatSetError {
                                        id,
                                        message_id,
                                        error,
                                    } => Some(PromptMessage::ChatSetError {
                                        id,
                                        message_id,
                                        error,
                                    }),
                                    Message::ChatClearError { id, message_id } => {
                                        Some(PromptMessage::ChatClearError { id, message_id })
                                    }
                                    // ChatSubmit goes from App → SDK, not SDK → App
                                    Message::ChatSubmit { .. } => None,
                                    // AI window start chat
                                    Message::AiStartChat {
                                        request_id,
                                        message,
                                        system_prompt,
                                        image,
                                        model_id,
                                        no_response,
                                    } => Some(PromptMessage::AiStartChat {
                                        request_id,
                                        message,
                                        system_prompt,
                                        image,
                                        model_id,
                                        no_response,
                                    }),
                                    // AI window focus
                                    Message::AiFocus { request_id } => {
                                        Some(PromptMessage::AiFocus { request_id })
                                    }
                                    other => {
                                        // Get the message type name for user feedback
                                        let msg_type = format!("{:?}", other);
                                        // Extract just the variant name (before any {})
                                        let type_name = msg_type
                                            .split('{')
                                            .next()
                                            .unwrap_or(&msg_type)
                                            .trim()
                                            .to_string();
                                        logging::log(
                                            "WARN",
                                            &format!("Unhandled message type: {}", type_name),
                                        );
                                        Some(PromptMessage::UnhandledMessage {
                                            message_type: type_name,
                                        })
                                    }
                                };

                                if let Some(prompt_msg) = prompt_msg {
                                    if tx.send_blocking(prompt_msg).is_err() {
                                        logging::log(
                                            "EXEC",
                                            "Prompt channel closed, reader exiting",
                                        );
                                        break;
                                    }
                                }
