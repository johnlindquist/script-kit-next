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
