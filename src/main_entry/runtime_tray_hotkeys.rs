        // Defer tray initialization until after window creation so startup-to-first-render
        // is not blocked by tray icon rendering/menu construction.
        let tray_ready = Arc::new(AtomicBool::new(false));
        let tray_ready_for_fallback = Arc::clone(&tray_ready);
        let window_for_tray = window;
        let app_entity_for_tray = app_entity.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            // Yield once so window creation and initial render can proceed first.
            Timer::after(std::time::Duration::from_millis(1)).await;

            let tray_manager = match cx.update(|_cx| match TrayManager::new() {
                Ok(tm) => {
                    logging::log("TRAY", "Tray icon initialized successfully (deferred)");
                    Some(tm)
                }
                Err(e) => {
                    logging::log(
                        "TRAY",
                        &format!("Failed to initialize tray icon (deferred): {}", e),
                    );
                    None
                }
            }) {
                Ok(manager) => manager,
                Err(_) => {
                    logging::log(
                        "TRAY",
                        "Deferred tray initialization aborted: app context update failed",
                    );
                    None
                }
            };

            let Some(tray_mgr) = tray_manager else {
                return;
            };

            tray_ready.store(true, Ordering::SeqCst);
            logging::log("TRAY", "Tray menu event handler started (event-driven)");
            let _menu_event_receiver = tray_mgr.menu_event_receiver();

            let (tray_event_tx, tray_event_rx) = async_channel::bounded(32);
            std::thread::spawn(move || {
                logging::log("TRAY", "Tray menu receiver bridge started (blocking recv)");
                let menu_rx = tray_icon::menu::MenuEvent::receiver();

                while let Ok(event) = menu_rx.recv() {
                    if tray_event_tx.send_blocking(event).is_err() {
                        logging::log(
                            "TRAY",
                            "Tray menu receiver bridge exiting: event channel closed",
                        );
                        break;
                    }
                }

                logging::log("TRAY", "Tray menu receiver bridge exiting");
            });

            while let Ok(event) = tray_event_rx.recv().await {
                // Convert event to action using type-safe IDs (pure function)
                let action = TrayManager::action_from_event(&event);

                // Handle side effects for LaunchAtLogin before the match
                if let Some(TrayMenuAction::LaunchAtLogin) = action {
                    if let Err(e) = tray_mgr.handle_action(TrayMenuAction::LaunchAtLogin) {
                        logging::log("TRAY", &format!("Failed to toggle login item: {}", e));
                    }
                }

                match action {
                    Some(TrayMenuAction::OpenScriptKit) => {
                        logging::log("TRAY", "Open Script Kit menu item clicked");
                        let window_inner = window_for_tray;
                        let app_entity_inner = app_entity_for_tray.clone();
                        let _ = cx.update(|cx| {
                            show_main_window_helper(window_inner, app_entity_inner, cx);
                        });
                    }
                    Some(TrayMenuAction::OpenNotes) => {
                        logging::log("TRAY", "Notes menu item clicked");
                        let _ = cx.update(|cx| {
                            if let Err(e) = notes::open_notes_window(cx) {
                                logging::log("TRAY", &format!("Failed to open notes window: {}", e));
                            }
                        });
                    }
                    Some(TrayMenuAction::OpenAiChat) => {
                        logging::log("TRAY", "AI Chat menu item clicked");
                        let _ = cx.update(|cx| {
                            if let Err(e) = ai::open_ai_window(cx) {
                                logging::log("TRAY", &format!("Failed to open AI window: {}", e));
                            }
                        });
                    }
                    Some(TrayMenuAction::LaunchAtLogin) => {
                        // Side effects (toggle + checkbox update) handled above
                        logging::log("TRAY", "Launch at Login toggled");
                    }
                    Some(TrayMenuAction::Settings) => {
                        logging::log("TRAY", "Settings menu item clicked");
                        // Open config file in editor
                        let editor = config_for_tray_actions.get_editor();
                        let config_path =
                            shellexpand::tilde("~/.scriptkit/kit/config.ts").to_string();

                        logging::log(
                            "TRAY",
                            &format!("Opening {} in editor '{}'", config_path, editor),
                        );
                        match std::process::Command::new(&editor)
                            .arg(&config_path)
                            .spawn()
                        {
                            Ok(_) => logging::log("TRAY", &format!("Spawned editor: {}", editor)),
                            Err(e) => logging::log(
                                "TRAY",
                                &format!("Failed to spawn editor '{}': {}", editor, e),
                            ),
                        }
                    }
                    Some(TrayMenuAction::OpenOnGitHub) => {
                        logging::log("TRAY", "Open on GitHub menu item clicked");
                        let url = "https://github.com/script-kit/app";
                        if let Err(e) = open::that(url) {
                            logging::log("TRAY", &format!("Failed to open GitHub URL: {}", e));
                        }
                    }
                    Some(TrayMenuAction::OpenManual) => {
                        logging::log("TRAY", "Manual menu item clicked");
                        let url = "https://scriptkit.com";
                        if let Err(e) = open::that(url) {
                            logging::log("TRAY", &format!("Failed to open manual URL: {}", e));
                        }
                    }
                    Some(TrayMenuAction::JoinCommunity) => {
                        logging::log("TRAY", "Join Community menu item clicked");
                        let url = "https://discord.gg/qnUX4XqJQd";
                        if let Err(e) = open::that(url) {
                            logging::log("TRAY", &format!("Failed to open Discord URL: {}", e));
                        }
                    }
                    Some(TrayMenuAction::FollowUs) => {
                        logging::log("TRAY", "Follow Us menu item clicked");
                        let url = "https://twitter.com/scriptkitapp";
                        if let Err(e) = open::that(url) {
                            logging::log("TRAY", &format!("Failed to open Twitter URL: {}", e));
                        }
                    }
                    Some(TrayMenuAction::Quit) => {
                        logging::log("TRAY", "Quit menu item clicked");
                        // Set shutdown flag FIRST - prevents new script spawns
                        // and triggers the shutdown monitor task for unified cleanup
                        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);

                        // Clean up processes and PID file before quitting
                        PROCESS_MANAGER.kill_all_processes();
                        PROCESS_MANAGER.remove_main_pid();
                        let _ = cx.update(|cx| {
                            cx.quit();
                        });
                        break;
                    }
                    None => {
                        logging::log("TRAY", "Unknown menu event received");
                    }
                }
            }

            logging::log("TRAY", "Tray menu event handler exiting");
        })
        .detach();

        // Fallback: If both hotkey AND tray fail, the user has no way to access the app!
        // Wait a short time for hotkey registration, then check if we need to show the window.
        let window_for_fallback = window;
        let app_entity_for_fallback = app_entity.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            // Wait 500ms for hotkey registration to complete (it runs in a separate thread)
            Timer::after(std::time::Duration::from_millis(500)).await;

            let hotkey_ok = hotkeys::is_main_hotkey_registered();
            let tray_ok = tray_ready_for_fallback.load(Ordering::SeqCst);

            if !hotkey_ok && !tray_ok {
                logging::log("APP", "");
                logging::log("APP", "╔════════════════════════════════════════════════════════════════════════════╗");
                logging::log("APP", "║  WARNING: Both hotkey and tray initialization failed!                     ║");
                logging::log("APP", "║  Showing window at startup as fallback entry point.                       ║");
                logging::log("APP", "║  Check logs for specific errors.                                          ║");
                logging::log("APP", "╚════════════════════════════════════════════════════════════════════════════╝");
                logging::log("APP", "");

                // Show window using the centralized helper
                let _ = cx.update(|cx| {
                    show_main_window_helper(window_for_fallback, app_entity_for_fallback, cx);
                });
            } else {
                logging::log("APP", &format!("Entry points available: hotkey={}, tray={}", hotkey_ok, tray_ok));
            }
        }).detach();

        // Main window hotkey listener - uses Entity<ScriptListApp> instead of WindowHandle
        let app_entity_for_hotkey = app_entity.clone();
        let window_for_hotkey = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Main hotkey listener started");
            while let Ok(hotkey_event) = hotkeys::hotkey_channel().1.recv().await {
                let _guard = logging::set_correlation_id(hotkey_event.correlation_id.clone());
                logging::log("VISIBILITY", "");
                logging::log("VISIBILITY", "╔════════════════════════════════════════════════════════════╗");
                logging::log("VISIBILITY", "║  HOTKEY TRIGGERED - TOGGLE WINDOW                          ║");
                logging::log("VISIBILITY", "╚════════════════════════════════════════════════════════════╝");

                let is_visible = script_kit_gpui::is_main_window_visible();
                logging::log("VISIBILITY", &format!("State: WINDOW_VISIBLE={}", is_visible));

                let app_entity_inner = app_entity_for_hotkey.clone();
                let window_inner = window_for_hotkey;

                if is_visible {
                    logging::log("VISIBILITY", "Decision: HIDE");
                    let _ = cx.update(move |cx: &mut gpui::App| {
                        hide_main_window_helper(app_entity_inner, cx);
                    });
                } else {
                    logging::log("VISIBILITY", "Decision: SHOW");
                    let _ = cx.update(move |cx: &mut gpui::App| {
                        show_main_window_helper(window_inner, app_entity_inner, cx);
                    });
                }
            }
            logging::log("HOTKEY", "Main hotkey listener exiting");
        }).detach();

        // Notes hotkey listener - event-driven via async_channel
        // The hotkey thread dispatches via GPUI's ForegroundExecutor, which wakes this task
        // This works even before main window activates because the executor is initialized first
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Notes hotkey listener started (event-driven)");
            // Event-driven: .recv().await blocks until a message arrives
            // This is more efficient than polling and responds immediately
            while let Ok(hotkey_event) = hotkeys::notes_hotkey_channel().1.recv().await {
                let _guard = logging::set_correlation_id(hotkey_event.correlation_id.clone());
                logging::log("HOTKEY", "Notes hotkey triggered - opening notes window");
                let _ = cx.update(|cx: &mut gpui::App| {
                    if let Err(e) = notes::open_notes_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open notes window: {}", e));
                    }
                });
            }
            logging::log("HOTKEY", "Notes hotkey listener exiting (channel closed)");
        }).detach();

        // AI hotkey listener - event-driven via async_channel
        // Same pattern as Notes hotkey - works immediately on app launch
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "AI hotkey listener started (event-driven)");
            // Event-driven: .recv().await blocks until a message arrives
            while let Ok(hotkey_event) = hotkeys::ai_hotkey_channel().1.recv().await {
                let _guard = logging::set_correlation_id(hotkey_event.correlation_id.clone());
                logging::log("HOTKEY", "AI hotkey triggered - opening AI window");
                let _ = cx.update(|cx: &mut gpui::App| {
                    if let Err(e) = ai::open_ai_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open AI window: {}", e));
                    }
                });
            }
            logging::log("HOTKEY", "AI hotkey listener exiting (channel closed)");
        }).detach();

        // Script/Scriptlet/App hotkey listener - event-driven via async_channel
        // Handles shortcuts from shortcuts.json for scriptlets, builtins, and apps
        let app_entity_for_scripts = app_entity.clone();
        let window_for_scripts = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Script shortcut listener started (event-driven)");
            while let Ok(hotkey_event) = hotkeys::script_hotkey_channel().1.recv().await {
                let _guard = logging::set_correlation_id(hotkey_event.correlation_id.clone());
                let command_id = hotkey_event.command_id;
                logging::log(
                    "HOTKEY",
                    &format!("Script shortcut received in main.rs: {}", command_id),
                );

                let id_clone = command_id.clone();
                let app_entity_inner = app_entity_for_scripts.clone();
                let window_inner = window_for_scripts;

                let _ = cx.update(move |cx: &mut gpui::App| {
                    logging::log(
                        "HOTKEY",
                        &format!("Executing command_id: {}", id_clone),
                    );

                    // Clear NEEDS_RESET before executing new command - we're starting a fresh script
                    // and shouldn't reset based on previous script's exit state
                    if NEEDS_RESET.swap(false, Ordering::SeqCst) {
                        logging::log(
                            "HOTKEY",
                            "Cleared NEEDS_RESET before executing new command",
                        );
                    }

                    // Use app_entity.update to access ScriptListApp directly
                    // Returns whether main window should be shown (apps/certain builtins don't need it)
                    let should_show_window = app_entity_inner.update(cx, |view, ctx| {
                        logging::log(
                            "HOTKEY",
                            "Inside app_entity update, calling execute_by_command_id_or_path",
                        );
                        view.execute_by_command_id_or_path(&id_clone, ctx)
                    });

                    // Only show window if command needs it AND it's currently hidden
                    if should_show_window && !script_kit_gpui::is_main_window_visible() {
                        logging::log("HOTKEY", "Command needs main window, showing it");
                        show_main_window_helper(window_inner, app_entity_inner.clone(), cx);
                    } else if !should_show_window {
                        logging::log("HOTKEY", "Command doesn't need main window (app/ai/notes)");
                    }
                });
            }
            logging::log("HOTKEY", "Script shortcut listener exiting (channel closed)");
        }).detach();

        // Deeplink listener - handles scriptkit:// URLs (same logic as hotkeys)
        let app_entity_for_deeplinks = app_entity.clone();
        let window_for_deeplinks = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("DEEPLINK", "Deeplink listener started (event-driven)");
            while let Ok(command_id) = deeplink_channel().1.recv().await {
                logging::log(
                    "DEEPLINK",
                    &format!("Processing deeplink command: {}", command_id),
                );

                // Handle special deeplink types
                if command_id.starts_with("notes/") {
                    // Notes deeplink - open notes window
                    // Note: Currently opens notes window; specific note navigation can be added later
                    let note_id = command_id.strip_prefix("notes/").unwrap_or("");
                    logging::log("DEEPLINK", &format!("Opening notes (note_id: {})", note_id));
                    let _ = cx.update(|cx| {
                        if let Err(e) = notes::open_notes_window(cx) {
                            logging::log("DEEPLINK", &format!("Failed to open notes: {}", e));
                        }
                    });
                    continue;
                }

                let id_clone = command_id.clone();
                let app_entity_inner = app_entity_for_deeplinks.clone();
                let window_inner = window_for_deeplinks;

                let _ = cx.update(move |cx: &mut gpui::App| {
                    logging::log(
                        "DEEPLINK",
                        &format!("Executing command_id: {}", id_clone),
                    );

                    // Clear NEEDS_RESET before executing new command - we're starting a fresh script
                    // and shouldn't reset based on previous script's exit state
                    if NEEDS_RESET.swap(false, Ordering::SeqCst) {
                        logging::log(
                            "DEEPLINK",
                            "Cleared NEEDS_RESET before executing new command",
                        );
                    }

                    // Use app_entity.update to access ScriptListApp directly
                    // Returns whether main window should be shown (apps/certain builtins don't need it)
                    let should_show_window = app_entity_inner.update(cx, |view, ctx| {
                        logging::log(
                            "DEEPLINK",
                            "Inside app_entity update, calling execute_by_command_id_or_path",
                        );
                        view.execute_by_command_id_or_path(&id_clone, ctx)
                    });

                    // Only show window if command needs it AND it's currently hidden
                    if should_show_window && !script_kit_gpui::is_main_window_visible() {
                        logging::log("DEEPLINK", "Command needs main window, showing it");
                        show_main_window_helper(window_inner, app_entity_inner.clone(), cx);
                    } else if !should_show_window {
                        logging::log("DEEPLINK", "Command doesn't need main window (app/ai/notes)");
                    }
                });
            }
            logging::log("DEEPLINK", "Deeplink listener exiting (channel closed)");
        }).detach();

        // Show window listener - handles requests to show main window after script exit
        // This is needed because prompt_handler doesn't have access to show_main_window_helper
        let app_entity_for_show = app_entity.clone();
        let window_for_show = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("VISIBILITY", "Show window listener started (event-driven)");
            let (_, rx) = script_kit_gpui::show_window_channel();
            while let Ok(()) = rx.recv().await {
                logging::log(
                    "VISIBILITY",
                    "Show window request received - bringing main menu back",
                );
                let app_entity_inner = app_entity_for_show.clone();
                let window_inner = window_for_show;
                let _ = cx.update(move |cx: &mut gpui::App| {
                    show_main_window_helper(window_inner, app_entity_inner, cx);
                });
            }
            logging::log("VISIBILITY", "Show window listener exiting (channel closed)");
        }).detach();
