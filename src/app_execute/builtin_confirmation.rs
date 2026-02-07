impl ScriptListApp {
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
