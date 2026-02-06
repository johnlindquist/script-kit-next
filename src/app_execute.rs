// App execution methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: execute_builtin, execute_app, execute_window_focus

impl ScriptListApp {
    fn execute_builtin(&mut self, entry: &builtins::BuiltInEntry, cx: &mut Context<Self>) {
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
                        // If modal fails, just proceed without confirmation (fallback)
                        let _ = confirm_sender.try_send((entry_id.clone(), true));
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
                                        format!("Failed to open AI: {}", e),
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
                            self.close_and_reset_window(cx);
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

                // All AI commands: reset state, hide main window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

                match cmd_type {
                    AiCommandType::OpenAi | AiCommandType::NewConversation => {
                        // Basic open/new conversation
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("AI command failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to open AI: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }

                    AiCommandType::ClearConversation => {
                        // TODO: Implement clear conversation
                        if let Err(e) = ai::open_ai_window(cx) {
                            logging::log("ERROR", &format!("AI command failed: {}", e));
                        }
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
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
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
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
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
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
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
                                    logging::log("ERROR", &format!("Failed to open AI: {}", e));
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
                            logging::log("ERROR", &format!("Failed to open AI: {}", e));
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
            // Utility Commands (Scratch Pad, Quick Terminal)
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
                }
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

    /// Execute an application directly from the main search results
    fn execute_app(&mut self, app: &app_launcher::AppInfo, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Launching app from search: {}", app.name));

        if let Err(e) = app_launcher::launch_application(app) {
            logging::log("ERROR", &format!("Failed to launch {}: {}", app.name, e));
            self.last_output = Some(SharedString::from(format!(
                "Failed to launch: {}",
                app.name
            )));
            cx.notify();
        } else {
            logging::log("EXEC", &format!("Launched app: {}", app.name));
            self.close_and_reset_window(cx);
        }
    }

    /// Focus a window from the main search results
    fn execute_window_focus(
        &mut self,
        window: &window_control::WindowInfo,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!("Focusing window: {} - {}", window.app, window.title),
        );

        if let Err(e) = window_control::focus_window(window.id) {
            logging::log("ERROR", &format!("Failed to focus window: {}", e));
            self.toast_manager.push(
                components::toast::Toast::error(
                    format!("Failed to focus window: {}", e),
                    &self.theme,
                )
                .duration_ms(Some(5000)),
            );
            cx.notify();
        } else {
            logging::log("EXEC", &format!("Focused window: {}", window.title));
            self.close_and_reset_window(cx);
        }
    }

    /// Show an API key configuration prompt.
    ///
    /// This creates an EnvPrompt that stores the key in the system keyring.
    /// Once stored, the key will be available to:
    /// - AI Chat window (via DetectedKeys::from_environment)
    /// - Scripts using `await env("SCRIPT_KIT_*_API_KEY")`
    fn show_api_key_prompt(
        &mut self,
        key_name: &str,
        prompt_text: &str,
        provider_name: &str,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!("Showing API key prompt for: {}", provider_name),
        );

        let id = format!("configure-{}", key_name.to_lowercase());
        let key = key_name.to_string();
        let prompt = Some(prompt_text.to_string());
        let secret = true; // API keys are always secrets

        // Store provider name for success message after completion
        self.pending_api_key_config = Some(provider_name.to_string());

        // Create submit callback that signals completion
        // The actual toast and view reset happens in handle_api_key_completion
        let completion_sender = self.api_key_completion_sender.clone();
        let provider_for_callback = provider_name.to_string();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id, value| {
                // Value being Some means the user submitted a value (key was saved)
                // Value being None means the user cancelled
                let success = value.is_some();
                logging::log(
                    "EXEC",
                    &format!(
                        "API key config callback: provider={}, success={}",
                        provider_for_callback, success
                    ),
                );
                // Signal completion to the app
                let _ = completion_sender.try_send((provider_for_callback.clone(), success));
            });

        // Check if key already exists in secrets (for UX messaging)
        // Use get_secret_info to get both existence and modification timestamp
        let secret_info = secrets::get_secret_info(&key);
        let exists_in_keyring = secret_info
            .as_ref()
            .map(|info| !info.value.is_empty())
            .unwrap_or(false);
        let modified_at = secret_info.map(|info| info.modified_at);

        if exists_in_keyring {
            logging::log(
                "EXEC",
                &format!(
                    "{} API key already configured (modified: {:?}) - showing update prompt",
                    provider_name, modified_at
                ),
            );
        }

        // Create EnvPrompt entity
        let focus_handle = self.focus_handle.clone();
        let env_prompt = prompts::EnvPrompt::new(
            id.clone(),
            key.clone(),
            prompt,
            Some(provider_name.to_string()), // title
            secret,
            focus_handle,
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            exists_in_keyring,
            modified_at,
        );

        let entity = cx.new(|_| env_prompt);
        self.current_view = AppView::EnvPrompt { id, entity };
        self.focused_input = FocusedInput::None; // EnvPrompt has its own focus handling
        self.pending_focus = Some(FocusTarget::EnvPrompt);

        // Resize to standard height for full-window centered layout
        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }

    /// Handle API key configuration completion.
    /// Called when the EnvPrompt callback signals completion.
    ///
    /// NOTE: This is called from render(), so we must use deferred resize via Window::defer
    /// to avoid layout issues where the macOS window resizes but GPUI's layout doesn't update.
    fn handle_api_key_completion(
        &mut self,
        provider: String,
        success: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pending_api_key_config = None;

        if success {
            // Show success toast
            self.toast_manager.push(
                components::toast::Toast::success(
                    format!("{} API key saved successfully", provider),
                    &self.theme,
                )
                .duration_ms(Some(3000)),
            );

            // Rebuild provider registry so new key is available next time chat opens
            self.rebuild_provider_registry_async(cx);
        }

        // Return to main menu
        self.reset_to_script_list(cx);

        // CRITICAL: Use deferred resize because this is called from render().
        // Synchronous resize (resize_to_view_sync) would resize the macOS window
        // but GPUI's layout system wouldn't update until the next frame,
        // causing the content to render at the wrong size (empty list bug).
        let target_height = window_resize::height_for_view(ViewType::ScriptList, 0);
        window.defer(cx, move |_window, _cx| {
            window_resize::resize_first_window_to_height(target_height);
        });

        cx.notify();
    }

    /// Enable Claude Code in config.ts and re-show the inline chat.
    ///
    /// This modifies the user's config.ts to enable Claude Code provider,
    /// reloads the config, and then re-opens the inline chat with
    /// the newly available Claude Code provider.
    pub fn enable_claude_code_in_config(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::config::editor::{self, ConfigWriteError, WriteOutcome};

        logging::log("EXEC", "Enabling Claude Code in config.ts");

        let config_path =
            std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());
        let bun_path = self.config.bun_path.as_deref();

        match editor::enable_claude_code_safely(&config_path, bun_path) {
            Ok(WriteOutcome::Written) => {
                logging::log("EXEC", "Claude Code enabled in config.ts");
            }
            Ok(WriteOutcome::Created) => {
                logging::log("EXEC", "Created new config.ts with Claude Code enabled");
            }
            Ok(WriteOutcome::AlreadySet) => {
                logging::log("EXEC", "Claude Code already enabled in config.ts");
            }
            Err(ConfigWriteError::ValidationFailed(reason)) => {
                logging::log(
                    "EXEC",
                    &format!("Config validation failed: {}", reason),
                );
                // Attempt to recover from backup
                match editor::recover_from_backup(&config_path, bun_path) {
                    Ok(true) => {
                        logging::log("EXEC", "Config restored from backup after validation failure");
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                "Failed to enable Claude Code (invalid config). Backup restored."
                                    .to_string(),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                    Ok(false) => {
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!(
                                    "Failed to enable Claude Code: {}. No backup available.",
                                    reason
                                ),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                    Err(recover_err) => {
                        logging::log(
                            "EXEC",
                            &format!("Backup recovery also failed: {}", recover_err),
                        );
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!(
                                    "Failed to enable Claude Code: {}. Recovery failed: {}",
                                    reason, recover_err
                                ),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                }
                cx.notify();
                return;
            }
            Err(e) => {
                logging::log("EXEC", &format!("Failed to enable Claude Code: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to enable Claude Code: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        }

        // Reload config and rebuild provider registry in background
        self.config = crate::config::load_config();
        self.rebuild_provider_registry_async(cx);

        // Check if Claude CLI is actually installed (this is an explicit user action,
        // so the brief sync check is acceptable for correct toast messaging)
        let claude_path = self
            .config
            .get_claude_code()
            .path
            .unwrap_or_else(|| "claude".to_string());
        let claude_available = std::process::Command::new(&claude_path)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if claude_available {
            self.toast_manager.push(
                components::toast::Toast::success(
                    "Claude Code enabled! Ready to use.".to_string(),
                    &self.theme,
                )
                .duration_ms(Some(3000)),
            );

            // Go back to main menu, then re-show inline chat
            self.go_back_or_close(window, cx);
            self.show_inline_ai_chat(None, cx);
        } else {
            self.toast_manager.push(
                components::toast::Toast::warning(
                    "Config saved! Install Claude CLI: npm install -g @anthropic-ai/claude-code"
                        .to_string(),
                    &self.theme,
                )
                .duration_ms(Some(8000)),
            );
            logging::log(
                "EXEC",
                "Claude Code config saved but CLI not found - user needs to install it",
            );
        }

        cx.notify();
    }

    /// Get the scratch pad file path
    fn get_scratch_pad_path() -> std::path::PathBuf {
        setup::get_kit_path().join("scratch-pad.md")
    }

    /// Open the scratch pad editor with auto-save functionality
    fn open_scratch_pad(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "Opening Scratch Pad");

        // Get or create scratch pad file path
        let scratch_path = Self::get_scratch_pad_path();

        // Ensure parent directory exists
        if let Some(parent) = scratch_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                logging::log(
                    "ERROR",
                    &format!("Failed to create scratch pad directory: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to create directory: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        }

        // Load existing content or create empty file
        let content = match std::fs::read_to_string(&scratch_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Create empty file
                if let Err(write_err) = std::fs::write(&scratch_path, "") {
                    logging::log(
                        "ERROR",
                        &format!("Failed to create scratch pad file: {}", write_err),
                    );
                }
                String::new()
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to read scratch pad: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to read scratch pad: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        };

        logging::log(
            "EXEC",
            &format!("Loaded scratch pad with {} bytes", content.len()),
        );

        // Create editor focus handle
        let editor_focus_handle = cx.focus_handle();

        // Create submit callback that saves and closes
        let scratch_path_clone = scratch_path.clone();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, value: Option<String>| {
                if let Some(content) = value {
                    // Save the content to disk
                    if let Err(e) = std::fs::write(&scratch_path_clone, &content) {
                        tracing::error!(error = %e, "Failed to save scratch pad on submit");
                    } else {
                        tracing::info!(bytes = content.len(), "Scratch pad saved on submit");
                    }
                }
            });

        // Get the target height for editor view (subtract footer height for unified footer)
        let editor_height = px(700.0 - window_resize::layout::FOOTER_HEIGHT);

        // Create the editor prompt
        let editor_prompt = EditorPrompt::with_height(
            "scratch-pad".to_string(),
            content,
            "markdown".to_string(), // Use markdown for nice highlighting
            editor_focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(editor_height),
        );

        let entity = cx.new(|_| editor_prompt);

        // Set up auto-save timer using weak reference
        let scratch_path_for_save = scratch_path;
        let entity_weak = entity.downgrade();
        cx.spawn(async move |_this, cx| {
            loop {
                // Auto-save every 2 seconds
                gpui::Timer::after(std::time::Duration::from_secs(2)).await;

                // Try to save the current content
                let save_result = cx.update(|cx| {
                    if let Some(entity) = entity_weak.upgrade() {
                        // Use update on the entity to get the correct Context<EditorPrompt>
                        let content: String = entity.update(cx, |editor, cx| editor.content(cx));
                        if let Err(e) = std::fs::write(&scratch_path_for_save, &content) {
                            tracing::warn!(error = %e, "Auto-save failed");
                        } else {
                            tracing::debug!(bytes = content.len(), "Auto-saved scratch pad");
                        }
                        true // Entity still exists
                    } else {
                        false // Entity dropped, stop the task
                    }
                });

                match save_result {
                    Ok(true) => continue,
                    Ok(false) | Err(_) => break, // Entity gone or context invalid
                }
            }
        })
        .detach();

        self.current_view = AppView::ScratchPadView {
            entity,
            focus_handle: editor_focus_handle,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::EditorPrompt);

        // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
        // to after the current GPUI update cycle completes.
        cx.spawn(async move |_this, _cx| {
            resize_to_view_sync(ViewType::EditorPrompt, 0);
        })
        .detach();
        cx.notify();
    }

    /// Open a terminal with a specific command (for fallback "Run in Terminal")
    pub fn open_terminal_with_command(&mut self, command: String, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Opening terminal with command: {}", command),
        );

        // Create submit callback that just closes on exit/escape
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {
                // Terminal exited - nothing special to do
            });

        // Get the target height for terminal view (subtract footer height)
        let term_height =
            window_resize::layout::MAX_HEIGHT - px(window_resize::layout::FOOTER_HEIGHT);

        // Create terminal with the specified command
        match term_prompt::TermPrompt::with_height(
            "fallback-terminal".to_string(),
            Some(command), // Run the specified command
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        ) {
            Ok(term_prompt) => {
                let entity = cx.new(|_| term_prompt);
                self.current_view = AppView::QuickTerminalView { entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::TermPrompt);
                // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                // to after the current GPUI update cycle completes. Synchronous Cocoa
                // setFrame: calls during render can trigger events that re-borrow GPUI state.
                cx.spawn(async move |_this, _cx| {
                    resize_to_view_sync(ViewType::TermPrompt, 0);
                })
                .detach();
                cx.notify();
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to create terminal: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to open terminal: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    // =========================================================================
    // File Search Implementation
    // =========================================================================
    //
    // BLOCKED: Requires the following changes to main.rs (not in worker reservations):
    //
    // 1. Add to AppView enum:
    //    ```rust
    //    /// Showing file search results (Spotlight/mdfind based)
    //    FileSearchView {
    //        query: String,
    //        selected_index: usize,
    //    },
    //    ```
    //
    // 2. Add to ScriptListApp struct:
    //    ```rust
    //    /// Cached file search results
    //    cached_file_results: Vec<file_search::FileResult>,
    //    /// Scroll handle for file search list
    //    file_search_scroll_handle: UniformListScrollHandle,
    //    ```
    //
    // 3. Add initialization in app_impl.rs ScriptListApp::new():
    //    ```rust
    //    cached_file_results: Vec::new(),
    //    file_search_scroll_handle: UniformListScrollHandle::new(),
    //    ```
    //
    // 4. Add render call in main.rs Render impl match arm:
    //    ```rust
    //    AppView::FileSearchView { query, selected_index } => {
    //        self.render_file_search(query.clone(), *selected_index, cx)
    //    }
    //    ```
    //
    // 5. Wire up in app_impl.rs execute_fallback():
    //    ```rust
    //    FallbackResult::SearchFiles { query } => {
    //        self.open_file_search(query, cx);
    //    }
    //    ```
    //
    // Once those are added, uncomment the method below.
    // =========================================================================

    /// Open file search with the given query
    ///
    /// This performs an mdfind-based file search and displays results in a Raycast-like UI.
    ///
    /// # Arguments
    /// * `query` - The search query (passed from the "Search Files" fallback action)
    ///
    /// # Usage
    /// Called when user selects "Search Files" fallback with a search term.
    /// Features:
    /// - Live search as user types (debounced)
    /// - File type icons (folder, document, image, audio, video, code, etc.)
    /// - File size and modified date display
    /// - Enter: Open file in default application
    /// - Cmd+Enter: Reveal in Finder
    pub fn open_file_search(&mut self, query: String, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Opening File Search with query: {}", query),
        );

        // Perform initial search or directory listing
        // Check if query looks like a directory path
        let results = if file_search::is_directory_path(&query) {
            logging::log(
                "EXEC",
                &format!("Detected directory path, listing: {}", query),
            );
            // Verify path is actually a directory before listing
            let expanded = file_search::expand_path(&query);
            let is_real_dir = expanded
                .as_deref()
                .map(|p| std::path::Path::new(p).is_dir())
                .unwrap_or(false);

            let dir_results = file_search::list_directory(&query, file_search::DEFAULT_CACHE_LIMIT);

            // Fallback to Spotlight search if path looks like directory but isn't
            if dir_results.is_empty() && !is_real_dir {
                logging::log(
                    "EXEC",
                    "Path mode not a real directory; falling back to Spotlight search",
                );
                file_search::search_files(&query, None, file_search::DEFAULT_SEARCH_LIMIT)
            } else {
                dir_results
            }
        } else {
            file_search::search_files(&query, None, file_search::DEFAULT_SEARCH_LIMIT)
        };
        logging::log(
            "EXEC",
            &format!("File search found {} results", results.len()),
        );

        // Cache the results
        self.cached_file_results = results;

        // Set up the view state
        self.filter_text = query.clone();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search files...".to_string());

        // Switch to file search view
        self.current_view = AppView::FileSearchView {
            query,
            selected_index: 0,
        };
        self.hovered_index = None;

        // Use standard height for file search view (same as window switcher)
        resize_to_view_sync(ViewType::ScriptList, 0);

        // Focus the main filter input so cursor blinks and typing works
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;

        // Initialize file search state for streaming
        self.file_search_gen = 0;
        self.file_search_cancel = None;
        self.file_search_display_indices.clear();

        // Compute initial display indices
        self.recompute_file_search_display_indices();

        cx.notify();
    }

    /// Sort directory listing results: directories first, then alphabetically
    pub fn sort_directory_results(&mut self) {
        // Sort the cached results in place
        self.cached_file_results.sort_by(|a, b| {
            let a_is_dir = matches!(a.file_type, crate::file_search::FileType::Directory);
            let b_is_dir = matches!(b.file_type, crate::file_search::FileType::Directory);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }

    /// Recompute file_search_display_indices based on current filter pattern
    ///
    /// This is called when:
    /// 1. Results change (new directory listing or search results)
    /// 2. Filter pattern changes (user types in existing directory)
    /// 3. Loading completes (final sort/rank)
    ///
    /// By computing this OUTSIDE of render, we ensure that animation tickers
    /// calling cx.notify() at 60fps don't re-run expensive Nucleo scoring.
    pub fn recompute_file_search_display_indices(&mut self) {
        // Get current filter pattern from the query
        let filter_pattern = if let AppView::FileSearchView { ref query, .. } = self.current_view {
            // Use frozen filter if set (during directory transitions)
            if let Some(ref frozen) = self.file_search_frozen_filter {
                frozen.clone()
            } else if let Some(parsed) = crate::file_search::parse_directory_path(query) {
                parsed.filter
            } else if !query.is_empty() {
                Some(query.clone())
            } else {
                None
            }
        } else {
            None
        };

        let cached_count = self.cached_file_results.len();

        // Compute display indices
        self.file_search_display_indices = if let Some(ref pattern) = filter_pattern {
            // Use Nucleo fuzzy matching and return only the indices, sorted by score
            let indices: Vec<usize> = crate::file_search::filter_results_nucleo_simple(
                &self.cached_file_results,
                pattern,
            )
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
            logging::log(
                "SEARCH",
                &format!(
                    "recompute_display_indices: pattern='{}' cached={} -> display={}",
                    pattern,
                    cached_count,
                    indices.len()
                ),
            );
            indices
        } else {
            // No filter - show all results in order
            logging::log(
                "SEARCH",
                &format!(
                    "recompute_display_indices: no_filter cached={} -> display={}",
                    cached_count, cached_count
                ),
            );
            (0..self.cached_file_results.len()).collect()
        };
    }

    /// Open the quick terminal
    fn open_quick_terminal(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "Opening Quick Terminal");

        // Create submit callback that just closes on exit/escape
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {
                // Terminal exited - nothing special to do
            });

        // Get the target height for terminal view (subtract footer height)
        let term_height =
            window_resize::layout::MAX_HEIGHT - px(window_resize::layout::FOOTER_HEIGHT);

        // Create terminal without a specific command (opens default shell)
        match term_prompt::TermPrompt::with_height(
            "quick-terminal".to_string(),
            None, // No command - opens default shell
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        ) {
            Ok(term_prompt) => {
                let entity = cx.new(|_| term_prompt);
                self.current_view = AppView::QuickTerminalView { entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::TermPrompt);
                // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                // to after the current GPUI update cycle completes. Synchronous Cocoa
                // setFrame: calls during render can trigger events that re-borrow GPUI state.
                cx.spawn(async move |_this, _cx| {
                    resize_to_view_sync(ViewType::TermPrompt, 0);
                })
                .detach();
                cx.notify();
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to create quick terminal: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to open terminal: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    /// Open the webcam prompt
    fn open_webcam(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "Opening Webcam prompt");

        let focus_handle = self.focus_handle.clone();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(|_id: String, value: Option<String>| {
                if let Some(data) = value {
                    logging::log("EXEC", &format!("Webcam capture data: {} bytes", data.len()));
                }
            });

        let webcam_prompt = prompts::WebcamPrompt::new(
            "webcam".to_string(),
            focus_handle,
            submit_callback,
            std::sync::Arc::clone(&self.theme),
        );

        let entity = cx.new(|_| webcam_prompt);

        // Zero-copy camera capture via AVFoundation.
        // Camera frames arrive as CVPixelBuffer on a dispatch queue,
        // then we poll and pass them to gpui::surface() — no CPU conversion.
        let entity_weak = entity.downgrade();

        let (frame_rx, capture_handle) = match crate::camera::start_capture(640) {
            Ok(pair) => pair,
            Err(err) => {
                logging::log("ERROR", &format!("Failed to start webcam: {}", err));
                // Still show the prompt with an error
                let entity_weak2 = entity.downgrade();
                let err_msg = err.to_string();
                cx.spawn(async move |_this, cx| {
                    let _ = cx.update(|cx| {
                        if let Some(entity) = entity_weak2.upgrade() {
                            entity.update(cx, |prompt, cx| {
                                prompt.set_error(err_msg, cx);
                            });
                        }
                    });
                })
                .detach();

                self.current_view = AppView::WebcamView { entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::AppRoot);
                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
                return;
            }
        };

        // Store the capture handle in the prompt — when the prompt entity is
        // dropped, the handle drops too, stopping the camera and releasing resources.
        entity.update(cx, |prompt, _cx| {
            prompt.capture_handle = Some(capture_handle);
        });

        // Async poller: drain CVPixelBuffers from channel, push latest to prompt.
        // Exits when the channel disconnects (CaptureHandle dropped) or the entity is gone.
        cx.spawn(async move |_this, cx| {
            loop {
                gpui::Timer::after(std::time::Duration::from_millis(16)).await;

                // Drain to latest frame, detect channel disconnect
                let mut latest = None;
                loop {
                    match frame_rx.try_recv() {
                        Ok(buf) => latest = Some(buf),
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                    }
                }

                let Some(buf) = latest else {
                    continue;
                };

                let result = cx.update(|cx| {
                    if let Some(entity) = entity_weak.upgrade() {
                        entity.update(cx, |prompt, cx| {
                            prompt.set_pixel_buffer(buf, cx);
                        });
                        true
                    } else {
                        false
                    }
                });

                match result {
                    Ok(true) => continue,
                    Ok(false) | Err(_) => break,
                }
            }
        })
        .detach();

        self.current_view = AppView::WebcamView { entity };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);

        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }

    /// Handle builtin confirmation modal result.
    /// Called when user confirms or cancels a dangerous action from the modal.
    fn handle_builtin_confirmation(
        &mut self,
        entry_id: String,
        confirmed: bool,
        cx: &mut Context<Self>,
    ) {
        if !confirmed {
            logging::log(
                "EXEC",
                &format!("Builtin confirmation cancelled: {}", entry_id),
            );
            return;
        }

        logging::log(
            "EXEC",
            &format!("Builtin confirmation accepted, executing: {}", entry_id),
        );

        // Find the builtin entry by ID and execute it
        let builtin_entries = builtins::get_builtin_entries(&self.config.get_builtins());
        if let Some(entry) = builtin_entries.iter().find(|b| b.id == entry_id) {
            // Execute the confirmed builtin action directly
            // Skip confirmation check since we're coming from the modal callback
            self.execute_builtin_confirmed(entry, cx);
        } else {
            logging::log(
                "ERROR",
                &format!("Builtin entry not found for confirmed action: {}", entry_id),
            );
        }
    }

    /// Execute a builtin that has already been confirmed.
    /// This skips the confirmation check and directly executes the action.
    fn execute_builtin_confirmed(
        &mut self,
        entry: &builtins::BuiltInEntry,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!(
                "Executing confirmed built-in: {} (id: {})",
                entry.name, entry.id
            ),
        );

        // Direct execution - same logic as execute_builtin but without confirmation check
        match &entry.feature {
            // System Actions that can be dangerous
            builtins::BuiltInFeature::SystemAction(action_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing confirmed system action: {:?}", action_type),
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
                            return;
                        }

                        // App control
                        SystemActionType::QuitScriptKit => {
                            logging::log("EXEC", "Quitting Script Kit (confirmed)");
                            cx.quit();
                            return;
                        }

                        // For other system actions that don't need confirmation,
                        // fall through to execute them
                        _ => {
                            // These shouldn't typically be confirmed, but handle gracefully
                            logging::log(
                                "EXEC",
                                &format!(
                                    "Executing non-dangerous system action: {:?}",
                                    action_type
                                ),
                            );
                            // Call the original execute_builtin for these
                            // Note: This creates a temp config with no confirmation to avoid loop
                            match action_type {
                                SystemActionType::ToggleDarkMode => {
                                    system_actions::toggle_dark_mode()
                                }
                                SystemActionType::ShowDesktop => system_actions::show_desktop(),
                                SystemActionType::MissionControl => {
                                    system_actions::mission_control()
                                }
                                SystemActionType::Launchpad => system_actions::launchpad(),
                                SystemActionType::ForceQuitApps => {
                                    system_actions::force_quit_apps()
                                }
                                SystemActionType::Volume0 => system_actions::set_volume(0),
                                SystemActionType::Volume25 => system_actions::set_volume(25),
                                SystemActionType::Volume50 => system_actions::set_volume(50),
                                SystemActionType::Volume75 => system_actions::set_volume(75),
                                SystemActionType::Volume100 => system_actions::set_volume(100),
                                SystemActionType::VolumeMute => system_actions::volume_mute(),
                                SystemActionType::ToggleDoNotDisturb => {
                                    system_actions::toggle_do_not_disturb()
                                }
                                SystemActionType::StartScreenSaver => {
                                    system_actions::start_screen_saver()
                                }
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
                                _ => Ok(()),
                            }
                        }
                    };

                    match result {
                        Ok(()) => {
                            logging::log("EXEC", "Confirmed system action executed successfully");
                            self.close_and_reset_window(cx);
                        }
                        Err(e) => {
                            logging::log(
                                "ERROR",
                                &format!("Confirmed system action failed: {}", e),
                            );
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

            // For any other builtin type that somehow got confirmed,
            // just execute it normally (shouldn't happen in practice)
            _ => {
                logging::log(
                    "WARN",
                    &format!("Unexpected confirmed builtin type: {:?}", entry.feature),
                );
            }
        }
    }
}
