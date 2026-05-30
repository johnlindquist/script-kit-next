                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAbout => {
                                logging::log("STDIN", "Opening About surface via stdin command");
                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown();
                                platform::show_main_window_without_activation();
                                window.activate_window();
                                sync_main_automation_window(current_main_automation_bounds(), true, true);
                                view.open_about_surface(
                                    std::sync::Arc::new(std::sync::RwLock::new(
                                        crate::updates::UpdateState::Idle,
                                    )),
                                    ctx,
                                );
                            }
                            ExternalCommand::OpenCreationFeedback { path, request_id: _ } => {
                                logging::log("STDIN", "Opening CreationFeedback surface via stdin command");
                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown();
                                platform::show_main_window_without_activation();
                                window.activate_window();
                                sync_main_automation_window(current_main_automation_bounds(), true, true);
                                view.open_creation_feedback_surface(path.map(std::path::PathBuf::from), ctx);
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening Agent Chat via openAi compatibility alias");
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenMiniAi => {
                                logging::log("STDIN", "Opening Agent Chat via openMiniAi compatibility alias");
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log(
                                    "STDIN",
                                    "Ignoring deprecated mock-data AI alias and opening Agent Chat",
                                );
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenMiniAiWithMockData => {
                                logging::log(
                                    "STDIN",
                                    "Ignoring deprecated mini mock-data AI alias and opening Agent Chat",
                                );
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenFocusedTextAgentChatWithMockData { text, instruction, request_id }
                            | ExternalCommand::OpenInlineAgentWithMockData { text, instruction, request_id } => {
                                logging::log("STDIN", "Opening focused-text Agent Chat mock fixture");
                                let text_length = text.as_ref().map(|value| value.len()).unwrap_or("Hello world".len());
                                let instruction_length = instruction
                                    .as_ref()
                                    .map(|value| value.trim().len())
                                    .unwrap_or(0);
                                let requested_submit = instruction_length > 0;
                                let result = view.open_focused_text_agent_chat_fixture(
                                    text,
                                    instruction,
                                    "focused_text_mock_fixture",
                                    ctx,
                                );
                                let ok = result.is_ok();
                                if let Err(error) = result {
                                    logging::log(
                                        "STDIN",
                                        &format!("Failed to open focused-text Agent Chat mock fixture: {error}"),
                                    );
                                }
                                if let Some(rid) = request_id {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::inline_agent_fixture_open_result(
                                                rid.to_string(),
                                                "mock".to_string(),
                                                ok,
                                                ok && requested_submit,
                                                text_length,
                                                instruction_length,
                                                if ok { None } else { Some("open_failed".to_string()) },
                                                if ok {
                                                    None
                                                } else {
                                                    Some("Focused-text Agent Chat mock fixture open failed".to_string())
                                                },
                                            ),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::OpenFocusedTextAgentChatFromFocusedFieldWithMockData { instruction, request_id } => {
                                logging::log("STDIN", "Opening focused-text Agent Chat live mock fixture");
                                let instruction_length = instruction
                                    .as_ref()
                                    .map(|value| value.trim().len())
                                    .unwrap_or(0);
                                let requested_submit = instruction_length > 0;
                                let result = view.open_focused_text_agent_chat_from_focused_field_mock_fixture(
                                    instruction,
                                    ctx,
                                );
                                let (ok, text_length, error_code, error_message) = match result {
                                    Ok(text_length) => (true, text_length, None, None),
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to open focused-text Agent Chat live mock fixture: {error}"),
                                        );
                                        let error_code = if error.contains("SCRIPT_KIT_FOCUSED_TEXT_LIVE_FIXTURE") {
                                            "gated_off"
                                        } else {
                                            "open_failed"
                                        };
                                        (
                                            false,
                                            0,
                                            Some(error_code.to_string()),
                                            Some("Focused-text Agent Chat live mock fixture open failed".to_string()),
                                        )
                                    }
                                };
                                if let Some(rid) = request_id {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::inline_agent_fixture_open_result(
                                                rid.to_string(),
                                                "live-mock".to_string(),
                                                ok,
                                                ok && requested_submit,
                                                text_length,
                                                instruction_length,
                                                error_code,
                                                error_message,
                                            ),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::OpenFocusedTextAgentChatWithPiData { text, instruction, request_id }
                            | ExternalCommand::OpenInlineAgentWithPiData { text, instruction, request_id } => {
                                logging::log("STDIN", "Opening focused-text Agent Chat real Pi fixture");
                                let text_length = text.as_ref().map(|value| value.len()).unwrap_or("Hello world".len());
                                let instruction_length = instruction
                                    .as_ref()
                                    .map(|value| value.trim().len())
                                    .unwrap_or(0);
                                let requested_submit = instruction_length > 0;
                                let result = view.open_focused_text_agent_chat_fixture(
                                    text,
                                    instruction,
                                    "focused_text_pi_fixture",
                                    ctx,
                                );
                                let ok = result.is_ok();
                                let (error_code, error_message) = match result {
                                    Ok(()) => (None, None),
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to open focused-text Agent Chat real Pi fixture: {error}"),
                                        );
                                        let error_text = error.to_string();
                                        if error_text.contains("SCRIPT_KIT_INLINE_AGENT_REAL_PI_FIXTURE") {
                                            (
                                                Some("gated_off".to_string()),
                                                Some("Focused-text Agent Chat real Pi fixture is gated off".to_string()),
                                            )
                                        } else {
                                            (
                                                Some("open_failed".to_string()),
                                                Some("Focused-text Agent Chat real Pi fixture open failed".to_string()),
                                            )
                                        }
                                    }
                                };
                                if let Some(rid) = request_id {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::inline_agent_fixture_open_result(
                                                rid.to_string(),
                                                "pi".to_string(),
                                                ok,
                                                ok && requested_submit,
                                                text_length,
                                                instruction_length,
                                                error_code,
                                                error_message,
                                            ),
                                        );
                                    }
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
                                        match capture_window_by_title_via_resolver(&title, false) {
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
                                                tracing::error!(
                                                    category = "STDIN",
                                                    event_type = "stdin_capture_window_failed",
                                                    requested_title = %title,
                                                    requested_path = %path,
                                                    error = %e,
                                                    correlation_id = %logging::current_correlation_id(),
                                                    "captureWindow failed before writing screenshot"
                                                );
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
                                let request_id_value = request_id.clone();
                                let request_id = request_id_value.as_deref();
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
                                            chat.set_input_in_window(text.clone(), window, cx);
                                            if submit {
                                                let _ = chat
                                                    .thread
                                                    .update(cx, |thread, cx| thread.submit_input(cx));
                                            }
                                        });
                                        Ok(())
                                    }
                                    _ => Err("Agent Chat view is not active".to_string()),
                                };
                                match &result {
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
                                if let Some(rid) = request_id_value {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::external_command_result(
                                                rid.to_string(),
                                                "setAcpInput".to_string(),
                                                result.is_ok(),
                                                result
                                                    .as_ref()
                                                    .err()
                                                    .map(|_| "agent_chat_inactive".to_string()),
                                                result.as_ref().err().cloned(),
                                            ),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAcpTestFixture {
                                ref phase,
                                ref user_text,
                                ref assistant_text,
                                ref request_id,
                            } => {
                                let request_id_value = request_id.clone();
                                let request_id = request_id_value.as_deref();
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_acp_command_received",
                                    command = "setAcpTestFixture",
                                    request_id = ?request_id,
                                    phase = %phase,
                                    user_text_len = user_text.as_ref().map(|text| text.len()).unwrap_or(0),
                                    assistant_text_len = assistant_text.as_ref().map(|text| text.len()).unwrap_or(0),
                                    "STDIN ACP command received"
                                );
                                let result = match &view.current_view {
                                    AppView::AcpChatView { entity } => {
                                        let entity = entity.clone();
                                        entity.update(ctx, |chat, cx| {
                                            chat.apply_test_fixture(
                                                phase,
                                                user_text.clone(),
                                                assistant_text.clone(),
                                                cx,
                                            )
                                        })
                                    }
                                    _ => Err("Agent Chat view is not active".to_string()),
                                };
                                match &result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "setAcpTestFixture",
                                            request_id = ?request_id,
                                            phase = %phase,
                                            status = "success",
                                            "STDIN ACP command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to set ACP test fixture: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "setAcpTestFixture",
                                            request_id = ?request_id,
                                            phase = %phase,
                                            status = "error",
                                            error = %error,
                                            "STDIN ACP command finished"
                                        );
                                    }
                                }
                                if let Some(rid) = request_id_value {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::external_command_result(
                                                rid.to_string(),
                                                "setAcpTestFixture".to_string(),
                                                result.is_ok(),
                                                result
                                                    .as_ref()
                                                    .err()
                                                    .map(|_| "agent_chat_inactive".to_string()),
                                                result.as_ref().err().cloned(),
                                            ),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::PasteClipboardIntoAcp { ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_acp_command_received",
                                    command = "pasteClipboardIntoAcp",
                                    request_id = ?request_id,
                                    "STDIN ACP command received"
                                );
                                let result = match &view.current_view {
                                    AppView::AcpChatView { entity } => {
                                        let entity = entity.clone();
                                        let pasted = entity
                                            .update(ctx, |chat, cx| chat.paste_text_from_clipboard(cx));
                                        if pasted {
                                            Ok(())
                                        } else {
                                            Err("clipboard is empty or text fetch failed"
                                                .to_string())
                                        }
                                    }
                                    _ => Err("Agent Chat view is not active".to_string()),
                                };
                                match result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "pasteClipboardIntoAcp",
                                            request_id = ?request_id,
                                            status = "success",
                                            "STDIN ACP command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to paste clipboard into ACP: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "pasteClipboardIntoAcp",
                                            request_id = ?request_id,
                                            status = "error",
                                            error = %error,
                                            "STDIN ACP command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::PushDictationResult {
                                ref transcript,
                                ref target,
                                ref request_id,
                            } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                let target_label = target.as_deref().unwrap_or("unspecified");
                                match view.deliver_stdin_dictation_result(
                                    transcript.clone(),
                                    target.as_deref(),
                                    ctx,
                                ) {
                                    Ok(delivery_target) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "push_dictation_result_delivered",
                                            command = "pushDictationResult",
                                            request_id = ?rid,
                                            transcript_len = transcript.len(),
                                            requested_target = target_label,
                                            delivery_target = ?delivery_target,
                                            "pushDictationResult RPC delivered through dictation pipeline"
                                        );
                                    }
                                    Err(error) => {
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "push_dictation_result_failed",
                                            command = "pushDictationResult",
                                            request_id = ?rid,
                                            transcript_len = transcript.len(),
                                            requested_target = target_label,
                                            error = %error,
                                            "pushDictationResult RPC failed"
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
                            ExternalCommand::GetConfigFingerprint { ref request_id } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                match crate::config::current_config_fingerprint_receipt() {
                                    Some(receipt) => {
                                        let json = serde_json::to_string(&receipt).unwrap_or_default();
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "config_fingerprint_result",
                                            command = "getConfigFingerprint",
                                            request_id = ?rid,
                                            ok = true,
                                            state = %json,
                                            "config.ts fingerprint snapshot"
                                        );
                                    }
                                    None => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "config_fingerprint_result",
                                            command = "getConfigFingerprint",
                                            request_id = ?rid,
                                            ok = false,
                                            error_code = "config_file_missing",
                                            "config.ts not found or metadata unreadable"
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
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), window, ctx);
                            }
                        }
