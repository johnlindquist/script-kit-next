/// Delay between hiding the main window and starting a synchronous screenshot capture.
const AI_CAPTURE_HIDE_SETTLE_MS: u64 = 150;

#[cfg(test)]
fn ai_open_failure_message(error: impl std::fmt::Display) -> String {
    format!("Failed to open AI: {}", error)
}

fn ai_capture_hide_settle_duration() -> std::time::Duration {
    std::time::Duration::from_millis(AI_CAPTURE_HIDE_SETTLE_MS)
}

fn ai_command_uses_hide_then_capture_flow(cmd_type: &builtins::AiCommandType) -> bool {
    matches!(
        cmd_type,
        builtins::AiCommandType::SendScreenToAi | builtins::AiCommandType::SendFocusedWindowToAi
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
    fn spawn_send_screen_to_ai_after_hide(&mut self, cx: &mut Context<Self>) {
        tracing::info!(
            action = "send_screen_to_ai_scheduled",
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
                            action = "send_screen_to_ai_rejected",
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
                        action = "send_screen_to_ai_captured",
                        width,
                        height,
                        size_bytes,
                        "Screen captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        this.open_ai_window_after_already_hidden(
                            "SendScreenToAi",
                            "send_screen_to_ai",
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            "Sent to AI",
                            cx,
                        );
                    })
                    .ok();
                }
                Err(error) => {
                    tracing::error!(
                        action = "send_screen_to_ai_capture_failed",
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

    fn spawn_send_focused_window_to_ai_after_hide(&mut self, cx: &mut Context<Self>) {
        tracing::info!(
            action = "send_focused_window_to_ai_scheduled",
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
                            action = "send_focused_window_to_ai_rejected",
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
                        action = "send_focused_window_to_ai_captured",
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
                            "send_focused_window_to_ai",
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            "Sent to AI",
                            cx,
                        );
                    })
                    .ok();
                }
                Err(error) => {
                    tracing::error!(
                        action = "send_focused_window_to_ai_capture_failed",
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
                    "AI Chat opened",
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

                let is_generate_script = matches!(cmd_type, AiCommandType::GenerateScript);
                let uses_hide_then_capture_flow = ai_command_uses_hide_then_capture_flow(cmd_type);
                if !is_generate_script {
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
                            "AI Chat opened",
                            cx,
                        );
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
                                    "Cleared AI conversations",
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

                    AiCommandType::SendScreenToAi => {
                        self.spawn_send_screen_to_ai_after_hide(cx);
                        Self::builtin_success(dctx, "ai_send_screen_dispatched")
                    }

                    AiCommandType::SendFocusedWindowToAi => {
                        self.spawn_send_focused_window_to_ai_after_hide(cx);
                        Self::builtin_success(dctx, "ai_send_focused_window_dispatched")
                    }

                    AiCommandType::SendSelectedTextToAi => {
                        match crate::selected_text::get_selected_text() {
                            Ok(text) if !text.is_empty() => {
                                let message = format!(
                                    "I've selected the following text:\n\n```\n{}\n```\n\nPlease help me with this.",
                                    text
                                );
                                tracing::info!(
                                    trace_id = %dctx.trace_id,
                                    text_len = text.len(),
                                    "Selected text captured"
                                );
                                self.open_ai_window_after_already_hidden(
                                    "SendSelectedTextToAi",
                                    &dctx.trace_id,
                                    DeferredAiWindowAction::SetInput {
                                        text: message,
                                        submit: false,
                                    },
                                    "Sent to AI",
                                    cx,
                                );
                                Self::builtin_success(dctx, "ai_send_selected_text")
                            }
                            Ok(_) => {
                                self.toast_manager.push(
                                    components::toast::Toast::info(
                                        "No text selected. Select some text first.",
                                        &self.theme,
                                    )
                                    .duration_ms(Some(TOAST_INFO_MS)),
                                );
                                cx.notify();
                                Self::builtin_success(dctx, "ai_send_selected_text_empty")
                            }
                            Err(e) => {
                                let message = format!("Failed to get selected text: {}", e);
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "ai_send_selected_text_failed",
                                )
                            }
                        }
                    }

                    AiCommandType::SendBrowserTabToAi => {
                        match platform::get_focused_browser_tab_url() {
                            Ok(url) => {
                                let message = format!(
                                    "I'm looking at this webpage:\n\n{}\n\nPlease help me analyze or understand its content.",
                                    url
                                );
                                tracing::info!(
                                    trace_id = %dctx.trace_id,
                                    "Browser URL captured"
                                );
                                self.open_ai_window_after_already_hidden(
                                    "SendBrowserTabToAi",
                                    &dctx.trace_id,
                                    DeferredAiWindowAction::SetInput {
                                        text: message,
                                        submit: false,
                                    },
                                    "Sent to AI",
                                    cx,
                                );
                                Self::builtin_success(dctx, "ai_send_browser_tab")
                            }
                            Err(e) => {
                                let message = format!("Failed to get browser URL: {}", e);
                                self.show_error_toast(message.clone(), cx);
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "ai_send_browser_tab_failed",
                                )
                            }
                        }
                    }

                    AiCommandType::SendScreenAreaToAi => {
                        match platform::capture_screen_area() {
                            Ok(Some(capture)) => {
                                let base64_data = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    &capture.png_data,
                                );
                                let message = format!(
                                    "[Screen area captured: {}x{} pixels]\n\nPlease analyze this selected screen area.",
                                    capture.width, capture.height
                                );
                                tracing::info!(
                                    action = "send_screen_area_to_ai",
                                    trace_id = %dctx.trace_id,
                                    width = capture.width,
                                    height = capture.height,
                                    file_size = capture.png_data.len(),
                                    "Screen area captured, sending to AI"
                                );
                                self.open_ai_window_after_already_hidden(
                                    "SendScreenAreaToAi",
                                    &dctx.trace_id,
                                    DeferredAiWindowAction::SetInputWithImage {
                                        text: message,
                                        image_base64: base64_data,
                                        submit: false,
                                    },
                                    "Sent to AI",
                                    cx,
                                );
                                Self::builtin_success(dctx, "ai_send_screen_area")
                            }
                            Ok(None) => {
                                tracing::info!(
                                    action = "send_screen_area_cancelled",
                                    trace_id = %dctx.trace_id,
                                    "Screen area selection cancelled by user"
                                );
                                cx.notify();
                                crate::action_helpers::DispatchOutcome::cancelled()
                                    .with_trace_id(dctx.trace_id.clone())
                                    .with_detail("ai_send_screen_area_cancelled")
                            }
                            Err(e) => {
                                let message = format!("Failed to capture screen area: {}", e);
                                self.show_error_toast(message.clone(), cx);
                                cx.notify();
                                Self::builtin_error(
                                    dctx,
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    message,
                                    "ai_send_screen_area_failed",
                                )
                            }
                        }
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
                }
            }

            // =========================================================================
            // Utility Commands (Scratch Pad, Quick Terminal, Process Manager)
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
}

#[cfg(test)]
mod builtin_execution_ai_feedback_tests {
    use super::{
        AI_CAPTURE_HIDE_SETTLE_MS, ai_capture_hide_settle_duration,
        ai_command_uses_hide_then_capture_flow, ai_open_failure_message,
        created_file_path_for_feedback, emoji_picker_label, favorites_loaded_message,
    };
    use crate::builtins::AiCommandType;
    use script_kit_gpui::emoji::{Emoji, EmojiCategory};
    use std::path::PathBuf;

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
            &AiCommandType::SendScreenToAi
        ));
        assert!(ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendFocusedWindowToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendScreenAreaToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendSelectedTextToAi
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
