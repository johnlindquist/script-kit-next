            builtins::BuiltInFeature::NotesCommand(cmd_type) => {
                logging::log("EXEC", &format!("Executing notes command: {:?}", cmd_type));

                use builtins::NotesCommandType;

                // All notes commands: reset state, hide main window, open notes
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

                let result = match cmd_type {
                    NotesCommandType::OpenNotes
                    | NotesCommandType::NewNote
                    | NotesCommandType::SearchNotes => notes::open_notes_window(cx),
                    NotesCommandType::QuickCapture => notes::quick_capture(cx),
                };

                if let Err(e) = result {
                    logging::log("ERROR", &format!("Notes command failed: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Notes command failed: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }

            // =========================================================================
            // AI Commands
            // =========================================================================
            builtins::BuiltInFeature::AiCommand(cmd_type) => {
                logging::log("EXEC", &format!("Executing AI command: {:?}", cmd_type));

                use builtins::AiCommandType;

                // All AI commands: reset state, hide main window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

                match cmd_type {
                    AiCommandType::OpenAi | AiCommandType::NewConversation => {
                        // Basic open/new conversation
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("AI command failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to open AI: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }

                    AiCommandType::ClearConversation => {
                        match ai::clear_all_chats() {
                            Ok(()) => {
                                // Force a fresh AI window state so cleared history is reflected immediately.
                                ai::close_ai_window(cx);
                                if let Err(e) = ai::open_ai_window(cx) {
                                    logging::log(
                                        "ERROR",
                                        &format!(
                                            "AI history cleared but failed to reopen AI window: {}",
                                            e
                                        ),
                                    );
                                    self.toast_manager.push(
                                        components::toast::Toast::error(
                                            format!(
                                                "AI history cleared, but failed to open AI: {}",
                                                e
                                            ),
                                            &self.theme,
                                        )
                                        .duration_ms(Some(5000)),
                                    );
                                    cx.notify();
                                } else {
                                    self.show_hud(
                                        "Cleared AI conversations".to_string(),
                                        Some(2000),
                                        cx,
                                    );
                                }
                            }
                            Err(e) => {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to clear AI conversations: {}", e),
                                );
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Failed to clear AI conversations: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }

                    AiCommandType::SendScreenToAi => {
                        // Capture entire screen and send to AI
                        match platform::capture_screen_screenshot() {
                            Ok((png_data, width, height)) => {
                                let base64_data = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    &png_data,
                                );
                                let message = format!(
                                    "[Screenshot captured: {}x{} pixels]\n\nPlease analyze this screenshot.",
                                    width, height
                                );
                                logging::log(
                                    "EXEC",
                                    &format!(
                                        "Screen captured: {}x{}, {} bytes",
                                        width,
                                        height,
                                        png_data.len()
                                    ),
                                );
                                if let Err(e) = ai::open_ai_window(cx) {
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
                                } else {
                                    // Set input with the screenshot context
                                    ai::set_ai_input_with_image(cx, &message, &base64_data, false);
                                }
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to capture screen: {}", e));
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Failed to capture screen: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }

                    AiCommandType::SendFocusedWindowToAi => {
                        // Capture the focused window (not our window) and send to AI
                        match platform::capture_focused_window_screenshot() {
                            Ok((png_data, width, height, window_title)) => {
                                let base64_data = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    &png_data,
                                );
                                let message = format!(
                                    "[Window: {} - {}x{} pixels]\n\nPlease analyze this window screenshot.",
                                    window_title, width, height
                                );
                                logging::log(
                                    "EXEC",
                                    &format!(
                                        "Window '{}' captured: {}x{}, {} bytes",
                                        window_title,
                                        width,
                                        height,
                                        png_data.len()
                                    ),
                                );
                                if let Err(e) = ai::open_ai_window(cx) {
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
                                } else {
                                    ai::set_ai_input_with_image(cx, &message, &base64_data, false);
                                }
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to capture window: {}", e));
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Failed to capture window: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }

                    AiCommandType::SendSelectedTextToAi => {
                        // Get selected text and send to AI
                        match crate::selected_text::get_selected_text() {
                            Ok(text) if !text.is_empty() => {
                                let message = format!(
                                    "I've selected the following text:\n\n```\n{}\n```\n\nPlease help me with this.",
                                    text
                                );
                                logging::log(
                                    "EXEC",
                                    &format!("Selected text captured: {} chars", text.len()),
                                );
                                if let Err(e) = ai::open_ai_window(cx) {
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
                                } else {
                                    ai::set_ai_input(cx, &message, false);
                                }
                            }
                            Ok(_) => {
                                // No text selected
                                self.toast_manager.push(
                                    components::toast::Toast::info(
                                        "No text selected. Select some text first.",
                                        &self.theme,
                                    )
                                    .duration_ms(Some(3000)),
                                );
                                cx.notify();
                            }
                            Err(e) => {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to get selected text: {}", e),
                                );
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Failed to get selected text: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }

                    AiCommandType::SendBrowserTabToAi => {
                        // Get browser URL and send to AI
                        match platform::get_focused_browser_tab_url() {
                            Ok(url) => {
                                let message = format!(
                                    "I'm looking at this webpage:\n\n{}\n\nPlease help me analyze or understand its content.",
                                    url
                                );
                                logging::log("EXEC", &format!("Browser URL captured: {}", url));
                                if let Err(e) = ai::open_ai_window(cx) {
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
                                } else {
                                    ai::set_ai_input(cx, &message, false);
                                }
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to get browser URL: {}", e));
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Failed to get browser URL: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }

                    AiCommandType::SendScreenAreaToAi => {
                        // Interactive screen area selection - for now just show a message
                        // Full implementation would need a selection UI overlay
                        self.toast_manager.push(
                            components::toast::Toast::info(
                                "Screen area selection coming soon. Use 'Send Screen to AI' for now.",
                                &self.theme,
                            )
                            .duration_ms(Some(3000)),
                        );
                        cx.notify();
                    }

                    AiCommandType::CreateAiPreset
                    | AiCommandType::ImportAiPresets
                    | AiCommandType::SearchAiPresets => {
                        // Preset management - open AI window with a future preset UI
                        self.toast_manager.push(
                            components::toast::Toast::info(
                                "AI Presets feature coming soon!",
                                &self.theme,
                            )
                            .duration_ms(Some(3000)),
                        );
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("Failed to open AI: {}", e));
                        }
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Script Commands
            // =========================================================================
            builtins::BuiltInFeature::ScriptCommand(cmd_type) => {
                logging::log("EXEC", &format!("Executing script command: {:?}", cmd_type));

                use builtins::ScriptCommandType;

                let (create_result, item_type) = match cmd_type {
                    ScriptCommandType::NewScript => {
                        (script_creation::create_new_script("untitled"), "script")
                    }
                    ScriptCommandType::NewExtension => {
                        // Generate a unique name with timestamp
                        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
                        let name = format!("my-extension-{}", timestamp);
                        (script_creation::create_new_extension(&name), "extension")
                    }
                };

                match create_result {
                    Ok(path) => {
                        logging::log("EXEC", &format!("Created new {}: {:?}", item_type, path));
                        if let Err(e) = script_creation::open_in_editor(&path, &self.config) {
                            logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!(
                                        "Created {} but failed to open editor: {}",
                                        item_type, e
                                    ),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        } else {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    format!("New {} created and opened in editor", item_type),
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        }
                        self.close_and_reset_window(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to create {}: {}", item_type, e));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to create {}: {}", item_type, e),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Permission Commands
            // =========================================================================
