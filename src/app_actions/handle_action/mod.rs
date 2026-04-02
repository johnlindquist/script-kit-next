use crate::action_helpers::{ActionOutcomeStatus, DispatchContext, DispatchOutcome};

/// A code block extracted from markdown with optional language hint.
struct CodeBlock {
    code: String,
    language: Option<String>,
}

/// Extract the last fenced code block (```lang\n...\n```) from markdown text.
fn extract_last_code_block(text: &str) -> Option<String> {
    extract_last_code_block_with_lang(text).map(|b| b.code)
}

/// Extract the last fenced code block with language hint.
fn extract_last_code_block_with_lang(text: &str) -> Option<CodeBlock> {
    let mut last_block: Option<CodeBlock> = None;
    let mut in_block = false;
    let mut current_code = String::new();
    let mut current_lang: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if in_block {
                last_block = Some(CodeBlock {
                    code: current_code.clone(),
                    language: current_lang.clone(),
                });
                current_code.clear();
                current_lang = None;
                in_block = false;
            } else {
                in_block = true;
                current_code.clear();
                // Parse language from ```typescript or ```ts etc.
                let lang = trimmed[3..].trim();
                current_lang = if lang.is_empty() {
                    None
                } else {
                    Some(lang.to_string())
                };
            }
        } else if in_block {
            if !current_code.is_empty() {
                current_code.push('\n');
            }
            current_code.push_str(line);
        }
    }
    last_block
}

// Action dispatch facade.
//
// This module splits the monolithic action handler into semantic submodules:
//   - clipboard.rs:  all clipboard_* actions
//   - scripts.rs:    script management (create, edit, remove, settings, quit)
//   - shortcuts.rs:  shortcut and alias configuration
//   - files.rs:      file search, reveal, copy path/deeplink
//   - scriptlets.rs: scriptlet editing, reveal, and dynamic actions

/// Maximum number of clipboard entries to cache for the clipboard history view.
const CLIPBOARD_CACHE_SIZE: usize = 100;

enum DeferredAiWindowAction {
    OpenOnly,
    SetInput { text: String, submit: bool },
    SetInputWithImage { text: String, image_base64: String, submit: bool },
    AddAttachment { path: String },
    ApplyPreset { preset_id: String },
}

impl DeferredAiWindowAction {
    fn name(&self) -> &'static str {
        match self {
            Self::OpenOnly => "open_only",
            Self::SetInput { submit: true, .. } => "set_input_submit",
            Self::SetInput { submit: false, .. } => "set_input",
            Self::SetInputWithImage { submit: true, .. } => "set_input_with_image_submit",
            Self::SetInputWithImage { submit: false, .. } => "set_input_with_image",
            Self::AddAttachment { .. } => "add_attachment",
            Self::ApplyPreset { .. } => "apply_preset",
        }
    }

    fn apply(self, cx: &mut App) -> Result<&'static str, String> {
        match self {
            Self::OpenOnly => Ok("open_only"),
            Self::SetInput { text, submit } => {
                ai::set_ai_input(cx, &text, submit)?;
                Ok("set_input")
            }
            Self::SetInputWithImage { text, image_base64, submit } => {
                ai::set_ai_input_with_image(cx, &text, &image_base64, submit)?;
                Ok("set_input_with_image")
            }
            Self::AddAttachment { path } => {
                ai::add_ai_attachment(cx, &path)?;
                Ok("add_attachment")
            }
            Self::ApplyPreset { preset_id } => {
                ai::apply_ai_preset(cx, &preset_id);
                Ok("apply_preset")
            }
        }
    }
}

impl ScriptListApp {
    /// Show an error toast and call cx.notify() to ensure the UI updates.
    ///
    /// Consolidates the repeated pattern of pushing an error toast, setting the
    /// duration to TOAST_ERROR_MS, and calling cx.notify().
    ///
    /// The optional `error_code` is logged for machine-readable diagnostics but
    /// never shown to the user.  Use the stable constants from
    /// `crate::action_helpers` (e.g. `ERROR_LAUNCH_FAILED`).
    fn show_error_toast(
        &mut self,
        message: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        self.show_error_toast_with_code(message, None, cx);
    }

    /// Like `show_error_toast` but also logs a stable error code.
    fn show_error_toast_with_code(
        &mut self,
        message: impl Into<String>,
        error_code: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let msg: String = message.into();
        if let Some(code) = error_code {
            tracing::warn!(
                error_code = code,
                message = %msg,
                "Action error"
            );
        }
        self.toast_manager.push(
            components::toast::Toast::error(msg, &self.theme)
                .duration_ms(Some(TOAST_ERROR_MS)),
        );
        cx.notify();
    }

    /// Copy text to the system clipboard with consistent success/error feedback.
    ///
    /// On success, shows a HUD with the given message and optionally hides the
    /// main window. On failure, shows an error toast.
    fn copy_to_clipboard_with_feedback(
        &mut self,
        text: &str,
        success_message: String,
        close_after: bool,
        cx: &mut Context<Self>,
    ) {
        let copy_result = {
            #[cfg(target_os = "macos")]
            {
                self.pbcopy(text)
                    .map_err(|e| format!("Clipboard write failed: {}", e))
            }

            #[cfg(not(target_os = "macos"))]
            {
                use arboard::Clipboard;
                Clipboard::new()
                    .and_then(|mut c| c.set_text(text))
                    .map_err(|e| format!("Clipboard write failed: {}", e))
            }
        };

        match copy_result {
            Ok(()) => {
                self.show_hud(success_message, Some(HUD_MEDIUM_MS), cx);
                if close_after {
                    self.hide_main_and_reset(cx);
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Clipboard write failed");
                self.show_error_toast("Failed to copy to clipboard", cx);
            }
        }
    }

    /// Show a consistent "not supported on this platform" warning toast.
    ///
    /// Uses Toast::warning (not error) per the feedback matrix — unsupported
    /// platform is a warning, not an error.  Internally logs with the
    /// `unsupported_platform` error code.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    fn show_unsupported_platform_toast(
        &mut self,
        feature: &str,
        cx: &mut Context<Self>,
    ) {
        tracing::warn!(
            error_code = crate::action_helpers::ERROR_UNSUPPORTED_PLATFORM,
            feature = feature,
            "Unsupported platform"
        );
        self.toast_manager.push(
            components::toast::Toast::warning(
                unsupported_platform_message(feature),
                &self.theme,
            )
            .duration_ms(Some(TOAST_WARNING_MS)),
        );
        cx.notify();
    }

    pub(crate) fn hide_main_and_reset(&self, cx: &mut Context<Self>) {
        if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
            let bounds = crate::window_state::PersistedWindowBounds::new(x, y, w, h);
            let displays = platform::get_macos_displays();
            let _ =
                crate::window_state::save_main_position_with_display_detection(bounds, &displays);
        }
        set_main_window_visible(false);
        NEEDS_RESET.store(true, Ordering::SeqCst);
        // Use deferred platform-specific hide that only hides the main window,
        // not the entire app (cx.hide() would hide HUD too).
        // Must be deferred to avoid RefCell reentrancy from macOS callbacks.
        platform::defer_hide_main_window(cx);
    }

    fn open_ai_window_after_main_hide(
        &mut self,
        source_action: &str,
        trace_id: &str,
        deferred_action: DeferredAiWindowAction,
        cx: &mut Context<Self>,
    ) {
        self.hide_main_and_reset(cx);
        self.open_ai_window_after_already_hidden(
            source_action,
            trace_id,
            deferred_action,
            cx,
        );
    }

    fn open_ai_window_after_already_hidden(
        &mut self,
        source_action: &str,
        trace_id: &str,
        deferred_action: DeferredAiWindowAction,
        cx: &mut Context<Self>,
    ) {
        let source_action = source_action.to_string();
        let trace_id = trace_id.to_string();
        let deferred_action_name = deferred_action.name();

        tracing::info!(
            category = "AI",
            event = "ai_handoff_defer_open_start",
            source_action = %source_action,
            trace_id = %trace_id,
            deferred_action = deferred_action_name,
            "Opening AI window after main window already hidden"
        );

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;

            let started_at = std::time::Instant::now();

            let open_result = cx.update(|cx| {
                ai::open_ai_window(cx).map_err(|error| error.to_string())?;
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
                    deferred_action.apply(cx)
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
                            deferred_action = deferred_action_name,
                            apply_stage,
                            duration_ms = started_at.elapsed().as_millis() as u64,
                            "AI handoff completed"
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
                            deferred_action = deferred_action_name,
                            error = %error,
                            duration_ms = started_at.elapsed().as_millis() as u64,
                            "Failed to open AI window after hiding main window"
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
    }

    /// Reveal a path and return completion back to the UI thread for HUD feedback.
    fn reveal_in_finder_with_feedback_async(
        &self,
        path: &std::path::Path,
        trace_id: &str,
    ) -> async_channel::Receiver<Result<(), String>> {
        let path_str = path.to_string_lossy().to_string();
        let trace_id = trace_id.to_string();
        let (result_tx, result_rx) = async_channel::bounded::<Result<(), String>>(1);

        std::thread::spawn(move || {
            let file_manager = if cfg!(target_os = "macos") {
                "Finder"
            } else if cfg!(target_os = "windows") {
                "Explorer"
            } else {
                "File Manager"
            };

            tracing::info!(
                category = "UI",
                event = "action_reveal_in_finder_start",
                trace_id = %trace_id,
                file_manager,
                path = %path_str,
                "Reveal in file manager started"
            );

            let reveal_result = match crate::file_search::reveal_in_finder(&path_str) {
                Ok(()) => {
                    tracing::info!(
                        category = "UI",
                        event = "action_reveal_in_finder_success",
                        trace_id = %trace_id,
                        file_manager,
                        path = %path_str,
                        "Reveal in file manager succeeded"
                    );
                    Ok(())
                }
                Err(error) => {
                    tracing::error!(
                        event = "action_reveal_in_finder_failed",
                        attempted = "reveal_in_finder",
                        trace_id = %trace_id,
                        file_manager,
                        path = %path_str,
                        error = %error,
                        "Reveal in file manager failed"
                    );
                    Err(format!("Failed to reveal in {}: {}", file_manager, error))
                }
            };

            let _ = result_tx.send_blocking(reveal_result);
        });

        result_rx
    }

    /// Launch the configured editor and return completion back to the UI thread for HUD feedback.
    fn launch_editor_with_feedback_async(
        &self,
        path: &std::path::Path,
        trace_id: &str,
    ) -> async_channel::Receiver<Result<(), String>> {
        let editor = self.config.get_editor();
        let path_str = path.to_string_lossy().to_string();
        let trace_id = trace_id.to_string();
        let (result_tx, result_rx) = async_channel::bounded::<Result<(), String>>(1);

        std::thread::spawn(move || {
            use std::process::Command;

            tracing::info!(
                category = "UI",
                event = "action_editor_launch_start",
                trace_id = %trace_id,
                editor = %editor,
                path = %path_str,
                "Editor launch started"
            );

            let launch_result = match Command::new(&editor).arg(&path_str).spawn() {
                Ok(_) => {
                    tracing::info!(
                        category = "UI",
                        event = "action_editor_launch_success",
                        trace_id = %trace_id,
                        editor = %editor,
                        path = %path_str,
                        "Editor launch succeeded"
                    );
                    Ok(())
                }
                Err(error) => {
                    tracing::error!(
                        event = "action_editor_launch_failed",
                        attempted = "launch_editor",
                        trace_id = %trace_id,
                        editor = %editor,
                        path = %path_str,
                        error = %error,
                        "Editor launch failed"
                    );
                    Err(format!("Failed to open in {}: {}", editor, error))
                }
            };

            let _ = result_tx.send_blocking(launch_result);
        });

        result_rx
    }

    /// Copy text to clipboard using pbcopy on macOS.
    /// Critical: This properly closes stdin before waiting to prevent hangs.
    #[cfg(target_os = "macos")]
    fn pbcopy(&self, text: &str) -> Result<(), std::io::Error> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

        // Take ownership of stdin, write, then drop to signal EOF
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
            // stdin is dropped here => EOF delivered to pbcopy
        }

        // Now it's safe to wait - pbcopy has received EOF
        let status = child.wait()?;
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "pbcopy exited with status: {}",
                status
            )));
        }
        Ok(())
    }

    /// Return the currently selected clipboard entry metadata when in ClipboardHistoryView.
    fn selected_clipboard_entry(&self) -> Option<clipboard_history::ClipboardEntryMeta> {
        if let Some(ref entry_id) = self.focused_clipboard_entry_id {
            if let Some(entry) = self
                .cached_clipboard_entries
                .iter()
                .find(|entry| &entry.id == entry_id)
            {
                return Some(entry.clone());
            }
        }

        let AppView::ClipboardHistoryView {
            filter,
            selected_index,
        } = &self.current_view
        else {
            return None;
        };

        select_clipboard_entry_meta(&self.cached_clipboard_entries, filter, *selected_index)
            .cloned()
    }

    /// Return true when the current view has any available actions.
    fn has_actions(&mut self) -> bool {
        match &self.current_view {
            AppView::AcpChatView { .. } => true,
            AppView::ClipboardHistoryView { .. } => {
                let has = self.selected_clipboard_entry().is_some();
                tracing::debug!(
                    event = "has_actions.clipboard",
                    has_selected_entry = has,
                    "has_actions (clipboard)",
                );
                has
            }
            _ => {
                let script_info = self.get_focused_script_info();
                let has_script_info = script_info.is_some();
                let script_name = script_info
                    .as_ref()
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let mut actions = Vec::new();

                if let Some(ref script) = script_info {
                    if script.is_scriptlet {
                        actions.extend(crate::actions::get_scriptlet_context_actions_with_custom(
                            script, None,
                        ));
                    } else {
                        actions.extend(crate::actions::get_script_context_actions(script));
                    }
                }

                let global_count_before = actions.len();
                actions.extend(crate::actions::get_global_actions());
                let result = !actions.is_empty();
                tracing::debug!(
                    event = "has_actions.check",
                    has_script_info = has_script_info,
                    script_name = %script_name,
                    script_actions = global_count_before,
                    total_actions = actions.len(),
                    result = result,
                    selected_index = self.selected_index,
                    "has_actions: script_info={}", has_script_info,
                );
                result
            }
        }
    }

    /// Return to script list after non-inline action handling.
    ///
    /// Centralizes state transition so actions don't directly mutate legacy
    /// focus fields (`pending_focus`) in multiple places.
    fn transition_to_script_list_after_action(&mut self, cx: &mut Context<Self>) {
        self.current_view = AppView::ScriptList;
        self.request_focus(FocusTarget::MainFilter, cx);
    }

    /// Simple percent-encoding for URL query strings.
    fn percent_encode_for_url(&self, input: &str) -> String {
        let mut encoded = String::with_capacity(input.len() * 3);
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    encoded.push(byte as char);
                }
                b' ' => encoded.push_str("%20"),
                _ => {
                    encoded.push('%');
                    encoded.push_str(&format!("{:02X}", byte));
                }
            }
        }
        encoded
    }

    /// Derive user-facing toast feedback from a `DispatchOutcome` at the
    /// dispatch boundary.
    ///
    /// Shows an error toast when the outcome carries an error with a
    /// user-facing message.  Success, NoEffect, and Cancelled outcomes
    /// produce no feedback here — success HUDs are the handler's
    /// responsibility since only the handler knows the right message.
    fn show_outcome_feedback(
        &mut self,
        outcome: &DispatchOutcome,
        cx: &mut Context<Self>,
    ) {
        if outcome.status == ActionOutcomeStatus::Error {
            if let Some(ref msg) = outcome.user_message {
                self.show_error_toast_with_code(
                    msg.clone(),
                    outcome.error_code,
                    cx,
                );
            }
        }
    }

    /// Handle action selection from the actions dialog
    fn handle_acp_chat_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let AppView::AcpChatView { ref entity } = self.current_view else {
            return DispatchOutcome::not_handled();
        };

        match action_id {
            "acp_copy_last_response" => {
                let entity = entity.clone();
                let last_response = entity.read(cx).thread.read(cx).messages.iter().rev().find(
                    |msg| {
                        matches!(
                            msg.role,
                            crate::ai::acp::thread::AcpThreadMessageRole::Assistant
                        )
                    },
                ).map(|msg| msg.body.to_string());

                if let Some(text) = last_response {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message =
                        Some("Copied last response to clipboard".to_string());
                    outcome
                } else {
                    DispatchOutcome::not_handled()
                }
            }
            "acp_new_conversation" => {
                // Clear messages but keep the session alive
                let entity = entity.clone();
                entity.update(cx, |chat, cx| {
                    chat.thread.update(cx, |thread, cx| {
                        thread.clear_messages(cx);
                    });
                    chat.collapsed_ids.clear();
                    cx.notify();
                });
                DispatchOutcome::success()
            }
            "acp_clear_conversation" => {
                // Close and reopen the ACP chat for a fresh session
                self.close_tab_ai_harness_terminal(cx);
                self.open_tab_ai_chat(cx);
                DispatchOutcome::success()
            }
            "acp_paste_to_frontmost" => {
                let last_response = entity.read(cx).thread.read(cx).messages.iter().rev().find(
                    |msg| {
                        matches!(
                            msg.role,
                            crate::ai::acp::thread::AcpThreadMessageRole::Assistant
                        )
                    },
                ).map(|msg| msg.body.to_string());

                if let Some(text) = last_response {
                    // Hide the window so the frontmost app regains focus
                    crate::platform::defer_hide_main_window(cx);
                    // Spawn a background thread to paste after a short delay
                    let text_for_paste = text.clone();
                    std::thread::spawn(move || {
                        // Small delay to let the frontmost app regain focus
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        let injector = crate::text_injector::TextInjector::new();
                        if let Err(e) = injector.paste_text(&text_for_paste) {
                            tracing::warn!(%e, "acp_paste_to_frontmost_failed");
                        }
                    });
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message = Some("Pasting to frontmost app\u{2026}".to_string());
                    outcome
                } else {
                    DispatchOutcome::not_handled()
                }
            }
            "acp_copy_all_code" => {
                let entity = entity.clone();
                let messages = &entity.read(cx).thread.read(cx).messages;
                let mut all_code = String::new();
                for msg in messages {
                    if matches!(
                        msg.role,
                        crate::ai::acp::thread::AcpThreadMessageRole::Assistant
                    ) {
                        // Extract all code blocks from this message
                        let mut in_block = false;
                        let mut current = String::new();
                        for line in msg.body.lines() {
                            if line.trim_start().starts_with("```") {
                                if in_block {
                                    if !current.is_empty() {
                                        if !all_code.is_empty() {
                                            all_code.push_str("\n\n");
                                        }
                                        all_code.push_str(&current);
                                    }
                                    current.clear();
                                    in_block = false;
                                } else {
                                    in_block = true;
                                    current.clear();
                                }
                            } else if in_block {
                                if !current.is_empty() {
                                    current.push('\n');
                                }
                                current.push_str(line);
                            }
                        }
                    }
                }
                if all_code.is_empty() {
                    let mut o = DispatchOutcome::success();
                    o.user_message = Some("No code blocks found".to_string());
                    o
                } else {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(all_code));
                    let mut o = DispatchOutcome::success();
                    o.user_message = Some("All code blocks copied".to_string());
                    o
                }
            }
            "acp_retry_last" => {
                let entity = entity.clone();
                let last_user_msg = entity
                    .read(cx)
                    .thread
                    .read(cx)
                    .messages
                    .iter()
                    .rev()
                    .find(|m| {
                        matches!(
                            m.role,
                            crate::ai::acp::thread::AcpThreadMessageRole::User
                        )
                    })
                    .map(|m| m.body.to_string());

                if let Some(text) = last_user_msg {
                    entity.update(cx, |chat, cx| {
                        chat.thread.update(cx, |thread, cx| {
                            thread.set_input(text, cx);
                            let _ = thread.submit_input(cx);
                        });
                    });
                    DispatchOutcome::success()
                } else {
                    let mut o = DispatchOutcome::success();
                    o.user_message = Some("No previous message to retry".to_string());
                    o
                }
            }
            "acp_save_as_script" => {
                let entity = entity.clone();
                let last_response = entity
                    .read(cx)
                    .thread
                    .read(cx)
                    .messages
                    .iter()
                    .rev()
                    .find(|m| {
                        matches!(
                            m.role,
                            crate::ai::acp::thread::AcpThreadMessageRole::Assistant
                        )
                    })
                    .map(|m| m.body.to_string());

                if let Some(text) = last_response {
                    let block = extract_last_code_block_with_lang(&text);
                    if let Some(block) = block {
                        let code = block.code;
                        let ext = match block.language.as_deref() {
                            Some("typescript" | "ts") => "ts",
                            Some("javascript" | "js") => "js",
                            Some("python" | "py") => "py",
                            Some("rust" | "rs") => "rs",
                            Some("bash" | "sh" | "zsh") => "sh",
                            _ => "ts", // Default to TypeScript for Script Kit
                        };
                        // Generate a script name from the first line
                        let name = code
                            .lines()
                            .find(|l| !l.trim().is_empty())
                            .and_then(|l| {
                                let trimmed = l.trim().trim_start_matches("//").trim();
                                if trimmed.len() > 3 && trimmed.len() < 50 {
                                    Some(
                                        trimmed
                                            .to_lowercase()
                                            .replace(' ', "-")
                                            .chars()
                                            .filter(|c| c.is_alphanumeric() || *c == '-')
                                            .collect::<String>(),
                                    )
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| {
                                format!(
                                    "ai-script-{}",
                                    chrono::Utc::now().format("%H%M%S")
                                )
                            });

                        let path = crate::setup::get_kit_path()
                            .join("kit")
                            .join("main")
                            .join("scripts")
                            .join(format!("{name}.{ext}"));

                        if let Err(e) = std::fs::write(&path, &code) {
                            tracing::warn!(%e, "acp_save_as_script_failed");
                        } else {
                            let mut o = DispatchOutcome::success();
                            o.user_message =
                                Some(format!("Saved as {name}.{ext}"));
                            return o;
                        }
                    }
                }
                let mut o = DispatchOutcome::success();
                o.user_message =
                    Some("No code block found in last response".to_string());
                o
            }
            "acp_run_last_code" => {
                let entity = entity.clone();
                let last_response = entity
                    .read(cx)
                    .thread
                    .read(cx)
                    .messages
                    .iter()
                    .rev()
                    .find(|m| {
                        matches!(
                            m.role,
                            crate::ai::acp::thread::AcpThreadMessageRole::Assistant
                        )
                    })
                    .map(|m| m.body.to_string());

                if let Some(text) = last_response {
                    if let Some(block) = extract_last_code_block_with_lang(&text) {
                        let lang = block
                            .language
                            .as_deref()
                            .unwrap_or("typescript")
                            .to_lowercase();

                        // Write to temp file
                        let ext = match lang.as_str() {
                            "typescript" | "ts" => "ts",
                            "javascript" | "js" => "js",
                            "python" | "py" => "py",
                            "bash" | "sh" | "zsh" | "shell" => "sh",
                            _ => "ts",
                        };
                        let name = format!("ai-run-{}.{ext}", chrono::Utc::now().format("%H%M%S"));
                        let tmp_dir = std::env::temp_dir().join("scriptkit-runs");
                        let _ = std::fs::create_dir_all(&tmp_dir);
                        let path = tmp_dir.join(&name);

                        if let Err(e) = std::fs::write(&path, &block.code) {
                            tracing::warn!(%e, "acp_run_last_code_write_failed");
                            let mut o = DispatchOutcome::success();
                            o.user_message = Some(format!("Failed to write temp file: {e}"));
                            return o;
                        }

                        // Pick the runner
                        let path_str = path.to_string_lossy().to_string();
                        let (cmd, args): (&str, Vec<String>) = match ext {
                            "ts" => ("bun", vec!["run".into(), path_str.clone()]),
                            "js" => ("node", vec![path_str.clone()]),
                            "py" => ("python3", vec![path_str.clone()]),
                            "sh" => ("bash", vec![path_str.clone()]),
                            _ => ("bun", vec!["run".into(), path_str.clone()]),
                        };
                        let cmd = cmd.to_string();

                        // Show "running..." message immediately
                        let thread = entity.read(cx).thread.clone();
                        thread.update(cx, |t, cx| {
                            t.push_system_message(
                                format!("Running `{name}`..."),
                                cx,
                            );
                        });

                        // Spawn async execution to avoid blocking the UI
                        let thread_for_result = thread.clone();
                        let path_clone = path.clone();
                        cx.spawn(async move |_this, cx| {
                            let result = cx
                                .background_executor()
                                .spawn(async move {
                                    std::process::Command::new(&cmd)
                                        .args(&args)
                                        .current_dir(std::env::temp_dir())
                                        .output()
                                })
                                .await;

                            // Clean up temp file
                            let _ = std::fs::remove_file(&path_clone);

                            let message = match result {
                                Ok(output) => {
                                    let stdout =
                                        String::from_utf8_lossy(&output.stdout).trim().to_string();
                                    let stderr =
                                        String::from_utf8_lossy(&output.stderr).trim().to_string();
                                    if output.status.success() {
                                        if stdout.is_empty() {
                                            "Finished (no output)".to_string()
                                        } else {
                                            format!("```\n{stdout}\n```")
                                        }
                                    } else {
                                        let out = if stderr.is_empty() {
                                            stdout
                                        } else {
                                            stderr
                                        };
                                        format!("Error (exit {}):\n```\n{out}\n```", output.status)
                                    }
                                }
                                Err(e) => format!("Failed to run: {e}"),
                            };

                            let _ = cx.update(|cx| {
                                thread_for_result.update(cx, |t, cx| {
                                    t.push_system_message(message, cx);
                                });
                            });
                        })
                        .detach();

                        return DispatchOutcome::success();
                    }
                }
                let mut o = DispatchOutcome::success();
                o.user_message = Some("No code block found".to_string());
                o
            }
            "acp_open_in_editor" => {
                let kit_path = crate::setup::get_kit_path();
                if let Err(e) = open::that(&kit_path) {
                    tracing::warn!(%e, "acp_open_in_editor_failed");
                }
                DispatchOutcome::success()
            }
            "acp_export_markdown" => {
                let entity = entity.clone();
                let messages = &entity.read(cx).thread.read(cx).messages;
                let mut md = String::from("# AI Chat Conversation\n\n");
                for msg in messages {
                    let role_label = match msg.role {
                        crate::ai::acp::thread::AcpThreadMessageRole::User => "**You**",
                        crate::ai::acp::thread::AcpThreadMessageRole::Assistant => "**Claude Code**",
                        crate::ai::acp::thread::AcpThreadMessageRole::Thought => "**Thinking**",
                        crate::ai::acp::thread::AcpThreadMessageRole::Tool => "**Tool**",
                        crate::ai::acp::thread::AcpThreadMessageRole::System => "**System**",
                        crate::ai::acp::thread::AcpThreadMessageRole::Error => "**Error**",
                    };
                    md.push_str(&format!("{role_label}\n\n{}\n\n---\n\n", msg.body));
                }
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(md));
                let mut outcome = DispatchOutcome::success();
                outcome.user_message =
                    Some("Conversation copied as markdown".to_string());
                outcome
            }
            "acp_show_history" => {
                let entries = crate::ai::acp::history::load_history();
                if entries.is_empty() {
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message = Some("No conversation history yet".to_string());
                    outcome
                } else {
                    // Format as markdown and copy to clipboard
                    let mut text = String::from("# Recent AI Conversations\n\n");
                    for (i, entry) in entries.iter().take(20).enumerate() {
                        let date = entry
                            .timestamp
                            .split('T')
                            .next()
                            .unwrap_or(&entry.timestamp);
                        text.push_str(&format!(
                            "{}. **{}** — {} messages, {}\n",
                            i + 1,
                            entry.first_message,
                            entry.message_count,
                            date,
                        ));
                    }
                    text.push_str(&format!(
                        "\n_Conversations saved in ~/.scriptkit/acp-conversations/_"
                    ));
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message = Some(format!(
                        "{} conversations copied to clipboard",
                        entries.len().min(20)
                    ));
                    outcome
                }
            }
            "acp_clear_history" => {
                // Delete history index and conversations directory
                let kit = crate::setup::get_kit_path();
                let _ = std::fs::remove_file(kit.join("acp-history.jsonl"));
                let _ = std::fs::remove_dir_all(kit.join("acp-conversations"));
                let mut o = DispatchOutcome::success();
                o.user_message = Some("Conversation history cleared".to_string());
                o
            }
            "acp_scroll_to_top" => {
                let entity = entity.clone();
                entity.update(cx, |chat, cx| {
                    chat.list_state.scroll_to(gpui::ListOffset {
                        item_ix: 0,
                        offset_in_item: gpui::px(0.),
                    });
                    cx.notify();
                });
                DispatchOutcome::success()
            }
            "acp_scroll_to_bottom" => {
                let entity = entity.clone();
                entity.update(cx, |chat, cx| {
                    chat.list_state.scroll_to_end();
                    cx.notify();
                });
                DispatchOutcome::success()
            }
            "acp_expand_all" => {
                let entity = entity.clone();
                entity.update(cx, |chat, cx| {
                    // Add all collapsible message IDs to collapsed_ids (which means expanded)
                    let ids: Vec<u64> = chat
                        .thread
                        .read(cx)
                        .messages
                        .iter()
                        .filter(|m| {
                            matches!(
                                m.role,
                                crate::ai::acp::thread::AcpThreadMessageRole::Thought
                                    | crate::ai::acp::thread::AcpThreadMessageRole::Tool
                            )
                        })
                        .map(|m| m.id)
                        .collect();
                    for id in ids {
                        chat.collapsed_ids.insert(id);
                    }
                    cx.notify();
                });
                DispatchOutcome::success()
            }
            "acp_collapse_all" => {
                let entity = entity.clone();
                entity.update(cx, |chat, cx| {
                    chat.collapsed_ids.clear();
                    cx.notify();
                });
                DispatchOutcome::success()
            }
            "acp_detach_window" => {
                let thread = entity.read(cx).thread.clone();
                let inherit_bounds = Some(window.bounds());
                tracing::info!(
                    event = "actions_detach_acp_requested",
                    has_inherited_bounds = true,
                );
                if let Err(e) =
                    crate::ai::acp::chat_window::open_chat_window_with_thread(
                        thread,
                        inherit_bounds,
                        cx,
                    )
                {
                    tracing::warn!(%e, "acp_detach_window_failed");
                    DispatchOutcome::success()
                } else {
                    self.close_acp_chat_to_script_list(false, cx);
                    let mut o = DispatchOutcome::success();
                    o.user_message = Some("Chat detached to window".to_string());
                    o
                }
            }
            "acp_reattach_panel" => {
                crate::ai::acp::chat_window::close_chat_window(cx);
                self.open_tab_ai_chat(cx);
                let mut o = DispatchOutcome::success();
                o.user_message = Some("Chat re-attached to panel".to_string());
                o
            }
            "acp_close" => {
                self.close_tab_ai_harness_terminal(cx);
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }

    fn handle_action(&mut self, action_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let start = std::time::Instant::now();

        let action_id_stripped = action_id
            .strip_prefix("clip:")
            .or_else(|| action_id.strip_prefix("file:"))
            .or_else(|| action_id.strip_prefix("chat:"))
            .unwrap_or(action_id.as_str())
            .to_string();

        let dctx = DispatchContext::for_action(&action_id_stripped);

        tracing::info!(
            category = "UI",
            action = %action_id_stripped,
            trace_id = %dctx.trace_id,
            surface = %dctx.surface,
            "Action dispatch started"
        );

        let should_transition_to_script_list =
            should_transition_to_script_list_after_action(&self.current_view);

        let selected_clipboard_entry = if action_id_stripped.starts_with("clipboard_") {
            self.selected_clipboard_entry()
        } else {
            None
        };
        // Clipboard actions handle their own transitions and notifications.
        let clipboard_outcome = self.handle_clipboard_action(&action_id_stripped, selected_clipboard_entry, &dctx, cx);
        if clipboard_outcome.was_handled() {
            log_dispatch_outcome(&action_id_stripped, &dctx.trace_id, "clipboard", &clipboard_outcome, &start);
            self.show_outcome_feedback(&clipboard_outcome, cx);
            return;
        }

        // Only script-list-hosted actions should force a ScriptList transition.
        if should_transition_to_script_list {
            self.transition_to_script_list_after_action(cx);
        }

        // Dispatch through handler chain, collecting the final outcome.
        let (handler, outcome) = {
            let o = self.handle_shortcut_alias_action(&action_id_stripped, &dctx, cx);
            if o.was_handled() {
                ("shortcut_alias", o)
            } else {
                let o = self.handle_script_action(&action_id_stripped, &dctx, window, cx);
                if o.was_handled() {
                    ("script", o)
                } else {
                    let o = self.handle_file_action(&action_id_stripped, &dctx, cx);
                    if o.was_handled() {
                        ("file", o)
                    } else {
                        let o = self.handle_scriptlet_action(&action_id_stripped, &dctx, cx);
                        if o.was_handled() {
                            ("scriptlet", o)
                        } else {
                            let o = self.handle_acp_chat_action(&action_id_stripped, window, cx);
                            if o.was_handled() {
                                ("acp_chat", o)
                            } else {
                                // SDK actions as final fallback — thread trace_id from dctx
                                ("sdk_fallback", self.trigger_sdk_action_with_trace(&action_id_stripped, &dctx.trace_id))
                            }
                        }
                    }
                }
            }
        };

        log_dispatch_outcome(&action_id_stripped, &dctx.trace_id, handler, &outcome, &start);
        self.show_outcome_feedback(&outcome, cx);
        cx.notify();
    }
}

/// Log structured outcome at the end of action dispatch.
fn log_dispatch_outcome(
    action_id: &str,
    trace_id: &str,
    handler: &str,
    outcome: &DispatchOutcome,
    start: &std::time::Instant,
) {
    tracing::info!(
        category = "UI",
        action = %action_id,
        trace_id = %trace_id,
        handler = handler,
        status = %outcome.status,
        error_code = outcome.error_code,
        duration_ms = start.elapsed().as_millis() as u64,
        "Action dispatch completed"
    );
}

// Include semantic submodules — each adds `impl ScriptListApp` methods.
include!("clipboard.rs");
include!("scripts.rs");
include!("shortcuts.rs");
include!("files.rs");
include!("scriptlets.rs");
