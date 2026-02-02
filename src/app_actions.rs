// Actions handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: handle_action, trigger_action_by_name

fn select_clipboard_entry_meta<'a>(
    entries: &'a [clipboard_history::ClipboardEntryMeta],
    filter: &str,
    selected_index: usize,
) -> Option<&'a clipboard_history::ClipboardEntryMeta> {
    if filter.is_empty() {
        if entries.is_empty() {
            return None;
        }
        let clamped_index = selected_index.min(entries.len().saturating_sub(1));
        return entries.get(clamped_index);
    }

    let filter_lower = filter.to_lowercase();
    let filtered_entries: Vec<_> = entries
        .iter()
        .filter(|entry| entry.text_preview.to_lowercase().contains(&filter_lower))
        .collect();

    if filtered_entries.is_empty() {
        return None;
    }

    let clamped_index = selected_index.min(filtered_entries.len().saturating_sub(1));
    filtered_entries.get(clamped_index).copied()
}

#[cfg(test)]
mod app_actions_tests {
    use super::select_clipboard_entry_meta;
    use crate::clipboard_history::{ClipboardEntryMeta, ContentType};

    fn entry(id: &str, preview: &str) -> ClipboardEntryMeta {
        ClipboardEntryMeta {
            id: id.to_string(),
            content_type: ContentType::Text,
            timestamp: 0,
            pinned: false,
            text_preview: preview.to_string(),
            image_width: None,
            image_height: None,
            byte_size: 0,
            ocr_text: None,
        }
    }

    #[test]
    fn test_select_clipboard_entry_meta_filters_and_clamps() {
        let entries = vec![entry("1", "Alpha"), entry("2", "Beta"), entry("3", "Gamma")];

        let filtered = select_clipboard_entry_meta(&entries, "et", 0).unwrap();
        assert_eq!(filtered.id, "2");

        let clamped = select_clipboard_entry_meta(&entries, "", 99).unwrap();
        assert_eq!(clamped.id, "3");
    }
}

impl ScriptListApp {
    /// Helper to hide main window and set reset flag.
    /// Uses platform::hide_main_window() to hide ONLY the main window,
    /// keeping other windows like HUD notifications visible.
    fn hide_main_and_reset(&self, _cx: &mut Context<Self>) {
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
            use std::process::Command;
            match Command::new("open").arg("-R").arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Revealed in Finder: {}", path_str)),
                Err(e) => logging::log("ERROR", &format!("Failed to reveal in Finder: {}", e)),
            }
        });
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
        logging::log("UI", &format!("Action selected: {}", action_id));

        let selected_clipboard_entry = if action_id.starts_with("clipboard_") {
            self.selected_clipboard_entry()
        } else {
            None
        };

        match action_id.as_str() {
            "clipboard_pin" | "clipboard_unpin" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
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

                        cx.notify();
                    }
                    Err(e) => {
                        logging::log(
                            "ERROR",
                            &format!("Failed to toggle clipboard pin: {}", e),
                        );
                        self.show_hud(
                            format!("Failed to update pin: {}", e),
                            Some(3000),
                            cx,
                        );
                    }
                }
                return;
            }
            "clipboard_share" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Clipboard entry content unavailable".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                };

                logging::log(
                    "UI",
                    &format!(
                        "Opening share sheet for clipboard entry {} ({:?})",
                        entry.id, entry.content_type
                    ),
                );

                match entry.content_type {
                    clipboard_history::ContentType::Text => {
                        crate::platform::show_share_sheet(
                            crate::platform::ShareSheetItem::Text(content),
                        );
                    }
                    clipboard_history::ContentType::Image => {
                        if let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                        {
                            crate::platform::show_share_sheet(
                                crate::platform::ShareSheetItem::ImagePng(png_bytes),
                            );
                        } else {
                            self.show_hud(
                                "Failed to decode clipboard image".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    }
                }
                return;
            }
            // Paste to active app and close window (Enter)
            "clipboard_paste" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                logging::log("CLIPBOARD", &format!("Paste entry: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        logging::log("CLIPBOARD", "Entry copied, simulating paste");
                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            if let Err(e) = selected_text::simulate_paste_with_cg() {
                                logging::log("ERROR", &format!("Failed to simulate paste: {}", e));
                            } else {
                                logging::log("CLIPBOARD", "Simulated Cmd+V paste");
                            }
                        });
                        self.show_hud("Pasted".to_string(), Some(1000), cx);
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to paste entry: {}", e));
                        self.show_hud(format!("Failed to paste: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            "clipboard_attach_to_ai" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Clipboard entry content unavailable".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                };

                logging::log(
                    "AI",
                    &format!(
                        "Attaching clipboard entry {} ({:?}) to AI chat",
                        entry.id, entry.content_type
                    ),
                );

                match entry.content_type {
                    clipboard_history::ContentType::Text => {
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            self.show_hud("Failed to open AI window".to_string(), Some(2000), cx);
                            return;
                        }
                        ai::set_ai_input(cx, &content, false);
                    }
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                        else {
                            self.show_hud(
                                "Failed to decode clipboard image".to_string(),
                                Some(2000),
                                cx,
                            );
                            return;
                        };

                        use base64::Engine;
                        let base64_data =
                            base64::engine::general_purpose::STANDARD.encode(&png_bytes);

                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            self.show_hud("Failed to open AI window".to_string(), Some(2000), cx);
                            return;
                        }
                        ai::set_ai_input_with_image(cx, "", &base64_data, false);
                    }
                }

                self.hide_main_and_reset(cx);
                return;
            }
            // Copy to clipboard without pasting (Cmd+Enter)
            "clipboard_copy" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                logging::log("CLIPBOARD", &format!("Copying entry to clipboard: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        logging::log("CLIPBOARD", "Entry copied to clipboard");
                        self.show_hud("Copied to clipboard".to_string(), Some(1500), cx);
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                        self.show_hud(format!("Failed to copy: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            // Paste and keep window open (Opt+Enter)
            "clipboard_paste_keep_open" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                logging::log("CLIPBOARD", &format!("Paste and keep open: {}", entry.id));
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        logging::log("CLIPBOARD", "Entry copied, simulating paste");
                        // Simulate Cmd+V paste after a brief delay
                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            if let Err(e) = selected_text::simulate_paste_with_cg() {
                                logging::log("ERROR", &format!("Failed to simulate paste: {}", e));
                            } else {
                                logging::log("CLIPBOARD", "Simulated Cmd+V paste");
                            }
                        });
                        self.show_hud("Pasted".to_string(), Some(1000), cx);
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                        self.show_hud(format!("Failed to paste: {}", e), Some(2500), cx);
                    }
                }
                return;
            }
            "clipboard_quick_look" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                if let Err(e) = clipboard_history::quick_look_entry(&entry) {
                    logging::log("ERROR", &format!("Quick Look failed: {}", e));
                    self.show_hud(format!("Quick Look failed: {}", e), Some(2500), cx);
                }
                return;
            }
            _ => {}
        }

        // Close the dialog and return to script list
        self.current_view = AppView::ScriptList;
        self.pending_focus = Some(FocusTarget::MainFilter);

        match action_id.as_str() {
            "create_script" => {
                logging::log("UI", "Create script action - opening scripts folder");
                let scripts_dir = shellexpand::tilde("~/.scriptkit/scripts").to_string();
                std::thread::spawn(move || {
                    use std::process::Command;
                    match Command::new("open").arg(&scripts_dir).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened scripts folder: {}", scripts_dir))
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open scripts folder: {}", e))
                        }
                    }
                });
                self.show_hud("Opened scripts folder".to_string(), Some(1500), cx);
                self.hide_main_and_reset(cx);
            }
            "run_script" => {
                logging::log("UI", "Run script action");
                self.execute_selected(cx);
            }
            "view_logs" => {
                logging::log("UI", "View logs action");
                self.toggle_logs(cx);
            }
            "reveal_in_finder" => {
                logging::log("UI", "Reveal in Finder action");
                // First check if we have a file search path (takes priority)
                let path_opt = if let Some(path) = self.file_search_actions_path.take() {
                    logging::log("UI", &format!("Reveal in Finder (file search): {}", path));
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
                    self.reveal_in_finder(&path);
                    self.show_hud("Revealed in Finder".to_string(), Some(1500), cx);
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud(
                        "Cannot reveal this item type in Finder".to_string(),
                        Some(2000),
                        cx,
                    );
                }
            }
            "copy_path" => {
                logging::log("UI", "Copy path action");
                // First check if we have a file search path (takes priority)
                let path_str = if let Some(path) = self.file_search_actions_path.take() {
                    logging::log("UI", &format!("Copy path (file search): {}", path));
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
                            Some(2000),
                            cx,
                        );
                    }
                    path_opt
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                    None
                };

                if let Some(path_str) = path_str {
                    #[cfg(target_os = "macos")]
                    {
                        match self.pbcopy(&path_str) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_str),
                                );
                                self.show_hud(format!("Copied: {}", path_str), Some(2000), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                self.show_hud("Failed to copy path".to_string(), Some(3000), cx);
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        match Clipboard::new().and_then(|mut c| c.set_text(&path_str)) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_str),
                                );
                                self.show_hud(format!("Copied: {}", path_str), Some(2000), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                self.show_hud("Failed to copy path".to_string(), Some(3000), cx);
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                }
            }
            "copy_deeplink" => {
                logging::log("UI", "Copy deeplink action");
                if let Some(result) = self.get_selected_result() {
                    let name = result.name();
                    let deeplink_name = crate::actions::to_deeplink_name(name);
                    let deeplink_url = format!("scriptkit://run/{}", deeplink_name);

                    #[cfg(target_os = "macos")]
                    {
                        match self.pbcopy(&deeplink_url) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied deeplink to clipboard: {}", deeplink_url),
                                );
                                self.show_hud(format!("Copied: {}", deeplink_url), Some(2000), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                self.show_hud(
                                    "Failed to copy deeplink".to_string(),
                                    Some(3000),
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
                                logging::log(
                                    "UI",
                                    &format!("Copied deeplink to clipboard: {}", deeplink_url),
                                );
                                self.show_hud(format!("Copied: {}", deeplink_url), Some(2000), cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy deeplink: {}", e));
                                self.show_hud(
                                    "Failed to copy deeplink".to_string(),
                                    Some(3000),
                                    cx,
                                );
                            }
                        }
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            // Handle both legacy "configure_shortcut" and new dynamic actions
            // "add_shortcut" and "update_shortcut" open the shortcut recorder
            "configure_shortcut" | "add_shortcut" | "update_shortcut" => {
                logging::log("UI", &format!("{} action", action_id));
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
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            // "remove_shortcut" removes the existing shortcut from the registry
            "remove_shortcut" => {
                logging::log("UI", "Remove shortcut action");
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
                                logging::log(
                                    "SHORTCUT",
                                    &format!("Removed shortcut for: {}", command_id),
                                );
                                self.show_hud("Shortcut removed".to_string(), Some(2000), cx);
                                // Refresh scripts to update shortcut display
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to remove shortcut: {}", e));
                                self.show_hud(
                                    format!("Failed to remove shortcut: {}", e),
                                    Some(3000),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot remove shortcut for this item type".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            // Alias actions: add_alias, update_alias open the alias input
            "add_alias" | "update_alias" => {
                logging::log("UI", &format!("{} action", action_id));
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
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            // "remove_alias" removes the existing alias from persistence
            "remove_alias" => {
                logging::log("UI", "Remove alias action");
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
                                logging::log(
                                    "ALIAS",
                                    &format!("Removed alias for: {}", command_id),
                                );
                                self.show_hud("Alias removed".to_string(), Some(2000), cx);
                                // Refresh scripts to update alias display and registry
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to remove alias: {}", e));
                                self.show_hud(
                                    format!("Failed to remove alias: {}", e),
                                    Some(3000),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot remove alias for this item type".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            "edit_script" => {
                logging::log("UI", "Edit script action");
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
                        self.edit_script(&path);
                        self.hide_main_and_reset(cx);
                    } else {
                        self.show_hud("Cannot edit this item type".to_string(), Some(2000), cx);
                    }
                } else {
                    self.show_hud("No script selected".to_string(), Some(2000), cx);
                }
            }
            "reload_scripts" => {
                logging::log("UI", "Reload scripts action");
                self.refresh_scripts(cx);
                self.show_hud("Scripts reloaded".to_string(), Some(1500), cx);
            }
            "settings" => {
                logging::log("UI", "Settings action - opening config.ts");

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
                        "code" | "cursor" => {
                            Command::new(&editor)
                                .arg("-r")
                                .arg(&config_dir)
                                .arg(&config_file)
                                .spawn()
                        }
                        // Zed: just the file (doesn't support folder context the same way)
                        "zed" => {
                            Command::new("zed")
                                .arg(&config_file)
                                .spawn()
                        }
                        // Sublime: -a (add to current window) + folder + file
                        "subl" => {
                            Command::new("subl")
                                .arg("-a")
                                .arg(&config_dir)
                                .arg(&config_file)
                                .spawn()
                        }
                        // Generic fallback: just open the file
                        _ => {
                            Command::new(&editor)
                                .arg(&config_file)
                                .spawn()
                        }
                    };

                    match result {
                        Ok(_) => logging::log("UI", &format!("Opened config.ts in {}", editor)),
                        Err(e) => logging::log("ERROR", &format!("Failed to open editor '{}': {}", editor, e)),
                    }
                });

                self.show_hud(format!("Opening config.ts in {}", editor_for_hud), Some(1500), cx);
                self.hide_main_and_reset(cx);
            }
            "quit" => {
                logging::log("UI", "Quit action");
                PROCESS_MANAGER.kill_all_processes();
                PROCESS_MANAGER.remove_main_pid();
                cx.quit();
                return; // Early return after quit - no notify needed
            }
            "__cancel__" => {
                logging::log("UI", "Actions dialog cancelled");
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
            }
            // File search specific actions
            "open_file" | "open_directory" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Opening file: {}", path));
                    let _ = crate::file_search::open_file(path);
                    self.file_search_actions_path = None;
                    self.close_and_reset_window(cx);
                }
            }
            "quick_look" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Quick Look: {}", path));
                    let _ = crate::file_search::quick_look(path);
                    self.file_search_actions_path = None;
                    // Don't close window for Quick Look - user may want to continue
                }
            }
            "open_with" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Open With: {}", path));
                    let _ = crate::file_search::open_with(path);
                    self.file_search_actions_path = None;
                }
            }
            "show_info" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Show Info: {}", path));
                    let _ = crate::file_search::show_info(path);
                    self.file_search_actions_path = None;
                }
            }
            "copy_filename" => {
                if let Some(ref path) = self.file_search_actions_path {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    logging::log("UI", &format!("Copy filename: {}", filename));
                    #[cfg(target_os = "macos")]
                    {
                        let _ = self.pbcopy(filename);
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        let _ = Clipboard::new().and_then(|mut c| c.set_text(filename));
                    }
                    self.show_hud(format!("Copied: {}", filename), Some(2000), cx);
                    self.file_search_actions_path = None;
                    self.hide_main_and_reset(cx);
                }
            }
            "clipboard_open_with" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Failed to load clipboard content".to_string(), Some(2000), cx);
                    return;
                };

                let full_entry = clipboard_history::ClipboardEntry {
                    id: entry.id.clone(),
                    content,
                    content_type: entry.content_type,
                    timestamp: entry.timestamp,
                    pinned: entry.pinned,
                    ocr_text: entry.ocr_text.clone(),
                };

                let temp_path = match clipboard_history::save_entry_to_temp_file(&full_entry) {
                    Ok(path) => path,
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to save temp file: {}", e));
                        self.show_hud("Failed to save temp file".to_string(), Some(2000), cx);
                        return;
                    }
                };

                #[cfg(target_os = "macos")]
                {
                    let path_str = temp_path.to_string_lossy().to_string();
                    if let Err(e) = crate::file_search::open_with(&path_str) {
                        logging::log("ERROR", &format!("Open With failed: {}", e));
                        self.show_hud("Failed to open \"Open With\"".to_string(), Some(2000), cx);
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let _ = temp_path;
                    self.show_hud(
                        "\"Open With\" is only supported on macOS".to_string(),
                        Some(2000),
                        cx,
                    );
                }
            }
            "clipboard_annotate_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "CleanShot actions are only available for images".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    if let Err(e) = clipboard_history::copy_entry_to_clipboard(&entry.id) {
                        logging::log("ERROR", &format!("Failed to copy image: {}", e));
                        self.show_hud("Failed to copy image".to_string(), Some(2000), cx);
                        return;
                    }

                    let url = "cleanshot://open-from-clipboard";
                    match std::process::Command::new("open").arg(url).spawn() {
                        Ok(_) => {
                            self.show_hud(
                                "Opening CleanShot X…".to_string(),
                                Some(1500),
                                cx,
                            );
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open CleanShot X: {}", e));
                            self.show_hud(
                                "Failed to open CleanShot X".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_hud(
                        "CleanShot actions are only supported on macOS".to_string(),
                        Some(2000),
                        cx,
                    );
                }
            }
            "clipboard_upload_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "CleanShot actions are only available for images".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_hud("Failed to load image content".to_string(), Some(2000), cx);
                        return;
                    };

                    let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content) else {
                        self.show_hud("Failed to decode image".to_string(), Some(2000), cx);
                        return;
                    };

                    let temp_path = std::env::temp_dir().join(format!(
                        "script-kit-clipboard-{}.png",
                        uuid::Uuid::new_v4()
                    ));

                    if let Err(e) = std::fs::write(&temp_path, png_bytes) {
                        logging::log("ERROR", &format!("Failed to write temp image: {}", e));
                        self.show_hud("Failed to save image".to_string(), Some(2000), cx);
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
                                Some(1500),
                                cx,
                            );
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open CleanShot X: {}", e));
                            self.show_hud(
                                "Failed to open CleanShot X".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_hud(
                        "CleanShot actions are only supported on macOS".to_string(),
                        Some(2000),
                        cx,
                    );
                }
            }
            "clipboard_ocr" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "OCR is only available for images".to_string(),
                        Some(2000),
                        cx,
                    );
                    return;
                }

                // Check if we already have cached OCR text
                if let Some(ref cached_text) = entry.ocr_text {
                    if !cached_text.trim().is_empty() {
                        logging::log("OCR", "Using cached OCR text");
                        #[cfg(target_os = "macos")]
                        {
                            let _ = self.pbcopy(cached_text);
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            use arboard::Clipboard;
                            let _ = Clipboard::new().and_then(|mut c| c.set_text(cached_text.clone()));
                        }
                        self.show_hud("Copied text from image".to_string(), Some(1500), cx);
                        self.hide_main_and_reset(cx);
                        return;
                    }
                }

                #[cfg(all(target_os = "macos", feature = "ocr"))]
                {
                    // Get image content
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_hud("Failed to load image content".to_string(), Some(2000), cx);
                        return;
                    };

                    // Decode to RGBA bytes for OCR
                    let Some((width, height, rgba_bytes)) = clipboard_history::decode_to_rgba_bytes(&content) else {
                        self.show_hud("Failed to decode image".to_string(), Some(2000), cx);
                        return;
                    };

                    logging::log("OCR", &format!("Starting OCR on {}x{} image", width, height));
                    self.show_hud("Extracting text...".to_string(), Some(1500), cx);

                    // Perform OCR synchronously (it runs on a background thread internally)
                    // For a truly async approach, we'd need to integrate with GPUI's async system
                    let entry_id = entry.id.clone();
                    match script_kit_gpui::ocr::extract_text_from_rgba(width, height, &rgba_bytes) {
                        Ok(text) => {
                            if text.trim().is_empty() {
                                logging::log("OCR", "No text found in image");
                                self.show_hud("No text found in image".to_string(), Some(2000), cx);
                            } else {
                                logging::log("OCR", &format!("Extracted {} characters", text.len()));
                                
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
                                    let _ = Clipboard::new().and_then(|mut c| c.set_text(text.clone()));
                                }
                                
                                self.show_hud("Copied text from image".to_string(), Some(1500), cx);
                                self.hide_main_and_reset(cx);
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("OCR failed: {}", e));
                            self.show_hud(format!("OCR failed: {}", e), Some(3000), cx);
                        }
                    }
                }

                #[cfg(not(all(target_os = "macos", feature = "ocr")))]
                {
                    self.show_hud(
                        "OCR is only supported on macOS".to_string(),
                        Some(2000),
                        cx,
                    );
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
                    self.show_hud("No matching entries to delete".to_string(), Some(2000), cx);
                    return;
                }

                let mut deleted = 0usize;
                let mut failed = 0usize;
                for id in ids_to_delete {
                    match clipboard_history::remove_entry(&id) {
                        Ok(()) => deleted += 1,
                        Err(e) => {
                            failed += 1;
                            logging::log(
                                "ERROR",
                                &format!("Failed to delete clipboard entry {}: {}", id, e),
                            );
                        }
                    }
                }

                self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                if let AppView::ClipboardHistoryView { selected_index, .. } = &mut self.current_view
                {
                    *selected_index = 0;
                    if let Some(first) = self.cached_clipboard_entries.first() {
                        self.focused_clipboard_entry_id = Some(first.id.clone());
                        self.clipboard_list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                    } else {
                        self.focused_clipboard_entry_id = None;
                    }
                }
                cx.notify();

                if failed == 0 {
                    self.show_hud(format!("Deleted {} entries", deleted), Some(2500), cx);
                } else {
                    self.show_hud(
                        format!("Deleted {}, failed {}", deleted, failed),
                        Some(3000),
                        cx,
                    );
                }
                return;
            }
            "clipboard_delete" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                match clipboard_history::remove_entry(&entry.id) {
                    Ok(()) => {
                        logging::log("UI", &format!("Deleted clipboard entry: {}", entry.id));
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
                                *selected_index = (*selected_index).min(filtered_entries.len().saturating_sub(1));
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

                        self.show_hud("Entry deleted".to_string(), Some(1500), cx);
                        cx.notify();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to delete clipboard entry: {}", e));
                        self.show_hud(format!("Delete failed: {}", e), Some(3000), cx);
                    }
                }
                return;
            }
            "clipboard_delete_all" => {
                // Delete all unpinned entries
                let unpinned_count = self.cached_clipboard_entries.iter().filter(|e| !e.pinned).count();
                
                if unpinned_count == 0 {
                    self.show_hud("No unpinned entries to delete".to_string(), Some(2000), cx);
                    return;
                }

                match clipboard_history::clear_unpinned_history() {
                    Ok(()) => {
                        logging::log("UI", &format!("Deleted {} unpinned clipboard entries", unpinned_count));
                        self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);

                        // Reset selection
                        if let AppView::ClipboardHistoryView {
                            selected_index,
                            ..
                        } = &mut self.current_view
                        {
                            *selected_index = 0;
                            if let Some(first) = self.cached_clipboard_entries.first() {
                                self.focused_clipboard_entry_id = Some(first.id.clone());
                            } else {
                                self.focused_clipboard_entry_id = None;
                            }
                        }

                        self.show_hud(format!("Deleted {} entries (pinned preserved)", unpinned_count), Some(2500), cx);
                        cx.notify();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to clear unpinned history: {}", e));
                        self.show_hud(format!("Delete failed: {}", e), Some(3000), cx);
                    }
                }
                return;
            }
            "clipboard_save_file" => {
                let Some(entry) = selected_clipboard_entry.clone() else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Clipboard content unavailable".to_string(), Some(2000), cx);
                    return;
                };

                // Determine filename and content based on type
                let (file_content, extension) = match entry.content_type {
                    clipboard_history::ContentType::Text => (content.into_bytes(), "txt"),
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content) else {
                            self.show_hud("Failed to decode image".to_string(), Some(2000), cx);
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
                        logging::log("UI", &format!("Saved clipboard to: {:?}", save_path));
                        self.show_hud(format!("Saved: {}", filename), Some(2500), cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to save file: {}", e));
                        self.show_hud(format!("Save failed: {}", e), Some(3000), cx);
                    }
                }
                return;
            }
            "clipboard_save_snippet" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(2000), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Text {
                    self.show_hud("Only text can be saved as snippet".to_string(), Some(2000), cx);
                    return;
                }

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Clipboard content unavailable".to_string(), Some(2000), cx);
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
                        logging::log("ERROR", &format!("Failed to create extensions dir: {}", e));
                        self.show_hud(format!("Failed to create snippets: {}", e), Some(3000), cx);
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
                let fence = if content.contains("```") { "~~~~" } else { "```" };
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
                    let header = "# Clipboard Snippets\n\nSnippets created from clipboard history.\n";
                    std::fs::write(&snippets_file, format!("{}{}", header, snippet_entry))
                };

                match result {
                    Ok(()) => {
                        logging::log("UI", &format!("Created snippet with keyword: {}", keyword));
                        self.show_hud(format!("Snippet created: type '{}' to paste", keyword), Some(3000), cx);
                        // Refresh scripts to pick up new snippet
                        self.refresh_scripts(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to save snippet: {}", e));
                        self.show_hud(format!("Save failed: {}", e), Some(3000), cx);
                    }
                }
                return;
            }

            // Scriptlet-specific actions
            "edit_scriptlet" => {
                logging::log("UI", "Edit scriptlet action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor (e.g., "/path/to/file.md#slug" -> "/path/to/file.md")
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::PathBuf::from(path_str);
                            self.edit_script(&path);
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            "reveal_scriptlet_in_finder" => {
                logging::log("UI", "Reveal scriptlet in Finder action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::Path::new(path_str);
                            self.reveal_in_finder(path);
                            self.show_hud("Revealed in Finder".to_string(), Some(1500), cx);
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            "copy_scriptlet_path" => {
                logging::log("UI", "Copy scriptlet path action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);

                            #[cfg(target_os = "macos")]
                            {
                                match self.pbcopy(path_str) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Copied: {}", path_str),
                                            Some(2000),
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                        self.show_hud(
                                            "Failed to copy path".to_string(),
                                            Some(3000),
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
                                        logging::log(
                                            "UI",
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Copied: {}", path_str),
                                            Some(2000),
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "ERROR",
                                            &format!("Failed to copy path: {}", e),
                                        );
                                        self.show_hud(
                                            "Failed to copy path".to_string(),
                                            Some(3000),
                                            cx,
                                        );
                                    }
                                }
                            }
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            "copy_content" => {
                logging::log("UI", "Copy content action");
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
                                            logging::log(
                                                "UI",
                                                &format!(
                                                    "Copied content to clipboard from: {}",
                                                    file_path
                                                ),
                                            );
                                            self.show_hud(
                                                "Content copied to clipboard".to_string(),
                                                Some(2000),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                            self.show_hud(
                                                "Failed to copy content".to_string(),
                                                Some(3000),
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
                                            logging::log(
                                                "UI",
                                                &format!(
                                                    "Copied content to clipboard from: {}",
                                                    file_path
                                                ),
                                            );
                                            self.show_hud(
                                                "Content copied to clipboard".to_string(),
                                                Some(2000),
                                                cx,
                                            );
                                        }
                                        Err(e) => {
                                            logging::log(
                                                "ERROR",
                                                &format!("Failed to copy content: {}", e),
                                            );
                                            self.show_hud(
                                                "Failed to copy content".to_string(),
                                                Some(3000),
                                                cx,
                                            );
                                        }
                                    }
                                }
                                self.hide_main_and_reset(cx);
                            }
                            Err(e) => {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to read file {}: {}", file_path, e),
                                );
                                self.show_hud(
                                    format!("Failed to read file: {}", e),
                                    Some(3000),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_hud(
                            "Cannot copy content for this item type".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            "reset_ranking" => {
                logging::log("UI", "Reset ranking action");
                // Get the frecency path from the focused script info
                if let Some(script_info) = self.get_focused_script_info() {
                    if let Some(ref frecency_path) = script_info.frecency_path {
                        // Remove the frecency entry for this item
                        if self.frecency_store.remove(frecency_path).is_some() {
                            // Save the updated frecency store
                            if let Err(e) = self.frecency_store.save() {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to save frecency after reset: {}", e),
                                );
                            }
                            // Invalidate the grouped cache AND refresh scripts to rebuild the list
                            // This ensures the item is immediately removed from the Suggested section
                            self.invalidate_grouped_cache();
                            self.refresh_scripts(cx);
                            logging::log("UI", &format!("Reset ranking for: {}", script_info.name));
                            self.show_hud(
                                format!("Ranking reset for \"{}\"", script_info.name),
                                Some(2000),
                                cx,
                            );
                        } else {
                            logging::log(
                                "UI",
                                &format!("No frecency entry found for: {}", frecency_path),
                            );
                            self.show_hud(
                                "Item has no ranking to reset".to_string(),
                                Some(2000),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud("Item has no ranking to reset".to_string(), Some(2000), cx);
                    }
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
                // Don't hide main window - stay in the main menu so user can see the change
                // The actions dialog is already closed by setting current_view = AppView::ScriptList
                // at the start of handle_action()
            }
            // Handle scriptlet actions defined via H3 headers
            action_id if action_id.starts_with("scriptlet_action:") => {
                let action_command = action_id.strip_prefix("scriptlet_action:").unwrap_or("");
                logging::log(
                    "UI",
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
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Scriptlet action '{}' executed successfully",
                                                            action.name
                                                        ),
                                                    );
                                                    self.show_hud(
                                                        format!("Executed: {}", action.name),
                                                        Some(2000),
                                                        cx,
                                                    );
                                                } else {
                                                    let error_msg = if exec_result.stderr.is_empty()
                                                    {
                                                        "Unknown error".to_string()
                                                    } else {
                                                        exec_result.stderr.clone()
                                                    };
                                                    logging::log(
                                                        "ERROR",
                                                        &format!(
                                                            "Scriptlet action '{}' failed: {}",
                                                            action.name, error_msg
                                                        ),
                                                    );
                                                    self.show_hud(
                                                        format!("Error: {}", error_msg),
                                                        Some(3000),
                                                        cx,
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                logging::log(
                                                    "ERROR",
                                                    &format!(
                                                        "Failed to execute scriptlet action '{}': {}",
                                                        action.name, e
                                                    ),
                                                );
                                                self.show_hud(
                                                    format!("Error: {}", e),
                                                    Some(3000),
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
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to read scriptlet file: {}", file_only),
                                );
                                false
                            }
                        } else {
                            false
                        };

                        if !action_found {
                            logging::log(
                                "ERROR",
                                &format!("Scriptlet action not found: {}", action_command),
                            );
                            self.show_hud("Scriptlet action not found".to_string(), Some(2000), cx);
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                } else {
                    self.show_hud("No item selected".to_string(), Some(2000), cx);
                }
            }
            _ => {
                // Handle SDK actions using shared helper
                self.trigger_sdk_action_internal(&action_id);
            }
        }

        cx.notify();
    }

    /// Internal helper for triggering SDK actions - used by both handle_action and trigger_action_by_name
    fn trigger_sdk_action_internal(&mut self, action_name: &str) {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                let send_result = if action.has_action {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action with handler: '{}' (has_action=true), sending ActionTriggered",
                            action_name
                        ),
                    );
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::action_triggered(
                            action_name.to_string(),
                            action.value.clone(),
                            self.arg_input.text().to_string(),
                        );
                        Some(sender.try_send(msg))
                    } else {
                        None
                    }
                } else if let Some(ref value) = action.value {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action without handler: '{}' (has_action=false), submitting value: {:?}",
                            action_name, value
                        ),
                    );
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::Submit {
                            id: "action".to_string(),
                            value: Some(value.clone()),
                        };
                        Some(sender.try_send(msg))
                    } else {
                        None
                    }
                } else {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action '{}' has no value and has_action=false",
                            action_name
                        ),
                    );
                    None
                };

                // Log any send errors
                if let Some(result) = send_result {
                    match result {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log(
                                "WARN",
                                &format!(
                                    "Response channel full - action '{}' dropped",
                                    action_name
                                ),
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                }
            } else {
                logging::log("UI", &format!("Unknown action: {}", action_name));
            }
        } else {
            logging::log("UI", &format!("Unknown action: {}", action_name));
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if actions.iter().any(|a| a.name == action_name) {
                logging::log(
                    "ACTIONS",
                    &format!("Triggering SDK action '{}' via shortcut", action_name),
                );
                self.trigger_sdk_action_internal(action_name);
                cx.notify();
                return true;
            }
        }
        false
    }
}
