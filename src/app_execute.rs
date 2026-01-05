// App execution methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: execute_builtin, execute_app, execute_window_focus

impl ScriptListApp {
    fn execute_builtin(&mut self, entry: &builtins::BuiltInEntry, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Executing built-in: {} (id: {})", entry.name, entry.id),
        );

        // Check if this command requires confirmation
        if self.config.requires_confirmation(&entry.id) {
            // Check if we're already in confirmation mode for this entry
            if self.pending_confirmation.as_ref() == Some(&entry.id) {
                // User confirmed - clear pending and proceed with execution
                logging::log("EXEC", &format!("Confirmed: {}", entry.id));
                self.pending_confirmation = None;
                // Fall through to execute
            } else {
                // First press - enter confirmation mode
                logging::log("EXEC", &format!("Awaiting confirmation: {}", entry.id));
                self.pending_confirmation = Some(entry.id.clone());
                cx.notify();
                return; // Don't execute yet
            }
        }

        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                logging::log("EXEC", "Opening Clipboard History");
                // Use cached entries for faster loading
                let entries = clipboard_history::get_cached_entries(100);
                logging::log(
                    "EXEC",
                    &format!("Loaded {} clipboard entries (cached)", entries.len()),
                );
                // Initial selected_index should be 0 (first entry)
                // Note: clipboard history uses a flat list without section headers
                self.current_view = AppView::ClipboardHistoryView {
                    entries,
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for clipboard history view
                defer_resize_to_view(ViewType::ScriptList, 0, cx);
                cx.notify();
            }
            builtins::BuiltInFeature::AppLauncher => {
                logging::log("EXEC", "Opening App Launcher");
                let apps = app_launcher::scan_applications().clone();
                logging::log("EXEC", &format!("Loaded {} applications", apps.len()));
                self.current_view = AppView::AppLauncherView {
                    apps,
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for app launcher view
                defer_resize_to_view(ViewType::ScriptList, 0, cx);
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
                // Load windows when view is opened (windows change frequently)
                match window_control::list_windows() {
                    Ok(windows) => {
                        logging::log("EXEC", &format!("Loaded {} windows", windows.len()));
                        self.current_view = AppView::WindowSwitcherView {
                            windows,
                            filter: String::new(),
                            selected_index: 0,
                        };
                        // Use standard height for window switcher view
                        defer_resize_to_view(ViewType::ScriptList, 0, cx);
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
                self.current_view = AppView::DesignGalleryView {
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for design gallery view
                defer_resize_to_view(ViewType::ScriptList, 0, cx);
                cx.notify();
            }
            builtins::BuiltInFeature::AiChat => {
                logging::log("EXEC", "Opening AI Chat window");
                // Reset state, hide main window, and open AI window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();
                if let Err(e) = ai::open_ai_window(cx) {
                    logging::log("ERROR", &format!("Failed to open AI window: {}", e));
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

            // =========================================================================
            // Window Actions (for frontmost window of the PREVIOUS app)
            // =========================================================================
            builtins::BuiltInFeature::WindowAction(action_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing window action: {:?}", action_type),
                );

                // Get the frontmost window of the app that was active before Script Kit.
                // Since Script Kit is an LSUIElement (accessory app), it doesn't take
                // menu bar ownership. The menu bar owner is the previously active app.
                match window_control::get_frontmost_window_of_previous_app() {
                    Ok(Some(target_window)) => {
                        use builtins::WindowActionType;
                        use window_control::TilePosition;

                        logging::log(
                            "EXEC",
                            &format!(
                                "Target window: {} - {} (id: {})",
                                target_window.app, target_window.title, target_window.id
                            ),
                        );

                        let result = match action_type {
                            WindowActionType::TileLeft => window_control::tile_window(
                                target_window.id,
                                TilePosition::LeftHalf,
                            ),
                            WindowActionType::TileRight => window_control::tile_window(
                                target_window.id,
                                TilePosition::RightHalf,
                            ),
                            WindowActionType::TileTop => {
                                window_control::tile_window(target_window.id, TilePosition::TopHalf)
                            }
                            WindowActionType::TileBottom => window_control::tile_window(
                                target_window.id,
                                TilePosition::BottomHalf,
                            ),
                            WindowActionType::Maximize => {
                                window_control::maximize_window(target_window.id)
                            }
                            WindowActionType::Minimize => {
                                window_control::minimize_window(target_window.id)
                            }
                        };

                        match result {
                            Ok(()) => {
                                logging::log("EXEC", "Window action executed successfully");
                                // Reset and hide - does the reset work immediately while hiding
                                self.close_and_reset_window(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Window action failed: {}", e));
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Window action failed: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }
                    Ok(None) => {
                        logging::log("WARN", "No windows found for previous app");
                        self.toast_manager.push(
                            components::toast::Toast::warning(
                                "No windows available to manage",
                                &self.theme,
                            )
                            .duration_ms(Some(3000)),
                        );
                        cx.notify();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to get target window: {}", e));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to find target window: {}", e),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                        cx.notify();
                    }
                }
            }

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

                // All AI commands: reset state, hide main window, open AI
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

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
                    ScriptCommandType::NewScriptlet => (
                        script_creation::create_new_extension("untitled"),
                        "extension",
                    ),
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
}
