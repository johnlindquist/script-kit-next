    logging::init();

    // Migrate from legacy ~/.kenv to new ~/.scriptkit structure (one-time migration)
    // This must happen BEFORE ensure_kit_setup() so the new path is used
    if setup::migrate_from_kenv() {
        logging::log("APP", "Migrated from ~/.kenv to ~/.scriptkit");
    }

    // Ensure ~/.scriptkit environment is properly set up (directories, SDK, config, etc.)
    // This is idempotent - it creates missing directories and files without overwriting user configs
    let setup_result = setup::ensure_kit_setup();
    if setup_result.is_fresh_install {
        logging::log(
            "APP",
            &format!(
                "Fresh install detected - created ~/.scriptkit at {}",
                setup_result.kit_path.display()
            ),
        );
    }
    for warning in &setup_result.warnings {
        logging::log("APP", &format!("Setup warning: {}", warning));
    }
    if !setup_result.bun_available {
        logging::log(
            "APP",
            "Warning: bun not found in PATH or common locations. Scripts may not run.",
        );
    }

    // Write main PID file for orphan detection on crash
    if let Err(e) = PROCESS_MANAGER.write_main_pid() {
        logging::log("APP", &format!("Failed to write main PID file: {}", e));
    } else {
        logging::log("APP", "Main PID file written");
    }

    // Clean up any orphaned processes from a previous crash
    let orphans_killed = PROCESS_MANAGER.cleanup_orphans();
    if orphans_killed > 0 {
        logging::log(
            "APP",
            &format!(
                "Cleaned up {} orphaned process(es) from previous session",
                orphans_killed
            ),
        );
    }

    // Register signal handlers for graceful shutdown
    // SAFETY: Signal handlers can only safely call async-signal-safe functions.
    // We ONLY set an atomic flag here. All cleanup (logging, killing processes,
    // removing PID files) happens in a GPUI task that monitors this flag.
    #[cfg(unix)]
    {
        extern "C" fn handle_signal(_sig: libc::c_int) {
            // ASYNC-SIGNAL-SAFE: Only set atomic flag
            // Do NOT call: logging, mutexes, heap allocation, or any Rust code
            // that might allocate or lock. The GPUI shutdown monitor task will
            // handle all cleanup on the main thread.
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        }

        unsafe {
            // Register handlers for common termination signals
            libc::signal(
                libc::SIGINT,
                handle_signal as *const () as libc::sighandler_t,
            );
            libc::signal(
                libc::SIGTERM,
                handle_signal as *const () as libc::sighandler_t,
            );
            libc::signal(
                libc::SIGHUP,
                handle_signal as *const () as libc::sighandler_t,
            );
            logging::log(
                "APP",
                "Signal handlers registered (SIGINT, SIGTERM, SIGHUP) - cleanup via GPUI task",
            );
        }
    }

    // Load config early so we can use it for hotkey registration AND clipboard history settings
    // This avoids duplicate config::load_config() calls (~100-300ms startup savings)
    let loaded_config = config::load_config();
    logging::log(
        "APP",
        &format!(
            "Loaded config: hotkey={:?}+{}, bun_path={:?}",
            loaded_config.hotkey.modifiers, loaded_config.hotkey.key, loaded_config.bun_path
        ),
    );
    clipboard_history::set_max_text_content_len(
        loaded_config.get_clipboard_history_max_text_length(),
    );

    // Initialize clipboard history monitoring (background thread)
    if let Err(e) = clipboard_history::init_clipboard_history() {
        logging::log(
            "APP",
            &format!("Failed to initialize clipboard history: {}", e),
        );
    } else {
        logging::log("APP", "Clipboard history monitoring initialized");
    }

    // Initialize text expansion system (background thread with keyboard monitoring)
    // This must be done early, before the GPUI run loop starts
    // Uses a global singleton so the manager can be updated when scriptlet files change
    #[cfg(target_os = "macos")]
    {
        // Spawn initialization in a thread to not block startup
        std::thread::spawn(move || {
            logging::log("KEYWORD", "Initializing text expansion system");

            match keyword_manager::init_keyword_manager() {
                Ok(Some(count)) => {
                    logging::log(
                        "KEYWORD",
                        &format!("Text expansion system enabled with {} triggers", count),
                    );
                }
                Ok(None) => {
                    logging::log(
                        "KEYWORD",
                        "Accessibility permissions not granted - text expansion disabled",
                    );
                    logging::log(
                        "KEYWORD",
                        "Enable in System Preferences > Privacy & Security > Accessibility",
                    );
                }
                Err(e) => {
                    logging::log(
                        "KEYWORD",
                        &format!("Failed to initialize text expansion: {}", e),
                    );
                }
            }
        });
    }

    // Clone before start_hotkey_listener consumes original
    let config_for_app = loaded_config.clone();

    // Start MCP server for AI agent integration
    // Server runs on localhost:43210 with Bearer token authentication
    // Discovery file written to ~/.scriptkit/server.json
    let _mcp_handle = match mcp_server::McpServer::with_defaults() {
        Ok(server) => match server.start() {
            Ok(handle) => {
                logging::log(
                    "MCP",
                    &format!(
                        "MCP server started on {} (token in ~/.scriptkit/agent-token)",
                        server.url()
                    ),
                );
                Some(handle)
            }
            Err(e) => {
                logging::log("MCP", &format!("Failed to start MCP server: {}", e));
                None
            }
        },
        Err(e) => {
            logging::log("MCP", &format!("Failed to create MCP server: {}", e));
            None
        }
    };

    hotkeys::start_hotkey_listener(loaded_config);

    // Start watchers and track which ones succeeded
    // We only spawn poll loops for watchers that successfully started
    // Note: Appearance watching is handled by GPUI's observe_window_appearance (set up on the window)
    let (mut config_watcher, config_rx) = watcher::ConfigWatcher::new();
    let config_watcher_ok = match config_watcher.start() {
        Ok(()) => {
            logging::log("APP", "Config watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start config watcher: {}", e));
            false
        }
    };

    let (mut script_watcher, script_rx) = watcher::ScriptWatcher::new();
    let script_watcher_ok = match script_watcher.start() {
        Ok(()) => {
            logging::log("APP", "Script watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start script watcher: {}", e));
            false
        }
    };

    // Create AppWatcher for live application cache updates
    let (mut app_watcher, app_rx) = watcher::AppWatcher::new();
    let app_watcher_ok = match app_watcher.start() {
        Ok(()) => {
            logging::log("APP", "App watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start app watcher: {}", e));
            false
        }
    };

    // Initialize script scheduler
    // Creates the scheduler and scans for scripts with // Cron: or // Schedule: metadata
    let (mut scheduler, scheduler_rx) = scheduler::Scheduler::new();
    let scheduled_count = scripts::register_scheduled_scripts(&scheduler);
    logging::log(
        "APP",
        &format!("Registered {} scheduled scripts", scheduled_count),
    );

    // Start the scheduler background thread (checks every 30 seconds for due scripts)
    if scheduled_count > 0 {
        if let Err(e) = scheduler.start() {
            logging::log("APP", &format!("Failed to start scheduler: {}", e));
        } else {
            logging::log("APP", "Scheduler started successfully");
        }
    } else {
        logging::log("APP", "No scheduled scripts found, scheduler not started");
    }

    // Wrap scheduler in Arc<Mutex<>> for thread-safe access (needed for re-scanning on file changes)
    let scheduler = Arc::new(Mutex::new(scheduler));

    // Register URL scheme handler for scriptkit:// deeplinks
    // This must be done before .run() as it's called on Application
    let app = Application::new();
    app.on_open_urls(|urls| {
        logging::log("DEEPLINK", &format!("Received {} URL(s)", urls.len()));
        for url in urls {
            logging::log("DEEPLINK", &format!("Processing URL: {}", url));
            if let Some(command_id) = parse_deeplink_url(&url) {
                logging::log("DEEPLINK", &format!("Parsed command_id: {}", command_id));
                // Send to channel for processing inside the app
                if deeplink_channel().0.try_send(command_id).is_err() {
                    logging::log(
                        "DEEPLINK",
                        "Failed to send command to channel (full or closed)",
                    );
                }
            }
        }
    });
