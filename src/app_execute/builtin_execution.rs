/// Small async yield (in ms) before opening the AI window to let pending GPUI operations complete.
const AI_WINDOW_ASYNC_YIELD_MS: u64 = 1;

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
fn applescript_list_literal(values: &[String]) -> String {
    let escaped_values = values
        .iter()
        .map(|value| format!("\"{}\"", crate::utils::escape_applescript_string(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{}}}", escaped_values)
}

#[cfg(target_os = "macos")]
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

fn quicklink_picker_label(quicklink: &script_kit_gpui::quicklinks::Quicklink) -> String {
    format!(
        "{}  {}",
        crate::utils::escape_applescript_string(&quicklink.name),
        crate::utils::escape_applescript_string(&quicklink.url_template)
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

    /// Shared dispatch for system actions — used by both the normal and confirmed paths.
    /// Maps a `SystemActionType` to its implementation, handles special cases
    /// (TestConfirmation, QuitScriptKit), and routes the result through
    /// `handle_system_action_result`.
    fn dispatch_system_action(
        &mut self,
        action_type: &builtins::SystemActionType,
        cx: &mut Context<Self>,
    ) {
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
                        .duration_ms(Some(TOAST_SUCCESS_MS)),
                    );
                    cx.notify();
                    return; // Don't hide window for test
                }

                // App control
                SystemActionType::QuitScriptKit => {
                    tracing::info!(message = %"Quitting Script Kit");
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
                SystemActionType::OpenPrivacySettings => system_actions::open_privacy_settings(),
                SystemActionType::OpenDisplaySettings => {
                    system_actions::open_display_settings()
                }
                SystemActionType::OpenSoundSettings => system_actions::open_sound_settings(),
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

            self.handle_system_action_result(result, action_type, cx);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = action_type;
            tracing::warn!(message = %"System actions only supported on macOS");
            self.show_unsupported_platform_toast("System actions", cx);
        }
    }

    /// Shared result handler for system actions — shows HUD on success, Toast on error.
    fn handle_system_action_result(
        &mut self,
        result: Result<(), String>,
        action_type: &builtins::SystemActionType,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(()) => {
                tracing::info!(message = %"System action executed successfully");
                if let Some(message) = self.system_action_feedback_message(action_type) {
                    cx.notify();
                    self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                    self.hide_main_and_reset(cx);
                } else {
                    self.close_and_reset_window(cx);
                }
            }
            Err(e) => {
                tracing::error!(message = %&format!("System action failed: {}", e));
                self.show_error_toast(format!("System action failed: {}", e), cx);
            }
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
        let trace_id = uuid::Uuid::new_v4().to_string();
        let start = std::time::Instant::now();

        tracing::info!(
            category = "BUILTIN",
            trace_id = %trace_id,
            builtin_id = %entry.id,
            builtin_name = %entry.name,
            "Builtin execution started"
        );

        // Clear any stale actions popup from previous view
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Check if this command requires confirmation - open modal if so
        if self.config.requires_confirmation(&entry.id) {
            tracing::info!(message = %&format!("Opening confirmation modal for: {}", entry.id),
            );

            // Clone what we need for the spawned task
            let entry_id = entry.id.clone();
            let entry_name = entry.name.clone();
            let query_owned = query_override.map(|s| s.to_string());

            // Spawn a task to show confirmation modal via confirm_with_modal helper
            cx.spawn(async move |this, cx| {
                let message = format!("Are you sure you want to {}?", entry_name);
                match confirm_with_modal(cx, message, "Yes", "Cancel").await {
                    Ok(true) => {
                        let _ = this.update(cx, |this, cx| {
                            this.handle_builtin_confirmation(
                                entry_id,
                                true,
                                query_owned,
                                cx,
                            );
                        });
                    }
                    Ok(false) => {
                        tracing::info!(
                            builtin_id = %entry_id,
                            "Builtin confirmation cancelled by user"
                        );
                    }
                    Err(e) => {
                        let _ = this.update(cx, |this, cx| {
                            tracing::error!(
                                builtin_id = %entry_id,
                                error = %e,
                                "failed to open confirmation modal"
                            );
                            this.show_error_toast_with_code(
                                "Failed to open confirmation dialog",
                                Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                cx,
                            );
                        });
                    }
                }
            })
            .detach();

            tracing::info!(
                category = "BUILTIN",
                trace_id = %trace_id,
                builtin_id = %entry.id,
                status = "awaiting_confirmation",
                duration_ms = start.elapsed().as_millis() as u64,
                "Builtin execution deferred to confirmation modal"
            );
            return; // Wait for modal callback
        }

        self.execute_builtin_inner(entry, query_override, &trace_id, start, cx);
    }

    /// Inner builtin executor — runs the actual action logic.
    /// Called from both the normal path (after confirmation check) and the
    /// confirmed path (after modal approval), ensuring a single implementation.
    fn execute_builtin_inner(
        &mut self,
        entry: &builtins::BuiltInEntry,
        query_override: Option<&str>,
        trace_id: &str,
        start: std::time::Instant,
        cx: &mut Context<Self>,
    ) {
        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                tracing::info!(message = %"Opening Clipboard History");
                // P0 FIX: Store data in self, view holds only state
                self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                self.focused_clipboard_entry_id = self
                    .cached_clipboard_entries
                    .first()
                    .map(|entry| entry.id.clone());
                tracing::info!(message = %&format!(
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
            builtins::BuiltInFeature::PasteSequentially => {
                tracing::info!(message = %"Opening Paste Sequentially");
                let prompt = prompts::PasteSequentialPrompt::new(
                    "builtin-paste-sequentially".to_string(),
                    self.focus_handle.clone(),
                    Arc::clone(&self.theme),
                );
                let entity = cx.new(|_| prompt);

                self.current_view = AppView::PasteSequentiallyView { entity };
                self.hovered_index = None;
                self.opened_from_main_menu = true;
                self.pending_focus = Some(FocusTarget::AppRoot);
                self.focused_input = FocusedInput::None;
                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
            }
            builtins::BuiltInFeature::Favorites => {
                tracing::info!(message = %"Opening Favorites");

                match crate::favorites::load_favorites() {
                    Ok(favorites) => {
                        if favorites.script_ids.is_empty() {
                            // Clear stale favorites filter to prevent state leak
                            self.active_favorites = None;
                            self.invalidate_filter_cache();
                            self.invalidate_grouped_cache();
                            self.sync_list_state();

                            self.toast_manager.push(
                                components::toast::Toast::info(
                                    "No favorites yet. Use Add to Favorites from an item action menu.",
                                    &self.theme,
                                )
                                .duration_ms(Some(TOAST_INFO_MS)),
                            );
                        } else {
                            // Store loaded favorites so the script list can filter by them
                            self.active_favorites = Some(favorites.script_ids.clone());
                            self.invalidate_filter_cache();
                            self.invalidate_grouped_cache();
                            self.sync_list_state();

                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    favorites_loaded_message(favorites.script_ids.len()),
                                    &self.theme,
                                )
                                .duration_ms(Some(TOAST_SUCCESS_MS)),
                            );
                        }
                    }
                    Err(error) => {
                        tracing::error!(message = %&format!("Failed to load favorites: {}", error));
                        self.show_error_toast(
                            format!("Failed to load favorites: {}", error),
                            cx,
                        );
                    }
                }
                cx.notify();
            }
            builtins::BuiltInFeature::AppLauncher => {
                tracing::info!(message = %"Opening App Launcher");
                // P0 FIX: Use self.apps which is already cached
                // Refresh apps list when opening launcher
                self.apps = app_launcher::scan_applications().clone();
                tracing::info!(message = %&format!("Loaded {} applications", self.apps.len()));
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
                tracing::info!(message = %&format!("Launching app: {}", app_name));
                // Find and launch the specific application
                let apps = app_launcher::scan_applications();
                if let Some(app) = apps.iter().find(|a| a.name == *app_name) {
                    if let Err(e) = app_launcher::launch_application(app) {
                        tracing::error!(message = %&format!("Failed to launch {}: {}", app_name, e));
                        self.show_error_toast(format!("Failed to launch {}: {}", app_name, e), cx);
                    } else {
                        tracing::info!(message = %&format!("Launched app: {}", app_name));
                        self.close_and_reset_window(cx);
                    }
                } else {
                    tracing::error!(message = %&format!("App not found: {}", app_name));
                    self.show_error_toast(format!("App not found: {}", app_name), cx);
                }
            }
            builtins::BuiltInFeature::WindowSwitcher => {
                tracing::info!(message = %"Opening Window Switcher");
                // P0 FIX: Store data in self, view holds only state
                // Load windows when view is opened (windows change frequently)
                match window_control::list_windows() {
                    Ok(windows) => {
                        tracing::info!(message = %&format!("Loaded {} windows", windows.len()));
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
                        tracing::error!(message = %&format!("Failed to list windows: {}", e));
                        self.show_error_toast(format!("Failed to list windows: {}", e), cx);
                    }
                }
                cx.notify();
            }
            builtins::BuiltInFeature::DesignGallery => {
                tracing::info!(message = %"Opening Design Gallery");
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
                tracing::info!(message = %"Opening AI Chat window");
                // Reset state and hide main window first
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::defer_hide_main_window(cx);

                // Defer AI window creation to avoid RefCell borrow conflicts
                // The reset_to_script_list calls cx.notify() which schedules a render,
                // opening a new window immediately can cause GPUI RefCell conflicts
                cx.spawn(async move |this, cx| {
                    // Small yield to let any pending GPUI operations complete
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(AI_WINDOW_ASYNC_YIELD_MS))
                        .await;

                    cx.update(|cx| {
                        if let Err(e) = ai::open_ai_window(cx) {
                            tracing::error!(message = %&format!("Failed to open AI window: {}", e));
                            let _ = this.update(cx, |this, cx| {
                                this.show_error_toast(ai_open_failure_message(&e), cx);
                            });
                        }
                    });
                })
                .detach();
            }
            builtins::BuiltInFeature::Notes => {
                tracing::info!(message = %"Opening Notes window");
                // Reset state, hide main window, and open Notes window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::defer_hide_main_window(cx);
                if let Err(e) = notes::open_notes_window(cx) {
                    tracing::error!(message = %&format!("Failed to open Notes window: {}", e));
                    self.show_error_toast(format!("Failed to open Notes: {}", e), cx);
                }
            }
            builtins::BuiltInFeature::EmojiPicker => {
                tracing::info!(message = %"correlation_id=builtin-emoji-picker-start action=show-emoji-grid",
                );
                self.filter_text = String::new();
                self.pending_filter_sync = true;
                self.pending_placeholder = Some("Search Emoji & Symbols...".to_string());
                self.current_view = AppView::EmojiPickerView {
                    filter: String::new(),
                    selected_index: 0,
                    selected_category: None,
                };
                self.hovered_index = None;
                self.opened_from_main_menu = true;
                self.pending_focus = Some(FocusTarget::MainFilter);
                cx.notify();
            }
            builtins::BuiltInFeature::Quicklinks => {
                tracing::info!(message = %"correlation_id=builtin-quicklinks-start action=show-quicklinks-list",
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
                            .duration_ms(Some(TOAST_INFO_MS)),
                        );
                        cx.notify();
                        return;
                    }

                    let quicklink_labels: Vec<String> =
                        quicklinks.iter().map(quicklink_picker_label).collect();
                    let default_query = self.filter_text.trim().to_string();

                    match choose_from_list("Select a quicklink to open", "Open", &quicklink_labels)
                    {
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
                                            tracing::info!(message = %&format!(
                                                    "correlation_id=builtin-quicklinks-cancelled id={}",
                                                    selected_quicklink.id
                                                ),
                                            );
                                            return;
                                        }
                                        Err(error) => {
                                            tracing::error!(message = %&format!(
                                                    "correlation_id=builtin-quicklinks-query-error id={} attempted=prompt-query error={}",
                                                    selected_quicklink.id, error
                                                ),
                                            );
                                            self.show_error_toast(
                                                format!(
                                                    "Failed to get quicklink query: {}",
                                                    error
                                                ),
                                                cx,
                                            );
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
                                        tracing::info!(message = %&format!(
                                                "correlation_id=builtin-quicklinks-opened id={} url={}",
                                                selected_quicklink.id, expanded_url
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Opened {}", selected_quicklink.name),
                                            Some(HUD_SHORT_MS),
                                            cx,
                                        );
                                        self.close_and_reset_window(cx);
                                    }
                                    Err(error) => {
                                        tracing::error!(message = %&format!(
                                                "correlation_id=builtin-quicklinks-open-failed id={} url={} error={}",
                                                selected_quicklink.id, expanded_url, error
                                            ),
                                        );
                                        self.show_error_toast(format!("Failed to open quicklink: {}", error), cx);
                                    }
                                }
                            } else {
                                tracing::error!(message = %&format!(
                                        "correlation_id=builtin-quicklinks-missing-selection selected_label=\"{}\"",
                                        selected_label
                                    ),
                                );
                                self.show_error_toast("Selected quicklink could not be resolved.", cx);
                            }
                        }
                        Ok(None) => {
                            tracing::info!(message = %"correlation_id=builtin-quicklinks-cancelled");
                        }
                        Err(error) => {
                            tracing::error!(message = %&format!(
                                    "correlation_id=builtin-quicklinks-list-error attempted=list-quicklinks error={}",
                                    error
                                ),
                            );
                            self.show_error_toast(format!("Failed to open Quicklinks: {}", error), cx);
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    tracing::warn!(message = %"correlation_id=builtin-quicklinks-unsupported platform=non-macos",
                    );
                    self.show_unsupported_platform_toast("Quicklinks", cx);
                }
            }
            builtins::BuiltInFeature::MenuBarAction(action) => {
                tracing::info!(message = %&format!(
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
                            tracing::info!(message = %"Menu action executed successfully");
                            self.close_and_reset_window(cx);
                        }
                        Err(e) => {
                            tracing::error!(message = %&format!("Menu action failed: {}", e));
                            self.show_error_toast(format!("Menu action failed: {}", e), cx);
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    tracing::warn!(message = %"Menu bar actions only supported on macOS");
                    self.show_unsupported_platform_toast("Menu bar actions", cx);
                }
            }

            // =========================================================================
            // System Actions
            // =========================================================================
            builtins::BuiltInFeature::SystemAction(action_type) => {
                tracing::info!(message = %&format!("Executing system action: {:?}", action_type));
                self.dispatch_system_action(action_type, cx);
            }

            // NOTE: Window Actions removed - now handled by window-management extension
            // SDK tileWindow() still works via protocol messages in execute_script.rs

            // =========================================================================
            // Notes Commands
            // =========================================================================
            builtins::BuiltInFeature::NotesCommand(cmd_type) => {
                tracing::info!(message = %&format!("Executing notes command: {:?}", cmd_type));

                use builtins::NotesCommandType;

                // All notes commands: reset state, hide main window, open notes
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
                    tracing::error!(message = %&format!("Notes command failed: {}", e));
                    self.show_error_toast(format!("Notes command failed: {}", e), cx);
                }
            }

            // =========================================================================
            // AI Commands
            // =========================================================================
            builtins::BuiltInFeature::AiCommand(cmd_type) => {
                tracing::info!(message = %&format!("Executing AI command: {:?}", cmd_type));

                use builtins::AiCommandType;

                let is_generate_script = matches!(cmd_type, AiCommandType::GenerateScript);
                if !is_generate_script {
                    // Most AI commands open a separate AI window.
                    script_kit_gpui::set_main_window_visible(false);
                    self.reset_to_script_list(cx);
                    platform::defer_hide_main_window(cx);
                }

                match cmd_type {
                    AiCommandType::OpenAi | AiCommandType::NewConversation => {
                        // Basic open/new conversation
                        if let Err(e) = ai::open_ai_window(cx) {
                            tracing::error!(message = %&format!("AI command failed: {}", e));
                            self.show_error_toast(ai_open_failure_message(&e), cx);
                        }
                    }

                    AiCommandType::ClearConversation => {
                        match ai::clear_all_chats() {
                            Ok(()) => {
                                // Force a fresh AI window state so cleared history is reflected immediately.
                                ai::close_ai_window(cx);
                                if let Err(e) = ai::open_ai_window(cx) {
                                    tracing::error!(message = %&format!(
                                            "AI history cleared but failed to reopen AI window: {}",
                                            e
                                        ),
                                    );
                                    self.show_error_toast(format!("AI history cleared, but failed to open AI: {}", e), cx);
                                } else {
                                    self.show_hud(
                                        "Cleared AI conversations".to_string(),
                                        Some(HUD_MEDIUM_MS),
                                        cx,
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(message = %&format!("Failed to clear AI conversations: {}", e),
                                );
                                self.show_error_toast(format!("Failed to clear AI conversations: {}", e), cx);
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
                                tracing::info!(message = %&format!(
                                        "Screen captured: {}x{}, {} bytes",
                                        width,
                                        height,
                                        png_data.len()
                                    ),
                                );
                                if let Err(e) = ai::open_ai_window(cx) {
                                    tracing::error!(message = %&ai_open_failure_message(&e));
                                    self.show_error_toast(ai_open_failure_message(&e), cx);
                                } else {
                                    // Set input with the screenshot context
                                    ai::set_ai_input_with_image(cx, &message, &base64_data, false);
                                }
                            }
                            Err(e) => {
                                tracing::error!(message = %&format!("Failed to capture screen: {}", e));
                                self.show_error_toast(format!("Failed to capture screen: {}", e), cx);
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
                                tracing::info!(message = %&format!(
                                        "Window '{}' captured: {}x{}, {} bytes",
                                        window_title,
                                        width,
                                        height,
                                        png_data.len()
                                    ),
                                );
                                if let Err(e) = ai::open_ai_window(cx) {
                                    tracing::error!(message = %&ai_open_failure_message(&e));
                                    self.show_error_toast(ai_open_failure_message(&e), cx);
                                } else {
                                    ai::set_ai_input_with_image(cx, &message, &base64_data, false);
                                }
                            }
                            Err(e) => {
                                tracing::error!(message = %&format!("Failed to capture window: {}", e));
                                self.show_error_toast(format!("Failed to capture window: {}", e), cx);
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
                                tracing::info!(message = %&format!("Selected text captured: {} chars", text.len()),
                                );
                                if let Err(e) = ai::open_ai_window(cx) {
                                    tracing::error!(message = %&ai_open_failure_message(&e));
                                    self.show_error_toast(ai_open_failure_message(&e), cx);
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
                                    .duration_ms(Some(TOAST_INFO_MS)),
                                );
                                cx.notify();
                            }
                            Err(e) => {
                                tracing::error!(message = %&format!("Failed to get selected text: {}", e),
                                );
                                self.show_error_toast(format!("Failed to get selected text: {}", e), cx);
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
                                tracing::info!(message = %&format!("Browser URL captured: {}", url));
                                if let Err(e) = ai::open_ai_window(cx) {
                                    tracing::error!(message = %&ai_open_failure_message(&e));
                                    self.show_error_toast(ai_open_failure_message(&e), cx);
                                } else {
                                    ai::set_ai_input(cx, &message, false);
                                }
                            }
                            Err(e) => {
                                tracing::error!(message = %&format!("Failed to get browser URL: {}", e));
                                self.show_error_toast(format!("Failed to get browser URL: {}", e), cx);
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
                            .duration_ms(Some(TOAST_INFO_MS)),
                        );
                        cx.notify();
                    }

                    AiCommandType::CreateAiPreset
                    | AiCommandType::ImportAiPresets
                    | AiCommandType::SearchAiPresets => {
                        // Preset management - open AI window with a future preset UI
                        match ai::open_ai_window(cx) {
                            Ok(()) => {
                                self.toast_manager.push(
                                    components::toast::Toast::info(
                                        "AI Presets feature coming soon!",
                                        &self.theme,
                                    )
                                    .duration_ms(Some(TOAST_INFO_MS)),
                                );
                            }
                            Err(e) => {
                                tracing::error!(message = %&ai_open_failure_message(&e));
                                self.show_error_toast(ai_open_failure_message(&e), cx);
                            }
                        }
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Script Commands
            // =========================================================================
            builtins::BuiltInFeature::ScriptCommand(cmd_type) => {
                tracing::info!(message = %&format!("Executing script command: {:?}", cmd_type));

                use builtins::ScriptCommandType;

                let target = match cmd_type {
                    ScriptCommandType::NewScript => prompts::NamingTarget::Script,
                    ScriptCommandType::NewExtension => prompts::NamingTarget::Extension,
                };
                self.show_naming_dialog(target, cx);
            }

            // =========================================================================
            // Permission Commands
            // =========================================================================
            builtins::BuiltInFeature::PermissionCommand(cmd_type) => {
                tracing::info!(message = %&format!("Executing permission command: {:?}", cmd_type),
                );

                use builtins::PermissionCommandType;

                match cmd_type {
                    PermissionCommandType::CheckPermissions => {
                        let status = permissions_wizard::check_all_permissions();
                        if status.all_granted() {
                            self.show_hud("All permissions granted!".to_string(), Some(HUD_SHORT_MS), cx);
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
                    }
                    PermissionCommandType::RequestAccessibility => {
                        let granted = permissions_wizard::request_accessibility_permission();
                        if granted {
                            self.show_hud("Accessibility permission granted!".to_string(), Some(HUD_SHORT_MS), cx);
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
                    }
                    PermissionCommandType::OpenAccessibilitySettings => {
                        if let Err(e) = permissions_wizard::open_accessibility_settings() {
                            tracing::error!(message = %&format!("Failed to open accessibility settings: {}", e),
                            );
                            self.show_error_toast(format!("Failed to open settings: {}", e), cx);
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
                tracing::info!(message = %&format!("Executing frecency command: {:?}", cmd_type),
                );

                use builtins::FrecencyCommandType;

                match cmd_type {
                    FrecencyCommandType::ClearSuggested => {
                        // Clear all frecency data
                        self.frecency_store.clear();
                        if let Err(e) = self.frecency_store.save() {
                            tracing::error!(message = %&format!("Failed to save frecency data: {}", e));
                            self.show_error_toast(format!("Failed to clear suggested: {}", e), cx);
                        } else {
                            tracing::info!(message = %"Cleared all suggested items");
                            // Invalidate the grouped cache so the UI updates
                            self.invalidate_grouped_cache();
                            // Reset the main input and window to clean state
                            self.reset_to_script_list(cx);
                            resize_to_view_sync(ViewType::ScriptList, 0);
                            self.show_hud("Suggested items cleared".to_string(), Some(HUD_SHORT_MS), cx);
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
                tracing::info!(message = %&format!("Executing settings command: {:?}", cmd_type),
                );

                use builtins::SettingsCommandType;

                match cmd_type {
                    SettingsCommandType::ResetWindowPositions => {
                        // Suppress position saving to prevent the bounds change callback
                        // from immediately re-saving after we delete the state file
                        crate::window_state::suppress_save();

                        // Reset all window positions to defaults
                        crate::window_state::reset_all_positions();
                        tracing::info!(message = %"Reset all window positions to defaults");

                        // Show toast confirmation
                        self.show_hud("Window positions reset - takes effect next open".to_string(), Some(HUD_SHORT_MS), cx);

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
                        tracing::info!(message = %"Opening Theme Chooser");
                        // Back up current theme for cancel/restore
                        self.theme_before_chooser = Some(self.theme.clone());
                        // Clear the shared input for fresh search (sync on next render)
                        self.filter_text = String::new();
                        self.pending_filter_sync = true;
                        self.pending_placeholder = Some("Search themes...".to_string());
                        // Start at the currently active theme
                        let start_index = theme::presets::find_current_preset_index(&self.theme);
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
                tracing::info!(message = %&format!("Executing utility command: {:?}", cmd_type),
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

                        tracing::info!(message = %&format!(
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
                                Some(HUD_2200_MS),
                                cx,
                            );
                        } else {
                            self.show_hud(
                                format!(
                                    "{} running script process(es). Details copied.",
                                    process_count
                                ),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    }
                    UtilityCommandType::StopAllProcesses => {
                        let process_count = crate::process_manager::PROCESS_MANAGER.active_count();
                        tracing::info!(message = %&format!(
                                "correlation_id=process-manager-stop-all requested_count={}",
                                process_count
                            ),
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
                    }
                }
            }

            // =========================================================================
            // Kit Store Commands
            // =========================================================================
            builtins::BuiltInFeature::KitStoreCommand(cmd_type) => {
                tracing::info!(message = %&format!("Executing kit store command: {:?}", cmd_type),
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
                    components::toast::Toast::info(message, &self.theme)
                        .duration_ms(Some(TOAST_INFO_MS)),
                );
                cx.notify();
            }

            // =========================================================================
            // File Search (Directory Navigation)
            // =========================================================================
            builtins::BuiltInFeature::Webcam => {
                tracing::info!(message = %"Opening Webcam");
                self.opened_from_main_menu = true;
                self.open_webcam(cx);
            }
            builtins::BuiltInFeature::FileSearch => {
                tracing::info!(message = %"Opening File Search");
                // Mark as opened from main menu - ESC will return to main menu
                self.opened_from_main_menu = true;
                self.open_file_search(String::new(), cx);
            }
        }

        tracing::info!(
            category = "BUILTIN",
            trace_id = %trace_id,
            builtin_id = %entry.id,
            status = "completed",
            duration_ms = start.elapsed().as_millis() as u64,
            "Builtin execution completed"
        );
    }
}

#[cfg(test)]
mod builtin_execution_ai_feedback_tests {
    use super::{
        ai_open_failure_message, created_file_path_for_feedback, emoji_picker_label,
        favorites_loaded_message, quicklink_picker_label,
    };
    use script_kit_gpui::emoji::{Emoji, EmojiCategory};
    use script_kit_gpui::quicklinks::Quicklink;
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

        assert_eq!(quicklink_picker_label(&quicklink), "Docs  https://docs.rs");
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
