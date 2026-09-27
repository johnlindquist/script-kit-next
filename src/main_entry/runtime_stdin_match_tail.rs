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
                            ExternalCommand::OpenCreationFeedback { path, receipt_path, receipt_status, verification_status, request_id: _ } => {
                                logging::log("STDIN", "Opening CreationFeedback surface via stdin command");
                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown();
                                platform::show_main_window_without_activation();
                                window.activate_window();
                                sync_main_automation_window(current_main_automation_bounds(), true, true);
                                let artifact_path = path
                                    .map(std::path::PathBuf::from)
                                    .unwrap_or_else(|| std::path::PathBuf::from("/tmp/script-kit-liquid-glass-feedback-fixture.ts"));
                                let payload = crate::prompts::CreationFeedbackPayload::fixture(
                                    artifact_path,
                                    receipt_path.map(std::path::PathBuf::from),
                                    receipt_status
                                        .as_deref()
                                        .map(crate::prompts::CreationFeedbackReceiptStatus::from_fixture_str),
                                    verification_status,
                                );
                                view.open_creation_feedback_payload(payload, ctx);
                            }
                            ExternalCommand::OpenConfirmPrompt { title, body: _, confirm_text: _, cancel_text: _, request_id: _ } => {
                                logging::log("STDIN", "Holding an irreversible fixture in the line via stdin command");
                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown();
                                platform::show_main_window_without_activation();
                                window.activate_window();
                                window_ops::queue_move(
                                    gpui::Bounds {
                                        origin: gpui::point(gpui::px(585.), gpui::px(177.)),
                                        size: gpui::size(
                                            gpui::px(750.),
                                            crate::window_resize::height_for_view(
                                                crate::window_resize::ViewType::MainWindow,
                                                0,
                                            ),
                                        ),
                                    },
                                    window,
                                    ctx,
                                );
                                sync_main_automation_window(current_main_automation_bounds(), true, true);
                                view.one_window_hold(
                                    "trash",
                                    title.unwrap_or_else(|| "Delete saved item".to_string()),
                                    Box::new(|_, _| {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "held_fixture_ran",
                                        );
                                    }),
                                    ctx,
                                );
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening Agent Chat via openAi compatibility alias");
                                view.one_window_open_chat(crate::one_window::chat_entry::ChatEntry::quick(), ctx);
                            }
                            ExternalCommand::OpenChatPromptFixture { ref request_id } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                view.handle_prompt_message(
                                    PromptMessage::ShowChat {
                                        id: "fixture:ordinary-chat-prompt".to_string(),
                                        placeholder: Some("Message".to_string()),
                                        messages: vec![crate::protocol::ChatPromptMessage::assistant(
                                            "Fixture response",
                                        )],
                                        hint: Some("Ordinary Script ChatPrompt".to_string()),
                                        footer: None,
                                        actions: None,
                                        model: None,
                                        models: Vec::new(),
                                        save_history: false,
                                        use_builtin_ai: false,
                                    },
                                    ctx,
                                );
                                tracing::info!(
                                    category = "STDIN",
                                    event = "chat_prompt_fixture_opened",
                                    command = "openChatPromptFixture",
                                    request_id = ?rid,
                                );
                            }
                            ExternalCommand::ClosePromptPopupNatively { ref target, ref request_id } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                match crate::components::inline_popup_window::close_prompt_popup_target_natively(target, ctx) {
                                    Ok((id, generation, native_window_number)) => tracing::info!(
                                        category = "STDIN",
                                        event = "prompt_popup_native_close_requested",
                                        command = "closePromptPopupNatively",
                                        request_id = ?rid,
                                        window_id = %id,
                                        generation,
                                        native_window_number,
                                    ),
                                    Err(error) => tracing::warn!(
                                        category = "STDIN",
                                        event = "prompt_popup_native_close_refused",
                                        command = "closePromptPopupNatively",
                                        request_id = ?rid,
                                        error = %error,
                                    ),
                                }
                            }
                            ExternalCommand::OpenMiniAi => {
                                logging::log("STDIN", "Opening Agent Chat via openMiniAi compatibility alias");
                                view.one_window_open_chat(crate::one_window::chat_entry::ChatEntry::quick(), ctx);
                            }
                            ExternalCommand::ShowAiCommandBar => {
                                logging::log("STDIN", "Ignoring showAiCommandBar: legacy AI window removed");
                            }
                            ExternalCommand::SimulateAiKey { key, modifiers } => {
                                logging::log(
                                    "STDIN",
                                    &format!("Ignoring simulateAiKey '{}' (modifiers: {:?}): legacy AI window removed", key, modifiers),
                                );
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
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_ai_command_finished",
                                    command = "setAiSearch",
                                    request_id = ?request_id,
                                    status = "unsupported",
                                    "setAiSearch removed with the legacy AI window"
                                );
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
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_ai_command_finished",
                                    command = "setAiInput",
                                    request_id = ?request_id,
                                    submit,
                                    status = "unsupported",
                                    "setAiInput removed with the legacy AI window; use setAgentChatInput"
                                );
                            }
                            ExternalCommand::SetAgentChatInput { text, submit, ref request_id } => {
                                let request_id_value = request_id.clone();
                                let request_id = request_id_value.as_deref();
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_agent_chat_command_received",
                                    command = "setAgentChatInput",
                                    request_id = ?request_id,
                                    submit,
                                    text_len = text.len(),
                                    "STDIN Agent Chat command received"
                                );
                                let entry = if submit {
                                    crate::one_window::chat_entry::ChatEntry::sending(text)
                                } else {
                                    crate::one_window::chat_entry::ChatEntry::seeded(text)
                                };
                                view.one_window_open_chat(entry, ctx);
                                let result: Result<(), String> = Ok(());
                                match &result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_agent_chat_command_finished",
                                            command = "setAgentChatInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "success",
                                            "STDIN Agent Chat command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to set Agent Chat input: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_agent_chat_command_finished",
                                            command = "setAgentChatInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "error",
                                            error = %error,
                                            "STDIN Agent Chat command finished"
                                        );
                                    }
                                }
                                if let Some(rid) = request_id_value {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::external_command_result(
                                                rid.to_string(),
                                                "setAgentChatInput".to_string(),
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
                            ExternalCommand::SetAgentChatTranscriptScroll {
                                item_ix,
                                offset_px,
                                ref request_id,
                            } => {
                                let request_id_value = request_id.clone();
                                let result = match &view.current_view {
                                    AppView::FlowSessionView { session_id } => view
                                        .conversations.flow_sessions
                                        .iter()
                                        .find(|(meta, _)| meta.id == *session_id)
                                        .map(|(_, entity)| entity.clone())
                                        .ok_or_else(|| "FlowSession entity is not active".to_string())
                                        .map(|entity| {
                                            entity.update(ctx, |chat, cx| {
                                                chat.set_transcript_geometry_scroll(
                                                    item_ix, offset_px, cx,
                                                );
                                            });
                                        }),
                                    _ => Err("Agent Chat view is not active".to_string()),
                                };
                                if let Some(rid) = request_id_value {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::external_command_result(
                                                rid.to_string(),
                                                "setAgentChatTranscriptScroll".to_string(),
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
                            ExternalCommand::PasteClipboardIntoAgentChat { ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_agent_chat_command_received",
                                    command = "pasteClipboardIntoAgentChat",
                                    request_id = ?request_id,
                                    "STDIN Agent Chat command received"
                                );
                                let result = match ctx.read_from_clipboard().and_then(|item| item.text()) {
                                    Some(text) => {
                                        view.one_window_open_chat(
                                            crate::one_window::chat_entry::ChatEntry::seeded(text),
                                            ctx,
                                        );
                                        Ok(())
                                    }
                                    None => Err("clipboard is empty or text fetch failed".to_string()),
                                };
                                match result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_agent_chat_command_finished",
                                            command = "pasteClipboardIntoAgentChat",
                                            request_id = ?request_id,
                                            status = "success",
                                            "STDIN Agent Chat command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to paste clipboard into Agent Chat: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_agent_chat_command_finished",
                                            command = "pasteClipboardIntoAgentChat",
                                            request_id = ?request_id,
                                            status = "error",
                                            error = %error,
                                            "STDIN Agent Chat command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::PushDictationResult {
                                ref transcript,
                                ref partial_transcript,
                                ref target,
                                freeze_only,
                                use_frozen_selection,
                                ref request_id,
                            } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                let target_label = target.as_deref().unwrap_or("unspecified");
                                let resolution =
                                    crate::dictation::resolve_final_or_partial_transcript(
                                        transcript,
                                        partial_transcript.as_deref(),
                                    );
                                match view.deliver_stdin_dictation_result(
                                    transcript.clone(),
                                    partial_transcript.as_deref(),
                                    target.as_deref(),
                                    freeze_only,
                                    use_frozen_selection,
                                    ctx,
                                ) {
                                    Ok(delivery_target) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "push_dictation_result_delivered",
                                            command = "pushDictationResult",
                                            request_id = ?rid,
                                            transcript_len = resolution.transcript.as_ref().map_or(0, String::len),
                                            final_transcript_len = resolution.final_len,
                                            partial_transcript_len = ?resolution.partial_len,
                                            partial_fallback_used = resolution.used_partial_fallback,
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
                                            transcript_len = resolution.transcript.as_ref().map_or(0, String::len),
                                            final_transcript_len = resolution.final_len,
                                            partial_transcript_len = ?resolution.partial_len,
                                            partial_fallback_used = resolution.used_partial_fallback,
                                            requested_target = target_label,
                                            error = %error,
                                            "pushDictationResult RPC failed"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::GetAiWindowState { ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "ai_window_state_result",
                                    command = "getAiWindowState",
                                    request_id = ?request_id,
                                    ok = false,
                                    error_code = "ai_window_removed",
                                    "legacy AI window removed"
                                );
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
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), ctx);
                            }
                        }
