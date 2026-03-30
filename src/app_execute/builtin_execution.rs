/// Delay between hiding the main window and starting a synchronous screenshot capture.
const AI_CAPTURE_HIDE_SETTLE_MS: u64 = 150;

/// Synthetic prompt ID used when the microphone-selection ArgPrompt is open.
/// Checked in `submit_arg_prompt_from_current_state` to intercept the submit
/// and persist the chosen device instead of sending a protocol message.
const BUILTIN_MIC_SELECT_PROMPT_ID: &str = "builtin:select-microphone";

/// Choice value representing "use system default" in the mic-selection prompt.
const BUILTIN_MIC_DEFAULT_VALUE: &str = "__system_default__";

#[cfg(test)]
fn ai_open_failure_message(error: impl std::fmt::Display) -> String {
    format!("Failed to open AI: {}", error)
}

#[derive(Debug)]
enum DeferredAiCapturedText {
    Ready(String),
    Empty(String),
}

fn ai_capture_hide_settle_duration() -> std::time::Duration {
    std::time::Duration::from_millis(AI_CAPTURE_HIDE_SETTLE_MS)
}

fn ai_command_keeps_main_window_visible(cmd_type: &builtins::AiCommandType) -> bool {
    match cmd_type {
        builtins::AiCommandType::GenerateScript => true,
        builtins::AiCommandType::MiniAi => false,
        _ => false,
    }
}

fn ai_command_uses_hide_then_capture_flow(cmd_type: &builtins::AiCommandType) -> bool {
    matches!(
        cmd_type,
        builtins::AiCommandType::GenerateScriptFromCurrentApp
            | builtins::AiCommandType::SendScreenToAi
            | builtins::AiCommandType::SendFocusedWindowToAi
            | builtins::AiCommandType::SendScreenAreaToAi
            | builtins::AiCommandType::SendSelectedTextToAi
            | builtins::AiCommandType::SendBrowserTabToAi
    )
}

#[cfg(test)]
fn favorites_loaded_message(count: usize) -> String {
    if count == 1 {
        "Loaded 1 favorite".to_string()
    } else {
        format!("Loaded {} favorites", count)
    }
}

#[cfg(test)]
fn created_file_path_for_feedback(path: &std::path::Path) -> std::path::PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    match std::env::current_dir() {
        Ok(current_dir) => current_dir.join(path),
        Err(_) => path.to_path_buf(),
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Retained for potential future AppleScript-based pickers
fn applescript_list_literal(values: &[String]) -> String {
    let escaped_values = values
        .iter()
        .map(|value| format!("\"{}\"", crate::utils::escape_applescript_string(value)))
        .join(", ");
    format!("{{{}}}", escaped_values)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Retained for potential future AppleScript-based pickers
fn choose_from_list(
    prompt: &str,
    ok_button: &str,
    values: &[String],
) -> Result<Option<String>, String> {
    if values.is_empty() {
        return Ok(None);
    }

    let list_literal = applescript_list_literal(values);
    let script = format!(
        r#"set selectedItem to choose from list {list_literal} with prompt "{prompt}" OK button name "{ok_button}" cancel button name "Cancel" without multiple selections allowed
if selectedItem is false then
    return ""
end if
return item 1 of selectedItem"#,
        list_literal = list_literal,
        prompt = crate::utils::escape_applescript_string(prompt),
        ok_button = crate::utils::escape_applescript_string(ok_button),
    );

    let selected = crate::platform::run_osascript(&script, "builtin_picker_choose_from_list")
        .map_err(|error| error.to_string())?;
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Retained for potential future AppleScript-based pickers
fn prompt_for_text(
    prompt: &str,
    default_value: &str,
    ok_button: &str,
) -> Result<Option<String>, String> {
    let script = format!(
        r#"try
set dialogResult to display dialog "{prompt}" default answer "{default_value}" buttons {{"Cancel", "{ok_button}"}} default button "{ok_button}"
return text returned of dialogResult
on error number -128
return ""
end try"#,
        prompt = crate::utils::escape_applescript_string(prompt),
        default_value = crate::utils::escape_applescript_string(default_value),
        ok_button = crate::utils::escape_applescript_string(ok_button),
    );

    let value = crate::platform::run_osascript(&script, "builtin_picker_prompt_for_text")
        .map_err(|error| error.to_string())?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[cfg(test)]
fn emoji_picker_label(emoji: &script_kit_gpui::emoji::Emoji) -> String {
    format!("{}  {}", emoji.emoji, emoji.name)
}


impl ScriptListApp {
    fn spawn_send_screen_to_ai_after_hide(&mut self, trace_id: &str, cx: &mut Context<Self>) {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "SendScreenToAi",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling screen capture for AI"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let capture_result = cx
                .background_executor()
                .spawn(async { platform::capture_screen_screenshot() })
                .await;

            match capture_result {
                Ok((png_data, width, height)) => {
                    let size_bytes = png_data.len();
                    if size_bytes > crate::prompts::chat::MAX_IMAGE_BYTES {
                        tracing::warn!(
                            category = "AI",
                            event = "ai_capture_rejected",
                            source_action = "SendScreenToAi",
                            trace_id = %trace_id,
                            size_bytes,
                            max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                            "Rejecting screen capture larger than 10 MB"
                        );
                        this.update(cx, |this, cx| {
                            this.show_error_toast(
                                "Screen capture exceeds 10 MB limit".to_string(),
                                cx,
                            );
                        })
                        .ok();
                        return;
                    }

                    let base64_data = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &png_data,
                    );
                    let message = format!(
                        "[Screenshot captured: {}x{} pixels]\n\nPlease analyze this screenshot.",
                        width, height
                    );

                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_completed",
                        source_action = "SendScreenToAi",
                        trace_id = %trace_id,
                        width,
                        height,
                        size_bytes,
                        "Screen captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        this.open_ai_window_after_already_hidden(
                            "SendScreenToAi",
                            &trace_id,
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            cx,
                        );
                    })
                    .ok();
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action = "SendScreenToAi",
                        trace_id = %trace_id,
                        error = %error,
                        "Failed to capture screen for AI"
                    );
                    let message = format!("Failed to capture screen: {}", error);
                    this.update(cx, |this, cx| {
                        this.show_error_toast(message, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn spawn_send_focused_window_to_ai_after_hide(&mut self, trace_id: &str, cx: &mut Context<Self>) {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "SendFocusedWindowToAi",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling focused window capture for AI"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let capture_result = cx
                .background_executor()
                .spawn(async { platform::capture_focused_window_screenshot() })
                .await;

            match capture_result {
                Ok(capture) => {
                    let size_bytes = capture.png_data.len();
                    if size_bytes > crate::prompts::chat::MAX_IMAGE_BYTES {
                        tracing::warn!(
                            category = "AI",
                            event = "ai_capture_rejected",
                            source_action = "SendFocusedWindowToAi",
                            trace_id = %trace_id,
                            size_bytes,
                            max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                            "Rejecting window capture larger than 10 MB"
                        );
                        this.update(cx, |this, cx| {
                            this.show_error_toast(
                                "Window capture exceeds 10 MB limit".to_string(),
                                cx,
                            );
                        })
                        .ok();
                        return;
                    }

                    let fallback_warning = capture.used_fallback.then(|| {
                        format!(
                            "No focused window found — captured '{}'",
                            capture.window_title
                        )
                    });
                    let base64_data = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &capture.png_data,
                    );
                    let message = format!(
                        "[Window: {} - {}x{} pixels]\n\nPlease analyze this window screenshot.",
                        capture.window_title, capture.width, capture.height
                    );

                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_completed",
                        source_action = "SendFocusedWindowToAi",
                        trace_id = %trace_id,
                        window_title = %capture.window_title,
                        width = capture.width,
                        height = capture.height,
                        size_bytes,
                        used_fallback = capture.used_fallback,
                        "Focused window captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        if let Some(warning_message) = fallback_warning {
                            this.toast_manager.push(
                                components::toast::Toast::warning(warning_message, &this.theme)
                                    .duration_ms(Some(TOAST_WARNING_MS)),
                            );
                            cx.notify();
                        }

                        this.open_ai_window_after_already_hidden(
                            "SendFocusedWindowToAi",
                            &trace_id,
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            cx,
                        );
                    })
                    .ok();
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action = "SendFocusedWindowToAi",
                        trace_id = %trace_id,
                        error = %error,
                        "Failed to capture focused window for AI"
                    );
                    let message = format!("Failed to capture window: {}", error);
                    this.update(cx, |this, cx| {
                        this.show_error_toast(message, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn spawn_send_screen_area_to_ai_after_hide(&mut self, trace_id: &str, cx: &mut Context<Self>) {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "SendScreenAreaToAi",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling screen area capture for AI"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let capture_result = cx
                .background_executor()
                .spawn(async { platform::capture_screen_area() })
                .await;

            match capture_result {
                Ok(Some(capture)) => {
                    let size_bytes = capture.png_data.len();
                    if size_bytes > crate::prompts::chat::MAX_IMAGE_BYTES {
                        tracing::warn!(
                            category = "AI",
                            event = "ai_capture_rejected",
                            source_action = "SendScreenAreaToAi",
                            trace_id = %trace_id,
                            size_bytes,
                            max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                            "Rejecting screen area capture larger than 10 MB"
                        );
                        this.update(cx, |this, cx| {
                            this.show_error_toast(
                                "Screen area capture exceeds 10 MB limit".to_string(),
                                cx,
                            );
                        })
                        .ok();
                        return;
                    }

                    let base64_data = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &capture.png_data,
                    );
                    let message = format!(
                        "[Screen area captured: {}x{} pixels]\n\nPlease analyze this selected screen area.",
                        capture.width, capture.height
                    );

                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_completed",
                        source_action = "SendScreenAreaToAi",
                        trace_id = %trace_id,
                        width = capture.width,
                        height = capture.height,
                        size_bytes,
                        "Screen area captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        this.open_ai_window_after_already_hidden(
                            "SendScreenAreaToAi",
                            &trace_id,
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            cx,
                        );
                    })
                    .ok();
                }
                Ok(None) => {
                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_cancelled",
                        source_action = "SendScreenAreaToAi",
                        trace_id = %trace_id,
                        "Screen area selection cancelled by user"
                    );
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action = "SendScreenAreaToAi",
                        trace_id = %trace_id,
                        error = %error,
                        "Failed to capture screen area for AI"
                    );
                    let message = format!("Failed to capture screen area: {}", error);
                    this.update(cx, |this, cx| {
                        this.show_error_toast(message, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_capture_text_to_ai_after_already_hidden<C, F>(
        &mut self,
        source_action: &'static str,
        trace_id: &str,
        capture_kind: &'static str,
        capture_fn: C,
        format_fn: F,
        cx: &mut Context<Self>,
    ) where
        C: FnOnce() -> Result<DeferredAiCapturedText, String> + Send + 'static,
        F: FnOnce(String) -> String + Send + 'static,
    {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action,
            trace_id = %trace_id,
            capture_kind,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Scheduled deferred AI text capture"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let (result_tx, result_rx) =
                async_channel::bounded::<Result<DeferredAiCapturedText, String>>(1);

            let trace_id_for_thread = trace_id.clone();
            std::thread::spawn(move || {
                let started_at = std::time::Instant::now();
                let result = capture_fn();

                let (success, result_state) = match &result {
                    Ok(DeferredAiCapturedText::Ready(_)) => (true, "ready"),
                    Ok(DeferredAiCapturedText::Empty(_)) => (true, "empty"),
                    Err(_) => (false, "error"),
                };

                tracing::info!(
                    category = "AI",
                    event = "ai_capture_completed",
                    source_action,
                    trace_id = %trace_id_for_thread,
                    capture_kind,
                    result_state,
                    success,
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    "Deferred AI text capture finished"
                );

                let _ = result_tx.send_blocking(result);
            });

            let Ok(result) = result_rx.recv().await else {
                return;
            };

            let _ = this.update(cx, |this, cx| match result {
                Ok(DeferredAiCapturedText::Ready(captured)) => {
                    this.open_ai_window_after_already_hidden(
                        source_action,
                        &trace_id,
                        DeferredAiWindowAction::SetInput {
                            text: format_fn(captured),
                            submit: false,
                        },
                        cx,
                    );
                }
                Ok(DeferredAiCapturedText::Empty(message)) => {
                    this.toast_manager.push(
                        components::toast::Toast::info(message, &this.theme)
                            .duration_ms(Some(TOAST_INFO_MS)),
                    );
                    cx.notify();
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action,
                        trace_id = %trace_id,
                        capture_kind,
                        error = %error,
                        "Deferred AI text capture failed"
                    );
                    let message = format!("Failed to capture content for AI Chat: {}", error);
                    this.toast_manager.push(
                        components::toast::Toast::error(message, &this.theme)
                            .duration_ms(Some(TOAST_CRITICAL_MS)),
                    );
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn spawn_send_selected_text_to_ai_after_hide(
        &mut self,
        trace_id: &str,
        cx: &mut Context<Self>,
    ) {
        self.spawn_capture_text_to_ai_after_already_hidden(
            "SendSelectedTextToAi",
            trace_id,
            "selected_text",
            || {
                crate::selected_text::get_selected_text()
                    .map_err(|error| error.to_string())
                    .map(|text| {
                        let trimmed = text.trim().to_string();
                        if trimmed.is_empty() {
                            DeferredAiCapturedText::Empty(
                                "No text selected. Select some text first.".to_string(),
                            )
                        } else {
                            DeferredAiCapturedText::Ready(trimmed)
                        }
                    })
            },
            |text| {
                format!(
                    "I've selected the following text:\n\n```\n{}\n```\n\nPlease help me with this.",
                    text
                )
            },
            cx,
        );
    }

    fn spawn_send_browser_tab_to_ai_after_hide(
        &mut self,
        trace_id: &str,
        cx: &mut Context<Self>,
    ) {
        self.spawn_capture_text_to_ai_after_already_hidden(
            "SendBrowserTabToAi",
            trace_id,
            "browser_url",
            || {
                platform::get_focused_browser_tab_url()
                    .map_err(|error| error.to_string())
                    .map(|url| {
                        let trimmed = url.trim().to_string();
                        if trimmed.is_empty() {
                            DeferredAiCapturedText::Empty(
                                "No browser URL found in the frontmost tab.".to_string(),
                            )
                        } else {
                            DeferredAiCapturedText::Ready(trimmed)
                        }
                    })
            },
            |url| {
                format!(
                    "I'm looking at this webpage:\n\n{}\n\nPlease help me analyze or understand its content.",
                    url
                )
            },
            cx,
        );
    }

    fn spawn_generate_script_from_current_app_after_hide(
        &mut self,
        trace_id: String,
        query_override: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let fallback_query = query_override.unwrap_or_else(|| self.filter_text.clone());

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "GenerateScriptFromCurrentApp",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling context capture for script generation"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let snapshot_result = cx
                .background_executor()
                .spawn(async { crate::menu_bar::load_frontmost_menu_snapshot() })
                .await;

            let selected_text = match crate::selected_text::get_selected_text() {
                Ok(text) if !text.trim().is_empty() => Some(text),
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(
                        trace_id = %trace_id,
                        error = %error,
                        "ai_generate_script_from_current_app.selected_text_unavailable"
                    );
                    None
                }
            };

            let browser_url = match platform::get_focused_browser_tab_url() {
                Ok(url) if !url.trim().is_empty() => Some(url),
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(
                        trace_id = %trace_id,
                        error = %error,
                        "ai_generate_script_from_current_app.browser_url_unavailable"
                    );
                    None
                }
            };

            // Build prompt outside entity borrow so we can show window safely.
            let prompt_or_error = match snapshot_result {
                Ok(snapshot) => {
                    let user_request =
                        crate::menu_bar::current_app_commands::normalize_generate_script_from_current_app_request(
                            Some(fallback_query.as_str()),
                        );

                    let (prompt, receipt) =
                        crate::menu_bar::current_app_commands::build_generate_script_prompt_from_snapshot(
                            snapshot,
                            user_request,
                            selected_text.as_deref(),
                            browser_url.as_deref(),
                        );

                    tracing::info!(
                        trace_id = %trace_id,
                        app_name = %receipt.app_name,
                        bundle_id = %receipt.bundle_id,
                        total_menu_items = receipt.total_menu_items,
                        included_menu_items = receipt.included_menu_items,
                        included_user_request = receipt.included_user_request,
                        included_selected_text = receipt.included_selected_text,
                        included_browser_url = receipt.included_browser_url,
                        "ai_generate_script_from_current_app.prompt_ready"
                    );

                    Ok(prompt)
                }
                Err(error) => Err(error),
            };

            match prompt_or_error {
                Ok(prompt) => {
                    // Platform calls — trigger macOS delegate callbacks.
                    // Safe here: no AppCell borrow is active.
                    script_kit_gpui::set_main_window_visible(true);
                    tracing::info!(
                        trace_id = %trace_id,
                        "ai_generate_script_from_current_app.showing_window"
                    );
                    crate::platform::show_main_window_without_activation();

                    // GPUI state changes inside entity borrow.
                    let _ = this.update(cx, |app, cx| {
                        app.dispatch_ai_script_generation_from_query(prompt, cx);
                    });
                }
                Err(error) => {
                    let _ = this.update(cx, |app, cx| {
                        let message = format!("Failed to capture current app context: {}", error);
                        app.show_error_toast(message.clone(), cx);
                        tracing::error!(
                            trace_id = %trace_id,
                            error = %error,
                            "ai_generate_script_from_current_app.capture_failed"
                        );
                    });
                }
            }
        })
        .detach();
    }

    /// Like `spawn_generate_script_from_current_app_after_hide`, but reuses an
    /// already-built recipe instead of recapturing live context after hide.
    ///
    /// This eliminates prompt drift: the prompt copied in the recipe is
    /// byte-for-byte the prompt sent to the AI generation path.
    fn spawn_generate_script_from_recipe_after_hide(
        &mut self,
        trace_id: String,
        recipe: crate::menu_bar::current_app_commands::CurrentAppCommandRecipe,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            category = "AI",
            event = "ai_recipe_generation_scheduled",
            source_action = "TurnThisIntoCommand",
            trace_id = %trace_id,
            recipe_prompt_bytes = recipe.prompt.len(),
            recipe_bundle_id = %recipe.prompt_receipt.bundle_id,
            recipe_included_selected_text = recipe.prompt_receipt.included_selected_text,
            recipe_included_browser_url = recipe.prompt_receipt.included_browser_url,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling recipe-based script generation (no recapture)"
        );

        platform::defer_hide_main_window(cx);

        let prompt =
            crate::menu_bar::current_app_commands::build_generated_script_prompt_from_recipe(
                &recipe,
            );

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            tracing::info!(
                trace_id = %trace_id,
                recipe_prompt_bytes = prompt.len(),
                recipe_bundle_id = %recipe.prompt_receipt.bundle_id,
                recipe_included_selected_text = recipe.prompt_receipt.included_selected_text,
                recipe_included_browser_url = recipe.prompt_receipt.included_browser_url,
                "ai_generate_script_from_recipe.prompt_ready"
            );

            // Platform calls — trigger macOS delegate callbacks.
            // Safe here: no AppCell borrow is active.
            script_kit_gpui::set_main_window_visible(true);
            tracing::info!(
                trace_id = %trace_id,
                "ai_generate_script_from_recipe.showing_window"
            );
            crate::platform::show_main_window_without_activation();

            // GPUI state changes inside entity borrow.
            let _ = this.update(cx, |app, cx| {
                app.dispatch_ai_script_generation_from_query(prompt, cx);
            });
        })
        .detach();
    }

    fn system_action_feedback_message(
        &self,
        action_type: &builtins::SystemActionType,
    ) -> Option<String> {
        let dark_mode_enabled = if matches!(action_type, builtins::SystemActionType::ToggleDarkMode)
        {
            system_actions::is_dark_mode().ok()
        } else {
            None
        };

        builtins::system_action_hud_message(*action_type, dark_mode_enabled)
    }

    /// Shared dispatch for system actions — used by both the normal and confirmed paths.
    /// Maps a `SystemActionType` to its implementation, handles special cases
    /// (TestConfirmation, QuitScriptKit), and routes the result through
    /// `handle_system_action_result`.
    /// Structured outcome logger for builtin execution paths.
    ///
    /// Emits a single log line with all fields needed for machine consumption
    /// and human debugging: builtin_id, trace_id, surface, handler, status,
    /// error_code, and duration_ms.
    fn log_builtin_outcome(
        builtin_id: &str,
        dctx: &crate::action_helpers::DispatchContext,
        handler: &str,
        outcome: &crate::action_helpers::DispatchOutcome,
        start: &std::time::Instant,
    ) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let trace_id = outcome
            .trace_id
            .as_deref()
            .unwrap_or(dctx.trace_id.as_str());

        match outcome.status {
            crate::action_helpers::ActionOutcomeStatus::Error => {
                tracing::error!(
                    category = "BUILTIN",
                    builtin_id = %builtin_id,
                    trace_id = %trace_id,
                    surface = %dctx.surface,
                    handler,
                    status = %outcome.status,
                    error_code = outcome.error_code,
                    duration_ms,
                    detail = ?outcome.detail,
                    "Builtin execution finished"
                );
            }
            _ => {
                tracing::info!(
                    category = "BUILTIN",
                    builtin_id = %builtin_id,
                    trace_id = %trace_id,
                    surface = %dctx.surface,
                    handler,
                    status = %outcome.status,
                    error_code = outcome.error_code,
                    duration_ms,
                    detail = ?outcome.detail,
                    "Builtin execution finished"
                );
            }
        }
    }

    /// Build a success outcome carrying the dispatch context's trace_id.
    fn builtin_success(
        dctx: &crate::action_helpers::DispatchContext,
        detail: impl Into<String>,
    ) -> crate::action_helpers::DispatchOutcome {
        crate::action_helpers::DispatchOutcome::success()
            .with_trace_id(dctx.trace_id.clone())
            .with_detail(detail)
    }

    /// Build an error outcome carrying the dispatch context's trace_id.
    fn builtin_error(
        dctx: &crate::action_helpers::DispatchContext,
        code: &'static str,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> crate::action_helpers::DispatchOutcome {
        crate::action_helpers::DispatchOutcome::error(code, message)
            .with_trace_id(dctx.trace_id.clone())
            .with_detail(detail)
    }

    fn dispatch_system_action(
        &mut self,
        action_type: &builtins::SystemActionType,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let start = std::time::Instant::now();

        tracing::info!(
            category = "BUILTIN",
            builtin_id = %dctx.action_id,
            trace_id = %dctx.trace_id,
            surface = %dctx.surface,
            action_type = ?action_type,
            status = "dispatched",
            "system_action_dispatch"
        );

        #[cfg(target_os = "macos")]
        {
            use builtins::SystemActionType;

            let result = match action_type {
                // Power management
                SystemActionType::EmptyTrash => system_actions::empty_trash(),
                SystemActionType::LockScreen => system_actions::lock_screen(),
                SystemActionType::Sleep => system_actions::sleep(),
                SystemActionType::Restart => system_actions::restart(),
                SystemActionType::ShutDown => system_actions::shut_down(),
                SystemActionType::LogOut => system_actions::log_out(),

                // UI controls
                SystemActionType::ToggleDarkMode => system_actions::toggle_dark_mode(),
                SystemActionType::ShowDesktop => system_actions::show_desktop(),
                SystemActionType::MissionControl => system_actions::mission_control(),
                SystemActionType::Launchpad => system_actions::launchpad(),
                SystemActionType::ForceQuitApps => system_actions::force_quit_apps(),

                // Volume controls (preset levels)
                SystemActionType::Volume0 => system_actions::set_volume(0),
                SystemActionType::Volume25 => system_actions::set_volume(25),
                SystemActionType::Volume50 => system_actions::set_volume(50),
                SystemActionType::Volume75 => system_actions::set_volume(75),
                SystemActionType::Volume100 => system_actions::set_volume(100),
                SystemActionType::VolumeMute => system_actions::volume_mute(),

                // Dev/test actions
                #[cfg(debug_assertions)]
                SystemActionType::TestConfirmation => {
                    self.toast_manager.push(
                        components::toast::Toast::success("Confirmation test passed!", &self.theme)
                            .duration_ms(Some(TOAST_SUCCESS_MS)),
                    );
                    cx.notify();
                    return Self::builtin_success(dctx, "system_action_test_confirmation");
                }

                // App control
                SystemActionType::QuitScriptKit => {
                    Self::prepare_script_kit_shutdown();
                    cx.quit();
                    return Self::builtin_success(dctx, "quit_script_kit");
                }

                // System utilities
                SystemActionType::ToggleDoNotDisturb => system_actions::toggle_do_not_disturb(),
                SystemActionType::StartScreenSaver => system_actions::start_screen_saver(),

                // System Preferences
                SystemActionType::OpenSystemPreferences => {
                    system_actions::open_system_preferences_main()
                }
                SystemActionType::OpenPrivacySettings => system_actions::open_privacy_settings(),
                SystemActionType::OpenDisplaySettings => system_actions::open_display_settings(),
                SystemActionType::OpenSoundSettings => system_actions::open_sound_settings(),
                SystemActionType::OpenNetworkSettings => system_actions::open_network_settings(),
                SystemActionType::OpenKeyboardSettings => system_actions::open_keyboard_settings(),
                SystemActionType::OpenBluetoothSettings => {
                    system_actions::open_bluetooth_settings()
                }
                SystemActionType::OpenNotificationsSettings => {
                    system_actions::open_notifications_settings()
                }
            };

            self.handle_system_action_result(
                result,
                action_type,
                dctx,
                start.elapsed(),
                cx,
            )
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = action_type;
            self.show_unsupported_platform_toast("System actions", cx);
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_UNSUPPORTED_PLATFORM,
                "System actions are not supported on this platform",
                "system_action_unsupported_platform",
            )
        }
    }

    /// Shared result handler for system actions — shows HUD on success, Toast on error.
    /// Returns a `DispatchOutcome` for structured logging at the call boundary.
    fn handle_system_action_result(
        &mut self,
        result: Result<(), String>,
        action_type: &builtins::SystemActionType,
        dctx: &crate::action_helpers::DispatchContext,
        elapsed: std::time::Duration,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let duration_ms = elapsed.as_millis() as u64;
        match result {
            Ok(()) => {
                tracing::info!(
                    category = "BUILTIN",
                    builtin_id = %dctx.action_id,
                    trace_id = %dctx.trace_id,
                    surface = %dctx.surface,
                    action_type = ?action_type,
                    status = "success",
                    duration_ms,
                    "system_action_dispatch"
                );
                if let Some(message) = self.system_action_feedback_message(action_type) {
                    cx.notify();
                    self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                    self.hide_main_and_reset(cx);
                } else {
                    self.close_and_reset_window(cx);
                }
                Self::builtin_success(dctx, format!("system_action::{action_type:?}"))
            }
            Err(error) => {
                tracing::error!(
                    category = "BUILTIN",
                    builtin_id = %dctx.action_id,
                    trace_id = %dctx.trace_id,
                    surface = %dctx.surface,
                    action_type = ?action_type,
                    status = "error",
                    error_code = crate::action_helpers::ERROR_LAUNCH_FAILED,
                    duration_ms,
                    error = %error,
                    "system_action_dispatch"
                );
                self.show_error_toast(format!("System action failed: {}", error), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_LAUNCH_FAILED,
                    format!("System action failed: {}", error),
                    format!("system_action::{action_type:?}; error={error}"),
                )
            }
        }
    }

    fn prepare_script_kit_shutdown() {
        tracing::info!(
            category = "UI",
            event = "prepare_script_kit_shutdown",
            "prepare_script_kit_shutdown"
        );
        PROCESS_MANAGER.kill_all_processes();
        PROCESS_MANAGER.remove_main_pid();
    }

    fn quit_script_kit_confirm_options() -> crate::confirm::ParentConfirmOptions {
        crate::confirm::ParentConfirmOptions::destructive(
            "Quit Script Kit",
            "Quit Script Kit and stop all running processes?",
            "Quit",
        )
    }

    fn builtin_confirmation_options(
        entry_id: &str,
        entry_name: &str,
    ) -> crate::confirm::ParentConfirmOptions {
        match entry_id {
            "builtin-quit-script-kit" => Self::quit_script_kit_confirm_options(),
            "builtin-shut-down" => crate::confirm::ParentConfirmOptions::destructive(
                "Shut Down Mac",
                "Shut down this Mac now?",
                "Shut Down",
            ),
            "builtin-restart" => crate::confirm::ParentConfirmOptions::destructive(
                "Restart Mac",
                "Restart this Mac now?",
                "Restart",
            ),
            "builtin-log-out" => crate::confirm::ParentConfirmOptions::destructive(
                "Log Out",
                "Log out of the current macOS session?",
                "Log Out",
            ),
            "builtin-empty-trash" => crate::confirm::ParentConfirmOptions::destructive(
                "Empty Trash",
                "Empty Trash now? This cannot be undone.",
                "Empty Trash",
            ),
            "builtin-sleep" => crate::confirm::ParentConfirmOptions {
                title: "Sleep Mac".into(),
                body: "Put this Mac to sleep now?".into(),
                confirm_text: "Sleep".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
            "builtin-force-quit" => crate::confirm::ParentConfirmOptions::destructive(
                "Force Quit Apps",
                "Open Force Quit Apps?",
                "Force Quit",
            ),
            "builtin-stop-all-processes" => crate::confirm::ParentConfirmOptions::destructive(
                "Stop All Processes",
                "Stop all running Script Kit processes?",
                "Stop All",
            ),
            "builtin-clear-suggested" => crate::confirm::ParentConfirmOptions::destructive(
                "Clear Suggested",
                "Clear suggested items and reset their ranking data?",
                "Clear Suggested",
            ),
            "builtin-test-confirmation" => crate::confirm::ParentConfirmOptions {
                title: "Test Confirmation".into(),
                body: "Open the confirmation test action?".into(),
                confirm_text: "Run Test".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
            _ => crate::confirm::ParentConfirmOptions {
                title: "Confirm".into(),
                body: format!("Are you sure you want to {}?", entry_name).into(),
                confirm_text: "Continue".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
        }
    }

    fn execute_builtin(&mut self, entry: &builtins::BuiltInEntry, cx: &mut Context<Self>) {
        self.execute_builtin_with_query(entry, None, cx);
    }

    fn execute_builtin_with_query(
        &mut self,
        entry: &builtins::BuiltInEntry,
        query_override: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let start = std::time::Instant::now();
        let dctx = crate::action_helpers::DispatchContext::for_builtin(&entry.id);

        tracing::info!(
            category = "BUILTIN",
            builtin_id = %entry.id,
            builtin_name = %entry.name,
            trace_id = %dctx.trace_id,
            surface = %dctx.surface,
            "Builtin execution started"
        );

        // Clear any stale actions popup from previous view
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Check if this command requires confirmation - open modal if so
        if self.config.requires_confirmation(&entry.id) {
            let confirmation_start = std::time::Instant::now();
            let entry_id = entry.id.clone();
            let query_owned = query_override.map(|s| s.to_string());
            let dctx_owned = dctx.clone();
            let confirm_options = Self::builtin_confirmation_options(&entry.id, &entry.name);

            // Spawn a task to show confirmation dialog via shared parent dialog helper
            cx.spawn(async move |this, cx| {
                match crate::confirm::confirm_with_parent_dialog(cx, confirm_options, &dctx_owned.trace_id).await
                {
                    Ok(true) => {
                        let _ = this.update(cx, |this, cx| {
                            this.handle_builtin_confirmation(
                                entry_id,
                                true,
                                query_owned,
                                &dctx_owned,
                                cx,
                            );
                        });
                    }
                    Ok(false) => {
                        let outcome = crate::action_helpers::DispatchOutcome::cancelled()
                            .with_trace_id(dctx_owned.trace_id.clone())
                            .with_detail("builtin_confirmation_cancelled");
                        let _ = this.update(cx, |_, _| {
                            Self::log_builtin_outcome(
                                &entry_id,
                                &dctx_owned,
                                "confirmation_gate",
                                &outcome,
                                &confirmation_start,
                            );
                        });
                    }
                    Err(e) => {
                        let _ = this.update(cx, |this, cx| {
                            tracing::error!(
                                builtin_id = %entry_id,
                                trace_id = %dctx_owned.trace_id,
                                error = %e,
                                "failed to open confirmation modal"
                            );
                            this.show_error_toast_with_code(
                                "Failed to open confirmation dialog",
                                Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                cx,
                            );
                            let outcome = Self::builtin_error(
                                &dctx_owned,
                                crate::action_helpers::ERROR_MODAL_FAILED,
                                "Failed to open confirmation dialog",
                                format!("confirmation_modal_error={e}"),
                            );
                            Self::log_builtin_outcome(
                                &entry_id,
                                &dctx_owned,
                                "confirmation_gate",
                                &outcome,
                                &confirmation_start,
                            );
                        });
                    }
                }
            })
            .detach();

            tracing::info!(
                category = "BUILTIN",
                trace_id = %dctx.trace_id,
                builtin_id = %entry.id,
                status = "awaiting_confirmation",
                duration_ms = start.elapsed().as_millis() as u64,
                "Builtin execution deferred to confirmation modal"
            );
            return; // Wait for modal callback
        }

        // All builtins now return DispatchOutcome — system actions are handled
        // inside execute_builtin_inner as well.
        let outcome = self.execute_builtin_inner(entry, query_override, &dctx, cx);

        Self::log_builtin_outcome(&entry.id, &dctx, "builtin_execution", &outcome, &start);
    }

    /// Open a filterable main-window builtin view with a consistent UX contract.
    ///
    /// Every filterable builtin should go through this helper so that focus,
    /// placeholder, filter reset, hover clearing, resize, and opened-from-menu
    /// state are always set the same way.
    fn open_builtin_filterable_view(
        &mut self,
        view: AppView,
        placeholder: &str,
        cx: &mut Context<Self>,
    ) {
        self.filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some(placeholder.to_string());
        self.current_view = view;
        self.hovered_index = None;
        self.opened_from_main_menu = true;
        resize_to_view_sync(ViewType::ScriptList, 0);
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    fn open_mini_main_window(&mut self, cx: &mut Context<Self>) {
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search scripts, apps, and commands…".to_string());
        self.current_view = AppView::ScriptList;
        self.main_window_mode = MainWindowMode::Mini;
        self.hovered_index = None;
        self.selected_index = 0;
        self.opened_from_main_menu = true;
        self.invalidate_grouped_cache();
        self.sync_list_state();
        let (grouped_items, _) = self.get_grouped_results_cached();
        let item_count = grouped_items.len();
        // Skip section headers — select first actual item so cmd+k works immediately
        let first_selectable = crate::list_item::GroupedListState::from_items(&grouped_items)
            .first_selectable;
        self.selected_index = first_selectable;
        tracing::info!(
            event = "open_mini_main_window",
            item_count = item_count,
            selected_index = self.selected_index,
            first_selectable = first_selectable,
            grouped_cache_key = %self.grouped_cache_key,
            computed_filter = %self.computed_filter_text,
            filter_text = %self.filter_text,
            pending_filter_sync = self.pending_filter_sync,
            "open_mini_main_window: items={}, selected={}",
            item_count,
            self.selected_index,
        );
        resize_to_view_sync(ViewType::MiniMainWindow, item_count);
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    /// Open a filterable builtin view with an initial filter value.
    ///
    /// Same UX contract as [`open_builtin_filterable_view`] but pre-fills the
    /// filter input instead of clearing it. Used by `DoInCurrentApp` to open
    /// the command palette with the user's query already typed.
    fn open_builtin_filterable_view_with_filter(
        &mut self,
        view: AppView,
        filter: &str,
        placeholder: &str,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            view = ?view,
            filter = %filter,
            placeholder = %placeholder,
            "open_builtin_filterable_view_with_filter — setting current_view and filter"
        );
        self.filter_text = filter.to_string();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some(placeholder.to_string());
        self.current_view = view;
        self.hovered_index = None;
        self.opened_from_main_menu = true;
        resize_to_view_sync(ViewType::ScriptList, 0);
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    /// Inner builtin executor — runs the actual action logic.
    /// Called from both the normal path (after confirmation check) and the
    /// confirmed path (after modal approval), ensuring a single implementation.
    ///
    /// Returns a `DispatchOutcome` so callers can log the real result instead
    /// of a synthetic success.
    fn execute_builtin_inner(
        &mut self,
        entry: &builtins::BuiltInEntry,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Clipboard History"
                );
                self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                self.focused_clipboard_entry_id = self
                    .cached_clipboard_entries
                    .first()
                    .map(|entry| entry.id.clone());
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    count = self.cached_clipboard_entries.len(),
                    "Loaded clipboard entries"
                );

                self.open_builtin_filterable_view(
                    AppView::ClipboardHistoryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search clipboard history...",
                    cx,
                );

                Self::builtin_success(dctx, "open_clipboard_history")
            }
            builtins::BuiltInFeature::PasteSequentially => {
                tracing::info!(
                    action = "paste_sequential",
                    event = "trigger",
                    trace_id = %dctx.trace_id,
                    "Paste Sequentially triggered"
                );
                match clipboard_history::advance_paste_sequence(&mut self.paste_sequential_state) {
                    clipboard_history::PasteSequentialOutcome::Pasted(entry_id) => {
                        tracing::info!(
                            action = "paste_sequential",
                            event = "paste_entry",
                            entry_id = %entry_id,
                            trace_id = %dctx.trace_id,
                            "Enqueuing sequential paste via serialized worker"
                        );
                        match clipboard_history::enqueue_sequential_paste(entry_id) {
                            Ok(()) => {
                                clipboard_history::commit_paste_sequence(
                                    &mut self.paste_sequential_state,
                                );
                                self.hide_main_and_reset(cx);
                                Self::builtin_success(dctx, "paste_sequential")
                            }
                            Err(clipboard_history::EnqueuePasteError::WorkerDisconnected) => {
                                tracing::error!(
                                    action = "paste_sequential",
                                    event = "enqueue_failed",
                                    error_code = "worker_disconnected",
                                    trace_id = %dctx.trace_id,
                                    "Paste worker is not running"
                                );
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        "Paste worker crashed — restart Script Kit",
                                        &self.theme,
                                    )
                                    .duration_ms(Some(TOAST_CRITICAL_MS)),
                                );
                                cx.notify();
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    "Paste worker crashed",
                                    "paste_sequential_worker_disconnected",
                                )
                            }
                        }
                    }
                    clipboard_history::PasteSequentialOutcome::Exhausted => {
                        tracing::info!(
                            action = "paste_sequential",
                            event = "sequence_exhausted",
                            trace_id = %dctx.trace_id,
                            "Sequential paste exhausted all entries"
                        );
                        self.show_hud("Sequence complete".to_string(), Some(HUD_SHORT_MS), cx);
                        Self::builtin_success(dctx, "paste_sequential_exhausted")
                    }
                    clipboard_history::PasteSequentialOutcome::Empty => {
                        tracing::info!(
                            action = "paste_sequential",
                            event = "history_empty",
                            trace_id = %dctx.trace_id,
                            "No clipboard history available for sequential paste"
                        );
                        self.show_hud("No clipboard history".to_string(), Some(HUD_SHORT_MS), cx);
                        Self::builtin_success(dctx, "paste_sequential_empty")
                    }
                }
            }
            builtins::BuiltInFeature::Favorites => {
                tracing::info!(
                    category = "BUILTIN",
                    action = "open_favorites_view",
                    trace_id = %dctx.trace_id,
                    "Opening Favorites browse view"
                );

                self.open_builtin_filterable_view(
                    AppView::FavoritesBrowseView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search favorites...",
                    cx,
                );

                Self::builtin_success(dctx, "open_favorites_view")
            }
            builtins::BuiltInFeature::AppLauncher => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening App Launcher"
                );
                self.apps = app_launcher::scan_applications().clone();
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    count = self.apps.len(),
                    "Loaded applications"
                );
                self.invalidate_filter_cache();
                self.invalidate_grouped_cache();
                self.sync_list_state();

                self.open_builtin_filterable_view(
                    AppView::AppLauncherView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search applications...",
                    cx,
                );

                Self::builtin_success(dctx, "open_app_launcher")
            }
            builtins::BuiltInFeature::App(app_name) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    app = %app_name,
                    "Launching app"
                );
                let apps = app_launcher::scan_applications();
                if let Some(app) = apps.iter().find(|a| a.name == *app_name) {
                    if let Err(e) = app_launcher::launch_application(app) {
                        let message = format!("Failed to launch {}: {}", app_name, e);
                        self.show_error_toast(message.clone(), cx);
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_LAUNCH_FAILED,
                            message,
                            format!("launch_app::{app_name}"),
                        )
                    } else {
                        self.close_and_reset_window(cx);
                        Self::builtin_success(dctx, format!("launch_app::{app_name}"))
                    }
                } else {
                    let message = format!("App not found: {}", app_name);
                    self.show_error_toast(message.clone(), cx);
                    Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        message,
                        format!("launch_app_not_found::{app_name}"),
                    )
                }
            }
            builtins::BuiltInFeature::WindowSwitcher => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Window Switcher"
                );
                match window_control::list_windows() {
                    Ok(windows) => {
                        tracing::info!(
                            category = "BUILTIN",
                            trace_id = %dctx.trace_id,
                            count = windows.len(),
                            "Loaded windows"
                        );
                        self.cached_windows = windows;

                        self.open_builtin_filterable_view(
                            AppView::WindowSwitcherView {
                                filter: String::new(),
                                selected_index: 0,
                            },
                            "Search windows...",
                            cx,
                        );

                        Self::builtin_success(dctx, "open_window_switcher")
                    }
                    Err(e) => {
                        let message = format!("Failed to list windows: {}", e);
                        self.show_error_toast(message.clone(), cx);
                        cx.notify();
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            message,
                            "open_window_switcher_failed",
                        )
                    }
                }
            }
            builtins::BuiltInFeature::DesignGallery => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Design Gallery"
                );

                self.open_builtin_filterable_view(
                    AppView::DesignGalleryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search designs...",
                    cx,
                );

                Self::builtin_success(dctx, "open_design_gallery")
            }
            #[cfg(feature = "storybook")]
            builtins::BuiltInFeature::DesignExplorer => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Design Explorer"
                );

                let explorer = cx.new(|cx| {
                    let mut browser = script_kit_gpui::storybook::StoryBrowser::new(cx);
                    browser.configure_for_design_explorer(
                        Some(script_kit_gpui::storybook::StorySurface::Footer),
                    );
                    browser
                });

                self.current_view = AppView::DesignExplorerView { entity: explorer };
                cx.notify();

                Self::builtin_success(dctx, "open_design_explorer")
            }
            builtins::BuiltInFeature::AiChat => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening AI Chat window"
                );
                self.open_ai_window_after_main_hide(
                    "AiChat",
                    &dctx.trace_id,
                    DeferredAiWindowAction::OpenOnly,
                    cx,
                );

                Self::builtin_success(dctx, "open_ai_chat_dispatched")
            }
            builtins::BuiltInFeature::Notes => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Notes window"
                );
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::defer_hide_main_window(cx);
                if let Err(e) = notes::open_notes_window(cx) {
                    let message = format!("Failed to open Notes: {}", e);
                    self.show_error_toast(message.clone(), cx);
                    Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_LAUNCH_FAILED,
                        message,
                        "open_notes_failed",
                    )
                } else {
                    Self::builtin_success(dctx, "open_notes")
                }
            }
            builtins::BuiltInFeature::EmojiPicker => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Emoji Picker"
                );
                // EmojiPicker has an extra selected_category field, so use the
                // shared helper for the common state and then set the view.
                self.open_builtin_filterable_view(
                    AppView::EmojiPickerView {
                        filter: String::new(),
                        selected_index: 0,
                        selected_category: None,
                    },
                    "Search Emoji & Symbols...",
                    cx,
                );

                Self::builtin_success(dctx, "open_emoji_picker")
            }
            builtins::BuiltInFeature::MenuBarAction(action) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    bundle_id = %action.bundle_id,
                    "Executing menu bar action"
                );
                #[cfg(target_os = "macos")]
                {
                    match script_kit_gpui::menu_executor::execute_menu_action(
                        &action.bundle_id,
                        &action.menu_path,
                    ) {
                        Ok(()) => {
                            self.close_and_reset_window(cx);
                            Self::builtin_success(dctx, "menu_bar_action")
                        }
                        Err(e) => {
                            let message = format!("Menu action failed: {}", e);
                            self.show_error_toast(message.clone(), cx);
                            Self::builtin_error(
                                dctx,
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                message,
                                "menu_bar_action_failed",
                            )
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.show_unsupported_platform_toast("Menu bar actions", cx);
                    Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_UNSUPPORTED_PLATFORM,
                        "Menu bar actions only supported on macOS",
                        "menu_bar_action_unsupported",
                    )
                }
            }

            // =========================================================================
            // System Actions
            // =========================================================================
            builtins::BuiltInFeature::SystemAction(action_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    action_type = ?action_type,
                    "Executing system action via inner path"
                );
                self.dispatch_system_action(action_type, dctx, cx)
            }

            // NOTE: Window Actions removed - now handled by window-management extension
            // SDK tileWindow() still works via protocol messages in execute_script.rs

            // =========================================================================
            // Notes Commands
            // =========================================================================
            builtins::BuiltInFeature::NotesCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    notes_command = ?cmd_type,
                    "Executing notes command"
                );

                use builtins::NotesCommandType;

                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::defer_hide_main_window(cx);

                let result = match cmd_type {
                    NotesCommandType::OpenNotes
                    | NotesCommandType::NewNote
                    | NotesCommandType::SearchNotes => notes::open_notes_window(cx),
                    NotesCommandType::QuickCapture => notes::quick_capture(cx),
                };

                if let Err(e) = result {
                    let message = format!("Notes command failed: {}", e);
                    self.show_error_toast(message.clone(), cx);
                    Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_LAUNCH_FAILED,
                        message,
                        format!("notes_command_failed::{cmd_type:?}"),
                    )
                } else {
                    Self::builtin_success(dctx, format!("notes_command::{cmd_type:?}"))
                }
            }

            // =========================================================================
            // AI Commands
            // =========================================================================
            builtins::BuiltInFeature::AiCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    ai_command = ?cmd_type,
                    "Executing AI command"
                );

                use builtins::AiCommandType;

                let keeps_main_window_visible = ai_command_keeps_main_window_visible(cmd_type);
                let uses_hide_then_capture_flow = ai_command_uses_hide_then_capture_flow(cmd_type);
                if !keeps_main_window_visible {
                    script_kit_gpui::set_main_window_visible(false);
                    self.reset_to_script_list(cx);
                    if uses_hide_then_capture_flow {
                        tracing::debug!(
                            action = ?cmd_type,
                            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
                            "Deferring main window hide to async capture flow"
                        );
                    } else {
                        platform::defer_hide_main_window(cx);
                    }
                }

                match cmd_type {
                    AiCommandType::OpenAi | AiCommandType::NewConversation => {
                        self.open_ai_window_after_already_hidden(
                            &format!("ai_command::{cmd_type:?}"),
                            &dctx.trace_id,
                            DeferredAiWindowAction::OpenOnly,
                            cx,
                        );
                        Self::builtin_success(dctx, format!("ai_command::{cmd_type:?}"))
                    }

                    AiCommandType::MiniAi => {
                        let source_action = format!("ai_command::{cmd_type:?}");
                        let trace_id = dctx.trace_id.to_string();

                        tracing::info!(
                            category = "AI",
                            event = "ai_handoff_defer_open_start",
                            source_action = %source_action,
                            trace_id = %trace_id,
                            deferred_action = "open_only",
                            "Opening mini AI window after main window already hidden"
                        );

                        cx.spawn(async move |this, cx| {
                            cx.background_executor()
                                .timer(std::time::Duration::from_millis(1))
                                .await;

                            let started_at = std::time::Instant::now();

                            let open_result = cx.update(|cx| {
                                crate::ai::open_mini_ai_window_from("builtin_mini_ai", cx)
                                    .map_err(|error| error.to_string())?;
                                Ok::<(), String>(())
                            });

                            if open_result.is_ok() {
                                let ready_now = cx.update(ai::is_ai_window_ready);
                                if !ready_now {
                                    cx.background_executor()
                                        .timer(std::time::Duration::from_millis(16))
                                        .await;
                                }
                            }

                            let handoff_result = open_result.and_then(|()| {
                                cx.update(|cx| {
                                    if !ai::is_ai_window_ready(cx) {
                                        return Err("AI window not ready after open".to_string());
                                    }
                                    DeferredAiWindowAction::OpenOnly.apply(cx)
                                })
                            });

                            match handoff_result {
                                Ok(apply_stage) => {
                                    let _ = this.update(cx, |_this, cx| {
                                        tracing::info!(
                                            category = "AI",
                                            event = "ai_handoff_defer_open_success",
                                            source_action = %source_action,
                                            trace_id = %trace_id,
                                            deferred_action = "open_only",
                                            apply_stage,
                                            duration_ms = started_at.elapsed().as_millis() as u64,
                                            "Mini AI handoff completed"
                                        );
                                        cx.notify();
                                    });
                                }
                                Err(error) => {
                                    let _ = this.update(cx, |this, cx| {
                                        tracing::error!(
                                            category = "AI",
                                            event = "ai_handoff_defer_open_failed",
                                            source_action = %source_action,
                                            trace_id = %trace_id,
                                            deferred_action = "open_only",
                                            error = %error,
                                            duration_ms = started_at.elapsed().as_millis() as u64,
                                            "Failed to open mini AI window after hiding main window"
                                        );
                                        this.show_error_toast(
                                            format!("Failed to send to AI Chat: {}", error),
                                            cx,
                                        );
                                    });
                                }
                            }
                        })
                        .detach();

                        Self::builtin_success(dctx, format!("ai_command::{cmd_type:?}"))
                    }

                    AiCommandType::ClearConversation => {
                        match ai::clear_all_chats() {
                            Ok(()) => {
                                ai::close_ai_window(cx);
                                self.open_ai_window_after_already_hidden(
                                    "ClearConversation",
                                    &dctx.trace_id,
                                    DeferredAiWindowAction::OpenOnly,
                                    cx,
                                );
                                Self::builtin_success(dctx, "ai_clear_conversation")
                            }
                            Err(e) => {
                                let message = format!("Failed to clear AI conversations: {}", e);
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "ai_clear_conversation_failed",
                                )
                            }
                        }
                    }

                    AiCommandType::GenerateScript => {
                        let query = query_override.unwrap_or(&self.filter_text).to_string();
                        self.dispatch_ai_script_generation_from_query(query, cx);
                        Self::builtin_success(dctx, "ai_generate_script_dispatched")
                    }

                    AiCommandType::GenerateScriptFromCurrentApp => {
                        self.spawn_generate_script_from_current_app_after_hide(
                            dctx.trace_id.to_string(),
                            query_override.map(|s| s.to_string()),
                            cx,
                        );
                        Self::builtin_success(
                            dctx,
                            "ai_generate_script_from_current_app_scheduled",
                        )
                    }

                    AiCommandType::SendScreenToAi => {
                        self.spawn_send_screen_to_ai_after_hide(&dctx.trace_id, cx);
                        Self::builtin_success(dctx, "ai_send_screen_scheduled")
                    }

                    AiCommandType::SendFocusedWindowToAi => {
                        self.spawn_send_focused_window_to_ai_after_hide(&dctx.trace_id, cx);
                        Self::builtin_success(dctx, "ai_send_focused_window_scheduled")
                    }

                    AiCommandType::SendSelectedTextToAi => {
                        self.spawn_send_selected_text_to_ai_after_hide(&dctx.trace_id, cx);
                        Self::builtin_success(dctx, "ai_send_selected_text_scheduled")
                    }

                    AiCommandType::SendBrowserTabToAi => {
                        self.spawn_send_browser_tab_to_ai_after_hide(&dctx.trace_id, cx);
                        Self::builtin_success(dctx, "ai_send_browser_tab_scheduled")
                    }

                    AiCommandType::SendScreenAreaToAi => {
                        self.spawn_send_screen_area_to_ai_after_hide(&dctx.trace_id, cx);
                        Self::builtin_success(dctx, "ai_send_screen_area_scheduled")
                    }

                    AiCommandType::CreateAiPreset => {
                        tracing::info!(
                            action = "create_ai_preset",
                            trace_id = %dctx.trace_id,
                            "Opening create AI preset form"
                        );
                        self.current_view = AppView::CreateAiPresetView {
                            name: String::new(),
                            system_prompt: String::new(),
                            model: String::new(),
                            active_field: 0,
                        };
                        self.pending_focus = Some(FocusTarget::AppRoot);
                        cx.notify();
                        Self::builtin_success(dctx, "ai_create_preset")
                    }

                    AiCommandType::ImportAiPresets => {
                        tracing::info!(
                            action = "import_ai_presets",
                            "Opening file picker for AI preset import"
                        );
                        let rx = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: false,
                            prompt: Some("Select AI presets JSON file".into()),
                            allowed_extensions: vec!["json".into()],
                        });

                        cx.spawn(async move |this, cx| {
                            match rx.await {
                                Ok(Ok(Some(paths))) => {
                                    if let Some(path) = paths.first() {
                                        // Validate file contents before importing
                                        let import_result = cx
                                            .background_executor()
                                            .spawn({
                                                let path = path.clone();
                                                async move {
                                                    let contents = std::fs::read_to_string(&path)
                                                        .map_err(|e| {
                                                        format!("Failed to read file: {}", e)
                                                    })?;
                                                    ai::presets::validate_presets_json(&contents)
                                                        .map_err(|e| {
                                                        format!("Invalid preset file: {}", e)
                                                    })?;
                                                    ai::presets::import_presets_from_file(&path)
                                                        .map_err(|e| {
                                                            format!("Import failed: {}", e)
                                                        })
                                                }
                                            })
                                            .await;

                                        let _ = this.update(cx, |this, cx| {
                                            match import_result {
                                                Ok(total) => {
                                                    tracing::info!(
                                                        total = total,
                                                        action = "import_presets_success",
                                                        "Imported AI presets via file picker"
                                                    );
                                                    this.show_hud(
                                                        format!(
                                                            "Imported presets ({} total)",
                                                            total
                                                        ),
                                                        Some(HUD_SHORT_MS),
                                                        cx,
                                                    );
                                                    ai::reload_ai_presets(cx);
                                                }
                                                Err(e) => {
                                                    tracing::error!(
                                                        error = %e,
                                                        action = "import_presets_failed",
                                                        "Failed to import presets"
                                                    );
                                                    this.show_error_toast(
                                                        format!("Failed to import presets: {}", e),
                                                        cx,
                                                    );
                                                }
                                            }
                                            cx.notify();
                                        });
                                    }
                                }
                                Ok(Ok(None)) => {
                                    tracing::info!(
                                        action = "import_presets_cancelled",
                                        "User cancelled import file picker"
                                    );
                                }
                                Ok(Err(e)) => {
                                    tracing::warn!(error = %e, "Import file picker returned error");
                                }
                                Err(_) => {
                                    tracing::warn!(
                                        "Import file picker channel closed unexpectedly"
                                    );
                                }
                            }
                        })
                        .detach();
                        // Async — outcome tracked in spawned task
                        Self::builtin_success(dctx, "ai_import_presets_dispatched")
                    }

                    AiCommandType::ExportAiPresets => {
                        tracing::info!(
                            action = "export_ai_presets",
                            trace_id = %dctx.trace_id,
                            "Opening save dialog for AI preset export"
                        );
                        let default_dir = ai::presets::get_presets_path()
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(crate::setup::get_kit_path);

                        let rx =
                            cx.prompt_for_new_path(&default_dir, Some("ai-presets-export.json"));

                        cx.spawn(async move |this, cx| match rx.await {
                            Ok(Ok(Some(path))) => {
                                let export_result = cx
                                    .background_executor()
                                    .spawn({
                                        let path = path.clone();
                                        async move {
                                            ai::presets::export_presets_to_file(&path)
                                                .map_err(|e| format!("Export failed: {}", e))
                                        }
                                    })
                                    .await;

                                let _ = this.update(cx, |this, cx| {
                                    match export_result {
                                        Ok(count) => {
                                            tracing::info!(
                                                count = count,
                                                path = %path.display(),
                                                action = "export_presets_success",
                                                "Exported AI presets via file picker"
                                            );
                                            this.show_hud(
                                                format!("Exported {} presets", count),
                                                Some(HUD_SHORT_MS),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                error = %e,
                                                action = "export_presets_failed",
                                                "Failed to export presets"
                                            );
                                            this.show_error_toast(
                                                format!("Failed to export presets: {}", e),
                                                cx,
                                            );
                                        }
                                    }
                                    cx.notify();
                                });
                            }
                            Ok(Ok(None)) => {
                                tracing::info!(
                                    action = "export_presets_cancelled",
                                    "User cancelled export save dialog"
                                );
                            }
                            Ok(Err(e)) => {
                                tracing::warn!(error = %e, "Export save dialog returned error");
                            }
                            Err(_) => {
                                tracing::warn!("Export save dialog channel closed unexpectedly");
                            }
                        })
                        .detach();
                        // Async — outcome tracked in spawned task
                        Self::builtin_success(dctx, "ai_export_presets_dispatched")
                    }

                    AiCommandType::SearchAiPresets => {
                        tracing::info!(
                            action = "search_ai_presets",
                            trace_id = %dctx.trace_id,
                            "Opening AI presets search"
                        );
                        self.current_view = AppView::SearchAiPresetsView {
                            filter: String::new(),
                            selected_index: 0,
                        };
                        self.pending_focus = Some(FocusTarget::MainFilter);
                        cx.notify();
                        Self::builtin_success(dctx, "ai_search_presets")
                    }
                }
            }

            // =========================================================================
            // Script Commands
            // =========================================================================
            builtins::BuiltInFeature::ScriptCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    script_command = ?cmd_type,
                    "Executing script command"
                );

                use builtins::ScriptCommandType;

                let target = match cmd_type {
                    ScriptCommandType::NewScript => prompts::NamingTarget::Script,
                    ScriptCommandType::NewExtension => prompts::NamingTarget::Extension,
                };
                self.show_naming_dialog(target, cx);
                Self::builtin_success(dctx, format!("script_command::{cmd_type:?}"))
            }

            // =========================================================================
            // Permission Commands
            // =========================================================================
            builtins::BuiltInFeature::PermissionCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    permission_command = ?cmd_type,
                    "Executing permission command"
                );

                use builtins::PermissionCommandType;

                match cmd_type {
                    PermissionCommandType::CheckPermissions => {
                        let status = permissions_wizard::check_all_permissions();
                        if status.all_granted() {
                            self.show_hud(
                                "All permissions granted!".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                        } else {
                            let missing: Vec<_> = status
                                .missing_permissions()
                                .iter()
                                .map(|p| p.permission_type.name())
                                .collect();
                            self.toast_manager.push(
                                components::toast::Toast::warning(
                                    format!("Missing permissions: {}", missing.join(", ")),
                                    &self.theme,
                                )
                                .duration_ms(Some(TOAST_WARNING_MS)),
                            );
                        }
                        cx.notify();
                        Self::builtin_success(dctx, "check_permissions")
                    }
                    PermissionCommandType::RequestAccessibility => {
                        let granted = permissions_wizard::request_accessibility_permission();
                        if granted {
                            self.show_hud(
                                "Accessibility permission granted!".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                        } else {
                            self.toast_manager.push(
                                components::toast::Toast::warning(
                                    "Accessibility permission not granted. Some features may not work.",
                                    &self.theme,
                                )
                                .duration_ms(Some(TOAST_WARNING_MS)),
                            );
                        }
                        cx.notify();
                        Self::builtin_success(dctx, "request_accessibility")
                    }
                    PermissionCommandType::OpenAccessibilitySettings => {
                        if let Err(e) = permissions_wizard::open_accessibility_settings() {
                            let message = format!("Failed to open settings: {}", e);
                            self.show_error_toast(message.clone(), cx);
                            Self::builtin_error(
                                dctx,
                                crate::action_helpers::ERROR_LAUNCH_FAILED,
                                message,
                                "open_accessibility_settings_failed",
                            )
                        } else {
                            self.close_and_reset_window(cx);
                            Self::builtin_success(dctx, "open_accessibility_settings")
                        }
                    }
                }
            }

            // =========================================================================
            // Frecency/Suggested Commands
            // =========================================================================
            builtins::BuiltInFeature::FrecencyCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    frecency_command = ?cmd_type,
                    "Executing frecency command"
                );

                use builtins::FrecencyCommandType;

                match cmd_type {
                    FrecencyCommandType::ClearSuggested => {
                        self.frecency_store.clear();
                        if let Err(e) = self.frecency_store.save() {
                            let message = format!("Failed to clear suggested: {}", e);
                            self.show_error_toast(message.clone(), cx);
                            cx.notify();
                            Self::builtin_error(
                                dctx,
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                message,
                                "clear_suggested_failed",
                            )
                        } else {
                            tracing::info!(
                                trace_id = %dctx.trace_id,
                                "Cleared all suggested items"
                            );
                            self.invalidate_grouped_cache();
                            self.reset_to_script_list(cx);
                            resize_to_view_sync(ViewType::ScriptList, 0);
                            self.show_hud(
                                "Suggested items cleared".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            cx.notify();
                            Self::builtin_success(dctx, "clear_suggested")
                        }
                    }
                }
            }

            // =========================================================================
            // Settings Commands (Reset Window Positions, etc.)
            // =========================================================================
            builtins::BuiltInFeature::SettingsCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    settings_command = ?cmd_type,
                    "Executing settings command"
                );

                use builtins::SettingsCommandType;

                match cmd_type {
                    SettingsCommandType::ResetWindowPositions => {
                        crate::window_state::suppress_save();
                        crate::window_state::reset_all_positions();
                        self.show_hud(
                            "Window positions reset - takes effect next open".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        self.close_and_reset_window(cx);
                        Self::builtin_success(dctx, "reset_window_positions")
                    }
                    SettingsCommandType::ConfigureVercelApiKey => {
                        self.show_api_key_prompt(
                            "SCRIPT_KIT_VERCEL_API_KEY",
                            "Enter your Vercel AI Gateway API key",
                            "Vercel AI Gateway",
                            cx,
                        );
                        Self::builtin_success(dctx, "configure_vercel_api_key")
                    }
                    SettingsCommandType::ConfigureOpenAiApiKey => {
                        self.show_api_key_prompt(
                            "SCRIPT_KIT_OPENAI_API_KEY",
                            "Enter your OpenAI API key",
                            "OpenAI",
                            cx,
                        );
                        Self::builtin_success(dctx, "configure_openai_api_key")
                    }
                    SettingsCommandType::ConfigureAnthropicApiKey => {
                        self.show_api_key_prompt(
                            "SCRIPT_KIT_ANTHROPIC_API_KEY",
                            "Enter your Anthropic API key",
                            "Anthropic",
                            cx,
                        );
                        Self::builtin_success(dctx, "configure_anthropic_api_key")
                    }
                    SettingsCommandType::ChooseTheme => {
                        self.theme_before_chooser = Some(self.theme.clone());
                        let start_index = theme::presets::find_current_preset_index(&self.theme);

                        self.open_builtin_filterable_view(
                            AppView::ThemeChooserView {
                                filter: String::new(),
                                selected_index: start_index,
                            },
                            "Search themes...",
                            cx,
                        );

                        Self::builtin_success(dctx, "choose_theme")
                    }
                    SettingsCommandType::SelectMicrophone => {
                        let prefs = crate::config::load_user_preferences();
                        let menu_items = match crate::dictation::list_input_device_menu_items(
                            prefs.dictation.selected_device_id.as_deref(),
                        ) {
                            Ok(items) => items,
                            Err(error) => {
                                tracing::error!(
                                    category = "DICTATION",
                                    error = %error,
                                    "Failed to enumerate microphone devices"
                                );
                                self.show_hud(
                                    format!("Failed to list microphones: {error}"),
                                    Some(HUD_SHORT_MS),
                                    cx,
                                );
                                return Self::builtin_error(
                                    dctx,
                                    "select_microphone_failed",
                                    "Failed to list microphones",
                                    error.to_string(),
                                );
                            }
                        };

                        let mut start_index: usize = 0;
                        let choices: Vec<Choice> = menu_items
                            .iter()
                            .enumerate()
                            .map(|(idx, item)| {
                                let value = match &item.action {
                                    crate::dictation::DictationDeviceSelectionAction::UseSystemDefault => {
                                        BUILTIN_MIC_DEFAULT_VALUE.to_string()
                                    }
                                    crate::dictation::DictationDeviceSelectionAction::UseDevice(id) => {
                                        id.0.clone()
                                    }
                                };
                                let name = if item.is_selected {
                                    if start_index == 0 && idx > 0 {
                                        start_index = idx;
                                    }
                                    format!("{} (current)", item.title)
                                } else {
                                    item.title.clone()
                                };
                                Choice {
                                    name,
                                    value,
                                    description: Some(item.subtitle.clone()),
                                    key: None,
                                    semantic_id: None,
                                }
                            })
                            .collect();

                        self.opened_from_main_menu = true;
                        self.arg_selected_index = start_index;
                        self.open_builtin_filterable_view(
                            AppView::ArgPrompt {
                                id: BUILTIN_MIC_SELECT_PROMPT_ID.to_string(),
                                placeholder: "Select microphone...".to_string(),
                                choices,
                                actions: None,
                            },
                            "Select microphone...",
                            cx,
                        );

                        Self::builtin_success(dctx, "select_microphone")
                    }
                }
            }

            // =========================================================================
            // Utility Commands (Scratch Pad, Quick Terminal, Claude Code Harness, Process Manager)
            // =========================================================================
            builtins::BuiltInFeature::UtilityCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    utility_command = ?cmd_type,
                    "Executing utility command"
                );

                use builtins::UtilityCommandType;

                match cmd_type {
                    UtilityCommandType::MiniMainWindow => {
                        tracing::info!(
                            category = "BUILTIN",
                            trace_id = %dctx.trace_id,
                            "Opening Mini Main Window"
                        );
                        self.open_mini_main_window(cx);
                        Self::builtin_success(dctx, "open_mini_main_window")
                    }
                    UtilityCommandType::ScratchPad => {
                        self.opened_from_main_menu = true;
                        self.open_scratch_pad(cx);
                        Self::builtin_success(dctx, "open_scratch_pad")
                    }
                    UtilityCommandType::QuickTerminal => {
                        self.opened_from_main_menu = true;
                        self.open_quick_terminal(cx);
                        Self::builtin_success(dctx, "open_quick_terminal")
                    }
                    UtilityCommandType::ClaudeCode => {
                        self.opened_from_main_menu = true;
                        self.open_claude_code_terminal(cx);
                        Self::builtin_success(dctx, "open_claude_code_terminal")
                    }
                    UtilityCommandType::ProcessManager => {
                        let processes =
                            crate::process_manager::PROCESS_MANAGER.get_active_processes_sorted();
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            active_process_count = processes.len(),
                            "process_manager.open_view"
                        );

                        self.cached_processes = processes;

                        self.open_builtin_filterable_view(
                            AppView::ProcessManagerView {
                                filter: String::new(),
                                selected_index: 0,
                            },
                            "Search running scripts...",
                            cx,
                        );

                        self.start_process_manager_refresh(cx);
                        Self::builtin_success(dctx, "open_process_manager")
                    }
                    UtilityCommandType::StopAllProcesses => {
                        let process_count = crate::process_manager::PROCESS_MANAGER.active_count();
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            requested_count = process_count,
                            "process_manager.stop_all"
                        );

                        if process_count == 0 {
                            self.show_hud(
                                "No running scripts to stop.".to_string(),
                                Some(HUD_2200_MS),
                                cx,
                            );
                        } else {
                            crate::process_manager::PROCESS_MANAGER.kill_all_processes();
                            self.show_hud(
                                format!("Stopped {} running script process(es).", process_count),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                            self.close_and_reset_window(cx);
                        }
                        Self::builtin_success(dctx, "stop_all_processes")
                    }
                    UtilityCommandType::InspectCurrentContext => {
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            "context_snapshot.inspect_requested"
                        );

                        let started_at = std::time::Instant::now();

                        let snapshot = crate::context_snapshot::capture_context_snapshot(
                            &crate::context_snapshot::CaptureContextOptions::default(),
                        );

                        match serde_json::to_string_pretty(&snapshot) {
                            Ok(json) => {
                                let receipt = crate::context_snapshot::build_inspection_receipt(
                                    &snapshot,
                                    json.len(),
                                );

                                tracing::info!(
                                    category = "CONTEXT",
                                    event = "context_snapshot_copied",
                                    trace_id = %dctx.trace_id,
                                    schema_version = receipt.schema_version,
                                    warning_count = receipt.warning_count,
                                    has_selected_text = receipt.has_selected_text,
                                    has_frontmost_app = receipt.has_frontmost_app,
                                    top_level_menu_count = receipt.top_level_menu_count,
                                    has_browser = receipt.has_browser,
                                    has_focused_window = receipt.has_focused_window,
                                    json_bytes = receipt.json_bytes,
                                    status = %receipt.status,
                                    duration_ms = started_at.elapsed().as_millis() as u64,
                                    "Copied current context snapshot to clipboard"
                                );

                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                                let hud_message =
                                    crate::context_snapshot::build_inspection_hud_message(&receipt);
                                self.show_hud(hud_message, Some(HUD_MEDIUM_MS), cx);
                                self.close_and_reset_window(cx);

                                Self::builtin_success(dctx, "inspect_current_context")
                            }
                            Err(e) => {
                                let message = format!(
                                    "Failed to serialize context snapshot: {}",
                                    e
                                );
                                tracing::error!(
                                    trace_id = %dctx.trace_id,
                                    error = %e,
                                    "context_snapshot.serialize_failed"
                                );
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "inspect_current_context_failed",
                                )
                            }
                        }
                    }
                    UtilityCommandType::TraceCurrentAppIntent => {
                        let raw_query_owned = query_override
                            .unwrap_or(&self.filter_text)
                            .to_string();
                        let effective_query =
                            crate::menu_bar::current_app_commands::normalize_trace_current_app_intent_request(
                                Some(&raw_query_owned),
                            )
                            .unwrap_or_default();

                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            raw_query = %raw_query_owned,
                            effective_query = %effective_query,
                            "current_app_intent_trace.requested"
                        );

                        match crate::menu_bar::load_frontmost_menu_snapshot() {
                            Ok(snapshot) => {
                                let trace_receipt =
                                    crate::menu_bar::current_app_commands::build_current_app_intent_trace_receipt(
                                        snapshot,
                                        Some(&raw_query_owned),
                                    );

                                match serde_json::to_string_pretty(&trace_receipt) {
                                    Ok(json) => {
                                        tracing::info!(
                                            category = "CURRENT_APP_TRACE",
                                            trace_id = %dctx.trace_id,
                                            app_name = %trace_receipt.app_name,
                                            bundle_id = %trace_receipt.bundle_id,
                                            raw_query = %trace_receipt.raw_query,
                                            effective_query = %trace_receipt.effective_query,
                                            normalized_query = %trace_receipt.normalized_query,
                                            action = %trace_receipt.action,
                                            filtered_entries = trace_receipt.filtered_entries,
                                            exact_matches = trace_receipt.exact_matches,
                                            candidate_count = trace_receipt.candidates.len(),
                                            has_prompt_preview = trace_receipt.prompt_preview.is_some(),
                                            json_bytes = json.len(),
                                            "current_app_intent_trace.copied"
                                        );

                                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                                        self.show_hud(
                                            format!(
                                                "Copied app intent trace: {} ({} exact / {} filtered)",
                                                trace_receipt.action,
                                                trace_receipt.exact_matches,
                                                trace_receipt.filtered_entries,
                                            ),
                                            Some(HUD_MEDIUM_MS),
                                            cx,
                                        );
                                        self.close_and_reset_window(cx);
                                        Self::builtin_success(dctx, "trace_current_app_intent")
                                    }
                                    Err(e) => {
                                        let message =
                                            format!("Failed to serialize current app intent trace: {}", e);
                                        tracing::error!(
                                            trace_id = %dctx.trace_id,
                                            error = %e,
                                            "current_app_intent_trace.serialize_failed"
                                        );
                                        self.show_error_toast(message.clone(), cx);
                                        Self::builtin_error(
                                            dctx,
                                            crate::action_helpers::ERROR_ACTION_FAILED,
                                            message,
                                            "trace_current_app_intent_serialize_failed",
                                        )
                                    }
                                }
                            }
                            Err(e) => {
                                let message = format!(
                                    "Failed to inspect current app intent: {}. Check Accessibility permission in System Settings \u{2192} Privacy & Security \u{2192} Accessibility, then refocus the target app and try again.",
                                    e
                                );
                                tracing::warn!(
                                    trace_id = %dctx.trace_id,
                                    error = %e,
                                    "current_app_intent_trace.capture_failed"
                                );
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "trace_current_app_intent_capture_failed",
                                )
                            }
                        }
                    }
                    UtilityCommandType::VerifyCurrentAppRecipe => {
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            "verify_current_app_recipe.requested"
                        );

                        let stored_recipe =
                            match crate::menu_bar::current_app_commands::load_current_app_command_recipe_from_clipboard()
                            {
                                Ok(recipe) => recipe,
                                Err(error) => {
                                    let message = format!("Verify Current App Recipe failed: {}", error);
                                    self.show_error_toast(message.clone(), cx);
                                    return Self::builtin_error(
                                        dctx,
                                        crate::action_helpers::ERROR_ACTION_FAILED,
                                        message,
                                        "verify_current_app_recipe_clipboard_failed",
                                    );
                                }
                            };

                        match crate::menu_bar::current_app_commands::load_frontmost_menu_snapshot() {
                            Ok(snapshot) => {
                                let selected_text = crate::selected_text::get_selected_text()
                                    .ok()
                                    .filter(|text| !text.trim().is_empty());

                                let browser_url = crate::platform::get_focused_browser_tab_url()
                                    .ok()
                                    .filter(|url| !url.trim().is_empty());

                                let verification =
                                    crate::menu_bar::current_app_commands::verify_current_app_command_recipe(
                                        &stored_recipe,
                                        snapshot,
                                        selected_text.as_deref(),
                                        browser_url.as_deref(),
                                    );

                                match serde_json::to_string_pretty(&verification) {
                                    Ok(json) => {
                                        tracing::info!(
                                            category = "CURRENT_APP_RECIPE_VERIFY",
                                            trace_id = %dctx.trace_id,
                                            expected_bundle_id = %verification.expected_bundle_id,
                                            actual_bundle_id = %verification.actual_bundle_id,
                                            expected_route = %verification.expected_route,
                                            actual_route = %verification.actual_route,
                                            prompt_matches = verification.prompt_matches,
                                            selected_text_expected = verification.selected_text_expected,
                                            selected_text_present = verification.selected_text_present,
                                            browser_url_expected = verification.browser_url_expected,
                                            browser_url_present = verification.browser_url_present,
                                            warning_count = verification.warning_count,
                                            status = %verification.status,
                                            json_bytes = json.len(),
                                            "verify_current_app_recipe.completed"
                                        );

                                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                                        self.show_hud(
                                            crate::menu_bar::current_app_commands::build_current_app_command_verification_hud_message(
                                                &verification,
                                            ),
                                            Some(HUD_MEDIUM_MS),
                                            cx,
                                        );
                                        self.close_and_reset_window(cx);

                                        Self::builtin_success(dctx, "verify_current_app_recipe")
                                    }
                                    Err(error) => {
                                        let message = format!(
                                            "Failed to serialize current app recipe verification: {}",
                                            error
                                        );
                                        self.show_error_toast(message.clone(), cx);
                                        Self::builtin_error(
                                            dctx,
                                            crate::action_helpers::ERROR_ACTION_FAILED,
                                            message,
                                            "verify_current_app_recipe_serialize_failed",
                                        )
                                    }
                                }
                            }
                            Err(error) => {
                                let message = format!(
                                    "Failed to verify current app recipe: {}. Check Accessibility permission in System Settings → Privacy & Security → Accessibility, then refocus the target app and try again.",
                                    error
                                );
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "verify_current_app_recipe_capture_failed",
                                )
                            }
                        }
                    }
                    UtilityCommandType::ReplayCurrentAppRecipe => {
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            "replay_current_app_recipe.requested"
                        );

                        let stored_recipe =
                            match crate::menu_bar::current_app_commands::load_current_app_command_recipe_from_clipboard()
                            {
                                Ok(recipe) => recipe,
                                Err(error) => {
                                    let message = format!("Replay Current App Recipe failed: {}", error);
                                    self.show_error_toast(message.clone(), cx);
                                    return Self::builtin_error(
                                        dctx,
                                        crate::action_helpers::ERROR_ACTION_FAILED,
                                        message,
                                        "replay_current_app_recipe_clipboard_failed",
                                    );
                                }
                            };

                        match crate::menu_bar::current_app_commands::load_frontmost_menu_snapshot() {
                            Ok(snapshot) => {
                                let (entries, snapshot_receipt) = snapshot.clone().into_entries_with_receipt();

                                let selected_text = crate::selected_text::get_selected_text()
                                    .ok()
                                    .filter(|text| !text.trim().is_empty());

                                let browser_url = crate::platform::get_focused_browser_tab_url()
                                    .ok()
                                    .filter(|url| !url.trim().is_empty());

                                let replay_receipt =
                                    crate::menu_bar::current_app_commands::build_replay_current_app_recipe_receipt(
                                        &stored_recipe,
                                        &entries,
                                        snapshot,
                                        selected_text.as_deref(),
                                        browser_url.as_deref(),
                                    );

                                tracing::info!(
                                    category = "CURRENT_APP_RECIPE_REPLAY",
                                    trace_id = %dctx.trace_id,
                                    action = %replay_receipt.action,
                                    status = %replay_receipt.verification.status,
                                    warning_count = replay_receipt.verification.warning_count,
                                    expected_bundle_id = %replay_receipt.verification.expected_bundle_id,
                                    actual_bundle_id = %replay_receipt.verification.actual_bundle_id,
                                    expected_route = %replay_receipt.verification.expected_route,
                                    actual_route = %replay_receipt.verification.actual_route,
                                    selected_entry_index = replay_receipt.selected_entry_index,
                                    "replay_current_app_recipe.resolved"
                                );

                                if replay_receipt.verification.warning_count > 0 {
                                    let json = match serde_json::to_string_pretty(&replay_receipt) {
                                        Ok(json) => json,
                                        Err(error) => {
                                            let message = format!(
                                                "Failed to serialize replay current app recipe receipt: {}",
                                                error
                                            );
                                            self.show_error_toast(message.clone(), cx);
                                            return Self::builtin_error(
                                                dctx,
                                                crate::action_helpers::ERROR_ACTION_FAILED,
                                                message,
                                                "replay_current_app_recipe_serialize_failed",
                                            );
                                        }
                                    };

                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));

                                    let message =
                                        crate::menu_bar::current_app_commands::build_replay_current_app_recipe_hud_message(
                                            &replay_receipt,
                                        );

                                    self.show_error_toast(
                                        format!("{}. Copied replay report to clipboard.", message),
                                        cx,
                                    );

                                    return Self::builtin_error(
                                        dctx,
                                        crate::action_helpers::ERROR_ACTION_FAILED,
                                        message,
                                        "replay_current_app_recipe_drift",
                                    );
                                }

                                match replay_receipt.action.as_str() {
                                    "execute_entry" => {
                                        let Some(entry_index) = replay_receipt.selected_entry_index else {
                                            let message =
                                                "Replay Current App Recipe resolved to execute_entry without an entry index"
                                                    .to_string();
                                            self.show_error_toast(message.clone(), cx);
                                            return Self::builtin_error(
                                                dctx,
                                                crate::action_helpers::ERROR_ACTION_FAILED,
                                                message,
                                                "replay_current_app_recipe_missing_entry_index",
                                            );
                                        };

                                        let entry = entries[entry_index].clone();
                                        self.execute_builtin_inner(
                                            &entry,
                                            Some(
                                                replay_receipt
                                                    .verification
                                                    .live_recipe
                                                    .effective_query
                                                    .as_str(),
                                            ),
                                            dctx,
                                            cx,
                                        )
                                    }
                                    "open_command_palette" => {
                                        let filter = replay_receipt
                                            .verification
                                            .live_recipe
                                            .effective_query
                                            .clone();

                                        self.cached_current_app_entries = entries;
                                        self.open_builtin_filterable_view_with_filter(
                                            AppView::CurrentAppCommandsView {
                                                filter: filter.clone(),
                                                selected_index: 0,
                                            },
                                            &filter,
                                            &snapshot_receipt.placeholder,
                                            cx,
                                        );

                                        Self::builtin_success(dctx, "replay_current_app_recipe_open_palette")
                                    }
                                    "generate_script" => {
                                        self.spawn_generate_script_from_recipe_after_hide(
                                            dctx.trace_id.to_string(),
                                            replay_receipt.verification.live_recipe.clone(),
                                            cx,
                                        );
                                        Self::builtin_success(
                                            dctx,
                                            "replay_current_app_recipe_generate_script",
                                        )
                                    }
                                    other => {
                                        let message = format!(
                                            "Replay Current App Recipe resolved to unsupported action: {}",
                                            other
                                        );
                                        self.show_error_toast(message.clone(), cx);
                                        Self::builtin_error(
                                            dctx,
                                            crate::action_helpers::ERROR_ACTION_FAILED,
                                            message,
                                            "replay_current_app_recipe_unknown_action",
                                        )
                                    }
                                }
                            }
                            Err(error) => {
                                let message = format!(
                                    "Failed to replay current app recipe: {}. Check Accessibility permission in System Settings → Privacy & Security → Accessibility, then refocus the target app and try again.",
                                    error
                                );
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "replay_current_app_recipe_capture_failed",
                                )
                            }
                        }
                    }
                    UtilityCommandType::TurnThisIntoCommand => {
                        let raw_query_owned = query_override
                            .unwrap_or(&self.filter_text)
                            .to_string();

                        let effective_query =
                            crate::menu_bar::current_app_commands::normalize_turn_this_into_a_command_request(
                                Some(&raw_query_owned),
                            )
                            .unwrap_or_default();

                        if effective_query.is_empty() {
                            let message =
                                "Type what you want to automate after \"Turn This Into a Command\"".to_string();
                            self.show_error_toast(message.clone(), cx);
                            Self::builtin_error(
                                dctx,
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                message,
                                "turn_this_into_command_missing_query",
                            )
                        } else {
                            tracing::info!(
                                trace_id = %dctx.trace_id,
                                raw_query = %raw_query_owned,
                                effective_query = %effective_query,
                                "turn_this_into_command.requested"
                            );

                            match crate::menu_bar::load_frontmost_menu_snapshot() {
                                Ok(snapshot) => {
                                    let selected_text = crate::selected_text::get_selected_text()
                                        .ok()
                                        .filter(|text| !text.trim().is_empty());

                                    let browser_url = crate::platform::get_focused_browser_tab_url()
                                        .ok()
                                        .filter(|url| !url.trim().is_empty());

                                    let recipe =
                                        crate::menu_bar::current_app_commands::build_current_app_command_recipe(
                                            snapshot,
                                            Some(&raw_query_owned),
                                            selected_text.as_deref(),
                                            browser_url.as_deref(),
                                        );

                                    match serde_json::to_string_pretty(&recipe) {
                                        Ok(json) => {
                                            tracing::info!(
                                                category = "CURRENT_APP_RECIPE",
                                                trace_id = %dctx.trace_id,
                                                app_name = %recipe.prompt_receipt.app_name,
                                                bundle_id = %recipe.prompt_receipt.bundle_id,
                                                effective_query = %recipe.effective_query,
                                                route = %recipe.trace.action,
                                                suggested_script_name = %recipe.suggested_script_name,
                                                candidate_count = recipe.trace.candidates.len(),
                                                included_selected_text = recipe.prompt_receipt.included_selected_text,
                                                included_browser_url = recipe.prompt_receipt.included_browser_url,
                                                json_bytes = json.len(),
                                                "turn_this_into_command.recipe_copied"
                                            );

                                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));

                                            self.show_hud(
                                                format!(
                                                    "Automation recipe copied: {}",
                                                    recipe.suggested_script_name,
                                                ),
                                                Some(HUD_MEDIUM_MS),
                                                cx,
                                            );

                                            self.spawn_generate_script_from_recipe_after_hide(
                                                dctx.trace_id.to_string(),
                                                recipe.clone(),
                                                cx,
                                            );

                                            Self::builtin_success(dctx, "turn_this_into_command")
                                        }
                                        Err(e) => {
                                            let message = format!(
                                                "Failed to serialize current app command recipe: {}",
                                                e
                                            );
                                            tracing::error!(
                                                trace_id = %dctx.trace_id,
                                                error = %e,
                                                "turn_this_into_command.serialize_failed"
                                            );
                                            self.show_error_toast(message.clone(), cx);
                                            Self::builtin_error(
                                                dctx,
                                                crate::action_helpers::ERROR_ACTION_FAILED,
                                                message,
                                                "turn_this_into_command_serialize_failed",
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    let message = format!(
                                        "Failed to capture current app command recipe: {}. Check Accessibility permission in System Settings \u{2192} Privacy & Security \u{2192} Accessibility, then refocus the target app and try again.",
                                        e
                                    );
                                    tracing::warn!(
                                        trace_id = %dctx.trace_id,
                                        error = %e,
                                        "turn_this_into_command.capture_failed"
                                    );
                                    self.show_error_toast(message.clone(), cx);
                                    Self::builtin_error(
                                        dctx,
                                        crate::action_helpers::ERROR_ACTION_FAILED,
                                        message,
                                        "turn_this_into_command_capture_failed",
                                    )
                                }
                            }
                        }
                    }
                    UtilityCommandType::DoInCurrentApp => {
                        let raw_query_owned = query_override
                            .unwrap_or(&self.filter_text)
                            .to_string();
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            raw_query = %raw_query_owned,
                            filter_text = %self.filter_text,
                            query_override = ?query_override,
                            "do_in_current_app.execution_entry — raw inputs"
                        );
                        let trimmed_query =
                            crate::menu_bar::current_app_commands::normalize_do_in_current_app_request(
                                Some(&raw_query_owned),
                            )
                            .unwrap_or_default()
                            .to_string();

                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            query = %trimmed_query,
                            "do_in_current_app.requested"
                        );

                        match crate::menu_bar::load_frontmost_menu_snapshot() {
                            Ok(snapshot) => {
                                let snapshot_for_recipe = snapshot.clone();
                                let (entries, snapshot_receipt) = snapshot.into_entries_with_receipt();

                                if entries.is_empty() && trimmed_query.is_empty() {
                                    let message = format!(
                                        "No enabled menu bar commands found for {}",
                                        snapshot_receipt.app_name
                                    );
                                    self.show_error_toast(message.clone(), cx);
                                    Self::builtin_error(
                                        dctx,
                                        crate::action_helpers::ERROR_ACTION_FAILED,
                                        message,
                                        "do_in_current_app_empty_snapshot",
                                    )
                                } else {
                                    let (action, intent_receipt) =
                                        crate::menu_bar::current_app_commands::resolve_do_in_current_app_intent(
                                            &entries,
                                            Some(&raw_query_owned),
                                        );

                                    tracing::info!(
                                        trace_id = %dctx.trace_id,
                                        app_name = %snapshot_receipt.app_name,
                                        bundle_id = %snapshot_receipt.bundle_id,
                                        leaf_entry_count = snapshot_receipt.leaf_entry_count,
                                        query = %trimmed_query,
                                        filtered_entries = intent_receipt.filtered_entries,
                                        exact_matches = intent_receipt.exact_matches,
                                        resolved_action = intent_receipt.action,
                                        "do_in_current_app.resolved"
                                    );

                                    match action {
                                        crate::menu_bar::current_app_commands::DoInCurrentAppAction::OpenCommandPalette => {
                                            tracing::info!(
                                                trace_id = %dctx.trace_id,
                                                cached_entries = entries.len(),
                                                filter = %trimmed_query,
                                                placeholder = %snapshot_receipt.placeholder,
                                                "do_in_current_app.action → OpenCommandPalette — switching to CurrentAppCommandsView"
                                            );
                                            self.cached_current_app_entries = entries;
                                            self.open_builtin_filterable_view_with_filter(
                                                AppView::CurrentAppCommandsView {
                                                    filter: trimmed_query.clone(),
                                                    selected_index: 0,
                                                },
                                                &trimmed_query,
                                                &snapshot_receipt.placeholder,
                                                cx,
                                            );
                                            Self::builtin_success(dctx, "do_in_current_app_open_palette")
                                        }
                                        crate::menu_bar::current_app_commands::DoInCurrentAppAction::ExecuteEntry(entry_index) => {
                                            tracing::info!(
                                                trace_id = %dctx.trace_id,
                                                entry_index = entry_index,
                                                entry_name = %entries[entry_index].name,
                                                "do_in_current_app.action → ExecuteEntry — running menu command directly"
                                            );
                                            let entry = entries[entry_index].clone();
                                            self.execute_builtin_inner(&entry, Some(&raw_query_owned), dctx, cx)
                                        }
                                        crate::menu_bar::current_app_commands::DoInCurrentAppAction::GenerateScript => {
                                            tracing::info!(
                                                trace_id = %dctx.trace_id,
                                                trimmed_query = %trimmed_query,
                                                "do_in_current_app.action → GenerateScript — routing through recipe flow"
                                            );

                                            let selected_text = crate::selected_text::get_selected_text()
                                                .ok()
                                                .filter(|text| !text.trim().is_empty());

                                            let browser_url = crate::platform::get_focused_browser_tab_url()
                                                .ok()
                                                .filter(|url| !url.trim().is_empty());

                                            // --- Automation memory lookup: replay/repair/generate ---
                                            let memory_decision = crate::ai::resolve_current_app_automation_from_memory(
                                                &raw_query_owned,
                                                &snapshot_for_recipe,
                                                &entries,
                                                selected_text.as_deref(),
                                                browser_url.as_deref(),
                                            );

                                            if let Ok(ref decision) = memory_decision {
                                                if let Some(ref replay) = decision.replay {
                                                    tracing::info!(
                                                        category = "CURRENT_APP_AUTOMATION_MEMORY",
                                                        trace_id = %dctx.trace_id,
                                                        action = %decision.action,
                                                        best_score = decision.best_score,
                                                        matched_slug = decision
                                                            .matched
                                                            .as_ref()
                                                            .map(|entry| entry.slug.as_str())
                                                            .unwrap_or(""),
                                                        reason = %decision.reason,
                                                        "do_in_current_app.memory_resolved"
                                                    );

                                                    match decision.action.as_str() {
                                                        "replay_recipe" => {
                                                            match replay.action.as_str() {
                                                                "execute_entry" => {
                                                                    if let Some(entry_index) = replay.selected_entry_index {
                                                                        if entry_index < entries.len() {
                                                                            let entry = entries[entry_index].clone();
                                                                            return self.execute_builtin_inner(
                                                                                &entry,
                                                                                Some(&raw_query_owned),
                                                                                dctx,
                                                                                cx,
                                                                            );
                                                                        }
                                                                    }
                                                                }
                                                                "open_command_palette" => {
                                                                    let filter = replay.verification.live_recipe.effective_query.clone();
                                                                    self.cached_current_app_entries = entries.clone();
                                                                    self.open_builtin_filterable_view_with_filter(
                                                                        AppView::CurrentAppCommandsView {
                                                                            filter: filter.clone(),
                                                                            selected_index: 0,
                                                                        },
                                                                        &filter,
                                                                        &snapshot_receipt.placeholder,
                                                                        cx,
                                                                    );
                                                                    return Self::builtin_success(
                                                                        dctx,
                                                                        "do_in_current_app_replay_memory_open_palette",
                                                                    );
                                                                }
                                                                "generate_script" => {
                                                                    self.spawn_generate_script_from_recipe_after_hide(
                                                                        dctx.trace_id.to_string(),
                                                                        replay.verification.live_recipe.clone(),
                                                                        cx,
                                                                    );
                                                                    return Self::builtin_success(
                                                                        dctx,
                                                                        "do_in_current_app_replay_memory_generate_script",
                                                                    );
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        "repair_recipe" => {
                                                            self.spawn_generate_script_from_recipe_after_hide(
                                                                dctx.trace_id.to_string(),
                                                                replay.verification.live_recipe.clone(),
                                                                cx,
                                                            );
                                                            return Self::builtin_success(
                                                                dctx,
                                                                "do_in_current_app_repair_memory_recipe",
                                                            );
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            // --- End automation memory lookup ---

                                            let recipe =
                                                crate::menu_bar::current_app_commands::build_current_app_command_recipe(
                                                    snapshot_for_recipe,
                                                    Some(&raw_query_owned),
                                                    selected_text.as_deref(),
                                                    browser_url.as_deref(),
                                                );

                                            match serde_json::to_string_pretty(&recipe) {
                                                Ok(json) => {
                                                    tracing::info!(
                                                        category = "CURRENT_APP_RECIPE",
                                                        trace_id = %dctx.trace_id,
                                                        app_name = %recipe.prompt_receipt.app_name,
                                                        bundle_id = %recipe.prompt_receipt.bundle_id,
                                                        effective_query = %recipe.effective_query,
                                                        route = %recipe.trace.action,
                                                        suggested_script_name = %recipe.suggested_script_name,
                                                        included_selected_text = recipe.prompt_receipt.included_selected_text,
                                                        included_browser_url = recipe.prompt_receipt.included_browser_url,
                                                        json_bytes = json.len(),
                                                        "do_in_current_app.recipe_prepared"
                                                    );
                                                }
                                                Err(error) => {
                                                    tracing::warn!(
                                                        trace_id = %dctx.trace_id,
                                                        error = %error,
                                                        "do_in_current_app.recipe_serialize_failed"
                                                    );
                                                }
                                            }

                                            self.spawn_generate_script_from_recipe_after_hide(
                                                dctx.trace_id.to_string(),
                                                recipe,
                                                cx,
                                            );
                                            Self::builtin_success(dctx, "do_in_current_app_generate_script_from_recipe")
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let message = format!("Failed to load frontmost app menu bar: {}", e);
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "do_in_current_app_capture_failed",
                                )
                            }
                        }
                    }
                    UtilityCommandType::CurrentAppCommands => {
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            "current_app_commands.open_requested"
                        );
                        match crate::menu_bar::load_frontmost_menu_snapshot() {
                            Ok(snapshot) => {
                                let (entries, receipt) = snapshot.into_entries_with_receipt();
                                let placeholder = receipt.placeholder.clone();
                                let app_name = receipt.app_name.clone();

                                tracing::info!(
                                    trace_id = %dctx.trace_id,
                                    app_name = %receipt.app_name,
                                    bundle_id = %receipt.bundle_id,
                                    top_level_menu_count = receipt.top_level_menu_count,
                                    leaf_entry_count = receipt.leaf_entry_count,
                                    placeholder = %receipt.placeholder,
                                    source = receipt.source,
                                    "current_app_commands.snapshot_ready"
                                );

                                if entries.is_empty() {
                                    let message = format!(
                                        "No enabled menu bar commands found for {}",
                                        app_name
                                    );
                                    tracing::warn!(
                                        trace_id = %dctx.trace_id,
                                        app_name = %app_name,
                                        "current_app_commands.no_entries"
                                    );
                                    self.show_error_toast(message.clone(), cx);
                                    Self::builtin_error(
                                        dctx,
                                        crate::action_helpers::ERROR_ACTION_FAILED,
                                        message,
                                        "current_app_commands_empty",
                                    )
                                } else {
                                    tracing::info!(
                                        trace_id = %dctx.trace_id,
                                        app_name = %app_name,
                                        entry_count = entries.len(),
                                        "current_app_commands.loaded"
                                    );
                                    self.cached_current_app_entries = entries;
                                    self.open_builtin_filterable_view(
                                        AppView::CurrentAppCommandsView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        },
                                        &placeholder,
                                        cx,
                                    );
                                    Self::builtin_success(dctx, "open_current_app_commands")
                                }
                            }
                            Err(e) => {
                                let message =
                                    format!("Failed to load frontmost app menu bar: {}", e);
                                tracing::warn!(
                                    trace_id = %dctx.trace_id,
                                    error = %e,
                                    "current_app_commands.capture_failed"
                                );
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "current_app_commands_capture_failed",
                                )
                            }
                        }
                    }
                }
            }

            // =========================================================================
            // Kit Store Commands
            // =========================================================================
            builtins::BuiltInFeature::KitStoreCommand(cmd_type) => {
                use builtins::KitStoreCommandType;
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    kit_store_command = ?cmd_type,
                    "Executing kit store command"
                );

                self.opened_from_main_menu = true;

                match cmd_type {
                    KitStoreCommandType::BrowseKits => {
                        self.current_view = AppView::BrowseKitsView {
                            query: String::new(),
                            selected_index: 0,
                            results: Vec::new(),
                        };
                        self.pending_focus = Some(FocusTarget::AppRoot);
                        cx.notify();

                        cx.spawn(async move |this, cx| {
                            let results = cx
                                .background_executor()
                                .spawn(async { Self::kit_store_search_results("") })
                                .await;
                            let _ = this.update(cx, |this, cx| {
                                if let AppView::BrowseKitsView {
                                    results: view_results,
                                    ..
                                } = &mut this.current_view
                                {
                                    *view_results = results;
                                    cx.notify();
                                }
                            });
                        })
                        .detach();
                        Self::builtin_success(dctx, "browse_kits_dispatched")
                    }
                    KitStoreCommandType::InstalledKits => {
                        let kits = Self::kit_store_list_installed();
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            installed_count = kits.len(),
                            "Loaded installed kits"
                        );
                        self.current_view = AppView::InstalledKitsView {
                            selected_index: 0,
                            kits,
                        };
                        self.pending_focus = Some(FocusTarget::AppRoot);
                        cx.notify();
                        Self::builtin_success(dctx, "installed_kits")
                    }
                    KitStoreCommandType::UpdateAllKits => {
                        cx.spawn(async move |this, cx| {
                            let (updated, failed) = cx
                                .background_executor()
                                .spawn(async {
                                    let kits =
                                        script_kit_gpui::kit_store::storage::list_installed_kits()
                                            .unwrap_or_default();
                                    let mut updated = 0usize;
                                    let mut failed = 0usize;
                                    for kit in &kits {
                                        let pull_output = std::process::Command::new("git")
                                            .arg("-C")
                                            .arg(&kit.path)
                                            .arg("pull")
                                            .arg("--ff-only")
                                            .output();
                                        match pull_output {
                                            Ok(output) if output.status.success() => {
                                                updated += 1;
                                            }
                                            _ => {
                                                failed += 1;
                                                tracing::warn!(
                                                    kit_name = %kit.name,
                                                    "Kit update-all failed for kit"
                                                );
                                            }
                                        }
                                    }
                                    (updated, failed)
                                })
                                .await;

                            let _ = this.update(cx, |this, cx| {
                                let message = if failed == 0 {
                                    format!("Updated {} kit(s) successfully", updated)
                                } else {
                                    format!("Updated {} kit(s), {} failed", updated, failed)
                                };
                                if failed > 0 {
                                    this.toast_manager.push(
                                        components::toast::Toast::error(message, &this.theme)
                                            .duration_ms(Some(TOAST_ERROR_MS)),
                                    );
                                } else {
                                    this.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                                }
                                cx.notify();
                            });
                        })
                        .detach();
                        // Async — outcome tracked in spawned task
                        Self::builtin_success(dctx, "update_all_kits_dispatched")
                    }
                }
            }

            // =========================================================================
            // File Search (Directory Navigation)
            // =========================================================================
            builtins::BuiltInFeature::Webcam => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Webcam"
                );
                self.opened_from_main_menu = true;
                self.open_webcam(cx);
                Self::builtin_success(dctx, "open_webcam")
            }
            builtins::BuiltInFeature::Dictation => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Dictation"
                );
                self.opened_from_main_menu = true;

                // Preflight: on the start edge, verify we have somewhere to
                // deliver transcribed text before beginning capture.
                if !crate::dictation::is_dictation_recording() {
                    if let Err(error) = self.ensure_dictation_delivery_target_available() {
                        let error_text = error.to_string();
                        tracing::error!(
                            category = "DICTATION",
                            error = %error_text,
                            "Dictation start preflight failed"
                        );
                        self.show_error_toast(
                            format!("Dictation unavailable: {error_text}"),
                            cx,
                        );
                        return Self::builtin_success(dctx, "dictation_preflight_failed");
                    }
                }

                match crate::dictation::toggle_dictation() {
                    Ok(crate::dictation::DictationToggleOutcome::Started) => {
                        let _ = crate::dictation::open_dictation_overlay(cx);
                        let _ = crate::dictation::update_dictation_overlay(
                            crate::dictation::DictationOverlayState {
                                phase: crate::dictation::DictationSessionPhase::Recording,
                                ..Default::default()
                            },
                            cx,
                        );
                        self.spawn_dictation_overlay_pump(cx);
                    }
                    Ok(crate::dictation::DictationToggleOutcome::Stopped(Some(capture))) => {
                        let _ = crate::dictation::update_dictation_overlay(
                            crate::dictation::DictationOverlayState {
                                phase: crate::dictation::DictationSessionPhase::Transcribing,
                                elapsed: capture.audio_duration,
                                ..Default::default()
                            },
                            cx,
                        );
                        let audio_duration = capture.audio_duration;
                        let chunks = capture.chunks;
                        cx.spawn(async move |this, cx| {
                            let transcript_result = cx
                                .background_executor()
                                .spawn(async move {
                                    crate::dictation::transcribe_captured_audio(&chunks)
                                })
                                .await;

                            let _ = this.update(cx, |this, cx| {
                                Self::handle_dictation_transcript(
                                    this,
                                    transcript_result,
                                    audio_duration,
                                    cx,
                                );
                            });
                        })
                        .detach();
                    }
                    Ok(crate::dictation::DictationToggleOutcome::Stopped(None)) => {
                        let _ = crate::dictation::close_dictation_overlay(cx);
                    }
                    Err(error) => {
                        tracing::error!(
                            category = "DICTATION",
                            error = %error,
                            "Failed to toggle dictation"
                        );
                        let _ = crate::dictation::update_dictation_overlay(
                            crate::dictation::DictationOverlayState {
                                phase: crate::dictation::DictationSessionPhase::Failed(
                                    error.to_string(),
                                ),
                                ..Default::default()
                            },
                            cx,
                        );
                        self.schedule_dictation_overlay_close(
                            cx,
                            std::time::Duration::from_millis(800),
                        );
                    }
                }
                Self::builtin_success(dctx, "dictation_toggle")
            }
            builtins::BuiltInFeature::FileSearch => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening File Search"
                );
                self.opened_from_main_menu = true;
                self.open_file_search(String::new(), cx);
                Self::builtin_success(dctx, "open_file_search")
            }
            // =========================================================================
            // Settings Hub
            // =========================================================================
            builtins::BuiltInFeature::Settings => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "Opening Settings"
                );
                self.opened_from_main_menu = true;
                self.current_view = AppView::SettingsView { selected_index: 0 };
                self.hovered_index = None;
                resize_to_view_sync(ViewType::ScriptList, 0);
                self.pending_focus = Some(FocusTarget::AppRoot);
                cx.notify();
                Self::builtin_success(dctx, "open_settings")
            }
        }
    }

    // =========================================================================
    // Dictation helpers — overlay pump, transcript delivery, scheduled cleanup
    // =========================================================================

    /// Periodically snapshot the live capture session and push state to the
    /// dictation overlay.  Runs every 50 ms and stops automatically when the
    /// session ends (i.e. `snapshot_overlay_state()` returns `None`).
    fn spawn_dictation_overlay_pump(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |_this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(50))
                    .await;
                let Some(state) = crate::dictation::snapshot_overlay_state() else {
                    break;
                };
                cx.update(|cx| {
                    let _ = crate::dictation::update_dictation_overlay(state, cx);
                });
            }
        })
        .detach();
    }

    /// Handle the result of background transcription: deliver the transcript
    /// to either the active prompt or the frontmost app, update the overlay,
    /// and schedule cleanup timers.
    fn handle_dictation_transcript(
        &mut self,
        result: anyhow::Result<Option<String>>,
        audio_duration: std::time::Duration,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(Some(transcript)) => {
                if self.try_set_prompt_input(transcript.clone(), cx) {
                    tracing::info!(
                        category = "DICTATION",
                        destination = ?crate::dictation::DictationDestination::ActivePrompt,
                        transcript_len = transcript.len(),
                        "Transcript delivered"
                    );

                    let _ = crate::dictation::update_dictation_overlay(
                        crate::dictation::DictationOverlayState {
                            phase: crate::dictation::DictationSessionPhase::Finished,
                            elapsed: audio_duration,
                            transcript: transcript.into(),
                            ..Default::default()
                        },
                        cx,
                    );
                    self.schedule_dictation_overlay_close(
                        cx,
                        std::time::Duration::from_millis(400),
                    );
                    self.schedule_dictation_transcriber_cleanup(
                        cx,
                        std::time::Duration::from_secs(300),
                    );
                } else {
                    // Guard: verify that a tracked external app target exists
                    // before attempting to paste to the frontmost app.
                    if let Err(error) = Self::ensure_dictation_frontmost_target_available() {
                        let error_text = error.to_string();
                        tracing::error!(
                            category = "DICTATION",
                            error = %error_text,
                            "Failed to resolve frontmost-app dictation target"
                        );
                        self.show_error_toast(
                            format!("Dictation paste failed: {error_text}"),
                            cx,
                        );
                        self.schedule_dictation_overlay_close(
                            cx,
                            std::time::Duration::from_millis(150),
                        );
                        self.schedule_dictation_transcriber_cleanup(
                            cx,
                            std::time::Duration::from_secs(300),
                        );
                        return;
                    }

                    // Show a brief done state before closing and pasting to
                    // the frontmost app, matching the prompt-first path UX.
                    let _ = crate::dictation::update_dictation_overlay(
                        crate::dictation::DictationOverlayState {
                            phase: crate::dictation::DictationSessionPhase::Finished,
                            elapsed: audio_duration,
                            transcript: transcript.clone().into(),
                            ..Default::default()
                        },
                        cx,
                    );

                    cx.spawn(async move |this, cx| {
                        // Brief pause so the user sees the done state.
                        cx.background_executor()
                            .timer(Self::dictation_done_state_duration())
                            .await;

                        // Close overlay and hide Script Kit windows so macOS
                        // returns keyboard focus to the target app before
                        // the CGEvent Cmd+V fires.
                        let yield_focus_result = match this.update(
                            cx,
                            |this, cx| this.yield_focus_for_dictation_paste(cx),
                        ) {
                            Ok(result) => result,
                            Err(error) => Err(anyhow::anyhow!(
                                "failed to update app state before paste: {error}"
                            )),
                        };

                        if let Err(error) = yield_focus_result {
                            let error_text = error.to_string();
                            let _ = this.update(cx, |this, cx| {
                                tracing::error!(
                                    category = "DICTATION",
                                    error = %error_text,
                                    "Failed to yield focus before dictation paste"
                                );
                                this.show_error_toast(
                                    format!(
                                        "Dictation paste failed before paste step: {error_text}"
                                    ),
                                    cx,
                                );
                                this.schedule_dictation_transcriber_cleanup(
                                    cx,
                                    std::time::Duration::from_secs(300),
                                );
                            });
                            return;
                        }

                        // Let macOS settle focus back to the target app.
                        cx.background_executor()
                            .timer(Self::dictation_focus_settle_duration())
                            .await;

                        let paste_result = cx
                            .background_executor()
                            .spawn({
                                let transcript = transcript.clone();
                                async move {
                                    crate::text_injector::TextInjector::new()
                                        .paste_text(&transcript)
                                }
                            })
                            .await;

                        let _ = this.update(cx, |this, cx| {
                            match paste_result {
                                Ok(()) => {
                                    tracing::info!(
                                        category = "DICTATION",
                                        destination = ?crate::dictation::DictationDestination::FrontmostApp,
                                        transcript_len = transcript.len(),
                                        "Transcript delivered"
                                    );
                                }
                                Err(ref error) => {
                                    tracing::error!(
                                        category = "DICTATION",
                                        error = %error,
                                        "Failed to paste dictation transcript"
                                    );
                                    this.show_error_toast(
                                        format!("Dictation paste failed: {error}"),
                                        cx,
                                    );
                                }
                            }
                            this.schedule_dictation_transcriber_cleanup(
                                cx,
                                std::time::Duration::from_secs(300),
                            );
                        });
                    })
                    .detach();
                }
            }
            Ok(None) => {
                // No speech detected — close overlay quietly.
                self.schedule_dictation_overlay_close(
                    cx,
                    std::time::Duration::from_millis(150),
                );
                self.schedule_dictation_transcriber_cleanup(
                    cx,
                    std::time::Duration::from_secs(300),
                );
            }
            Err(error) => {
                tracing::error!(
                    category = "DICTATION",
                    error = %error,
                    "Transcription failed"
                );
                let _ = crate::dictation::update_dictation_overlay(
                    crate::dictation::DictationOverlayState {
                        phase: crate::dictation::DictationSessionPhase::Failed(
                            error.to_string(),
                        ),
                        elapsed: audio_duration,
                        ..Default::default()
                    },
                    cx,
                );
                self.schedule_dictation_overlay_close(
                    cx,
                    std::time::Duration::from_millis(800),
                );
                self.schedule_dictation_transcriber_cleanup(
                    cx,
                    std::time::Duration::from_secs(300),
                );
            }
        }
    }

    const DICTATION_DONE_STATE_MS: u64 = 75;
    const DICTATION_FOCUS_SETTLE_MS: u64 = 100;

    fn dictation_done_state_duration() -> std::time::Duration {
        std::time::Duration::from_millis(Self::DICTATION_DONE_STATE_MS)
    }

    fn dictation_focus_settle_duration() -> std::time::Duration {
        std::time::Duration::from_millis(Self::DICTATION_FOCUS_SETTLE_MS)
    }

    /// Ensure a new dictation session has somewhere valid to send text.
    ///
    /// Allowed start conditions:
    /// - a Script Kit prompt is active and can accept dictated text, or
    /// - the frontmost-app tracker already has a previously tracked external target.
    fn ensure_dictation_delivery_target_available(&self) -> anyhow::Result<()> {
        if self.can_accept_dictation_into_prompt() {
            return Ok(());
        }
        Self::ensure_dictation_frontmost_target_available()
    }

    /// Verify that the frontmost-app tracker has a previously-tracked
    /// external app target before attempting a dictation paste.
    fn ensure_dictation_frontmost_target_available() -> anyhow::Result<()> {
        use anyhow::Context as _;
        crate::frontmost_app_tracker::get_last_real_app_bundle_id()
            .context("no previously tracked frontmost app is available for dictation paste")?;
        Ok(())
    }

    /// Close the dictation overlay and hide the main window, propagating
    /// errors so the caller can abort the paste if either step fails.
    fn yield_focus_for_dictation_paste(
        &mut self,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        use anyhow::Context as _;
        crate::dictation::close_dictation_overlay(cx)
            .context("failed to close dictation overlay before paste")?;
        if script_kit_gpui::is_main_window_visible() {
            script_kit_gpui::set_main_window_visible(false);
            platform::defer_hide_main_window(cx);
        }
        Ok(())
    }

    /// Schedule the overlay window to close after a delay.
    fn schedule_dictation_overlay_close(
        &mut self,
        cx: &mut Context<Self>,
        delay: std::time::Duration,
    ) {
        cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            cx.update(|cx| {
                let _ = crate::dictation::close_dictation_overlay(cx);
            });
        })
        .detach();
    }

    /// Schedule the cached Whisper transcriber to be unloaded after an idle
    /// timeout.
    fn schedule_dictation_transcriber_cleanup(
        &mut self,
        cx: &mut Context<Self>,
        delay: std::time::Duration,
    ) {
        cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            cx.update(|_cx| {
                crate::dictation::maybe_unload_transcriber();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod builtin_execution_ai_feedback_tests {
    use super::{
        AI_CAPTURE_HIDE_SETTLE_MS, ai_capture_hide_settle_duration,
        ai_command_keeps_main_window_visible, ai_command_uses_hide_then_capture_flow,
        ai_open_failure_message, created_file_path_for_feedback, emoji_picker_label,
        favorites_loaded_message,
    };
    use crate::builtins::AiCommandType;
    use script_kit_gpui::emoji::{Emoji, EmojiCategory};
    use std::path::PathBuf;

    #[test]
    fn only_plain_generate_script_keeps_main_window_visible() {
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::GenerateScript
        ));
        assert!(!ai_command_keeps_main_window_visible(
            &AiCommandType::GenerateScriptFromCurrentApp
        ));
        assert!(!ai_command_keeps_main_window_visible(
            &AiCommandType::SendScreenToAi
        ));
        assert!(!ai_command_keeps_main_window_visible(
            &AiCommandType::OpenAi
        ));
        assert!(!ai_command_keeps_main_window_visible(
            &AiCommandType::MiniAi
        ));
    }

    #[test]
    fn test_ai_open_failure_message_includes_error_details() {
        assert_eq!(
            ai_open_failure_message("window init failed"),
            "Failed to open AI: window init failed"
        );
    }

    #[test]
    fn test_favorites_loaded_message_uses_singular_for_one() {
        assert_eq!(favorites_loaded_message(1), "Loaded 1 favorite");
    }

    #[test]
    fn test_favorites_loaded_message_uses_plural_for_many() {
        assert_eq!(favorites_loaded_message(3), "Loaded 3 favorites");
    }

    #[test]
    fn test_ai_capture_commands_use_hide_then_capture_flow_only_for_sync_screenshots() {
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::GenerateScriptFromCurrentApp
        ));
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendScreenToAi
        ));
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendFocusedWindowToAi
        ));
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendScreenAreaToAi
        ));
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendSelectedTextToAi
        ));
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendBrowserTabToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::MiniAi
        ));
    }

    #[test]
    fn test_ai_capture_hide_settle_duration_matches_constant() {
        assert_eq!(
            ai_capture_hide_settle_duration(),
            std::time::Duration::from_millis(AI_CAPTURE_HIDE_SETTLE_MS)
        );
    }

    #[test]
    fn test_ai_capture_hide_settle_duration_waits_150ms() {
        assert_eq!(AI_CAPTURE_HIDE_SETTLE_MS, 150);
        assert_eq!(
            ai_capture_hide_settle_duration(),
            std::time::Duration::from_millis(150)
        );
    }

    #[test]
    fn test_emoji_picker_label_includes_emoji_and_name() {
        let emoji = Emoji {
            emoji: "🚀",
            name: "rocket",
            keywords: &["launch", "ship"],
            category: EmojiCategory::TravelPlaces,
        };

        assert_eq!(emoji_picker_label(&emoji), "🚀  rocket");
    }


    #[test]
    fn test_created_file_path_for_feedback_returns_same_path_when_already_absolute() {
        let absolute_path = PathBuf::from("/tmp/new-script.ts");
        let feedback_path = created_file_path_for_feedback(&absolute_path);

        assert_eq!(feedback_path, absolute_path);
    }

    #[test]
    fn test_created_file_path_for_feedback_joins_current_dir_when_relative() {
        let relative_path = PathBuf::from("new-script.ts");
        let current_dir = std::env::current_dir().expect("current dir should be available");
        let feedback_path = created_file_path_for_feedback(&relative_path);

        assert_eq!(feedback_path, current_dir.join(relative_path));
    }
}
