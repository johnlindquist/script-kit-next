/// Dispatch and execution of user-triggered actions (copy, paste, reveal, terminal, AI, etc.).
use crate::action_helpers::{ActionOutcomeStatus, DispatchContext, DispatchOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AsyncExternalToolFeedbackAction {
    RevealInFileManager,
    LaunchEditor,
}

impl AsyncExternalToolFeedbackAction {
    fn failure_message(self, tool_name: &str, error: impl std::fmt::Display) -> String {
        match self {
            Self::RevealInFileManager => format!("Failed to reveal in {tool_name}: {error}"),
            Self::LaunchEditor => format!("Failed to open in {tool_name}: {error}"),
        }
    }
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

include!("deferred_agent_chat.rs");

impl ScriptListApp {
    /// Show an error toast and call cx.notify() to ensure the UI updates.
    ///
    /// Consolidates the repeated pattern of pushing an error toast, setting the
    /// duration to TOAST_ERROR_MS, and calling cx.notify().
    ///
    /// The optional `error_code` is logged for machine-readable diagnostics but
    /// never shown to the user.  Use the stable constants from
    /// `crate::action_helpers` (e.g. `ERROR_LAUNCH_FAILED`).
    fn show_error_toast(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.show_error_toast_with_code(message, None, cx);
    }

    /// Show a short success toast in the main window.
    fn show_success_toast(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        let msg: String = message.into();
        self.toast_manager.push(
            components::toast::Toast::success(msg, &self.theme).duration_ms(Some(TOAST_SUCCESS_MS)),
        );
        cx.notify();
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
            components::toast::Toast::error(msg, &self.theme).duration_ms(Some(TOAST_ERROR_MS)),
        );
        cx.notify();
    }

    /// Copy application text with destination-accurate success/error feedback.
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
        match crate::platform::copy_text(text) {
            Ok(receipt) => {
                let message = match receipt.destination() {
                    crate::runtime_policy::CopyDestination::SystemClipboard => success_message,
                    crate::runtime_policy::CopyDestination::OwnedProcessLocal => {
                        receipt.feedback("Copied".to_owned())
                    }
                };
                self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                if close_after {
                    self.hide_main_and_reset(cx);
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Text copy failed");
                self.show_error_toast(format!("Failed to copy text: {e}"), cx);
            }
        }
    }

    /// Show a consistent "not supported on this platform" warning toast.
    ///
    /// Uses Toast::warning (not error) per the feedback matrix — unsupported
    /// platform is a warning, not an error.  Internally logs with the
    /// `unsupported_platform` error code.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    fn show_unsupported_platform_toast(&mut self, feature: &str, cx: &mut Context<Self>) {
        tracing::warn!(
            error_code = crate::action_helpers::ERROR_UNSUPPORTED_PLATFORM,
            feature = feature,
            "Unsupported platform"
        );
        self.toast_manager.push(
            components::toast::Toast::warning(unsupported_platform_message(feature), &self.theme)
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

    /// Hands an action's words, image or file to the model's chat in the main window.
    fn open_chat_for_action(
        &mut self,
        source_action: &str,
        trace_id: &str,
        action: DeferredAgentChatAction,
        cx: &mut Context<Self>,
    ) {
        let name = action.name();
        let failure = action.failure_message("");
        match action.entry() {
            Ok(entry) => {
                tracing::info!(
                    category = "AI",
                    event = "ai_handoff_to_chat",
                    source_action,
                    trace_id,
                    action = name,
                );
                self.one_window_open_chat(entry, cx);
            }
            Err(error) => {
                tracing::error!(
                    category = "AI",
                    event = "ai_handoff_to_chat_failed",
                    source_action,
                    trace_id,
                    action = name,
                    error = %error,
                );
                self.show_error_toast(format!("{failure}{error}"), cx);
            }
        }
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
            let feedback_action = AsyncExternalToolFeedbackAction::RevealInFileManager;

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
                    Err(feedback_action.failure_message(file_manager, error))
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
        let feedback_action = AsyncExternalToolFeedbackAction::LaunchEditor;
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
                    Err(feedback_action.failure_message(&editor, error))
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

    fn selected_dictation_history_entry(&self) -> Option<crate::dictation::DictationHistoryEntry> {
        let AppView::DictationHistoryView {
            filter,
            selected_index,
            visible_limit,
        } = &self.current_view
        else {
            return None;
        };

        self.dictation_history_current_or_previous_page(filter, *visible_limit)
            .and_then(|page| page.rows.into_iter().nth(*selected_index))
    }

    /// Return true when the current view has any available actions.
    fn has_actions(&mut self) -> bool {
        match &self.current_view {
            AppView::ClipboardHistoryView { .. } => {
                let has = self.selected_clipboard_entry().is_some();
                tracing::debug!(
                    event = "has_actions.clipboard",
                    has_selected_entry = has,
                    "has_actions (clipboard)",
                );
                has
            }
            AppView::DictationHistoryView { .. } => {
                self.selected_dictation_history_entry().is_some()
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
                // Run 12 Pass 7 — Power Syntax composer states ALWAYS have
                // Cmd+K actions available (cancel, copy filter, default time,
                // edit argv, …). The legacy `has_actions` only counted script-
                // and global-rows, so composing `+cal …` with no script match
                // would silently swallow Cmd+K. Treat the composer states as
                // self-sufficient for the gate.
                let composing_power_syntax = {
                    let raw = self.filter_text();
                    self.menu_syntax_mode.capture_for(raw).is_some()
                        || self.menu_syntax_mode.command_for(raw).is_some()
                        || self.menu_syntax_mode.advanced_query_for(raw).is_some()
                };
                // Run 13 Pass 1 (user bug report) — Cmd+K on the main script
                // list MUST always open the actions dialog, even when the
                // selected entry has no script-context actions and
                // get_global_actions() is currently empty. The legacy gate
                // here returned false in that case, silently swallowing the
                // keystroke and matching the user's "Cmd+K doesn't work at
                // all" report. Always-true on ScriptList lets the dialog
                // surface its built-in/global rows (or an empty placeholder)
                // so the keystroke is visible and discoverable.
                let on_script_list = matches!(self.current_view, AppView::ScriptList);
                let result = !actions.is_empty() || composing_power_syntax || on_script_list;
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
        self.transition_current_view_and_rekey_main_automation_surface(AppView::ScriptList);
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
    fn show_outcome_feedback(&mut self, outcome: &DispatchOutcome, cx: &mut Context<Self>) {
        if outcome.status == ActionOutcomeStatus::Error {
            if let Some(ref msg) = outcome.user_message {
                self.show_error_toast_with_code(msg.clone(), outcome.error_code, cx);
            }
        }
    }

    fn current_main_prompt_handoff_payload(
        &mut self,
        adapter_id: crate::ai::agent_prompt_handoff::AgentPromptHandoffAdapterId,
    ) -> Result<
        crate::ai::agent_prompt_handoff::AgentPromptHandoffPayload,
        crate::ai::agent_prompt_handoff::AgentPromptHandoffError,
    > {
        let raw = self.filter_text().to_string();
        if raw.trim().is_empty() {
            return Err(crate::ai::agent_prompt_handoff::AgentPromptHandoffError::EmptyPrompt);
        }

        self.set_spine_parse_from_filter_and_cursor(&raw, raw.len());
        let plan = crate::spine::prompt_plan::build_spine_prompt_plan_with_aliases(
            &self.spine_parse,
            &self.spine_mention_aliases,
        );
        let cwd = self
            .spine_cwd_for_agent_chat_launch()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("/"));
        crate::ai::agent_prompt_handoff::compile_handoff_payload_from_spine_plan(
            adapter_id,
            raw,
            cwd,
            None,
            Vec::new(),
            plan,
        )
    }

    pub(crate) fn launch_prompt_target_from_main_prompt(
        &mut self,
        adapter_id: crate::ai::agent_prompt_handoff::AgentPromptHandoffAdapterId,
        cx: &mut Context<Self>,
    ) -> Result<
        crate::ai::agent_prompt_handoff::AgentPromptHandoffReceipt,
        crate::ai::agent_prompt_handoff::AgentPromptHandoffError,
    > {
        let payload = self.current_main_prompt_handoff_payload(adapter_id)?;
        let receipt = crate::ai::agent_prompt_handoff::launch_prompt_handoff(&payload)?;
        tracing::info!(
            target: "script_kit::agent_handoff",
            event = "main_prompt_handoff_succeeded",
            adapter_id = %receipt.adapter_id,
            action_id = %receipt.action_id,
            dry_run = receipt.dry_run,
            prompt_chars = receipt.prompt_chars,
            prompt_sha256 = %receipt.diagnostic_prompt_fingerprint(),
            spawned = receipt.spawned,
            pid = ?receipt.pid,
        );
        cx.notify();
        Ok(receipt)
    }

    pub(crate) fn export_prompt_from_main_prompt(
        &mut self,
        action: crate::ai::agent_prompt_handoff::AgentPromptActionId,
        cx: &mut Context<Self>,
    ) -> Result<
        crate::ai::agent_prompt_handoff::AgentPromptExportReceipt,
        crate::ai::agent_prompt_handoff::AgentPromptHandoffError,
    > {
        let payload = self.current_main_prompt_handoff_payload(
            crate::ai::agent_prompt_handoff::AgentPromptHandoffAdapterId::CmuxCodex,
        )?;
        let receipt = crate::ai::agent_prompt_handoff::export_prompt(&payload, action)?;
        tracing::info!(
            target: "script_kit::agent_handoff",
            event = "main_prompt_export_succeeded",
            action_id = %receipt.action_id,
            dry_run = receipt.dry_run,
            export_kind = %receipt.export_kind,
            context_part_count = receipt.context_part_count,
            prompt_builder_segment_count = receipt.prompt_builder_segment_count,
            clipboard_written = receipt.clipboard_written,
            prompt_chars = receipt.prompt_chars,
            prompt_sha256 = %receipt.diagnostic_prompt_fingerprint(),
            path_sha256 = ?receipt.diagnostic_path_fingerprint(),
            url_sha256 = ?receipt.diagnostic_url_fingerprint(),
        );
        cx.notify();
        Ok(receipt)
    }

    fn handle_main_prompt_handoff_action(
        &mut self,
        action_id: &str,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let Some(adapter_id) = crate::ai::agent_prompt_handoff::adapter_from_action_id(action_id)
        else {
            return DispatchOutcome::not_handled();
        };

        if !matches!(self.current_view, AppView::ScriptList) {
            return DispatchOutcome::not_handled();
        }

        match self.launch_prompt_target_from_main_prompt(adapter_id, cx) {
            Ok(receipt) => {
                let mut outcome = DispatchOutcome::success();
                outcome.user_message = Some(format!("Sent prompt to {}", receipt.adapter_id));
                outcome
            }
            Err(error) => DispatchOutcome::error(
                crate::action_helpers::ERROR_ACTION_FAILED,
                error.user_message(),
            ),
        }
    }

    fn handle_main_prompt_export_action(
        &mut self,
        action_id: &str,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let Some(prompt_action) =
            crate::ai::agent_prompt_handoff::prompt_action_from_action_id(action_id)
        else {
            return DispatchOutcome::not_handled();
        };

        if !matches!(self.current_view, AppView::ScriptList) {
            return DispatchOutcome::not_handled();
        }

        match self.export_prompt_from_main_prompt(prompt_action, cx) {
            Ok(receipt) => {
                let mut outcome = DispatchOutcome::success();
                outcome.user_message = Some(match receipt.export_kind.as_str() {
                    "file" => receipt
                        .path
                        .as_deref()
                        .map(|path| format!("Exported prompt to {path}"))
                        .unwrap_or_else(|| "Exported prompt to file".to_string()),
                    "gist" => receipt
                        .url
                        .as_deref()
                        .map(|url| format!("Exported prompt to {url}"))
                        .unwrap_or_else(|| "Exported prompt to gist".to_string()),
                    "clipboard" => "Copied prompt to clipboard".to_string(),
                    _ => "Exported prompt".to_string(),
                });
                outcome
            }
            Err(error) => DispatchOutcome::error(
                crate::action_helpers::ERROR_ACTION_FAILED,
                error.user_message(),
            ),
        }
    }

    fn handle_action(&mut self, action_id: String, window: &mut Window, cx: &mut Context<Self>) {
        if action_id.starts_with("brain_inbox_open:") {
            if !self.open_brain_inbox_action(&action_id, cx) {
                self.show_error_toast(
                    "The inbox changed. Open Brain Inbox in Actions and choose again.",
                    cx,
                );
            }
            return;
        }

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

        let is_prompt_action =
            crate::ai::agent_prompt_handoff::is_prompt_action_id(&action_id_stripped);
        let should_transition_to_script_list =
            should_transition_to_script_list_after_action(&self.current_view) && !is_prompt_action;

        let selected_clipboard_entry = if action_id_stripped.starts_with("clipboard_") {
            self.selected_clipboard_entry()
        } else {
            None
        };
        // Clipboard actions handle their own transitions and notifications.
        let clipboard_outcome =
            self.handle_clipboard_action(&action_id_stripped, selected_clipboard_entry, &dctx, cx);
        if clipboard_outcome.was_handled() {
            log_dispatch_outcome(
                &action_id_stripped,
                &dctx.trace_id,
                "clipboard",
                &clipboard_outcome,
                &start,
            );
            self.show_outcome_feedback(&clipboard_outcome, cx);
            return;
        }

        let selected_dictation_entry = if action_id_stripped.starts_with("dictation_history_") {
            self.selected_dictation_history_entry()
        } else {
            None
        };
        let dictation_outcome = self.handle_dictation_history_action(
            &action_id_stripped,
            selected_dictation_entry,
            &dctx,
            window,
            cx,
        );
        if dictation_outcome.was_handled() {
            log_dispatch_outcome(
                &action_id_stripped,
                &dctx.trace_id,
                "dictation_history",
                &dictation_outcome,
                &start,
            );
            self.show_outcome_feedback(&dictation_outcome, cx);
            return;
        }

        let favorites_outcome =
            self.handle_favorites_action(&action_id_stripped, &dctx, window, cx);
        if favorites_outcome.was_handled() {
            log_dispatch_outcome(
                &action_id_stripped,
                &dctx.trace_id,
                "favorites",
                &favorites_outcome,
                &start,
            );
            self.show_outcome_feedback(&favorites_outcome, cx);
            return;
        }

        // Only script-list-hosted actions should force a ScriptList transition.
        if should_transition_to_script_list {
            self.transition_to_script_list_after_action(cx);
        }

        // Dispatch through handler chain, collecting the final outcome.
        let (handler, outcome) = {
            let o = self.handle_shortcut_alias_action(&action_id_stripped, &dctx, window, cx);
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
                        let o = self.handle_app_action(&action_id_stripped, &dctx, window, cx);
                        if o.was_handled() {
                            ("app", o)
                        } else {
                            let o = self.handle_scriptlet_action(&action_id_stripped, &dctx, cx);
                            if o.was_handled() {
                                ("scriptlet", o)
                            } else {
                                let o =
                                    self.handle_main_prompt_handoff_action(&action_id_stripped, cx);
                                if o.was_handled() {
                                    ("main_prompt_handoff", o)
                                } else {
                                    let o = self
                                        .handle_main_prompt_export_action(&action_id_stripped, cx);
                                    if o.was_handled() {
                                        ("main_prompt_export", o)
                                    } else {
                                        // SDK actions as final fallback — thread trace_id from dctx
                                        (
                                            "sdk_fallback",
                                            self.trigger_sdk_action_with_trace(
                                                &action_id_stripped,
                                                &dctx.trace_id,
                                            ),
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        if matches!(self.current_view, AppView::ScriptList) {
            self.flush_pending_main_menu_query(cx);
        }

        log_dispatch_outcome(
            &action_id_stripped,
            &dctx.trace_id,
            handler,
            &outcome,
            &start,
        );
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
include!("paste.rs");
include!("dictation_history.rs");
include!("favorites.rs");
include!("scripts.rs");
include!("shortcuts.rs");
include!("files.rs");
include!("apps.rs");
include!("scriptlets.rs");
