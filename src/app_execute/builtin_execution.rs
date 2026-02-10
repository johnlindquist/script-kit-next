fn ai_open_failure_message(error: impl std::fmt::Display) -> String {
    format!("Failed to open AI: {}", error)
}

fn favorites_loaded_message(count: usize) -> String {
    if count == 1 {
        "Loaded 1 favorite".to_string()
    } else {
        format!("Loaded {} favorites", count)
    }
}

#[cfg(target_os = "macos")]
fn applescript_escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(target_os = "macos")]
fn applescript_list_literal(values: &[String]) -> String {
    let escaped_values = values
        .iter()
        .map(|value| format!("\"{}\"", applescript_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{}}}", escaped_values)
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Result<String, String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|error| {
            format!(
                "failed to spawn osascript for builtin picker flow: {}",
                error
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "osascript exited with status {} for builtin picker flow: {}",
            output.status,
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(target_os = "macos")]
fn choose_from_list(prompt: &str, ok_button: &str, values: &[String]) -> Result<Option<String>, String> {
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
        prompt = applescript_escape(prompt),
        ok_button = applescript_escape(ok_button),
    );

    let selected = run_osascript(&script)?;
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

#[cfg(target_os = "macos")]
fn prompt_for_text(prompt: &str, default_value: &str, ok_button: &str) -> Result<Option<String>, String> {
    let script = format!(
        r#"try
set dialogResult to display dialog "{prompt}" default answer "{default_value}" buttons {{"Cancel", "{ok_button}"}} default button "{ok_button}"
return text returned of dialogResult
on error number -128
return ""
end try"#,
        prompt = applescript_escape(prompt),
        default_value = applescript_escape(default_value),
        ok_button = applescript_escape(ok_button),
    );

    let value = run_osascript(&script)?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn emoji_picker_label(emoji: &script_kit_gpui::emoji::Emoji) -> String {
    format!("{}  {}", emoji.emoji, emoji.name)
}

fn quicklink_picker_label(quicklink: &script_kit_gpui::quicklinks::Quicklink) -> String {
    format!(
        "{}  {}",
        applescript_escape(&quicklink.name),
        applescript_escape(&quicklink.url_template)
    )
}

impl ScriptListApp {
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

    fn execute_builtin(&mut self, entry: &builtins::BuiltInEntry, cx: &mut Context<Self>) {
        self.execute_builtin_with_query(entry, None, cx);
    }

    fn execute_builtin_with_query(
        &mut self,
        entry: &builtins::BuiltInEntry,
        query_override: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!("Executing built-in: {} (id: {})", entry.name, entry.id),
        );

        // Clear any stale actions popup from previous view
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Check if this command requires confirmation - open modal if so
        if self.config.requires_confirmation(&entry.id) {
            logging::log(
                "EXEC",
                &format!("Opening confirmation modal for: {}", entry.id),
            );

            // Clone what we need for the spawned task
            let entry_id = entry.id.clone();
            let entry_name = entry.name.clone();
            let confirm_sender = self.builtin_confirm_sender.clone();

            // Spawn a task to open the confirm modal
            // We need to do this async because we need App context for open_confirm_window
            cx.spawn(async move |this, cx| {
                cx.update(|cx| {
                    // Get main window bounds from native API for positioning
                    let main_bounds = if let Some((x, y, w, h)) = platform::get_main_window_bounds()
                    {
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
                        // Fallback: use sensible defaults
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

                    // Create the callback that sends result via channel
                    let sender = confirm_sender.clone();
                    let id_for_callback = entry_id.clone();
                    let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                        logging::log(
                            "EXEC",
                            &format!(
                                "Confirmation modal result for {}: {}",
                                id_for_callback,
                                if confirmed { "confirmed" } else { "cancelled" }
                            ),
                        );
                        // Send the result to be processed in render()
                        let _ = sender.try_send((id_for_callback.clone(), confirmed));
                    });

                    // Open the confirm modal
                    let message = format!("Are you sure you want to {}?", entry_name);
                    if let Err(e) = open_confirm_window(
                        cx,
                        main_bounds,
                        None, // display_id - let system choose based on position
                        message,
                        Some("Yes".to_string()),
                        Some("Cancel".to_string()),
                        on_choice,
                    ) {
                        logging::log(
                            "ERROR",
                            &format!("Failed to open confirmation modal: {}", e),
                        );
                        logging::log(
                            "EXEC",
                            &format!(
                                "Skipping dangerous action '{}' because confirmation modal failed to open",
                                entry_id
                            ),
                        );
                    }
                })
                .ok();

                // Notify main window to re-render (in case UI state changed)
                this.update(cx, |_this, cx| {
                    cx.notify();
                })
                .ok();
            })
            .detach();

            return; // Wait for modal callback
        }

        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                logging::log("EXEC", "Opening Clipboard History");
                // P0 FIX: Store data in self, view holds only state
                self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                self.focused_clipboard_entry_id = self
                    .cached_clipboard_entries
                    .first()
                    .map(|entry| entry.id.clone());
                logging::log(
                    "EXEC",
                    &format!(
                        "Loaded {} clipboard entries (cached)",
                        self.cached_clipboard_entries.len()
                    ),
                );
                // Clear the shared input for fresh search (sync on next render)
                self.filter_text = String::new();
                self.pending_filter_sync = true;
                self.pending_placeholder = Some("Search clipboard history...".to_string());
                // Initial selected_index should be 0 (first entry)
                // Note: clipboard history uses a flat list without section headers
                self.current_view = AppView::ClipboardHistoryView {
                    filter: String::new(),
                    selected_index: 0,
                };
                self.hovered_index = None;
                // Mark as opened from main menu - ESC will return to main menu
                self.opened_from_main_menu = true;
                // Use standard height for clipboard history view
                resize_to_view_sync(ViewType::ScriptList, 0);
                // Focus the main filter input so cursor blinks and typing works
                self.pending_focus = Some(FocusTarget::MainFilter);
                self.focused_input = FocusedInput::MainFilter;
                cx.notify();
            }
            builtins::BuiltInFeature::Favorites => {
                logging::log("EXEC", "Opening Favorites");

                match crate::favorites::load_favorites() {
                    Ok(favorites) => {
                        if favorites.script_ids.is_empty() {
                            self.toast_manager.push(
                                components::toast::Toast::info(
                                    "No favorites yet. Use Add to Favorites from an item action menu.",
                                    &self.theme,
                                )
                                .duration_ms(Some(3500)),
                            );
                        } else {
                            self.toast_manager.push(
                                components::toast::Toast::info(
                                    favorites_loaded_message(favorites.script_ids.len()),
                                    &self.theme,
                                )
                                .duration_ms(Some(2500)),
                            );
                        }
                    }
                    Err(error) => {
                        logging::log("ERROR", &format!("Failed to load favorites: {}", error));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to load favorites: {}", error),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                }
                cx.notify();
            }
            builtins::BuiltInFeature::AppLauncher => {
                logging::log("EXEC", "Opening App Launcher");
                // P0 FIX: Use self.apps which is already cached
                // Refresh apps list when opening launcher
                self.apps = app_launcher::scan_applications().clone();
                logging::log("EXEC", &format!("Loaded {} applications", self.apps.len()));
                // Invalidate caches since apps changed
                self.invalidate_filter_cache();
                self.invalidate_grouped_cache();
                // Sync list state so when user returns to ScriptList, the count is correct
                self.sync_list_state();
                // Clear the shared input for fresh search (sync on next render)
                self.filter_text = String::new();
                self.pending_filter_sync = true;
                self.pending_placeholder = Some("Search applications...".to_string());
                self.current_view = AppView::AppLauncherView {
                    filter: String::new(),
                    selected_index: 0,
                };
                self.hovered_index = None;
                // Mark as opened from main menu - ESC will return to main menu
                self.opened_from_main_menu = true;
                // Use standard height for app launcher view
                resize_to_view_sync(ViewType::ScriptList, 0);
                // Focus the main filter input so cursor blinks and typing works
                self.pending_focus = Some(FocusTarget::MainFilter);
                self.focused_input = FocusedInput::MainFilter;
                cx.notify();
            }
            builtins::BuiltInFeature::App(app_name) => {
                logging::log("EXEC", &format!("Launching app: {}", app_name));
                // Find and launch the specific application
                let apps = app_launcher::scan_applications();
                if let Some(app) = apps.iter().find(|a| a.name == *app_name) {
                    if let Err(e) = app_launcher::launch_application(app) {
                        logging::log("ERROR", &format!("Failed to launch {}: {}", app_name, e));
                        self.last_output = Some(SharedString::from(format!(
                            "Failed to launch: {}",
                            app_name
                        )));
                    } else {
                        logging::log("EXEC", &format!("Launched app: {}", app_name));
                        self.close_and_reset_window(cx);
                    }
                } else {
                    logging::log("ERROR", &format!("App not found: {}", app_name));
                    self.last_output =
                        Some(SharedString::from(format!("App not found: {}", app_name)));
                }
                cx.notify();
            }
            builtins::BuiltInFeature::WindowSwitcher => {
                logging::log("EXEC", "Opening Window Switcher");
                // P0 FIX: Store data in self, view holds only state
                // Load windows when view is opened (windows change frequently)
                match window_control::list_windows() {
                    Ok(windows) => {
                        logging::log("EXEC", &format!("Loaded {} windows", windows.len()));
                        self.cached_windows = windows;
                        // Clear the shared input for fresh search (sync on next render)
                        self.filter_text = String::new();
                        self.pending_filter_sync = true;
                        self.pending_placeholder = Some("Search windows...".to_string());
                        self.current_view = AppView::WindowSwitcherView {
                            filter: String::new(),
                            selected_index: 0,
                        };
                        self.hovered_index = None;
                        // Mark as opened from main menu - ESC will return to main menu
                        self.opened_from_main_menu = true;
                        // Use standard height for window switcher view
                        resize_to_view_sync(ViewType::ScriptList, 0);
                        // Focus the main filter input so cursor blinks and typing works
                        self.pending_focus = Some(FocusTarget::MainFilter);
                        self.focused_input = FocusedInput::MainFilter;
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to list windows: {}", e));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to list windows: {}", e),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                }
                cx.notify();
            }
            builtins::BuiltInFeature::DesignGallery => {
                logging::log("EXEC", "Opening Design Gallery");
                // Clear the shared input for fresh search (sync on next render)
                self.filter_text = String::new();
                self.pending_filter_sync = true;
                self.pending_placeholder = Some("Search designs...".to_string());
                self.current_view = AppView::DesignGalleryView {
                    filter: String::new(),
                    selected_index: 0,
                };
                self.hovered_index = None;
                // Mark as opened from main menu - ESC will return to main menu
                self.opened_from_main_menu = true;
                // Use standard height for design gallery view
                resize_to_view_sync(ViewType::ScriptList, 0);
                // Focus the main filter input so cursor blinks and typing works
                self.pending_focus = Some(FocusTarget::MainFilter);
                self.focused_input = FocusedInput::MainFilter;
                cx.notify();
            }
            builtins::BuiltInFeature::AiChat => {
                logging::log("EXEC", "Opening AI Chat window");
                // Reset state and hide main window first
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

                // Defer AI window creation to avoid RefCell borrow conflicts
                // The reset_to_script_list calls cx.notify() which schedules a render,
                // opening a new window immediately can cause GPUI RefCell conflicts
                cx.spawn(async move |this, cx| {
                    // Small yield to let any pending GPUI operations complete
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(1))
                        .await;

                    cx.update(|cx| {
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            let _ = this.update(cx, |this, cx| {
                                this.toast_manager.push(
                                    components::toast::Toast::error(
                                        ai_open_failure_message(&e),
                                        &this.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            });
                        }
                    })
                    .ok();
                })
                .detach();
            }
            builtins::BuiltInFeature::Notes => {
                logging::log("EXEC", "Opening Notes window");
                // Reset state, hide main window, and open Notes window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();
                if let Err(e) = notes::open_notes_window(cx) {
                    logging::log("ERROR", &format!("Failed to open Notes window: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Failed to open Notes: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            builtins::BuiltInFeature::EmojiPicker => {
                logging::log(
                    "EXEC",
                    "correlation_id=builtin-emoji-picker-start action=show-emoji-list",
                );

                #[cfg(target_os = "macos")]
                {
                    let emoji_labels: Vec<String> = script_kit_gpui::emoji::EMOJIS
                        .iter()
                        .map(emoji_picker_label)
                        .collect();

                    match choose_from_list("Select an emoji to copy", "Copy", &emoji_labels) {
                        Ok(Some(selected_label)) => {
                            if let Some(index) =
                                emoji_labels.iter().position(|label| label == &selected_label)
                            {
                                let selected_emoji = script_kit_gpui::emoji::EMOJIS[index].emoji;
                                let clipboard_item =
                                    gpui::ClipboardItem::new_string(selected_emoji.to_string());
                                cx.write_to_clipboard(clipboard_item);
                                logging::log(
                                    "EXEC",
                                    &format!(
                                        "correlation_id=builtin-emoji-picker-success emoji=\"{}\"",
                                        selected_emoji
                                    ),
                                );
                                self.show_hud(
                                    format!("Copied {} to clipboard", selected_emoji),
                                    Some(1600),
                                    cx,
                                );
                                self.close_and_reset_window(cx);
                            } else {
                                logging::log(
                                    "ERROR",
                                    &format!(
                                        "correlation_id=builtin-emoji-picker-missing-selection selected_label=\"{}\"",
                                        selected_label
                                    ),
                                );
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        "Selected emoji could not be resolved.",
                                        &self.theme,
                                    )
                                    .duration_ms(Some(3500)),
                                );
                                cx.notify();
                            }
                        }
                        Ok(None) => {
                            logging::log(
                                "EXEC",
                                "correlation_id=builtin-emoji-picker-cancelled",
                            );
                        }
                        Err(error) => {
                            logging::log(
                                "ERROR",
                                &format!(
                                    "correlation_id=builtin-emoji-picker-error attempted=list-emojis error={}",
                                    error
                                ),
                            );
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to open Emoji Picker: {}", error),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    logging::log(
                        "WARN",
                        "correlation_id=builtin-emoji-picker-unsupported platform=non-macos",
                    );
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "Emoji Picker currently requires macOS.",
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                    cx.notify();
                }
            }
            builtins::BuiltInFeature::Quicklinks => {
                logging::log(
                    "EXEC",
                    "correlation_id=builtin-quicklinks-start action=show-quicklinks-list",
                );

                #[cfg(target_os = "macos")]
                {
                    let quicklinks = script_kit_gpui::quicklinks::load_quicklinks();
                    if quicklinks.is_empty() {
                        self.toast_manager.push(
                            components::toast::Toast::info(
                                "No quicklinks found. Add quicklinks to ~/.scriptkit/quicklinks.json",
                                &self.theme,
                            )
                            .duration_ms(Some(3500)),
                        );
                        cx.notify();
                        return;
                    }

                    let quicklink_labels: Vec<String> =
                        quicklinks.iter().map(quicklink_picker_label).collect();
                    let default_query = self.filter_text.trim().to_string();

                    match choose_from_list("Select a quicklink to open", "Open", &quicklink_labels) {
                        Ok(Some(selected_label)) => {
                            if let Some(index) = quicklink_labels
                                .iter()
                                .position(|label| label == &selected_label)
                            {
                                let selected_quicklink = &quicklinks[index];
                                let query = if script_kit_gpui::quicklinks::has_query_placeholder(
                                    &selected_quicklink.url_template,
                                ) {
                                    match prompt_for_text(
                                        "Enter quicklink query",
                                        &default_query,
                                        "Open",
                                    ) {
                                        Ok(Some(value)) => value,
                                        Ok(None) => {
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "correlation_id=builtin-quicklinks-cancelled id={}",
                                                    selected_quicklink.id
                                                ),
                                            );
                                            return;
                                        }
                                        Err(error) => {
                                            logging::log(
                                                "ERROR",
                                                &format!(
                                                    "correlation_id=builtin-quicklinks-query-error id={} attempted=prompt-query error={}",
                                                    selected_quicklink.id, error
                                                ),
                                            );
                                            self.toast_manager.push(
                                                components::toast::Toast::error(
                                                    format!(
                                                        "Failed to get quicklink query: {}",
                                                        error
                                                    ),
                                                    &self.theme,
                                                )
                                                .duration_ms(Some(5000)),
                                            );
                                            cx.notify();
                                            return;
                                        }
                                    }
                                } else {
                                    String::new()
                                };

                                let expanded_url = script_kit_gpui::quicklinks::expand_url(
                                    &selected_quicklink.url_template,
                                    query.trim(),
                                );
                                match open::that(&expanded_url) {
                                    Ok(_) => {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "correlation_id=builtin-quicklinks-opened id={} url={}",
                                                selected_quicklink.id, expanded_url
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Opened {}", selected_quicklink.name),
                                            Some(1700),
                                            cx,
                                        );
                                        self.close_and_reset_window(cx);
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "ERROR",
                                            &format!(
                                                "correlation_id=builtin-quicklinks-open-failed id={} url={} error={}",
                                                selected_quicklink.id, expanded_url, error
                                            ),
                                        );
                                        self.toast_manager.push(
                                            components::toast::Toast::error(
                                                format!("Failed to open quicklink: {}", error),
                                                &self.theme,
                                            )
                                            .duration_ms(Some(5000)),
                                        );
                                        cx.notify();
                                    }
                                }
                            } else {
                                logging::log(
                                    "ERROR",
                                    &format!(
                                        "correlation_id=builtin-quicklinks-missing-selection selected_label=\"{}\"",
                                        selected_label
                                    ),
                                );
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        "Selected quicklink could not be resolved.",
                                        &self.theme,
                                    )
                                    .duration_ms(Some(3500)),
                                );
                                cx.notify();
                            }
                        }
                        Ok(None) => {
                            logging::log("EXEC", "correlation_id=builtin-quicklinks-cancelled");
                        }
                        Err(error) => {
                            logging::log(
                                "ERROR",
                                &format!(
                                    "correlation_id=builtin-quicklinks-list-error attempted=list-quicklinks error={}",
                                    error
                                ),
                            );
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to open Quicklinks: {}", error),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    logging::log(
                        "WARN",
                        "correlation_id=builtin-quicklinks-unsupported platform=non-macos",
                    );
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "Quicklinks currently requires macOS.",
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                    cx.notify();
                }
            }
            builtins::BuiltInFeature::MenuBarAction(action) => {
                logging::log(
                    "EXEC",
                    &format!(
                        "Executing menu bar action: {} -> {}",
                        action.bundle_id,
                        action.menu_path.join(" → ")
                    ),
                );
                // Execute menu action via accessibility API
                #[cfg(target_os = "macos")]
                {
                    match script_kit_gpui::menu_executor::execute_menu_action(
                        &action.bundle_id,
                        &action.menu_path,
                    ) {
                        Ok(()) => {
                            logging::log("EXEC", "Menu action executed successfully");
                            self.close_and_reset_window(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Menu action failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Menu action failed: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    logging::log("WARN", "Menu bar actions only supported on macOS");
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "Menu bar actions are only supported on macOS",
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                    cx.notify();
                }
            }

            // =========================================================================
            // System Actions
            // =========================================================================
            builtins::BuiltInFeature::SystemAction(action_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing system action: {:?}", action_type),
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
                                components::toast::Toast::success(
                                    "Confirmation test passed!",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                            cx.notify();
                            return; // Don't hide window for test
                        }

                        // App control
                        SystemActionType::QuitScriptKit => {
                            logging::log("EXEC", "Quitting Script Kit");
                            cx.quit();
                            return;
                        }

                        // System utilities
                        SystemActionType::ToggleDoNotDisturb => {
                            system_actions::toggle_do_not_disturb()
                        }
                        SystemActionType::StartScreenSaver => system_actions::start_screen_saver(),

                        // System Preferences
                        SystemActionType::OpenSystemPreferences => {
                            system_actions::open_system_preferences_main()
                        }
                        SystemActionType::OpenPrivacySettings => {
                            system_actions::open_privacy_settings()
                        }
                        SystemActionType::OpenDisplaySettings => {
                            system_actions::open_display_settings()
                        }
                        SystemActionType::OpenSoundSettings => {
                            system_actions::open_sound_settings()
                        }
                        SystemActionType::OpenNetworkSettings => {
                            system_actions::open_network_settings()
                        }
                        SystemActionType::OpenKeyboardSettings => {
                            system_actions::open_keyboard_settings()
                        }
                        SystemActionType::OpenBluetoothSettings => {
                            system_actions::open_bluetooth_settings()
                        }
                        SystemActionType::OpenNotificationsSettings => {
                            system_actions::open_notifications_settings()
                        }
                    };

                    match result {
                        Ok(()) => {
                            logging::log("EXEC", "System action executed successfully");
                            if let Some(message) = self.system_action_feedback_message(action_type)
                            {
                                cx.notify();
                                self.show_hud(message, Some(2000), cx);
                                self.hide_main_and_reset(cx);
                            } else {
                                self.close_and_reset_window(cx);
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("System action failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("System action failed: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    logging::log("WARN", "System actions only supported on macOS");
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "System actions are only supported on macOS",
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                    cx.notify();
                }
            }

            // NOTE: Window Actions removed - now handled by window-management extension
            // SDK tileWindow() still works via protocol messages in execute_script.rs

            // =========================================================================
            // Notes Commands
            // =========================================================================

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

                let is_generate_script = matches!(cmd_type, AiCommandType::GenerateScript);
                if !is_generate_script {
                    // Most AI commands open a separate AI window.
                    script_kit_gpui::set_main_window_visible(false);
                    self.reset_to_script_list(cx);
                    platform::hide_main_window();
                }

                match cmd_type {
                    AiCommandType::OpenAi | AiCommandType::NewConversation => {
                        // Basic open/new conversation
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("AI command failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    ai_open_failure_message(&e),
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

                    AiCommandType::GenerateScript => {
                        let query = query_override.unwrap_or(&self.filter_text).to_string();
                        self.dispatch_ai_script_generation_from_query(query, cx);
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
                                    logging::log("ERROR", &ai_open_failure_message(&e));
                                    self.toast_manager.push(
                                        components::toast::Toast::error(
                                            ai_open_failure_message(&e),
                                            &self.theme,
                                        )
                                        .duration_ms(Some(5000)),
                                    );
                                    cx.notify();
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
                                    logging::log("ERROR", &ai_open_failure_message(&e));
                                    self.toast_manager.push(
                                        components::toast::Toast::error(
                                            ai_open_failure_message(&e),
                                            &self.theme,
                                        )
                                        .duration_ms(Some(5000)),
                                    );
                                    cx.notify();
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
                                    logging::log("ERROR", &ai_open_failure_message(&e));
                                    self.toast_manager.push(
                                        components::toast::Toast::error(
                                            ai_open_failure_message(&e),
                                            &self.theme,
                                        )
                                        .duration_ms(Some(5000)),
                                    );
                                    cx.notify();
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
                                    logging::log("ERROR", &ai_open_failure_message(&e));
                                    self.toast_manager.push(
                                        components::toast::Toast::error(
                                            ai_open_failure_message(&e),
                                            &self.theme,
                                        )
                                        .duration_ms(Some(5000)),
                                    );
                                    cx.notify();
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
                            logging::log("ERROR", &ai_open_failure_message(&e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    ai_open_failure_message(&e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
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

            builtins::BuiltInFeature::PermissionCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing permission command: {:?}", cmd_type),
                );

                use builtins::PermissionCommandType;

                match cmd_type {
                    PermissionCommandType::CheckPermissions => {
                        let status = permissions_wizard::check_all_permissions();
                        if status.all_granted() {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "All permissions granted!",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
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
                                .duration_ms(Some(5000)),
                            );
                        }
                        cx.notify();
                    }
                    PermissionCommandType::RequestAccessibility => {
                        let granted = permissions_wizard::request_accessibility_permission();
                        if granted {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "Accessibility permission granted!",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        } else {
                            self.toast_manager.push(
                                components::toast::Toast::warning(
                                    "Accessibility permission not granted. Some features may not work.",
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        }
                        cx.notify();
                    }
                    PermissionCommandType::OpenAccessibilitySettings => {
                        if let Err(e) = permissions_wizard::open_accessibility_settings() {
                            logging::log(
                                "ERROR",
                                &format!("Failed to open accessibility settings: {}", e),
                            );
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to open settings: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        } else {
                            self.close_and_reset_window(cx);
                        }
                    }
                }
            }

            // =========================================================================
            // Frecency/Suggested Commands
            // =========================================================================
            builtins::BuiltInFeature::FrecencyCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing frecency command: {:?}", cmd_type),
                );

                use builtins::FrecencyCommandType;

                match cmd_type {
                    FrecencyCommandType::ClearSuggested => {
                        // Clear all frecency data
                        self.frecency_store.clear();
                        if let Err(e) = self.frecency_store.save() {
                            logging::log("ERROR", &format!("Failed to save frecency data: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to clear suggested: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        } else {
                            logging::log("EXEC", "Cleared all suggested items");
                            // Invalidate the grouped cache so the UI updates
                            self.invalidate_grouped_cache();
                            // Reset the main input and window to clean state
                            self.reset_to_script_list(cx);
                            resize_to_view_sync(ViewType::ScriptList, 0);
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "Suggested items cleared",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        }
                        // Note: cx.notify() is called by reset_to_script_list, but we still need it for error case
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Settings Commands (Reset Window Positions, etc.)
            // =========================================================================
            builtins::BuiltInFeature::SettingsCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing settings command: {:?}", cmd_type),
                );

                use builtins::SettingsCommandType;

                match cmd_type {
                    SettingsCommandType::ResetWindowPositions => {
                        // Suppress position saving to prevent the bounds change callback
                        // from immediately re-saving after we delete the state file
                        crate::window_state::suppress_save();

                        // Reset all window positions to defaults
                        crate::window_state::reset_all_positions();
                        logging::log("EXEC", "Reset all window positions to defaults");

                        // Show toast confirmation
                        self.toast_manager.push(
                            components::toast::Toast::success(
                                "Window positions reset - takes effect next open",
                                &self.theme,
                            )
                            .duration_ms(Some(3000)),
                        );

                        // Close and reset window - this hides the window which is required
                        // for the reset to take effect (as the toast message states)
                        self.close_and_reset_window(cx);
                    }
                    SettingsCommandType::ConfigureVercelApiKey => {
                        self.show_api_key_prompt(
                            "SCRIPT_KIT_VERCEL_API_KEY",
                            "Enter your Vercel AI Gateway API key",
                            "Vercel AI Gateway",
                            cx,
                        );
                    }
                    SettingsCommandType::ConfigureOpenAiApiKey => {
                        self.show_api_key_prompt(
                            "SCRIPT_KIT_OPENAI_API_KEY",
                            "Enter your OpenAI API key",
                            "OpenAI",
                            cx,
                        );
                    }
                    SettingsCommandType::ConfigureAnthropicApiKey => {
                        self.show_api_key_prompt(
                            "SCRIPT_KIT_ANTHROPIC_API_KEY",
                            "Enter your Anthropic API key",
                            "Anthropic",
                            cx,
                        );
                    }
                    SettingsCommandType::ChooseTheme => {
                        logging::log("EXEC", "Opening Theme Chooser");
                        // Back up current theme for cancel/restore
                        self.theme_before_chooser = Some(self.theme.clone());
                        // Clear the shared input for fresh search (sync on next render)
                        self.filter_text = String::new();
                        self.pending_filter_sync = true;
                        self.pending_placeholder = Some("Search themes...".to_string());
                        // Start at the currently active theme
                        let start_index =
                            theme::presets::find_current_preset_index(&self.theme);
                        self.current_view = AppView::ThemeChooserView {
                            filter: String::new(),
                            selected_index: start_index,
                        };
                        self.hovered_index = None;
                        self.opened_from_main_menu = true;
                        resize_to_view_sync(ViewType::ScriptList, 0);
                        // Focus the main filter input so cursor blinks and typing works
                        self.pending_focus = Some(FocusTarget::MainFilter);
                        self.focused_input = FocusedInput::MainFilter;
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Utility Commands (Scratch Pad, Quick Terminal, Process Manager)
            // =========================================================================
            builtins::BuiltInFeature::UtilityCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing utility command: {:?}", cmd_type),
                );

                use builtins::UtilityCommandType;

                match cmd_type {
                    UtilityCommandType::ScratchPad => {
                        // Mark as opened from main menu - ESC will return to main menu
                        self.opened_from_main_menu = true;
                        self.open_scratch_pad(cx);
                    }
                    UtilityCommandType::QuickTerminal => {
                        // Mark as opened from main menu - ESC will return to main menu
                        self.opened_from_main_menu = true;
                        self.open_quick_terminal(cx);
                    }
                    UtilityCommandType::ProcessManager => {
                        let process_count = crate::process_manager::PROCESS_MANAGER.active_count();
                        let report =
                            crate::process_manager::PROCESS_MANAGER.format_active_process_report(8);

                        logging::log(
                            "EXEC",
                            &format!(
                                "correlation_id=process-manager-inspect active_process_count={}",
                                process_count
                            ),
                        );

                        // Always copy details so users can inspect full paths quickly.
                        let clipboard_item = gpui::ClipboardItem::new_string(report.clone());
                        cx.write_to_clipboard(clipboard_item);

                        if process_count == 0 {
                            self.show_hud(
                                "No running scripts. Process report copied.".to_string(),
                                Some(2200),
                                cx,
                            );
                        } else {
                            self.show_hud(
                                format!(
                                    "{} running script process(es). Details copied.",
                                    process_count
                                ),
                                Some(2600),
                                cx,
                            );
                        }
                    }
                    UtilityCommandType::StopAllProcesses => {
                        let process_count = crate::process_manager::PROCESS_MANAGER.active_count();
                        logging::log(
                            "EXEC",
                            &format!(
                                "correlation_id=process-manager-stop-all requested_count={}",
                                process_count
                            ),
                        );

                        if process_count == 0 {
                            self.show_hud("No running scripts to stop.".to_string(), Some(2200), cx);
                        } else {
                            crate::process_manager::PROCESS_MANAGER.kill_all_processes();
                            self.show_hud(
                                format!("Stopped {} running script process(es).", process_count),
                                Some(2600),
                                cx,
                            );
                            self.close_and_reset_window(cx);
                        }
                    }
                }
            }

            // =========================================================================
            // Kit Store Commands
            // =========================================================================
            builtins::BuiltInFeature::KitStoreCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing kit store command: {:?}", cmd_type),
                );

                use builtins::KitStoreCommandType;

                let message = match cmd_type {
                    KitStoreCommandType::BrowseKits => "Kit Store browsing is coming soon.",
                    KitStoreCommandType::InstalledKits => {
                        "Installed kit management is coming soon."
                    }
                    KitStoreCommandType::UpdateAllKits => "Kit update flow is coming soon.",
                };

                self.toast_manager.push(
                    components::toast::Toast::info(message, &self.theme).duration_ms(Some(3000)),
                );
                cx.notify();
            }

            // =========================================================================
            // File Search (Directory Navigation)
            // =========================================================================
            builtins::BuiltInFeature::Webcam => {
                logging::log("EXEC", "Opening Webcam");
                self.opened_from_main_menu = true;
                self.open_webcam(cx);
            }
            builtins::BuiltInFeature::FileSearch => {
                logging::log("EXEC", "Opening File Search");
                // Mark as opened from main menu - ESC will return to main menu
                self.opened_from_main_menu = true;
                self.open_file_search(String::new(), cx);
            }

        }
    }
}

#[cfg(test)]
mod builtin_execution_ai_feedback_tests {
    use super::{
        ai_open_failure_message, emoji_picker_label, favorites_loaded_message, quicklink_picker_label,
    };
    use script_kit_gpui::emoji::{Emoji, EmojiCategory};
    use script_kit_gpui::quicklinks::Quicklink;

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
    fn test_quicklink_picker_label_includes_name_and_url_template() {
        let quicklink = Quicklink {
            id: "ql-1".to_string(),
            name: "Docs".to_string(),
            url_template: "https://docs.rs".to_string(),
            icon: None,
        };

        assert_eq!(
            quicklink_picker_label(&quicklink),
            "Docs  https://docs.rs"
        );
    }
}
