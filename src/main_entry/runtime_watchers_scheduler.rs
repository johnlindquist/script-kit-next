        // Note: Appearance watching is now handled by GPUI's observe_window_appearance
        // (set up during window creation above), replacing the custom AppearanceWatcher.

        // Config reload watcher - watches ~/.scriptkit/kit/config.ts for changes
        // Only spawn if watcher started successfully
        // Uses adaptive polling: starts at 200ms, increases to 2s when idle
        if config_watcher_ok {
            let app_entity_for_config = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                let mut idle_count = 0u32;
                loop {
                    // Adaptive polling: 200ms when active, up to 2000ms when idle
                    // After 5 idle checks (1s), increase to 500ms
                    // After 10 idle checks (3.5s), increase to 2000ms
                    let poll_interval = if idle_count < 5 {
                        200
                    } else if idle_count < 10 {
                        500
                    } else {
                        2000
                    };
                    Timer::after(std::time::Duration::from_millis(poll_interval)).await;

                    if config_rx.try_recv().is_ok() {
                        idle_count = 0; // Reset on activity
                        logging::log("APP", "Config file changed, reloading");
                        let _ = cx.update(|cx| {
                            app_entity_for_config.update(cx, |view, ctx| {
                                view.update_config(ctx);
                            });
                        });
                    } else {
                        idle_count = idle_count.saturating_add(1);
                    }
                }
            }).detach();
        }

        // Script/scriptlets reload watcher - watches ~/.scriptkit/*/scripts/ and ~/.scriptkit/*/scriptlets/
        // Uses incremental updates for scriptlet files, full reload for scripts
        // Also re-scans for scheduled scripts to pick up new/modified schedules
        // Only spawn if watcher started successfully
        // Uses adaptive polling: starts at 200ms, increases to 2s when idle
        if script_watcher_ok {
            let app_entity_for_scripts = app_entity.clone();
            let scheduler_for_scripts = scheduler.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                use watcher::ScriptReloadEvent;
                let mut idle_count = 0u32;

                loop {
                    // Adaptive polling: 200ms when active, up to 2000ms when idle
                    let poll_interval = if idle_count < 5 {
                        200
                    } else if idle_count < 10 {
                        500
                    } else {
                        2000
                    };
                    Timer::after(std::time::Duration::from_millis(poll_interval)).await;

                    // Drain all pending events
                    let mut had_events = false;
                    while let Ok(event) = script_rx.try_recv() {
                        had_events = true;
                        match event {
                            ScriptReloadEvent::FileChanged(path) | ScriptReloadEvent::FileCreated(path) => {
                                // Check if it's a scriptlet file (markdown in scriptlets directory)
                                let is_scriptlet = path.extension().map(|e| e == "md").unwrap_or(false);

                                if is_scriptlet {
                                    logging::log("APP", &format!("Scriptlet file changed: {}", path.display()));
                                    let path_clone = path.clone();
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.handle_scriptlet_file_change(&path_clone, false, ctx);
                                        });
                                    });
                                } else {
                                    logging::log("APP", &format!("Script file changed: {}", path.display()));
                                    // Re-scan for scheduled scripts when script files change
                                    if let Ok(scheduler_guard) = scheduler_for_scripts.lock() {
                                        let new_count = scripts::register_scheduled_scripts(&scheduler_guard);
                                        if new_count > 0 {
                                            logging::log("APP", &format!("Re-registered {} scheduled scripts after file change", new_count));
                                        }
                                    }
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.refresh_scripts(ctx);
                                        });
                                    });
                                }
                            }
                            ScriptReloadEvent::FileDeleted(path) => {
                                let is_scriptlet = path.extension().map(|e| e == "md").unwrap_or(false);

                                if is_scriptlet {
                                    logging::log("APP", &format!("Scriptlet file deleted: {}", path.display()));
                                    let path_clone = path.clone();
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.handle_scriptlet_file_change(&path_clone, true, ctx);
                                        });
                                    });
                                } else {
                                    logging::log("APP", &format!("Script file deleted: {}", path.display()));
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.refresh_scripts(ctx);
                                        });
                                    });
                                }
                            }
                            ScriptReloadEvent::FullReload => {
                                logging::log("APP", "Full script/scriptlet reload requested");
                                // Re-scan for scheduled scripts
                                if let Ok(scheduler_guard) = scheduler_for_scripts.lock() {
                                    let new_count = scripts::register_scheduled_scripts(&scheduler_guard);
                                    if new_count > 0 {
                                        logging::log("APP", &format!("Re-registered {} scheduled scripts after full reload", new_count));
                                    }
                                }
                                let _ = cx.update(|cx| {
                                    app_entity_for_scripts.update(cx, |view, ctx| {
                                        view.refresh_scripts(ctx);
                                    });
                                });
                            }
                        }
                    }

                    // Update idle count for adaptive polling
                    if had_events {
                        idle_count = 0;
                    } else {
                        idle_count = idle_count.saturating_add(1);
                    }
                }
            }).detach();
        }

        // App watcher poll loop - watches /Applications and ~/Applications for changes
        // When apps are added/removed/updated, refresh the app launcher cache
        // Only spawn if watcher started successfully
        if app_watcher_ok {
            let app_entity_for_apps = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                use watcher::AppReloadEvent;

                logging::log("APP", "App watcher poll loop started (event-driven)");

                // Event-driven: blocks until app change event received
                while let Ok(event) = app_rx.recv().await {
                    logging::log("APP", &format!("App watcher event: {:?}", event));

                    match event {
                        AppReloadEvent::AppAdded(path) => {
                            logging::log("APP", &format!("App added: {}", path.display()));
                        }
                        AppReloadEvent::AppRemoved(path) => {
                            logging::log("APP", &format!("App removed: {}", path.display()));
                        }
                        AppReloadEvent::AppUpdated(path) => {
                            logging::log("APP", &format!("App updated: {}", path.display()));
                        }
                        AppReloadEvent::FullReload => {
                            logging::log("APP", "Full app reload requested");
                        }
                    }

                    // Trigger cache refresh (scan_applications updates the in-memory cache)
                    let _ = app_launcher::scan_applications();

                    // Notify UI to re-fetch cached apps and invalidate search caches
                    // This ensures new apps appear in search results immediately
                    let _ = cx.update(|cx| {
                        app_entity_for_apps.update(cx, |view, ctx| {
                            view.refresh_apps(ctx);
                        });
                    });
                }

                logging::log("APP", "App watcher poll loop exiting (channel closed)");
            }).detach();
        }

        // NOTE: Prompt message listener is now spawned per-script in execute_interactive()
        // using event-driven async_channel instead of 50ms polling

        // Scheduler event handler - runs scripts when their cron schedule triggers
        // Uses std::sync::mpsc::Receiver which requires a polling approach
        let _window_for_scheduler = window;
        std::thread::spawn(move || {
            logging::log("APP", "Scheduler event handler started");

            loop {
                // Check shutdown flag - exit loop if shutting down
                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                    logging::log("SCHEDULER", "Shutdown requested, exiting scheduler event handler");
                    break;
                }

                // Use recv_timeout to periodically check for events without blocking forever
                match scheduler_rx.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(event) => {
                        match event {
                            scheduler::SchedulerEvent::RunScript(path) => {
                                // Check shutdown flag before spawning new scripts
                                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                                    logging::log("SCHEDULER", &format!("Skipping scheduled script (shutdown in progress): {}", path.display()));
                                    continue;
                                }

                                logging::log("SCHEDULER", &format!("Executing scheduled script: {}", path.display()));

                                // Execute the script using the existing executor infrastructure
                                // This spawns it in the background without blocking the scheduler
                                let path_str = path.to_string_lossy().to_string();

                                // Use bun to run the script directly (non-interactive for scheduled scripts)
                                // Find bun path (same logic as executor)
                                let bun_path = std::env::var("BUN_PATH")
                                    .ok()
                                    .or_else(|| {
                                        // Check common locations
                                        for candidate in &[
                                            "/opt/homebrew/bin/bun",
                                            "/usr/local/bin/bun",
                                            std::env::var("HOME").ok().map(|h| format!("{}/.bun/bin/bun", h)).unwrap_or_default().as_str(),
                                        ] {
                                            if std::path::Path::new(candidate).exists() {
                                                return Some(candidate.to_string());
                                            }
                                        }
                                        None
                                    })
                                    .unwrap_or_else(|| "bun".to_string());

                                // Spawn bun process to run the script
                                match std::process::Command::new(&bun_path)
                                    .arg("run")
                                    .arg("--preload")
                                    .arg(format!("{}/.scriptkit/sdk/kit-sdk.ts", std::env::var("HOME").unwrap_or_default()))
                                    .arg(&path_str)
                                    .stdout(std::process::Stdio::piped())
                                    .stderr(std::process::Stdio::piped())
                                    .spawn()
                                {
                                    Ok(child) => {
                                        let pid = child.id();
                                        // Track the process
                                        PROCESS_MANAGER.register_process(pid, &path_str);
                                        logging::log("SCHEDULER", &format!("Spawned scheduled script PID {}: {}", pid, path_str));

                                        // Wait for completion in a separate thread to not block scheduler
                                        let path_for_log = path_str.clone();
                                        std::thread::spawn(move || {
                                            match child.wait_with_output() {
                                                Ok(output) => {
                                                    // Unregister the process now that it's done
                                                    PROCESS_MANAGER.unregister_process(pid);

                                                    if output.status.success() {
                                                        logging::log("SCHEDULER", &format!("Scheduled script completed: {}", path_for_log));
                                                    } else {
                                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                                        logging::log("SCHEDULER", &format!("Scheduled script failed: {} - {}", path_for_log, stderr));
                                                    }
                                                }
                                                Err(e) => {
                                                    // Unregister on error too
                                                    PROCESS_MANAGER.unregister_process(pid);
                                                    logging::log("SCHEDULER", &format!("Scheduled script error: {} - {}", path_for_log, e));
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        logging::log("SCHEDULER", &format!("Failed to spawn scheduled script: {} - {}", path_str, e));
                                    }
                                }
                            }
                            scheduler::SchedulerEvent::Error(msg) => {
                                logging::log("SCHEDULER", &format!("Scheduler error: {}", msg));
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Normal timeout, continue loop
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        logging::log("APP", "Scheduler event channel disconnected, exiting handler");
                        break;
                    }
                }
            }
        });

        // Test command file watcher - allows smoke tests to trigger script execution
        // SECURITY: This feature is ONLY enabled in debug builds to prevent local privilege escalation.
        // In release builds, any process that can write to /tmp could trigger script execution.
        #[cfg(debug_assertions)]
        {
            let app_entity_for_test = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                logging::log("TEST", "Debug command file watcher enabled (debug build only)");
                let cmd_file = std::path::PathBuf::from("/tmp/script-kit-gpui-cmd.txt");
                loop {
                    Timer::after(std::time::Duration::from_millis(500)).await;

                    if cmd_file.exists() {
                        if let Ok(content) = std::fs::read_to_string(&cmd_file) {
                            let _ = std::fs::remove_file(&cmd_file); // Remove immediately to prevent re-processing

                            for line in content.lines() {
                                if line.starts_with("run:") {
                                    let script_name = line.strip_prefix("run:").unwrap_or("").trim();
                                    logging::log("TEST", &format!("Test command: run script '{}'", script_name));

                                    let script_name_owned = script_name.to_string();
                                    let app_entity_inner = app_entity_for_test.clone();
                                    let _ = cx.update(|cx| {
                                        app_entity_inner.update(cx, |view, ctx| {
                                            // Find and run the script interactively
                                            if let Some(script) = view.scripts.iter().find(|s| s.name == script_name_owned || s.path.to_string_lossy().contains(&script_name_owned)).cloned() {
                                                logging::log("TEST", &format!("Found script: {}", script.name));
                                                view.execute_interactive(&script, ctx);
                                            } else {
                                                logging::log("TEST", &format!("Script not found: {}", script_name_owned));
                                            }
                                        });
                                    });
                                }
                            }
                        }
                    }
                }
            }).detach();
        }
