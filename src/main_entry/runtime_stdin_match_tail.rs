                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening AI window via stdin command");
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenMiniAi => {
                                logging::log("STDIN", "Opening mini AI window via stdin command");
                                if let Err(e) = ai::open_mini_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open mini AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log("STDIN", "Opening AI window with mock data via stdin command");
                                // First insert mock data
                                if let Err(e) = ai::insert_mock_data() {
                                    logging::log("STDIN", &format!("Failed to insert mock data: {}", e));
                                } else {
                                    logging::log("STDIN", "Mock data inserted successfully");
                                }
                                // Then open the window
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenMiniAiWithMockData => {
                                logging::log("STDIN", "Opening mini AI window with mock data via stdin command");
                                if let Err(e) = ai::insert_mock_data() {
                                    logging::log("STDIN", &format!("Failed to insert mock data: {}", e));
                                } else {
                                    logging::log("STDIN", "Mock data inserted successfully");
                                }
                                if let Err(e) = ai::open_mini_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open mini AI window: {}", e));
                                }
                            }
                            ExternalCommand::ShowAiCommandBar => {
                                logging::log("STDIN", "Showing AI command bar via stdin command");
                                ai::show_ai_command_bar(ctx);
                            }
                            ExternalCommand::SimulateAiKey { key, modifiers } => {
                                logging::log(
                                    "STDIN",
                                    &format!("Simulating AI key: '{}' with modifiers: {:?}", key, modifiers),
                                );
                                ai::simulate_ai_key(ctx, &key, modifiers);
                            }
                            ExternalCommand::CaptureWindow { title, path } => {
                                logging::log("STDIN", &format!("Capturing window with title '{}' to '{}'", title, path));
                                match validate_capture_window_output_path(&path) {
                                    Ok(validated_path) => {
                                        match capture_window_by_title(&title, false) {
                                            Ok((png_data, width, height)) => {
                                                let mut can_write = true;
                                                if let Some(parent) = validated_path.parent() {
                                                    if let Err(e) = std::fs::create_dir_all(parent) {
                                                        can_write = false;
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Failed to create screenshot directory '{}': {}",
                                                                parent.display(),
                                                                e
                                                            ),
                                                        );
                                                    }
                                                }

                                                if can_write {
                                                    if let Err(e) = std::fs::write(&validated_path, &png_data) {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!("Failed to write screenshot: {}", e),
                                                        );
                                                    } else {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Screenshot saved: {} ({}x{})",
                                                                validated_path.display(),
                                                                width,
                                                                height
                                                            ),
                                                        );
                                                    }
                                                } else {
                                                    tracing::warn!(
                                                        category = "STDIN",
                                                        event_type = "stdin_capture_window_dir_create_failed",
                                                        requested_path = %path,
                                                        resolved_path = %validated_path.display(),
                                                        correlation_id = %logging::current_correlation_id(),
                                                        "Skipping screenshot write due to directory creation failure"
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                logging::log("STDIN", &format!("Failed to capture window: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let correlation_id = logging::current_correlation_id();
                                        tracing::warn!(
                                            category = "STDIN",
                                            event_type = "stdin_capture_window_path_rejected",
                                            requested_path = %path,
                                            reason = %e,
                                            correlation_id = %correlation_id,
                                            "Rejected captureWindow output path"
                                        );
                                        logging::log(
                                            "STDIN",
                                            &format!("Rejected captureWindow path '{}': {}", path, e),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAiSearch { text, ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_ai_command_received",
                                    command = "setAiSearch",
                                    request_id = ?request_id,
                                    text_len = text.len(),
                                    "STDIN AI command received"
                                );
                                match ai::set_ai_search(ctx, &text) {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiSearch",
                                            request_id = ?request_id,
                                            status = "success",
                                            "STDIN AI command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log("STDIN", &format!("Failed to set AI search filter: {}", error));
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiSearch",
                                            request_id = ?request_id,
                                            status = "error",
                                            error = %error,
                                            "STDIN AI command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAiInput { text, submit, ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_ai_command_received",
                                    command = "setAiInput",
                                    request_id = ?request_id,
                                    submit,
                                    text_len = text.len(),
                                    "STDIN AI command received"
                                );
                                match ai::set_ai_input(ctx, &text, submit) {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "success",
                                            "STDIN AI command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log("STDIN", &format!("Failed to set AI input: {}", error));
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "error",
                                            error = %error,
                                            "STDIN AI command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAcpInput { text, submit, ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_acp_command_received",
                                    command = "setAcpInput",
                                    request_id = ?request_id,
                                    submit,
                                    text_len = text.len(),
                                    "STDIN ACP command received"
                                );
                                let result = match &view.current_view {
                                    AppView::AcpChatView { entity } => {
                                        let entity = entity.clone();
                                        entity.update(ctx, |chat, cx| {
                                            chat.set_input(text.clone(), cx);
                                            if submit {
                                                let _ = chat
                                                    .thread
                                                    .update(cx, |thread, cx| thread.submit_input(cx));
                                            }
                                        });
                                        Ok(())
                                    }
                                    _ => Err("ACP chat view is not active".to_string()),
                                };
                                match result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "setAcpInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "success",
                                            "STDIN ACP command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to set ACP input: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "setAcpInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "error",
                                            error = %error,
                                            "STDIN ACP command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::GetAiWindowState { ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                match ai::get_ai_window_state(ctx) {
                                    Some(snapshot) => {
                                        let json = serde_json::to_string(&snapshot).unwrap_or_default();
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "ai_window_state_result",
                                            command = "getAiWindowState",
                                            request_id = ?request_id,
                                            ok = true,
                                            state = %json,
                                            "AI window state snapshot"
                                        );
                                    }
                                    None => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "ai_window_state_result",
                                            command = "getAiWindowState",
                                            request_id = ?request_id,
                                            ok = false,
                                            error_code = "ai_window_not_open",
                                            "AI window not open or entity dropped"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::ShowGrid { grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, ref depth } => {
                                logging::log("STDIN", &format!(
                                    "ShowGrid: size={}, bounds={}, box_model={}, guides={}, dimensions={}, depth={:?}",
                                    grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, depth
                                ));
                                let options = protocol::GridOptions {
                                    grid_size,
                                    show_bounds,
                                    show_box_model,
                                    show_alignment_guides,
                                    show_dimensions,
                                    depth: depth.clone(),
                                    color_scheme: None,
                                };
                                view.show_grid(options, ctx);
                            }
                            ExternalCommand::HideGrid => {
                                logging::log("STDIN", "HideGrid: hiding debug grid overlay");
                                view.hide_grid(ctx);
                            }
                            ExternalCommand::ExecuteFallback { ref fallback_id, ref input } => {
                                logging::log("STDIN", &format!("ExecuteFallback: id='{}', input='{}'", fallback_id, input));
                                execute_fallback_action(view, fallback_id, input, window, ctx);
                            }
                            ExternalCommand::ShowShortcutRecorder { ref command_id, ref command_name } => {
                                logging::log("STDIN", &format!("ShowShortcutRecorder: command_id='{}', command_name='{}'", command_id, command_name));
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), ctx);
                            }
                        }
