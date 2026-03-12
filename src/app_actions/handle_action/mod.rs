use crate::action_helpers::{ActionOutcomeStatus, DispatchContext, DispatchOutcome};

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
    SetInput { text: String },
    SetInputWithImage { text: String, image_base64: String },
    AddAttachment { path: String },
}

impl DeferredAiWindowAction {
    fn name(&self) -> &'static str {
        match self {
            Self::SetInput { .. } => "set_input",
            Self::SetInputWithImage { .. } => "set_input_with_image",
            Self::AddAttachment { .. } => "add_attachment",
        }
    }

    fn apply(self, cx: &mut App) {
        match self {
            Self::SetInput { text } => ai::set_ai_input(cx, &text, false),
            Self::SetInputWithImage { text, image_base64 } => {
                ai::set_ai_input_with_image(cx, &text, &image_base64, false);
            }
            Self::AddAttachment { path } => ai::add_ai_attachment(cx, &path),
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
        deferred_action: DeferredAiWindowAction,
        success_message: &'static str,
        cx: &mut Context<Self>,
    ) {
        let deferred_action_name = deferred_action.name();
        tracing::info!(
            category = "AI",
            event = "action_attach_to_ai_defer_open_start",
            deferred_action = deferred_action_name,
            "Hiding main window before opening AI window"
        );

        self.hide_main_and_reset(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;

            let open_result = cx.update(|cx| {
                ai::open_ai_window(cx).map_err(|error| error.to_string())?;
                deferred_action.apply(cx);
                Ok::<(), String>(())
            });

            match open_result {
                Ok(()) => {
                    let _ = this.update(cx, |this, cx| {
                        this.show_hud(success_message.to_string(), Some(HUD_SHORT_MS), cx);
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = this.update(cx, |this, cx| {
                        tracing::error!(
                            category = "AI",
                            event = "action_attach_to_ai_defer_open_failed",
                            attempted = "open_ai_window_after_main_hide",
                            deferred_action = deferred_action_name,
                            error = %error,
                            "Failed to open AI window after hiding main window"
                        );
                        this.show_error_toast("Failed to open AI window", cx);
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
            AppView::ClipboardHistoryView { .. } => self.selected_clipboard_entry().is_some(),
            _ => {
                let script_info = self.get_focused_script_info();
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

                actions.extend(crate::actions::get_global_actions());
                !actions.is_empty()
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
    fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
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
                let o = self.handle_script_action(&action_id_stripped, &dctx, cx);
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
                            // SDK actions as final fallback
                            ("sdk_fallback", self.trigger_sdk_action_internal(&action_id_stripped))
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
