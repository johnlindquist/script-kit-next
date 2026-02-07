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
