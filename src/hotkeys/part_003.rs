#[allow(dead_code)]
pub(crate) fn start_hotkey_listener(config: config::Config) {
    std::thread::spawn(move || {
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to create hotkey manager: {}", e));
                return;
            }
        };

        if MAIN_MANAGER.set(Mutex::new(manager)).is_err() {
            logging::log("HOTKEY", "Manager already initialized (unexpected)");
            return;
        }

        let manager_guard = match MAIN_MANAGER.get().unwrap().lock() {
            Ok(g) => g,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to lock manager: {}", e));
                return;
            }
        };

        // Register main hotkey using unified registration
        if register_builtin_hotkey(&manager_guard, HotkeyAction::Main, &config.hotkey).is_some() {
            MAIN_HOTKEY_REGISTERED.store(true, Ordering::Relaxed);
        }

        // Register notes hotkey (only if configured - no default)
        if let Some(notes_hotkey) = config.get_notes_hotkey() {
            register_builtin_hotkey(&manager_guard, HotkeyAction::Notes, &notes_hotkey);
        }
        // Register AI and logs hotkeys
        if let Some(ai_hotkey) = config.get_ai_hotkey() {
            register_builtin_hotkey(&manager_guard, HotkeyAction::Ai, &ai_hotkey);
        }
        if let Some(logs_hotkey) = config.get_logs_hotkey() {
            register_builtin_hotkey(&manager_guard, HotkeyAction::ToggleLogs, &logs_hotkey);
        }

        // Register script shortcuts
        let mut script_count = 0;

        let all_scripts = scripts::read_scripts();
        for script in &all_scripts {
            if let Some(ref shortcut) = script.shortcut {
                let path = script.path.to_string_lossy().to_string();
                if register_script_hotkey_internal(&manager_guard, &path, shortcut, &script.name)
                    .is_some()
                {
                    script_count += 1;
                }
            }
        }

        let all_scriptlets = scripts::load_scriptlets();
        for scriptlet in &all_scriptlets {
            if let Some(ref shortcut) = scriptlet.shortcut {
                let path = scriptlet
                    .file_path
                    .clone()
                    .unwrap_or_else(|| scriptlet.name.clone());
                if register_script_hotkey_internal(&manager_guard, &path, shortcut, &scriptlet.name)
                    .is_some()
                {
                    script_count += 1;
                }
            }
        }

        logging::log(
            "HOTKEY",
            &format!("Registered {} script/scriptlet shortcuts", script_count),
        );

        // Track which command IDs have been registered to avoid duplicates
        let mut registered_commands: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // Priority 1: Load shortcuts from config.ts commands field (highest priority)
        let mut config_count = 0;
        if let Some(commands) = &config.commands {
            for (command_id, cmd_config) in commands {
                if let Some(hotkey_config) = &cmd_config.shortcut {
                    let shortcut_str = hotkey_config.to_shortcut_string();
                    if register_script_hotkey_internal(
                        &manager_guard,
                        command_id,
                        &shortcut_str,
                        command_id,
                    )
                    .is_some()
                    {
                        registered_commands.insert(command_id.clone());
                        config_count += 1;
                    }
                }
            }
            if config_count > 0 {
                logging::log(
                    "HOTKEY",
                    &format!(
                        "Registered {} command shortcuts from config.ts",
                        config_count
                    ),
                );
            }
        }

        // Priority 2: Load user shortcut overrides from shortcuts.json
        // (skips commands already registered from config.ts)
        let mut override_count = 0;
        match crate::shortcuts::load_shortcut_overrides() {
            Ok(overrides) => {
                for (command_id, shortcut) in overrides {
                    // Skip if already registered from config.ts
                    if registered_commands.contains(&command_id) {
                        logging::log(
                            "HOTKEY",
                            &format!(
                                "Skipping shortcuts.json entry for '{}' (config.ts takes priority)",
                                command_id
                            ),
                        );
                        continue;
                    }

                    let shortcut_str = shortcut.to_canonical_string();
                    if register_script_hotkey_internal(
                        &manager_guard,
                        &command_id,
                        &shortcut_str,
                        &command_id,
                    )
                    .is_some()
                    {
                        override_count += 1;
                    }
                }
                if override_count > 0 {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Registered {} user shortcut overrides from shortcuts.json",
                            override_count
                        ),
                    );
                }
            }
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to load shortcuts.json: {}", e));
            }
        }

        // Log routing table summary
        {
            let routes_guard = routes().read();
            logging::log(
                "HOTKEY",
                &format!(
                    "Routing table: main={:?}, notes={:?}, ai={:?}, logs={:?}, scripts={}",
                    routes_guard.main_id,
                    routes_guard.notes_id,
                    routes_guard.ai_id,
                    routes_guard.logs_id,
                    routes_guard.script_paths.len()
                ),
            );
        }

        drop(manager_guard);
        let receiver = GlobalHotKeyEvent::receiver();

        loop {
            if let Ok(event) = receiver.recv() {
                if event.state != HotKeyState::Pressed {
                    continue;
                }

                // Look up action in unified routing table (fast read lock)
                let action = {
                    let routes_guard = routes().read();
                    routes_guard.get_action(event.id)
                };

                match action {
                    Some(HotkeyAction::Main) => {
                        // Set correlation ID for this hotkey event
                        let correlation_id = format!("hotkey:main:{}", Uuid::new_v4());
                        let _guard = logging::set_correlation_id(correlation_id.clone());

                        let count = HOTKEY_TRIGGER_COUNT.fetch_add(1, Ordering::Relaxed);
                        // NON-BLOCKING: Use try_send to prevent hotkey thread from blocking
                        if hotkey_channel()
                            .0
                            .try_send(HotkeyEvent {
                                correlation_id: correlation_id.clone(),
                            })
                            .is_err()
                        {
                            logging::log("HOTKEY", "Main hotkey channel full/closed");
                        }
                        logging::log(
                            "HOTKEY",
                            &format!("Main hotkey pressed (trigger #{})", count + 1),
                        );
                    }
                    Some(HotkeyAction::Notes) => {
                        // Set correlation ID for this hotkey event
                        let correlation_id = format!("hotkey:notes:{}", Uuid::new_v4());
                        let _guard = logging::set_correlation_id(correlation_id.clone());

                        logging::log(
                            "HOTKEY",
                            "Notes hotkey pressed - dispatching to main thread",
                        );
                        dispatch_notes_hotkey(HotkeyEvent { correlation_id });
                    }
                    Some(HotkeyAction::Ai) => {
                        // Set correlation ID for this hotkey event
                        let correlation_id = format!("hotkey:ai:{}", Uuid::new_v4());
                        let _guard = logging::set_correlation_id(correlation_id.clone());

                        logging::log("HOTKEY", "AI hotkey pressed - dispatching to main thread");
                        dispatch_ai_hotkey(HotkeyEvent { correlation_id });
                    }
                    Some(HotkeyAction::ToggleLogs) => {
                        // Set correlation ID for this hotkey event
                        let correlation_id = format!("hotkey:logs:{}", Uuid::new_v4());
                        let _guard = logging::set_correlation_id(correlation_id.clone());

                        logging::log("HOTKEY", "Logs hotkey pressed - toggling log capture");
                        // Toggle capture immediately (no need for main thread dispatch)
                        let (is_capturing, path) = logging::toggle_capture();
                        if is_capturing {
                            if let Some(p) = path {
                                logging::log(
                                    "HOTKEY",
                                    &format!("Log capture STARTED: {}", p.display()),
                                );
                            }
                        } else if let Some(p) = path {
                            logging::log(
                                "HOTKEY",
                                &format!("Log capture STOPPED: {}", p.display()),
                            );
                        }
                        // Send to channel for UI notification (HUD)
                        if logs_hotkey_channel()
                            .0
                            .try_send(HotkeyEvent { correlation_id })
                            .is_err()
                        {
                            logging::log("HOTKEY", "Logs hotkey channel full/closed");
                        }
                    }
                    Some(HotkeyAction::Script(path)) => {
                        // Set correlation ID for this hotkey event
                        let correlation_id = format!("hotkey:script:{}:{}", path, Uuid::new_v4());
                        let _guard = logging::set_correlation_id(correlation_id.clone());

                        // Start benchmark timing for hotkey → chat latency analysis
                        logging::bench_start(&format!("hotkey:{}", path));
                        logging::log("HOTKEY", &format!("Script shortcut triggered: {}", path));
                        // NON-BLOCKING: Use try_send to prevent hotkey thread from blocking
                        if script_hotkey_channel()
                            .0
                            .try_send(ScriptHotkeyEvent {
                                command_id: path.clone(),
                                correlation_id,
                            })
                            .is_err()
                        {
                            logging::log(
                                "HOTKEY",
                                &format!("Script channel full/closed for {}", path),
                            );
                        }
                    }
                    None => {
                        // Set correlation ID even for unknown hotkey events
                        let _guard = logging::set_correlation_id(format!(
                            "hotkey:unknown:{}",
                            Uuid::new_v4()
                        ));

                        // Unknown hotkey ID - can happen during hot-reload transitions
                        logging::log("HOTKEY", &format!("Unknown hotkey event id={}", event.id));
                    }
                }
            }
        }
    });
}
