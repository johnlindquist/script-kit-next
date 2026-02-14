impl ScriptListApp {
    fn hide_main_and_reset(&self, _cx: &mut Context<Self>) {
        if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
            let bounds = crate::window_state::PersistedWindowBounds::new(x, y, w, h);
            let displays = platform::get_macos_displays();
            let _ =
                crate::window_state::save_main_position_with_display_detection(bounds, &displays);
        }
        set_main_window_visible(false);
        NEEDS_RESET.store(true, Ordering::SeqCst);
        // Use platform-specific hide that only hides the main window,
        // not the entire app (cx.hide() would hide HUD too)
        platform::hide_main_window();
    }

    /// Helper to reveal a path in Finder (macOS)
    fn reveal_in_finder(&self, path: &std::path::Path) {
        let path_str = path.to_string_lossy().to_string();
        std::thread::spawn(move || {
            let file_manager = if cfg!(target_os = "macos") {
                "Finder"
            } else if cfg!(target_os = "windows") {
                "Explorer"
            } else {
                "File Manager"
            };

            match crate::file_search::reveal_in_finder(&path_str) {
                Ok(_) => tracing::info!(
                    category = "UI",
                    action = "reveal_in_finder",
                    file_manager,
                    path = %path_str,
                    "Revealed path in file manager"
                ),
                Err(error) => tracing::error!(
                    action = "reveal_in_finder",
                    file_manager,
                    path = %path_str,
                    error = %error,
                    "Failed to reveal path in file manager"
                ),
            }
        });
    }

    /// Reveal a path and return completion back to the UI thread for HUD feedback.
    fn reveal_in_finder_with_feedback_async(
        &self,
        path: &std::path::Path,
    ) -> async_channel::Receiver<Result<(), String>> {
        let path_str = path.to_string_lossy().to_string();
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
                file_manager,
                path = %path_str,
                "Reveal in file manager started"
            );

            let reveal_result = match crate::file_search::reveal_in_finder(&path_str) {
                Ok(()) => {
                    tracing::info!(
                        category = "UI",
                        event = "action_reveal_in_finder_success",
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
    ) -> async_channel::Receiver<Result<(), String>> {
        let editor = self.config.get_editor();
        let path_str = path.to_string_lossy().to_string();
        let (result_tx, result_rx) = async_channel::bounded::<Result<(), String>>(1);

        std::thread::spawn(move || {
            use std::process::Command;

            tracing::info!(
                category = "UI",
                event = "action_editor_launch_start",
                editor = %editor,
                path = %path_str,
                "Editor launch started"
            );

            let launch_result = match Command::new(&editor).arg(&path_str).spawn() {
                Ok(_) => {
                    tracing::info!(
                        category = "UI",
                        event = "action_editor_launch_success",
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
        child.wait()?;
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

    /// Handle action selection from the actions dialog
    fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
        tracing::info!(
            category = "UI",
            action = %action_id,
            "Action selected"
        );

        let action_id = action_id
            .strip_prefix("clip:")
            .or_else(|| action_id.strip_prefix("file:"))
            .or_else(|| action_id.strip_prefix("chat:"))
            .unwrap_or(action_id.as_str());

        let should_transition_to_script_list =
            should_transition_to_script_list_after_action(&self.current_view);

        let selected_clipboard_entry = if action_id.starts_with("clipboard_") {
            self.selected_clipboard_entry()
        } else {
            None
        };

        match action_id {
            "clipboard_pin" | "clipboard_unpin" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let result = if action_id == "clipboard_pin" {
                    clipboard_history::pin_entry(&entry.id)
                } else {
                    clipboard_history::unpin_entry(&entry.id)
                };

                match result {
                    Ok(()) => {
                        // Refresh cached entries (pin/unpin updates cache ordering)
                        self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);

                        // Keep selection on the same entry when possible
                        if let AppView::ClipboardHistoryView {
                            filter,
                            selected_index,
                        } = &mut self.current_view
                        {
                            let filtered_entries: Vec<_> = if filter.is_empty() {
                                self.cached_clipboard_entries.iter().enumerate().collect()
                            } else {
                                let filter_lower = filter.to_lowercase();
                                self.cached_clipboard_entries
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, e)| {
                                        e.text_preview.to_lowercase().contains(&filter_lower)
                                    })
                                    .collect()
                            };

                            if let Some(new_index) =
                                filtered_entries.iter().position(|(_, e)| e.id == entry.id)
                            {
                                *selected_index = new_index;
                            } else if !filtered_entries.is_empty() {
                                *selected_index =
                                    (*selected_index).min(filtered_entries.len().saturating_sub(1));
                            } else {
                                *selected_index = 0;
                            }

                            if !filtered_entries.is_empty() {
                                self.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            }
                            self.focused_clipboard_entry_id = filtered_entries
                                .get(*selected_index)
                                .map(|(_, entry)| entry.id.clone());
                        }

                        if let Some(message) = clipboard_pin_action_success_hud(action_id) {
                            self.show_hud(message.to_string(), Some(HUD_SHORT_MS), cx);
                        }
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to toggle clipboard pin: {}", e));
                        self.show_hud(format!("Failed to update pin: {}", e), Some(HUD_LONG_MS), cx);
                    }
                }
                return;
            }
            "clipboard_share" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Clipboard entry content unavailable".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                };

                tracing::info!(category = "UI", message = ?
                    &format!(
                        "Opening share sheet for clipboard entry {} ({:?})",
                        entry.id, entry.content_type
                    ),
                );

                let share_result = match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => {
                        crate::platform::show_share_sheet(crate::platform::ShareSheetItem::Text(
                            content,
                        ));
                        Ok(())
                    }
                    clipboard_history::ContentType::Image => {
                        if let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content) {
                            crate::platform::show_share_sheet(
                                crate::platform::ShareSheetItem::ImagePng(png_bytes),
                            );
                            Ok(())
                        } else {
                            Err("Failed to decode clipboard image".to_string())
                        }
                    }
                };

                match share_result {
                    Ok(()) => self.show_hud("Share sheet opened".to_string(), Some(HUD_SHORT_MS), cx),
                    Err(message) => self.show_hud(message, Some(HUD_MEDIUM_MS), cx),
                }
                return;
            }
            // Paste to active app and close window (Enter)
            "clipboard_paste" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                tracing::info!(category = "CLIPBOARD", message = ? &format!("Paste entry: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        tracing::info!(category = "CLIPBOARD", message = ? "Entry copied, simulating paste");
                        cx.spawn(async move |_this, _cx| {
                            Timer::after(std::time::Duration::from_millis(50)).await;
                            if let Err(e) = selected_text::simulate_paste_with_cg() {
                                tracing::error!(message = ? &format!("Failed to simulate paste: {}", e));
                            } else {
                                tracing::info!(category = "CLIPBOARD", message = ? "Simulated Cmd+V paste");
                            }
                        })
                        .detach();
                        self.show_hud("Pasted".to_string(), Some(1000), cx);
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to paste entry: {}", e));
                        self.show_hud(format!("Failed to paste: {}", e), Some(2500), cx);
                    }
                }
                cx.notify();
                return;
            }
            "clipboard_attach_to_ai" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Clipboard entry content unavailable".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                };

                tracing::info!(
                    category = "AI",
                    entry_id = %entry.id,
                    content_type = ?entry.content_type,
                    "Attaching clipboard entry to AI chat"
                );

                match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => {
                        if let Err(e) = ai::open_ai_window(cx) {
                            tracing::error!(message = ? &format!("Failed to open AI window: {}", e));
                            self.show_hud("Failed to open AI window".to_string(), Some(HUD_MEDIUM_MS), cx);
                            return;
                        }
                        ai::set_ai_input(cx, &content, false);
                    }
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                        else {
                            self.show_hud(
                                "Failed to decode clipboard image".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                            return;
                        };

                        use base64::Engine;
                        let base64_data =
                            base64::engine::general_purpose::STANDARD.encode(&png_bytes);

                        if let Err(e) = ai::open_ai_window(cx) {
                            tracing::error!(message = ? &format!("Failed to open AI window: {}", e));
                            self.show_hud("Failed to open AI window".to_string(), Some(HUD_MEDIUM_MS), cx);
                            return;
                        }
                        ai::set_ai_input_with_image(cx, "", &base64_data, false);
                    }
                }

                self.show_hud("Attached to AI".to_string(), Some(HUD_SHORT_MS), cx);
                self.hide_main_and_reset(cx);
                cx.notify();
                return;
            }
            // Copy to clipboard without pasting (Cmd+Enter)
            "clipboard_copy" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                tracing::info!(category = "CLIPBOARD", message = ?
                    &format!("Copying entry to clipboard: {}", entry.id),
                );
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        tracing::info!(category = "CLIPBOARD", message = ? "Entry copied to clipboard");
                        self.show_hud("Copied to clipboard".to_string(), Some(HUD_SHORT_MS), cx);
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to copy entry: {}", e));
                        self.show_hud(format!("Failed to copy: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            // Paste and keep window open (Opt+Enter)
            "clipboard_paste_keep_open" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                tracing::info!(category = "CLIPBOARD", message = ? &format!("Paste and keep open: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        tracing::info!(category = "CLIPBOARD", message = ? "Entry copied, simulating paste");
                        // Simulate Cmd+V paste after a brief delay
                        cx.spawn(async move |_this, _cx| {
                            Timer::after(std::time::Duration::from_millis(50)).await;
                            if let Err(e) = selected_text::simulate_paste_with_cg() {
                                tracing::error!(message = ? &format!("Failed to simulate paste: {}", e));
                            } else {
                                tracing::info!(category = "CLIPBOARD", message = ? "Simulated Cmd+V paste");
                            }
                        })
                        .detach();
                        self.show_hud("Pasted".to_string(), Some(1000), cx);
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to copy entry: {}", e));
                        self.show_hud(format!("Failed to paste: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            "clipboard_quick_look" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if let Err(e) = clipboard_history::quick_look_entry(&entry) {
                    tracing::error!(message = ? &format!("Quick Look failed: {}", e));
                    self.show_hud(format!("Quick Look failed: {}", e), Some(2500), cx);
                }
                return;
            }

            _ => {}
        }

        // Only script-list-hosted actions should force a ScriptList transition.
        if should_transition_to_script_list {
            self.transition_to_script_list_after_action(cx);
        }

        match action_id {
            "create_script" => {
                tracing::info!(category = "UI", message = ? "Create script action - opening scripts folder");
                let scripts_dir = shellexpand::tilde("~/.scriptkit/scripts").to_string();
                std::thread::spawn(move || {
                    use std::process::Command;
                    match Command::new("open").arg(&scripts_dir).spawn() {
                        Ok(_) => {
                            tracing::info!(category = "UI", message = ? &format!("Opened scripts folder: {}", scripts_dir))
                        }
                        Err(e) => {
                            tracing::error!(message = ? &format!("Failed to open scripts folder: {}", e))
                        }
                    }
                });
                self.show_hud("Opened scripts folder".to_string(), Some(HUD_SHORT_MS), cx);
                self.hide_main_and_reset(cx);
            }
            "run_script" => {
                tracing::info!(category = "UI", message = ? "Run script action");
                self.execute_selected(cx);
            }
            "view_logs" => {
                tracing::info!(category = "UI", message = ? "View logs action");
                self.toggle_logs(cx);
            }
            "reveal_in_finder" => {
                tracing::info!(category = "UI", message = ? "Reveal in Finder action");
                // First check if we have a file search path (takes priority)
                let path_opt = if let Some(path) = self.file_search_actions_path.take() {
                    tracing::info!(category = "UI", message = ? &format!("Reveal in Finder (file search): {}", path));
                    Some(std::path::PathBuf::from(path))
                } else if let Some(result) = self.get_selected_result() {
                    // Fall back to main menu selected result
                    match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::App(m) => Some(m.app.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => None,
                        scripts::SearchResult::BuiltIn(_) => None,
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(_) => None,
                    }
                } else {
                    None
                };

                if let Some(path) = path_opt {
                    let reveal_result_rx = self.reveal_in_finder_with_feedback_async(&path);
                    cx.spawn(async move |this, cx| {
                        let Ok(reveal_result) = reveal_result_rx.recv().await else {
                            return;
                        };

                        this.update(cx, |this, cx| match reveal_result {
                            Ok(()) => {
                                this.show_hud("Opened in Finder".to_string(), Some(HUD_SHORT_MS), cx);
                                this.hide_main_and_reset(cx);
                            }
                            Err(message) => {
                                this.show_hud(message, Some(2500), cx);
                            }
                        })
                        .ok();
                    })
                    .detach();
                } else {
                    self.show_hud(
                        "Cannot reveal this item type in Finder".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "copy_path" => {
                tracing::info!(category = "UI", message = ? "Copy path action");
                // First check if we have a file search path (takes priority)
                let path_str = if let Some(path) = self.file_search_actions_path.take() {
                    tracing::info!(category = "UI", message = ? &format!("Copy path (file search): {}", path));
                    Some(path)
                } else if let Some(result) = self.get_selected_result() {
                    // Fall back to main menu selected result
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(m.script.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::App(m) => {
                            Some(m.app.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Agent(m) => {
                            Some(m.agent.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Scriptlet(_) => None,
                        scripts::SearchResult::BuiltIn(_) => None,
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(_) => None,
                    };
                    if path_opt.is_none() {
                        self.show_hud(
                            "Cannot copy path for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    path_opt
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    None
                };

                if let Some(path_str) = path_str {
                    #[cfg(target_os = "macos")]
                    {
                        match self.pbcopy(&path_str) {
                            Ok(_) => {
                                tracing::info!(category = "UI", message = ?
                                    &format!("Copied path to clipboard: {}", path_str),
                                );
                                self.show_hud(format!("Copied: {}", path_str), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ? &format!("pbcopy failed: {}", e));
                                self.show_hud("Failed to copy path".to_string(), Some(HUD_LONG_MS), cx);
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        match Clipboard::new().and_then(|mut c| c.set_text(&path_str)) {
                            Ok(_) => {
                                tracing::info!(category = "UI", message = ?
                                    &format!("Copied path to clipboard: {}", path_str),
                                );
                                self.show_hud(format!("Copied: {}", path_str), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ? &format!("Failed to copy path: {}", e));
                                self.show_hud("Failed to copy path".to_string(), Some(HUD_LONG_MS), cx);
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                }
            }
            "copy_deeplink" => {
                tracing::info!(category = "UI", message = ? "Copy deeplink action");
                if let Some(result) = self.get_selected_result() {
                    let name = result.name();
                    let deeplink_name = crate::actions::to_deeplink_name(name);
                    let deeplink_url = format!("scriptkit://run/{}", deeplink_name);

                    #[cfg(target_os = "macos")]
                    {
                        match self.pbcopy(&deeplink_url) {
                            Ok(_) => {
                                tracing::info!(category = "UI", message = ?
                                    &format!("Copied deeplink to clipboard: {}", deeplink_url),
                                );
                                self.show_hud(format!("Copied: {}", deeplink_url), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ? &format!("pbcopy failed: {}", e));
                                self.show_hud(
                                    "Failed to copy deeplink".to_string(),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        match Clipboard::new().and_then(|mut c| c.set_text(&deeplink_url)) {
                            Ok(_) => {
                                tracing::info!(category = "UI", message = ?
                                    &format!("Copied deeplink to clipboard: {}", deeplink_url),
                                );
                                self.show_hud(format!("Copied: {}", deeplink_url), Some(HUD_MEDIUM_MS), cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ? &format!("Failed to copy deeplink: {}", e));
                                self.show_hud(
                                    "Failed to copy deeplink".to_string(),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // Handle both legacy "configure_shortcut" and new dynamic actions
            // "add_shortcut" and "update_shortcut" open the shortcut recorder
            "configure_shortcut" | "add_shortcut" | "update_shortcut" => {
                tracing::info!(category = "UI", message = ? &format!("{} action", action_id));
                if let Some(result) = self.get_selected_result() {
                    match result {
                        // Scripts: open the script file to edit // Shortcut: comment
                        scripts::SearchResult::Script(m) => {
                            self.edit_script(&m.script.path);
                            self.hide_main_and_reset(cx);
                        }
                        scripts::SearchResult::Agent(m) => {
                            self.edit_script(&m.agent.path);
                            self.hide_main_and_reset(cx);
                        }
                        // Non-scripts: show inline shortcut recorder
                        scripts::SearchResult::Scriptlet(m) => {
                            let command_id = format!("scriptlet/{}", m.scriptlet.name);
                            let command_name = m.scriptlet.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            let command_id = format!("builtin/{}", m.entry.id);
                            let command_name = m.entry.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::App(m) => {
                            // Use bundle ID if available, otherwise use name
                            let command_id = if let Some(ref bundle_id) = m.app.bundle_id {
                                format!("app/{}", bundle_id)
                            } else {
                                format!("app/{}", m.app.name.to_lowercase().replace(' ', "-"))
                            };
                            let command_name = m.app.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::Window(_) => {
                            self.show_hud(
                                "Window shortcuts not supported - windows are transient"
                                    .to_string(),
                                Some(2500),
                                cx,
                            );
                        }
                        scripts::SearchResult::Fallback(m) => {
                            match &m.fallback {
                                crate::fallbacks::collector::FallbackItem::Builtin(b) => {
                                    let command_id = format!("fallback/{}", m.fallback.name());
                                    let command_name = b.name.to_string();
                                    self.show_shortcut_recorder(command_id, command_name, cx);
                                }
                                crate::fallbacks::collector::FallbackItem::Script(s) => {
                                    // Script-based fallback - open the script
                                    self.edit_script(&s.script.path);
                                    self.hide_main_and_reset(cx);
                                }
                            }
                        }
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // "remove_shortcut" removes the existing shortcut from the registry
            "remove_shortcut" => {
                tracing::info!(category = "UI", message = ? "Remove shortcut action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(format!("script/{}", m.script.name))
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            Some(format!("scriptlet/{}", m.scriptlet.name))
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            Some(format!("builtin/{}", m.entry.id))
                        }
                        scripts::SearchResult::App(m) => {
                            if let Some(ref bundle_id) = m.app.bundle_id {
                                Some(format!("app/{}", bundle_id))
                            } else {
                                Some(format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                ))
                            }
                        }
                        scripts::SearchResult::Agent(m) => Some(format!("agent/{}", m.agent.name)),
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(m) => {
                            Some(format!("fallback/{}", m.fallback.name()))
                        }
                    };

                    if let Some(command_id) = command_id_opt {
                        // Remove the shortcut override from persistence
                        match crate::shortcuts::remove_shortcut_override(&command_id) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "SHORTCUT",
                                    command_id = %command_id,
                                    "Removed shortcut override"
                                );
                                self.show_hud("Shortcut removed".to_string(), Some(HUD_MEDIUM_MS), cx);
                                // Refresh scripts to update shortcut display
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ? &format!("Failed to remove shortcut: {}", e));
                                self.show_hud(
                                    format!("Failed to remove shortcut: {}", e),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot remove shortcut for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // Alias actions: add_alias, update_alias open the alias input
            "add_alias" | "update_alias" => {
                tracing::info!(category = "UI", message = ? &format!("{} action", action_id));
                if let Some(result) = self.get_selected_result() {
                    let (command_id, command_name) = match result {
                        scripts::SearchResult::Script(m) => {
                            (format!("script/{}", m.script.name), m.script.name.clone())
                        }
                        scripts::SearchResult::Scriptlet(m) => (
                            format!("scriptlet/{}", m.scriptlet.name),
                            m.scriptlet.name.clone(),
                        ),
                        scripts::SearchResult::BuiltIn(m) => {
                            (format!("builtin/{}", m.entry.id), m.entry.name.clone())
                        }
                        scripts::SearchResult::App(m) => {
                            let id = if let Some(ref bundle_id) = m.app.bundle_id {
                                format!("app/{}", bundle_id)
                            } else {
                                format!("app/{}", m.app.name.to_lowercase().replace(' ', "-"))
                            };
                            (id, m.app.name.clone())
                        }
                        scripts::SearchResult::Agent(m) => {
                            (format!("agent/{}", m.agent.name), m.agent.name.clone())
                        }
                        scripts::SearchResult::Window(_) => {
                            self.show_hud(
                                "Window aliases not supported - windows are transient".to_string(),
                                Some(2500),
                                cx,
                            );
                            return;
                        }
                        scripts::SearchResult::Fallback(m) => (
                            format!("fallback/{}", m.fallback.name()),
                            m.fallback.name().to_string(),
                        ),
                    };
                    self.show_alias_input(command_id, command_name, cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            // "remove_alias" removes the existing alias from persistence
            "remove_alias" => {
                tracing::info!(category = "UI", message = ? "Remove alias action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(format!("script/{}", m.script.name))
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            Some(format!("scriptlet/{}", m.scriptlet.name))
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            Some(format!("builtin/{}", m.entry.id))
                        }
                        scripts::SearchResult::App(m) => {
                            if let Some(ref bundle_id) = m.app.bundle_id {
                                Some(format!("app/{}", bundle_id))
                            } else {
                                Some(format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                ))
                            }
                        }
                        scripts::SearchResult::Agent(m) => Some(format!("agent/{}", m.agent.name)),
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(m) => {
                            Some(format!("fallback/{}", m.fallback.name()))
                        }
                    };

                    if let Some(command_id) = command_id_opt {
                        // Remove the alias override from persistence
                        match crate::aliases::remove_alias_override(&command_id) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "ALIAS",
                                    command_id = %command_id,
                                    "Removed alias override"
                                );
                                self.show_hud("Alias removed".to_string(), Some(HUD_MEDIUM_MS), cx);
                                // Refresh scripts to update alias display and registry
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ? &format!("Failed to remove alias: {}", e));
                                self.show_hud(
                                    format!("Failed to remove alias: {}", e),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot remove alias for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "edit_script" => {
                tracing::info!(category = "UI", message = ? "Edit script action");
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => None,
                        scripts::SearchResult::BuiltIn(_) => None,
                        scripts::SearchResult::App(_) => None,
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(_) => None,
                    };

                    if let Some(path) = path_opt {
                        let editor_launch_rx = self.launch_editor_with_feedback_async(&path);
                        cx.spawn(async move |this, cx| {
                            let Ok(launch_result) = editor_launch_rx.recv().await else {
                                return;
                            };

                            this.update(cx, |this, cx| match launch_result {
                                Ok(()) => {
                                    this.hide_main_and_reset(cx);
                                }
                                Err(message) => {
                                    this.show_hud(message, Some(HUD_LONG_MS), cx);
                                }
                            })
                            .ok();
                        })
                        .detach();
                    } else {
                        self.show_hud("Cannot edit this item type".to_string(), Some(HUD_MEDIUM_MS), cx);
                    }
                } else {
                    self.show_hud("No script selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                }
            }
            "remove_script" | "delete_script" => {
                tracing::info!(category = "UI", message = ? &format!("{} action", action_id));

                let Some(result) = self.get_selected_result() else {
                    self.show_hud("No script selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(target) = script_removal_target_from_result(&result) else {
                    self.show_hud("Cannot remove this item type".to_string(), Some(2500), cx);
                    return;
                };

                if !target.path.exists() {
                    self.show_hud(format!("{} no longer exists", target.name), Some(2500), cx);
                    self.refresh_scripts(cx);
                    cx.notify();
                    return;
                }

                let message = format!(
                    "Move this {} to Trash?\n\n{}",
                    target.item_kind, target.name
                );

                cx.spawn(async move |this, cx| {
                    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);
                    let open_result = cx.update(|cx| {
                        let main_bounds =
                            if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(x as f32),
                                        y: gpui::px(y as f32),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(w as f32),
                                        height: gpui::px(h as f32),
                                    },
                                }
                            } else {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(100.0),
                                        y: gpui::px(100.0),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(600.0),
                                        height: gpui::px(400.0),
                                    },
                                }
                            };

                        let sender = confirm_tx.clone();
                        let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                            let _ = sender.try_send(confirmed);
                        });

                        open_confirm_window(
                            cx,
                            main_bounds,
                            None,
                            message,
                            Some("Move to Trash".to_string()),
                            Some("Cancel".to_string()),
                            on_choice,
                        )
                    });

                    match open_result {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            this.update(cx, |this, cx| {
                                tracing::error!(message = ?
                                    &format!("Failed to open confirmation modal: {}", e),
                                );
                                this.show_hud(
                                    "Failed to open confirmation dialog".to_string(),
                                    Some(2500),
                                    cx,
                                );
                            })
                            .ok();
                            return;
                        }
                        Err(_) => return,
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    this.update(cx, move |this, cx| match move_path_to_trash(&target.path) {
                        Ok(()) => {
                            tracing::info!(category = "UI", message = ?
                                &format!(
                                    "Moved {} '{}' to trash: {}",
                                    target.item_kind,
                                    target.name,
                                    target.path.display()
                                ),
                            );
                            this.refresh_scripts(cx);
                            this.show_hud(
                                format!("Moved '{}' to Trash", target.name),
                                Some(2200),
                                cx,
                            );
                            this.hide_main_and_reset(cx);
                            cx.notify();
                        }
                        Err(e) => {
                            tracing::error!(message = ?
                                &format!(
                                    "Failed to move {} '{}' to trash ({}): {}",
                                    target.item_kind,
                                    target.name,
                                    target.path.display(),
                                    e
                                ),
                            );
                            this.show_hud(format!("Failed to remove: {}", e), Some(3200), cx);
                        }
                    })
                    .ok();
                })
                .detach();
                return;
            }
            "reload_scripts" => {
                tracing::info!(category = "UI", message = ? "Reload scripts action");
                self.refresh_scripts(cx);
                self.show_hud("Scripts reloaded".to_string(), Some(HUD_SHORT_MS), cx);
            }
            "settings" => {
                tracing::info!(category = "UI", message = ? "Settings action - opening config.ts");

                // Get editor from config
                let editor = self.config.get_editor();
                let config_dir = shellexpand::tilde("~/.scriptkit/kit").to_string();
                let config_file = format!("{}/config.ts", config_dir);

                // Clone editor for HUD message before moving into thread
                let editor_for_hud = editor.clone();

                // Spawn editor in background thread
                std::thread::spawn(move || {
                    use std::process::Command;

                    // Editor-specific arguments for opening folder with file focused
                    let result = match editor.as_str() {
                        // VS Code and Cursor: -r (reuse window) + folder + file
                        "code" | "cursor" => Command::new(&editor)
                            .arg("-r")
                            .arg(&config_dir)
                            .arg(&config_file)
                            .spawn(),
                        // Zed: just the file (doesn't support folder context the same way)
                        "zed" => Command::new("zed").arg(&config_file).spawn(),
                        // Sublime: -a (add to current window) + folder + file
                        "subl" => Command::new("subl")
                            .arg("-a")
                            .arg(&config_dir)
                            .arg(&config_file)
                            .spawn(),
                        // Generic fallback: just open the file
                        _ => Command::new(&editor).arg(&config_file).spawn(),
                    };

                    match result {
                        Ok(_) => tracing::info!(category = "UI", message = ? &format!("Opened config.ts in {}", editor)),
                        Err(e) => tracing::error!(message = ?
                            &format!("Failed to open editor '{}': {}", editor, e),
                        ),
                    }
                });

                self.show_hud(
                    format!("Opening config.ts in {}", editor_for_hud),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                self.hide_main_and_reset(cx);
            }
            "quit" => {
                tracing::info!(category = "UI", message = ? "Quit action");
                PROCESS_MANAGER.kill_all_processes();
                PROCESS_MANAGER.remove_main_pid();
                cx.quit();
                return; // Early return after quit - no notify needed
            }
            "__cancel__" => {
                tracing::info!(category = "UI", message = ? "Actions dialog cancelled");
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
            }
            // File search specific actions
            "open_file" | "open_directory" | "quick_look" | "open_with" | "show_info" => {
                if let Some(path) = self.file_search_actions_path.clone() {
                    tracing::info!(category = "UI", message = ? &format!("File action '{}': {}", action_id, path));

                    let result = match action_id {
                        "open_file" | "open_directory" => crate::file_search::open_file(&path),
                        "quick_look" => crate::file_search::quick_look(&path),
                        "open_with" => crate::file_search::open_with(&path),
                        "show_info" => crate::file_search::show_info(&path),
                        _ => Ok(()),
                    };

                    match result {
                        Ok(()) => {
                            if let Some(message) = file_search_action_success_hud(action_id) {
                                self.show_hud(message.to_string(), Some(HUD_SHORT_MS), cx);
                            }
                            self.file_search_actions_path = None;
                            if action_id == "open_file" || action_id == "open_directory" {
                                self.hide_main_and_reset(cx);
                            }
                        }
                        Err(e) => {
                            tracing::error!(message = ?
                                &format!(
                                    "File search action '{}' failed for '{}': {}",
                                    action_id, path, e
                                ),
                            );
                            let prefix = file_search_action_error_hud_prefix(action_id)
                                .unwrap_or("Action failed");
                            self.show_hud(format!("{}: {}", prefix, e), Some(HUD_LONG_MS), cx);
                            self.file_search_actions_path = None;
                        }
                    }
                }
            }
            "copy_filename" => {
                if let Some(ref path) = self.file_search_actions_path {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    tracing::info!(category = "UI", message = ? &format!("Copy filename: {}", filename));
                    #[cfg(target_os = "macos")]
                    {
                        let _ = self.pbcopy(filename);
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        let _ = Clipboard::new().and_then(|mut c| c.set_text(filename));
                    }
                    self.show_hud(format!("Copied: {}", filename), Some(HUD_MEDIUM_MS), cx);
                    self.file_search_actions_path = None;
                    self.hide_main_and_reset(cx);
                }
            }
            "clipboard_open_with" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Failed to load clipboard content".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                };

                let full_entry = clipboard_history::ClipboardEntry {
                    id: entry.id.clone(),
                    content,
                    content_type: entry.content_type,
                    timestamp: entry.timestamp,
                    pinned: entry.pinned,
                    ocr_text: entry.ocr_text.clone(),
                    source_app_name: None,
                    source_app_bundle_id: None,
                };

                let temp_path = match clipboard_history::save_entry_to_temp_file(&full_entry) {
                    Ok(path) => path,
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to save temp file: {}", e));
                        self.show_hud("Failed to save temp file".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    }
                };

                #[cfg(target_os = "macos")]
                {
                    let path_str = temp_path.to_string_lossy().to_string();
                    if let Err(e) = crate::file_search::open_with(&path_str) {
                        tracing::error!(message = ? &format!("Open With failed: {}", e));
                        self.show_hud("Failed to open \"Open With\"".to_string(), Some(HUD_MEDIUM_MS), cx);
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let _ = temp_path;
                    self.show_hud(
                        "\"Open With\" is only supported on macOS".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "clipboard_annotate_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "CleanShot actions are only available for images".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    if let Err(e) = clipboard_history::copy_entry_to_clipboard(&entry.id) {
                        tracing::error!(message = ? &format!("Failed to copy image: {}", e));
                        self.show_hud("Failed to copy image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    }

                    let url = "cleanshot://open-from-clipboard";
                    match std::process::Command::new("open").arg(url).spawn() {
                        Ok(_) => {
                            self.show_hud("Opening CleanShot X…".to_string(), Some(HUD_SHORT_MS), cx);
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            tracing::error!(message = ? &format!("Failed to open CleanShot X: {}", e));
                            self.show_hud("Failed to open CleanShot X".to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_hud(
                        "CleanShot actions are only supported on macOS".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "clipboard_upload_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "CleanShot actions are only available for images".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_hud("Failed to load image content".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content) else {
                        self.show_hud("Failed to decode image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    let temp_path = std::env::temp_dir()
                        .join(format!("script-kit-clipboard-{}.png", uuid::Uuid::new_v4()));

                    if let Err(e) = std::fs::write(&temp_path, png_bytes) {
                        tracing::error!(message = ? &format!("Failed to write temp image: {}", e));
                        self.show_hud("Failed to save image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    }

                    let path_str = temp_path.to_string_lossy();
                    let encoded_path = self.percent_encode_for_url(&path_str);
                    let url = format!(
                        "cleanshot://open-annotate?filepath={}&action=upload",
                        encoded_path
                    );

                    match std::process::Command::new("open").arg(&url).spawn() {
                        Ok(_) => {
                            self.show_hud(
                                "Opening CleanShot X upload…".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            tracing::error!(message = ? &format!("Failed to open CleanShot X: {}", e));
                            self.show_hud("Failed to open CleanShot X".to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_hud(
                        "CleanShot actions are only supported on macOS".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "clipboard_ocr" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "OCR is only available for images".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                // Check if we already have cached OCR text
                if let Some(ref cached_text) = entry.ocr_text {
                    if !cached_text.trim().is_empty() {
                        tracing::debug!(category = "OCR", message = ? "Using cached OCR text");
                        #[cfg(target_os = "macos")]
                        {
                            let _ = self.pbcopy(cached_text);
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            use arboard::Clipboard;
                            let _ =
                                Clipboard::new().and_then(|mut c| c.set_text(cached_text.clone()));
                        }
                        self.show_hud("Copied text from image".to_string(), Some(HUD_SHORT_MS), cx);
                        self.hide_main_and_reset(cx);
                        cx.notify();
                        return;
                    }
                }

                #[cfg(all(target_os = "macos", feature = "ocr"))]
                {
                    // Get image content
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_hud("Failed to load image content".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    // Decode to RGBA bytes for OCR
                    let Some((width, height, rgba_bytes)) =
                        clipboard_history::decode_to_rgba_bytes(&content)
                    else {
                        self.show_hud("Failed to decode image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    tracing::debug!(category = "OCR", message = ?
                        &format!("Starting OCR on {}x{} image", width, height),
                    );
                    self.show_hud("Extracting text...".to_string(), Some(HUD_SHORT_MS), cx);

                    // Perform OCR synchronously (it runs on a background thread internally)
                    // For a truly async approach, we'd need to integrate with GPUI's async system
                    let entry_id = entry.id.clone();
                    match script_kit_gpui::ocr::extract_text_from_rgba(width, height, &rgba_bytes) {
                        Ok(text) => {
                            if text.trim().is_empty() {
                                tracing::debug!(category = "OCR", message = ? "No text found in image");
                                self.show_hud("No text found in image".to_string(), Some(HUD_MEDIUM_MS), cx);
                            } else {
                                tracing::debug!(category = "OCR", message = ?
                                    &format!("Extracted {} characters", text.len()),
                                );

                                // Cache the OCR result
                                let _ = clipboard_history::update_ocr_text(&entry_id, &text);

                                // Copy to clipboard
                                #[cfg(target_os = "macos")]
                                {
                                    let _ = self.pbcopy(&text);
                                }
                                #[cfg(not(target_os = "macos"))]
                                {
                                    use arboard::Clipboard;
                                    let _ =
                                        Clipboard::new().and_then(|mut c| c.set_text(text.clone()));
                                }

                                self.show_hud("Copied text from image".to_string(), Some(HUD_SHORT_MS), cx);
                                self.hide_main_and_reset(cx);
                            }
                        }
                        Err(e) => {
                            tracing::error!(message = ? &format!("OCR failed: {}", e));
                            self.show_hud(format!("OCR failed: {}", e), Some(HUD_LONG_MS), cx);
                        }
                    }
                }

                #[cfg(not(all(target_os = "macos", feature = "ocr")))]
                {
                    self.show_hud("OCR is only supported on macOS".to_string(), Some(HUD_MEDIUM_MS), cx);
                }
            }
            // Clipboard delete actions
            "clipboard_delete_multiple" => {
                let filter_text = match &self.current_view {
                    AppView::ClipboardHistoryView { filter, .. } => filter.trim().to_string(),
                    _ => String::new(),
                };

                if filter_text.is_empty() {
                    self.show_hud(
                        "Type in search first, then use Delete Entries...".to_string(),
                        Some(2500),
                        cx,
                    );
                    return;
                }

                let filter_lower = filter_text.to_lowercase();
                let ids_to_delete: Vec<String> = self
                    .cached_clipboard_entries
                    .iter()
                    .filter(|entry| entry.text_preview.to_lowercase().contains(&filter_lower))
                    .map(|entry| entry.id.clone())
                    .collect();

                if ids_to_delete.is_empty() {
                    self.show_hud("No matching entries to delete".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                }

                let delete_count = ids_to_delete.len();
                let message = format!(
                    "Are you sure you want to delete these {} matching clipboard entries?",
                    delete_count
                );

                cx.spawn(async move |this, cx| {
                    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);
                    let open_result = cx.update(|cx| {
                        let main_bounds =
                            if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(x as f32),
                                        y: gpui::px(y as f32),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(w as f32),
                                        height: gpui::px(h as f32),
                                    },
                                }
                            } else {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(100.0),
                                        y: gpui::px(100.0),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(600.0),
                                        height: gpui::px(400.0),
                                    },
                                }
                            };

                        let sender = confirm_tx.clone();
                        let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                            let _ = sender.try_send(confirmed);
                        });

                        open_confirm_window(
                            cx,
                            main_bounds,
                            None,
                            message,
                            Some("Yes".to_string()),
                            Some("Cancel".to_string()),
                            on_choice,
                        )
                    });

                    match open_result {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            this.update(cx, |this, cx| {
                                tracing::error!(message = ?
                                    &format!("Failed to open confirmation modal: {}", e),
                                );
                                this.show_hud(
                                    "Failed to open confirmation dialog".to_string(),
                                    Some(2500),
                                    cx,
                                );
                            })
                            .ok();
                            return;
                        }
                        Err(_) => return,
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    this.update(cx, move |this, cx| {
                        let mut deleted = 0usize;
                        let mut failed = 0usize;
                        for id in ids_to_delete {
                            match clipboard_history::remove_entry(&id) {
                                Ok(()) => deleted += 1,
                                Err(e) => {
                                    failed += 1;
                                    tracing::error!(message = ?
                                        &format!("Failed to delete clipboard entry {}: {}", id, e),
                                    );
                                }
                            }
                        }

                        this.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                        if let AppView::ClipboardHistoryView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = 0;
                            if let Some(first) = this.cached_clipboard_entries.first() {
                                this.focused_clipboard_entry_id = Some(first.id.clone());
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(0, ScrollStrategy::Top);
                            } else {
                                this.focused_clipboard_entry_id = None;
                            }
                        }
                        cx.notify();

                        if failed == 0 {
                            this.show_hud(format!("Deleted {} entries", deleted), Some(2500), cx);
                        } else {
                            this.show_hud(
                                format!("Deleted {}, failed {}", deleted, failed),
                                Some(HUD_LONG_MS),
                                cx,
                            );
                        }
                    })
                    .ok();
                })
                .detach();
                return;
            }
            "clipboard_delete" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                match clipboard_history::remove_entry(&entry.id) {
                    Ok(()) => {
                        tracing::info!(category = "UI", message = ? &format!("Deleted clipboard entry: {}", entry.id));
                        // Refresh cached entries
                        self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);

                        // Update selection in ClipboardHistoryView
                        if let AppView::ClipboardHistoryView {
                            filter,
                            selected_index,
                        } = &mut self.current_view
                        {
                            let filtered_entries: Vec<_> = if filter.is_empty() {
                                self.cached_clipboard_entries.iter().enumerate().collect()
                            } else {
                                let filter_lower = filter.to_lowercase();
                                self.cached_clipboard_entries
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, e)| {
                                        e.text_preview.to_lowercase().contains(&filter_lower)
                                    })
                                    .collect()
                            };

                            // Keep selection in bounds after deletion
                            if !filtered_entries.is_empty() {
                                *selected_index =
                                    (*selected_index).min(filtered_entries.len().saturating_sub(1));
                                self.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                self.focused_clipboard_entry_id = filtered_entries
                                    .get(*selected_index)
                                    .map(|(_, entry)| entry.id.clone());
                            } else {
                                *selected_index = 0;
                                self.focused_clipboard_entry_id = None;
                            }
                        }

                        self.show_hud("Entry deleted".to_string(), Some(HUD_SHORT_MS), cx);
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to delete clipboard entry: {}", e));
                        self.show_hud(format!("Delete failed: {}", e), Some(HUD_LONG_MS), cx);
                    }
                }
                return;
            }
            "clipboard_delete_all" => {
                // Delete all unpinned entries
                let unpinned_count = self
                    .cached_clipboard_entries
                    .iter()
                    .filter(|e| !e.pinned)
                    .count();

                if unpinned_count == 0 {
                    self.show_hud("No unpinned entries to delete".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                }

                let message = format!(
                    "Are you sure you want to delete all {} unpinned clipboard entries?",
                    unpinned_count
                );

                cx.spawn(async move |this, cx| {
                    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);
                    let open_result = cx.update(|cx| {
                        let main_bounds =
                            if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(x as f32),
                                        y: gpui::px(y as f32),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(w as f32),
                                        height: gpui::px(h as f32),
                                    },
                                }
                            } else {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(100.0),
                                        y: gpui::px(100.0),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(600.0),
                                        height: gpui::px(400.0),
                                    },
                                }
                            };

                        let sender = confirm_tx.clone();
                        let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                            let _ = sender.try_send(confirmed);
                        });

                        open_confirm_window(
                            cx,
                            main_bounds,
                            None,
                            message,
                            Some("Yes".to_string()),
                            Some("Cancel".to_string()),
                            on_choice,
                        )
                    });

                    match open_result {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            this.update(cx, |this, cx| {
                                tracing::error!(message = ?
                                    &format!("Failed to open confirmation modal: {}", e),
                                );
                                this.show_hud(
                                    "Failed to open confirmation dialog".to_string(),
                                    Some(2500),
                                    cx,
                                );
                            })
                            .ok();
                            return;
                        }
                        Err(_) => return,
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    this.update(cx, move |this, cx| {
                        match clipboard_history::clear_unpinned_history() {
                            Ok(()) => {
                                tracing::info!(category = "UI", message = ?
                                    &format!(
                                        "Deleted {} unpinned clipboard entries",
                                        unpinned_count
                                    ),
                                );
                                this.cached_clipboard_entries =
                                    clipboard_history::get_cached_entries(100);

                                // Reset selection
                                if let AppView::ClipboardHistoryView { selected_index, .. } =
                                    &mut this.current_view
                                {
                                    *selected_index = 0;
                                    if let Some(first) = this.cached_clipboard_entries.first() {
                                        this.focused_clipboard_entry_id = Some(first.id.clone());
                                    } else {
                                        this.focused_clipboard_entry_id = None;
                                    }
                                }

                                this.show_hud(
                                    format!(
                                        "Deleted {} entries (pinned preserved)",
                                        unpinned_count
                                    ),
                                    Some(2500),
                                    cx,
                                );
                                cx.notify();
                            }
                            Err(e) => {
                                tracing::error!(message = ?
                                    &format!("Failed to clear unpinned history: {}", e),
                                );
                                this.show_hud(format!("Delete failed: {}", e), Some(HUD_LONG_MS), cx);
                            }
                        }
                    })
                    .ok();
                })
                .detach();
                return;
            }

            "clipboard_save_file" => {
                let Some(entry) = selected_clipboard_entry.clone() else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Clipboard content unavailable".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                // Determine filename and content based on type
                let (file_content, extension) = match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => (content.into_bytes(), "txt"),
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                        else {
                            self.show_hud("Failed to decode image".to_string(), Some(HUD_MEDIUM_MS), cx);
                            return;
                        };
                        (png_bytes, "png")
                    }
                };

                // Get save location (Desktop or home)
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
                let desktop = home.join("Desktop");
                let save_dir = if desktop.exists() { desktop } else { home };

                // Generate unique filename with timestamp
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let filename = format!("clipboard-{}.{}", timestamp, extension);
                let save_path = save_dir.join(&filename);

                match std::fs::write(&save_path, &file_content) {
                    Ok(()) => {
                        tracing::info!(category = "UI", message = ? &format!("Saved clipboard to: {:?}", save_path));
                        self.show_hud(format!("Saved to: {}", save_path.display()), Some(HUD_LONG_MS), cx);
                        self.reveal_in_finder(&save_path);
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to save file: {}", e));
                        self.show_hud(format!("Save failed: {}", e), Some(HUD_LONG_MS), cx);
                    }
                }
                return;
            }
            "clipboard_save_snippet" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Text {
                    self.show_hud(
                        "Only text can be saved as snippet".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Clipboard content unavailable".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                // Generate a default keyword from the first few words
                let default_keyword: String = content
                    .chars()
                    .take(20)
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
                    .to_lowercase();
                let default_keyword = if default_keyword.is_empty() {
                    "snippet".to_string()
                } else {
                    default_keyword
                };

                // Create snippet file in extensions directory
                let kenv = dirs::home_dir()
                    .map(|h| h.join(".kenv"))
                    .unwrap_or_else(|| std::path::PathBuf::from("/"));
                let extensions_dir = kenv.join("extensions");
                let snippets_file = extensions_dir.join("clipboard-snippets.md");

                // Ensure extensions directory exists
                if !extensions_dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(&extensions_dir) {
                        tracing::error!(message = ? &format!("Failed to create extensions dir: {}", e));
                        self.show_hud(format!("Failed to create snippets: {}", e), Some(HUD_LONG_MS), cx);
                        return;
                    }
                }

                // Generate unique keyword with timestamp suffix
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() % 10000)
                    .unwrap_or(0);
                let keyword = format!("{}-{}", default_keyword, timestamp);

                // Create snippet entry with proper fence handling
                let fence = if content.contains("```") {
                    "~~~~"
                } else {
                    "```"
                };
                let snippet_entry = format!(
                    "\n## {}\n\n{}\nname: {}\ntool: paste\nkeyword: {}\n{}\n\n{}paste\n{}\n{}\n",
                    keyword, fence, keyword, keyword, fence, fence, content, fence
                );

                // Append to snippets file
                let result = if snippets_file.exists() {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(&snippets_file)
                        .and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(snippet_entry.as_bytes())
                        })
                } else {
                    let header =
                        "# Clipboard Snippets\n\nSnippets created from clipboard history.\n";
                    std::fs::write(&snippets_file, format!("{}{}", header, snippet_entry))
                };

                match result {
                    Ok(()) => {
                        tracing::info!(category = "UI", message = ? &format!("Created snippet with keyword: {}", keyword));
                        self.show_hud(
                            format!("Snippet created: type '{}' to paste", keyword),
                            Some(HUD_LONG_MS),
                            cx,
                        );
                        // Refresh scripts to pick up new snippet
                        self.refresh_scripts(cx);
                    }
                    Err(e) => {
                        tracing::error!(message = ? &format!("Failed to save snippet: {}", e));
                        self.show_hud(format!("Save failed: {}", e), Some(HUD_LONG_MS), cx);
                    }
                }
                return;
            }

            // Scriptlet-specific actions
            "edit_scriptlet" => {
                tracing::info!(category = "UI", message = ? "Edit scriptlet action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor (e.g., "/path/to/file.md#slug" -> "/path/to/file.md")
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::PathBuf::from(path_str);
                            let editor_launch_rx = self.launch_editor_with_feedback_async(&path);
                            cx.spawn(async move |this, cx| {
                                let Ok(launch_result) = editor_launch_rx.recv().await else {
                                    return;
                                };

                                this.update(cx, |this, cx| match launch_result {
                                    Ok(()) => {
                                        this.hide_main_and_reset(cx);
                                    }
                                    Err(message) => {
                                        this.show_hud(message, Some(HUD_LONG_MS), cx);
                                    }
                                })
                                .ok();
                            })
                            .detach();
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "reveal_scriptlet_in_finder" => {
                tracing::info!(category = "UI", message = ? "Reveal scriptlet in Finder action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::Path::new(path_str);
                            let reveal_result_rx = self.reveal_in_finder_with_feedback_async(path);
                            cx.spawn(async move |this, cx| {
                                let Ok(reveal_result) = reveal_result_rx.recv().await else {
                                    return;
                                };

                                this.update(cx, |this, cx| match reveal_result {
                                    Ok(()) => {
                                        this.show_hud(
                                            "Opened in Finder".to_string(),
                                            Some(HUD_SHORT_MS),
                                            cx,
                                        );
                                        this.hide_main_and_reset(cx);
                                    }
                                    Err(message) => {
                                        this.show_hud(message, Some(2500), cx);
                                    }
                                })
                                .ok();
                            })
                            .detach();
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "copy_scriptlet_path" => {
                tracing::info!(category = "UI", message = ? "Copy scriptlet path action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);

                            #[cfg(target_os = "macos")]
                            {
                                match self.pbcopy(path_str) {
                                    Ok(_) => {
                                        tracing::info!(category = "UI", message = ?
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Copied: {}", path_str),
                                            Some(HUD_MEDIUM_MS),
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        tracing::error!(message = ? &format!("pbcopy failed: {}", e));
                                        self.show_hud(
                                            "Failed to copy path".to_string(),
                                            Some(HUD_LONG_MS),
                                            cx,
                                        );
                                    }
                                }
                            }

                            #[cfg(not(target_os = "macos"))]
                            {
                                use arboard::Clipboard;
                                match Clipboard::new().and_then(|mut c| c.set_text(path_str)) {
                                    Ok(_) => {
                                        tracing::info!(category = "UI", message = ?
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Copied: {}", path_str),
                                            Some(HUD_MEDIUM_MS),
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        tracing::error!(message = ?
                                            &format!("Failed to copy path: {}", e),
                                        );
                                        self.show_hud(
                                            "Failed to copy path".to_string(),
                                            Some(HUD_LONG_MS),
                                            cx,
                                        );
                                    }
                                }
                            }
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }

            "copy_content" => {
                tracing::info!(category = "UI", message = ? "Copy content action");
                if let Some(result) = self.get_selected_result() {
                    // Get the file path based on the result type
                    let file_path_opt: Option<String> = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(m.script.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Agent(m) => {
                            Some(m.agent.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            // Extract just the path without the anchor (e.g., "/path/to/file.md#slug" -> "/path/to/file.md")
                            m.scriptlet
                                .file_path
                                .as_ref()
                                .map(|p| p.split('#').next().unwrap_or(p).to_string())
                        }
                        _ => None,
                    };

                    if let Some(file_path) = file_path_opt {
                        // Read the file content
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => {
                                #[cfg(target_os = "macos")]
                                {
                                    match self.pbcopy(&content) {
                                        Ok(_) => {
                                            tracing::info!(category = "UI", message = ?
                                                &format!(
                                                    "Copied content to clipboard from: {}",
                                                    file_path
                                                ),
                                            );
                                            self.show_hud(
                                                "Content copied to clipboard".to_string(),
                                                Some(HUD_MEDIUM_MS),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!(message = ? &format!("pbcopy failed: {}", e));
                                            self.show_hud(
                                                "Failed to copy content".to_string(),
                                                Some(HUD_LONG_MS),
                                                cx,
                                            );
                                        }
                                    }
                                }

                                #[cfg(not(target_os = "macos"))]
                                {
                                    use arboard::Clipboard;
                                    match Clipboard::new().and_then(|mut c| c.set_text(&content)) {
                                        Ok(_) => {
                                            tracing::info!(category = "UI", message = ?
                                                &format!(
                                                    "Copied content to clipboard from: {}",
                                                    file_path
                                                ),
                                            );
                                            self.show_hud(
                                                "Content copied to clipboard".to_string(),
                                                Some(HUD_MEDIUM_MS),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!(message = ?
                                                &format!("Failed to copy content: {}", e),
                                            );
                                            self.show_hud(
                                                "Failed to copy content".to_string(),
                                                Some(HUD_LONG_MS),
                                                cx,
                                            );
                                        }
                                    }
                                }
                                self.hide_main_and_reset(cx);
                            }
                            Err(e) => {
                                tracing::error!(message = ?
                                    &format!("Failed to read file {}: {}", file_path, e),
                                );
                                self.show_hud(
                                    format!("Failed to read file: {}", e),
                                    Some(HUD_LONG_MS),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot copy content for this item type".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "reset_ranking" => {
                tracing::info!(category = "UI", message = ? "Reset ranking action");
                // Get the frecency path from the focused script info
                if let Some(script_info) = self.get_focused_script_info() {
                    if let Some(ref frecency_path) = script_info.frecency_path {
                        // Remove the frecency entry for this item
                        if self.frecency_store.remove(frecency_path).is_some() {
                            // Save the updated frecency store
                            if let Err(e) = self.frecency_store.save() {
                                tracing::error!(message = ?
                                    &format!("Failed to save frecency after reset: {}", e),
                                );
                            }
                            // Invalidate the grouped cache AND refresh scripts to rebuild the list
                            // This ensures the item is immediately removed from the Suggested section
                            self.invalidate_grouped_cache();
                            self.refresh_scripts(cx);
                            tracing::info!(category = "UI", message = ? &format!("Reset ranking for: {}", script_info.name));
                            self.show_hud(
                                format!("Ranking reset for \"{}\"", script_info.name),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        } else {
                            tracing::info!(category = "UI", message = ?
                                &format!("No frecency entry found for: {}", frecency_path),
                            );
                            self.show_hud(
                                "Item has no ranking to reset".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud("Item has no ranking to reset".to_string(), Some(HUD_MEDIUM_MS), cx);
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
                // Don't hide main window - stay in the main menu so user can see the change
                // The actions dialog is already closed by setting current_view = AppView::ScriptList
                // at the start of handle_action()
            }
            // Handle scriptlet actions defined via H3 headers
            action_id if action_id.starts_with("scriptlet_action:") => {
                let action_command = action_id.strip_prefix("scriptlet_action:").unwrap_or("");
                tracing::info!(category = "UI", message = ?
                    &format!("Scriptlet action triggered: {}", action_command),
                );

                // Find the scriptlet and execute its action
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(scriptlet_match) = result {
                        // Get the file path from the UI scriptlet type
                        // The file_path contains path#slug format
                        let file_path = scriptlet_match.scriptlet.file_path.clone();
                        let scriptlet_command = scriptlet_match.scriptlet.command.clone();

                        // We need to re-parse the markdown file to get the full scriptlet with actions
                        // because scripts::types::Scriptlet is a simplified type without actions
                        let action_found = if let Some(ref path_with_anchor) = file_path {
                            // Extract just the file path (before #anchor)
                            let file_only = path_with_anchor
                                .split('#')
                                .next()
                                .unwrap_or(path_with_anchor);

                            // Read and parse the markdown file
                            if let Ok(content) = std::fs::read_to_string(file_only) {
                                let parsed_scriptlets = scriptlets::parse_markdown_as_scriptlets(
                                    &content,
                                    Some(file_only),
                                );

                                // Find the matching scriptlet by command
                                let target_command = scriptlet_command.clone().unwrap_or_default();
                                if let Some(full_scriptlet) = parsed_scriptlets
                                    .iter()
                                    .find(|s| s.command == target_command)
                                {
                                    // Find the action in the scriptlet
                                    if let Some(action) = full_scriptlet
                                        .actions
                                        .iter()
                                        .find(|a| a.command == action_command)
                                    {
                                        // Create a scriptlet for executing the action
                                        let action_scriptlet = scriptlets::Scriptlet {
                                            name: action.name.clone(),
                                            command: action.command.clone(),
                                            tool: action.tool.clone(),
                                            scriptlet_content: action.code.clone(),
                                            inputs: action.inputs.clone(),
                                            group: full_scriptlet.group.clone(),
                                            preview: None,
                                            metadata: scriptlets::ScriptletMetadata {
                                                shortcut: action.shortcut.clone(),
                                                description: action.description.clone(),
                                                ..Default::default()
                                            },
                                            typed_metadata: None,
                                            schema: None,
                                            kit: full_scriptlet.kit.clone(),
                                            source_path: full_scriptlet.source_path.clone(),
                                            actions: vec![], // Actions don't have nested actions
                                        };

                                        // Pass the parent scriptlet's content to the action
                                        // This allows actions to use {{content}} to access the
                                        // parent's code (e.g., the URL for `open` tool scriptlets)
                                        let mut inputs = std::collections::HashMap::new();
                                        inputs.insert(
                                            "content".to_string(),
                                            full_scriptlet.scriptlet_content.trim().to_string(),
                                        );
                                        let options = executor::ScriptletExecOptions {
                                            inputs,
                                            ..Default::default()
                                        };
                                        match executor::run_scriptlet(&action_scriptlet, options) {
                                            Ok(exec_result) => {
                                                if exec_result.success {
                                                    tracing::info!(category = "UI", message = ?
                                                        &format!(
                                                            "Scriptlet action '{}' executed successfully",
                                                            action.name
                                                        ),
                                                    );
                                                    self.show_hud(
                                                        format!("Executed: {}", action.name),
                                                        Some(HUD_MEDIUM_MS),
                                                        cx,
                                                    );
                                                } else {
                                                    let error_msg = if exec_result.stderr.is_empty()
                                                    {
                                                        "Unknown error".to_string()
                                                    } else {
                                                        exec_result.stderr.clone()
                                                    };
                                                    tracing::error!(message = ?
                                                        &format!(
                                                            "Scriptlet action '{}' failed: {}",
                                                            action.name, error_msg
                                                        ),
                                                    );
                                                    self.show_hud(
                                                        format!("Error: {}", error_msg),
                                                        Some(HUD_LONG_MS),
                                                        cx,
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!(message = ?
                                                    &format!(
                                                        "Failed to execute scriptlet action '{}': {}",
                                                        action.name, e
                                                    ),
                                                );
                                                self.show_hud(
                                                    format!("Error: {}", e),
                                                    Some(HUD_LONG_MS),
                                                    cx,
                                                );
                                            }
                                        }
                                        self.hide_main_and_reset(cx);
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                tracing::error!(message = ?
                                    &format!("Failed to read scriptlet file: {}", file_only),
                                );
                                false
                            }
                        } else {
                            false
                        };

                        if !action_found {
                            tracing::error!(message = ?
                                &format!("Scriptlet action not found: {}", action_command),
                            );
                            self.show_hud("Scriptlet action not found".to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }

            _ => {
                // Handle SDK actions using shared helper
                self.trigger_sdk_action_internal(action_id);
            }
        }

        cx.notify();
    }
}
