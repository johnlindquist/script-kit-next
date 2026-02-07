            PromptMessage::ScriptExit => {
                logging::log("VISIBILITY", "=== ScriptExit message received ===");
                let was_visible = script_kit_gpui::is_main_window_visible();
                let script_hid_window = script_kit_gpui::script_requested_hide();
                logging::log(
                    "VISIBILITY",
                    &format!(
                        "WINDOW_VISIBLE was: {}, SCRIPT_REQUESTED_HIDE was: {}",
                        was_visible, script_hid_window
                    ),
                );

                // Reset the script-requested-hide flag
                script_kit_gpui::set_script_requested_hide(false);
                logging::log("VISIBILITY", "SCRIPT_REQUESTED_HIDE reset to: false");

                // Set flag so next hotkey show will reset to script list
                NEEDS_RESET.store(true, Ordering::SeqCst);
                logging::log("VISIBILITY", "NEEDS_RESET set to: true");

                self.reset_to_script_list(cx);
                logging::log("VISIBILITY", "reset_to_script_list() called");

                // If the script had hidden the window (e.g., for getSelectedText),
                // request showing the main window so the menu comes back
                if script_hid_window {
                    logging::log(
                        "VISIBILITY",
                        "Script had hidden window - requesting show main window",
                    );
                    script_kit_gpui::request_show_main_window();
                } else {
                    // Script didn't hide window, so it was user-initiated hide or already visible
                    // Restore window height to main menu size in case a prompt (like EnvPrompt)
                    // had shrunk the window
                    resize_to_view_sync(ViewType::ScriptList, 0);
                    script_kit_gpui::set_main_window_visible(false);
                    logging::log(
                        "VISIBILITY",
                        "Script didn't hide window - restored height and keeping visibility state",
                    );
                }
            }
            PromptMessage::HideWindow => {
                logging::log("VISIBILITY", "=== HideWindow message received ===");
                let was_visible = script_kit_gpui::is_main_window_visible();
                logging::log(
                    "VISIBILITY",
                    &format!("WINDOW_VISIBLE was: {}", was_visible),
                );

                // CRITICAL: Update visibility state so hotkey toggle works correctly
                script_kit_gpui::set_main_window_visible(false);
                logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

                // Mark that script requested hide - so ScriptExit knows to show window again
                script_kit_gpui::set_script_requested_hide(true);
                logging::log("VISIBILITY", "SCRIPT_REQUESTED_HIDE set to: true");

                // Set flag so next hotkey show will reset to script list
                NEEDS_RESET.store(true, Ordering::SeqCst);
                logging::log("VISIBILITY", "NEEDS_RESET set to: true");

                // Hide main window only (not entire app) to keep HUD visible
                platform::hide_main_window();
                logging::log(
                    "VISIBILITY",
                    "platform::hide_main_window() called - main window hidden, HUDs remain visible",
                );
            }
            PromptMessage::OpenBrowser { url } => {
                logging::log("UI", &format!("Opening browser: {}", url));
                #[cfg(target_os = "macos")]
                {
                    match std::process::Command::new("open").arg(&url).spawn() {
                        Ok(_) => logging::log(
                            "UI",
                            &format!("Successfully opened URL in browser: {}", url),
                        ),
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open URL '{}': {}", url, e))
                        }
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    match std::process::Command::new("xdg-open").arg(&url).spawn() {
                        Ok(_) => logging::log(
                            "UI",
                            &format!("Successfully opened URL in browser: {}", url),
                        ),
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open URL '{}': {}", url, e))
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                {
                    match std::process::Command::new("cmd")
                        .args(["/C", "start", &url])
                        .spawn()
                    {
                        Ok(_) => logging::log(
                            "UI",
                            &format!("Successfully opened URL in browser: {}", url),
                        ),
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open URL '{}': {}", url, e))
                        }
                    }
                }
            }
            PromptMessage::RunScript { path } => {
                logging::log("EXEC", &format!("RunScript command received: {}", path));

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

                logging::log("EXEC", &format!("Executing script: {}", script_name));
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
                logging::log(
                    "ERROR",
                    &format!(
                        "Script error received: {} (exit code: {:?})",
                        error_message, exit_code
                    ),
                );

                // CRITICAL: Show error via HUD (highly visible floating window)
                // This ensures the user sees the error even if the main window is hidden/dismissed
                // HUD appears at bottom-center of screen for 5 seconds
                let hud_message = if error_message.chars().count() > 60 {
                    // Use chars().take() to safely handle multi-byte UTF-8 characters
                    let truncated: String = error_message.chars().take(57).collect();
                    format!("Script Error: {}...", truncated)
                } else {
                    format!("Script Error: {}", error_message)
                };
                self.show_hud(hud_message, Some(5000), cx);

                // Also create in-app toast with expandable details (for when window is visible)
                // Use stderr_output if available, otherwise use stack_trace
                let details_text = stderr_output.clone().or_else(|| stack_trace.clone());
                let toast = Toast::error(error_message.clone(), &self.theme)
                    .details_opt(details_text.clone())
                    .duration_ms(Some(10000)); // 10 seconds for errors

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
                                logging::log("UI", "Error copied to clipboard");
                            }
                        }),
                    ))
                } else {
                    toast
                };

                // Log suggestions if present
                if !suggestions.is_empty() {
                    logging::log("ERROR", &format!("Suggestions: {:?}", suggestions));
                }

                // Push toast to manager
                let toast_id = self.toast_manager.push(toast);
                logging::log(
                    "UI",
                    &format!(
                        "Toast created for script error: {} (id: {})",
                        script_path, toast_id
                    ),
                );

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
                    .duration_ms(Some(8000));

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
                logging::log(
                    "WARN",
                    &format!("Displaying unhandled message warning: {}", message_type),
                );

                let toast = Toast::warning(unhandled_message_warning(&message_type), &self.theme)
                    .duration_ms(Some(5000));

                self.toast_manager.push(toast);
                cx.notify();
            }
