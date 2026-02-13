{
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
app.run(move |cx: &mut App| {
        logging::log("APP", "GPUI Application starting");

        // Warm up the secrets cache in background thread
        // This pre-decrypts secrets.age so AI chat opens instantly instead of
        // waiting ~7s for sequential keyring lookups
        secrets::warmup_cache();

        // Configure as accessory app FIRST, before any windows are created
        // This is equivalent to LSUIElement=true in Info.plist:
        // - No Dock icon
        // - No menu bar ownership (critical for window actions to work)
        platform::configure_as_accessory_app();

        // Start frontmost app tracker - watches for app activations and pre-fetches menu bar items
        // Must be started after configure_as_accessory_app() so we're correctly classified
        #[cfg(target_os = "macos")]
        frontmost_app_tracker::start_tracking();

        // Register bundled JetBrains Mono font
        // This makes "JetBrains Mono" available as a font family for the editor
        register_bundled_fonts(cx);

        // Initialize gpui-component (theme, context providers)
        // Must be called before opening windows that use Root wrapper
        gpui_component::init(cx);

        // Initialize confirm dialog key bindings (Escape, Enter, Space)
        confirm::init_confirm_bindings(cx);

        // Initialize the theme cache FIRST (before any render calls)
        // This ensures get_cached_theme() returns correct data from first render
        theme::init_theme_cache();

        // Sync Script Kit theme with gpui-component's ThemeColor system
        // This ensures all gpui-component widgets use our colors
        theme::sync_gpui_component_theme(cx);

        // Start the centralized theme service for hot-reload
        // This replaces per-window theme watchers and ensures all windows
        // stay in sync with theme.json changes
        theme::service::ensure_theme_service(cx);

        // Calculate window bounds: try saved position first, then eye-line
        let window_size = size(px(750.), initial_window_height());
        let default_bounds = calculate_eye_line_bounds_on_mouse_display(window_size);
        let displays = platform::get_macos_displays();
        let bounds = window_state::get_initial_bounds(
            window_state::WindowRole::Main,
            default_bounds,
            &displays,
        );

        // Load theme to determine window background appearance (vibrancy)
        let initial_theme = theme::load_theme();
        let window_background = if initial_theme.is_vibrancy_enabled() {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };
        logging::log(
            "THEME",
            &format!(
                "Window background appearance: {:?} (vibrancy_enabled={})",
                window_background,
                initial_theme.is_vibrancy_enabled()
            ),
        );

        // Store the ScriptListApp entity for direct access (needed since Root wraps the view)
        let app_entity_holder: Arc<Mutex<Option<Entity<ScriptListApp>>>> = Arc::new(Mutex::new(None));
        let app_entity_for_closure = app_entity_holder.clone();

        // Capture bun_available for use in window creation
        let bun_available = setup_result.bun_available;
        let config_for_tray_actions = config_for_app.clone();

        // Root is required for gpui_component's InputState focus tracking
        let window: WindowHandle<Root> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                is_movable: true,
                window_background,
                show: false, // Start hidden - only show on hotkey press
                focus: false, // Don't focus on creation
                // CRITICAL: Use PopUp for Raycast-like behavior
                // Creates NSPanel with NonactivatingPanel style, allowing keyboard
                // input without activating the application (preserves previous app focus)
                kind: WindowKind::PopUp,
                ..Default::default()
            },
            |window, cx| {
                logging::log("APP", "Window opened, creating ScriptListApp wrapped in Root");
                let view = cx.new(|cx| ScriptListApp::new(config_for_app, bun_available, window, cx));
                // Store the entity for external access
                *app_entity_for_closure.lock().unwrap_or_else(|e| e.into_inner()) = Some(view.clone());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();

        // Extract the app entity for use in callbacks
        let app_entity = app_entity_holder.lock().unwrap_or_else(|e| e.into_inner()).clone().expect("App entity should be set");

        // Set initial focus via the Root window
        // We access the app entity within the window context to properly focus it
        let app_entity_for_focus = app_entity.clone();
        window
            .update(cx, |_root, win, root_cx| {
                app_entity_for_focus.update(root_cx, |view, ctx| {
                    let focus_handle = view.focus_handle(ctx);
                    win.focus(&focus_handle, ctx);
                    logging::log("APP", "Focus set on ScriptListApp via Root");

                    // Subscribe to window bounds changes to save position when user drags the window.
                    // This persists the position per-display so it can be restored on next show.
                    // Store subscription in view to keep it alive.
                    view.bounds_subscription = Some(ctx.observe_window_bounds(win, |_view, win, _ctx| {
                        // Only save if window is visible (avoid saving during initial positioning)
                        if is_main_window_visible() {
                            if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                let displays = platform::get_macos_displays();
                                let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                    logging::log(
                                        "WINDOW_BOUNDS",
                                        &format!(
                                            "Bounds changed - saving position for display {}: ({:.0}, {:.0})",
                                            window_state::display_key(display),
                                            x,
                                            y
                                        ),
                                    );
                                    window_state::save_main_position_for_display(display, bounds);
                                }
                            }
                        }
                        // Suppress unused variable warning - we need win to access window bounds
                        let _ = win;
                    }));

                    // Observe window appearance changes (GPUI fires this when macOS changes light/dark mode)
                    view.appearance_subscription = Some(ctx.observe_window_appearance(win, |view, _win, ctx| {
                        logging::log("APP", "System appearance changed, reloading theme");

                        // Invalidate the cached appearance detection so
                        // detect_system_appearance() gets the fresh value
                        theme::invalidate_appearance_cache();

                        // Reload the theme cache so get_cached_theme() returns
                        // fresh colors (used by vibrancy backgrounds, etc.)
                        let theme = theme::reload_theme_cache();
                        let is_dark = theme.should_use_dark_vibrancy();

                        // Reconfigure vibrancy materials on NSVisualEffectViews
                        platform::configure_window_vibrancy_material_for_appearance(is_dark);

                        // Update all secondary windows (Notes, AI, Actions)
                        platform::update_all_secondary_windows_appearance(is_dark);

                        // Sync gpui-component theme with new system appearance
                        theme::sync_gpui_component_theme(ctx);

                        // Update the app entity theme
                        view.update_theme(ctx);

                        // Notify all registered windows to re-render with new colors
                        windows::notify_all_windows(ctx);
                    }));
                });
            })
            .unwrap();

        // Register the main window with WindowManager before tray init
        // This must happen after GPUI creates the window but before tray creates its windows
        // so we can reliably find our main window by its expected size (~750x500)
        window_manager::find_and_register_main_window();

        // HACK: Swizzle GPUI's BlurredView IMMEDIATELY after window creation
        // GPUI hides the native macOS CAChameleonLayer (vibrancy tint) on every frame.
        // By swizzling now (before any rendering), we preserve the native tint effect.
        // This gives us Raycast/Spotlight-like vibrancy appearance.
        platform::swizzle_gpui_blurred_view();

        // Window starts hidden - no activation, no panel configuration yet
        // Panel will be configured on first show via hotkey
        // WINDOW_VISIBLE is already false by default (static initializer)
        logging::log("HOTKEY", "Window created but not shown (use hotkey to show)");

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

                                if !scheduler::try_acquire_scheduled_run_slot() {
                                    logging::log(
                                        "SCHEDULER",
                                        &format!(
                                            "Skipping scheduled script (concurrency limit reached: active={}, limit={}): {}",
                                            scheduler::active_scheduled_run_count(),
                                            scheduler::SCHEDULED_SCRIPT_MAX_CONCURRENT_RUNS,
                                            path.display()
                                        ),
                                    );
                                    continue;
                                }

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
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::piped())
                                    .spawn()
                                {
                                    Ok(mut child) => {
                                        let pid = child.id();
                                        // Track the process
                                        PROCESS_MANAGER.register_process(pid, &path_str);
                                        logging::log("SCHEDULER", &format!("Spawned scheduled script PID {}: {}", pid, path_str));

                                        // Wait for completion in a separate thread to not block scheduler
                                        let path_for_log = path_str.clone();
                                        std::thread::spawn(move || {
                                            let mut captured_stderr = String::new();
                                            let mut capture_was_truncated = false;

                                            if let Some(stderr) = child.stderr.take() {
                                                match scheduler::read_limited_stderr(
                                                    stderr,
                                                    scheduler::SCHEDULED_SCRIPT_STDERR_CAPTURE_MAX_BYTES,
                                                ) {
                                                    Ok((stderr_output, was_truncated)) => {
                                                        captured_stderr = stderr_output;
                                                        capture_was_truncated = was_truncated;
                                                    }
                                                    Err(e) => {
                                                        logging::log(
                                                            "SCHEDULER",
                                                            &format!(
                                                                "Failed to read scheduled script stderr stream: {} (pid={}): {}",
                                                                path_for_log, pid, e
                                                            ),
                                                        );
                                                    }
                                                }
                                            }

                                            let wait_result = child.wait();

                                            // Unregister and release slot now that execution has finished
                                            PROCESS_MANAGER.unregister_process(pid);
                                            scheduler::release_scheduled_run_slot();

                                            match wait_result {
                                                Ok(status) => {
                                                    if status.success() {
                                                        logging::log(
                                                            "SCHEDULER",
                                                            &format!(
                                                                "Scheduled script completed: {} (pid={})",
                                                                path_for_log, pid
                                                            ),
                                                        );
                                                    } else {
                                                        let (mut stderr_for_log, log_was_truncated) =
                                                            scheduler::truncate_scheduler_stderr_for_log(
                                                                &captured_stderr,
                                                                scheduler::SCHEDULED_SCRIPT_STDERR_LOG_MAX_BYTES,
                                                            );
                                                        if stderr_for_log.trim().is_empty() {
                                                            stderr_for_log = "<no stderr output>".to_string();
                                                        }
                                                        if capture_was_truncated {
                                                            stderr_for_log.push_str(
                                                                " [stderr capture truncated at 64KB]",
                                                            );
                                                        }
                                                        if log_was_truncated {
                                                            stderr_for_log.push_str(
                                                                " [stderr log truncated at 4KB]",
                                                            );
                                                        }
                                                        logging::log(
                                                            "SCHEDULER",
                                                            &format!(
                                                                "Scheduled script failed: {} (pid={}, status={}): {}",
                                                                path_for_log, pid, status, stderr_for_log
                                                            ),
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    logging::log(
                                                        "SCHEDULER",
                                                        &format!(
                                                            "Scheduled script wait error: {} (pid={}): {}",
                                                            path_for_log, pid, e
                                                        ),
                                                    );
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        scheduler::release_scheduled_run_slot();
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

// External command listener - receives commands via stdin (event-driven, no polling)
let stdin_rx = start_stdin_listener();
let window_for_stdin = window;
let app_entity_for_stdin = app_entity.clone();

// Track if we've received any stdin commands (for timeout warning)
static STDIN_RECEIVED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// Spawn a timeout warning task - helps AI agents detect when they forgot to use stdin protocol
cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
    Timer::after(std::time::Duration::from_secs(2)).await;
    if !STDIN_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
        logging::log("STDIN", "");
        logging::log(
            "STDIN",
            "╔════════════════════════════════════════════════════════════════════════════╗",
        );
        logging::log(
            "STDIN",
            "║  WARNING: No stdin JSON received after 2 seconds                          ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  If you're testing, use the stdin JSON protocol:                          ║",
        );
        logging::log(
            "STDIN",
            "║  echo '{\"type\":\"run\",\"path\":\"...\"}' | ./target/debug/script-kit-gpui     ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  Command line args do NOT work:                                           ║",
        );
        logging::log(
            "STDIN",
            "║  ./target/debug/script-kit-gpui test.ts  # WRONG - does nothing!          ║",
        );
        logging::log(
            "STDIN",
            "╚════════════════════════════════════════════════════════════════════════════╝",
        );
        logging::log("STDIN", "");
    }
})
.detach();

cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    logging::log("STDIN", "Async stdin command handler started");

    // Event-driven: recv().await yields until a command arrives
    while let Ok(ExternalCommandEnvelope {
        command: cmd,
        correlation_id,
    }) = stdin_rx.recv().await
    {
        let _guard = logging::set_correlation_id(correlation_id);
        // Mark that we've received stdin (clears the timeout warning)
        STDIN_RECEIVED.store(true, std::sync::atomic::Ordering::SeqCst);
        logging::log(
            "STDIN",
            &format!("Processing external command type={}", cmd.command_type()),
        );

        let app_entity_inner = app_entity_for_stdin.clone();
        let _ = cx.update(|cx| {
            // Use the Root window to get Window reference, then update the app entity
            let _ = window_for_stdin.update(cx, |_root, window, root_cx| {
                app_entity_inner.update(root_cx, |view, ctx| {
                    // Note: We have both `window` from Root and `view` from entity here
                    // ctx is Context<ScriptListApp>, window is &mut Window
                    match cmd {
                            ExternalCommand::Run { ref path, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Executing script: {}", rid, path));

                                // NOTE: This is a simplified show path for script execution.
                                // We show the window, then immediately run the script.
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown(); // Focus grace period
                                platform::ensure_move_to_active_space();

                                // Use Window::defer via window_ops to coalesce and defer window move.
                                // This avoids RefCell borrow conflicts from synchronous macOS window operations.
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                window_ops::queue_move(bounds, window, ctx);

                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    platform::swizzle_gpui_blurred_view();
                                    // Configure vibrancy based on actual theme colors
                                    let theme = theme::load_theme();
                                    let is_dark = theme.should_use_dark_vibrancy();
                                    platform::configure_window_vibrancy_material_for_appearance(is_dark);
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                // Show window WITHOUT activating (floating panel behavior)
                                platform::show_main_window_without_activation();
                                window.activate_window();

                                // Send AI window to back so it doesn't come forward with main menu
                                platform::send_ai_window_to_back();

                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);

                                // Send RunScript message to be handled
                                view.handle_prompt_message(PromptMessage::RunScript { path: path.clone() }, ctx);
                            }
                            ExternalCommand::Show { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Showing window", rid));

                                // NOTE: This is a simplified show path for explicit stdin commands.
                                // Unlike the hotkey handler, we don't need NEEDS_RESET handling
                                // because this is an explicit show (not a toggle).
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown(); // Focus grace period
                                platform::ensure_move_to_active_space();

                                // Position window - try per-display saved position first, then fall back to eye-line
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let displays = platform::get_macos_displays();
                                let bounds = if let Some((mouse_x, mouse_y)) = platform::get_global_mouse_position() {
                                    // Try to restore saved position for the mouse display
                                    if let Some((saved, display)) =
                                        window_state::get_main_position_for_mouse_display(mouse_x, mouse_y, &displays)
                                    {
                                        // Validate the saved position is still visible
                                        if window_state::is_bounds_visible(&saved, &displays) {
                                            logging::log(
                                                "STDIN",
                                                &format!(
                                                    "Restoring saved position for display {}: ({:.0}, {:.0})",
                                                    window_state::display_key(&display),
                                                    saved.x,
                                                    saved.y
                                                ),
                                            );
                                            // Use saved position but with current window height (may have changed)
                                            gpui::Bounds {
                                                origin: gpui::point(px(saved.x as f32), px(saved.y as f32)),
                                                size: window_size,
                                            }
                                        } else {
                                            logging::log("STDIN", "Saved position no longer visible, using eye-line");
                                            platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                        }
                                    } else {
                                        logging::log("STDIN", "No saved position for this display, using eye-line");
                                        platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                    }
                                } else {
                                    logging::log("STDIN", "Could not get mouse position, using eye-line");
                                    platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                };
                                window_ops::queue_move(bounds, window, ctx);

                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    platform::swizzle_gpui_blurred_view();
                                    // Configure vibrancy based on actual theme colors
                                    let theme = theme::load_theme();
                                    let is_dark = theme.should_use_dark_vibrancy();
                                    platform::configure_window_vibrancy_material_for_appearance(is_dark);
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                // Show window WITHOUT activating (floating panel behavior)
                                platform::show_main_window_without_activation();
                                window.activate_window();

                                // Send AI window to back so it doesn't come forward with main menu
                                platform::send_ai_window_to_back();

                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);
                            }
                            ExternalCommand::Hide { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Hiding main window", rid));

                                // Save window position for the current display BEFORE hiding
                                if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                    let displays = platform::get_macos_displays();
                                    let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                    if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                        logging::log(
                                            "STDIN",
                                            &format!(
                                                "Saving position for display {}: ({:.0}, {:.0})",
                                                window_state::display_key(display),
                                                x,
                                                y
                                            ),
                                        );
                                        window_state::save_main_position_for_display(display, bounds);
                                    }
                                }

                                script_kit_gpui::set_main_window_visible(false);

                                // Check if Notes or AI windows are open
                                let notes_open = notes::is_notes_window_open();
                                let ai_open = ai::is_ai_window_open();

                                // CRITICAL: Only hide main window if Notes/AI are open
                                // ctx.hide() hides the ENTIRE app (all windows)
                                if notes_open || ai_open {
                                    logging::log("STDIN", "Using hide_main_window() - secondary windows are open");
                                    platform::hide_main_window();
                                } else {
                                    ctx.hide();
                                }
                            }
                            ExternalCommand::SetFilter { ref text, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Setting filter to: '{}'", rid, text));
                                view.set_filter_text_immediate(text.clone(), window, ctx);
                                let _ = view.get_filtered_results_cached(); // Update cache
                            }
                            ExternalCommand::TriggerBuiltin { ref name } => {
                                logging::log("STDIN", &format!("Triggering built-in: '{}'", name));
                                // Opened via protocol command - ESC should close window (not return to main menu)
                                view.opened_from_main_menu = false;
                                // Match built-in name and trigger the corresponding feature
                                match name.to_lowercase().as_str() {
                                    "design-gallery" | "designgallery" | "design gallery" => {
                                        view.current_view = AppView::DesignGalleryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    // P0 FIX: Store data in self, view holds only state
                                    "clipboard" | "clipboard-history" | "clipboardhistory" => {
                                        view.cached_clipboard_entries =
                                            clipboard_history::get_cached_entries(100);
                                        view.focused_clipboard_entry_id = view
                                            .cached_clipboard_entries
                                            .first()
                                            .map(|entry| entry.id.clone());
                                        view.current_view = AppView::ClipboardHistoryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    // P0 FIX: Use existing self.apps, view holds only state
                                    "apps" | "app-launcher" | "applauncher" => {
                                        view.current_view = AppView::AppLauncherView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    "file-search" | "filesearch" | "files" | "searchfiles" => {
                                        view.open_file_search(String::new(), ctx);
                                    }
                                    "emoji" | "emoji-picker" | "emojipicker" => {
                                        view.filter_text = String::new();
                                        view.pending_filter_sync = true;
                                        view.pending_placeholder = Some("Search Emoji & Symbols...".to_string());
                                        view.current_view = AppView::EmojiPickerView {
                                            filter: String::new(),
                                            selected_index: 0,
                                            selected_category: None,
                                        };
                                        view.hovered_index = None;
                                        view.pending_focus = Some(FocusTarget::MainFilter);
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    _ => {
                                        logging::log("ERROR", &format!("Unknown built-in: '{}'", name));
                                    }
                                }
                            }

                            ExternalCommand::SimulateKey { ref key, ref modifiers } => {
                                logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

                                // Parse modifiers
                                let has_cmd = modifiers.contains(&KeyModifier::Cmd);
                                let has_shift = modifiers.contains(&KeyModifier::Shift);
                                let _has_alt = modifiers.contains(&KeyModifier::Alt);
                                let _has_ctrl = modifiers.contains(&KeyModifier::Ctrl);

                                // Handle key based on current view
                                let key_lower = key.to_lowercase();

                                match &view.current_view {
                                    AppView::ScriptList => {
                                        // Main script list key handling
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle actions");
                                            view.toggle_actions(ctx, window);
                                        } else if view.fallback_mode && !view.cached_fallbacks.is_empty() {
                                            // Handle keys in fallback mode
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    // Tab with filter text opens inline AI chat (even in fallback mode)
                                                    if !view.filter_text.is_empty() && !view.show_actions_popup {
                                                        let query = view.filter_text.clone();
                                                        view.filter_text.clear();
                                                        view.show_inline_ai_chat(Some(query), ctx);
                                                    }
                                                }
                                                "up" | "arrowup" => {
                                                    if view.fallback_selected_index > 0 {
                                                        view.fallback_selected_index -= 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    if view.fallback_selected_index < view.cached_fallbacks.len().saturating_sub(1) {
                                                        view.fallback_selected_index += 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute fallback");
                                                    view.execute_selected_fallback(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter (exit fallback mode)");
                                                    view.clear_filter(window, ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in fallback mode", key_lower));
                                                }
                                            }
                                        } else {
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    // Tab with filter text opens inline AI chat
                                                    if !view.filter_text.is_empty() && !view.show_actions_popup {
                                                        let query = view.filter_text.clone();
                                                        view.filter_text.clear();
                                                        view.show_inline_ai_chat(Some(query), ctx);
                                                    }
                                                }
                                                "up" | "arrowup" => {
                                                    // Use move_selection_up to properly skip section headers
                                                    view.move_selection_up(ctx);
                                                }
                                                "down" | "arrowdown" => {
                                                    // Use move_selection_down to properly skip section headers
                                                    view.move_selection_down(ctx);
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute selected");
                                                    view.execute_selected(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter or hide");
                                                    if !view.filter_text.is_empty() {
                                                        view.clear_filter(window, ctx);
                                                    } else {
                                                        // Save window position for the current display BEFORE hiding
                                                        if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                                            let displays = platform::get_macos_displays();
                                                            let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                                            if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                                                window_state::save_main_position_for_display(display, bounds);
                                                            }
                                                        }
                                                        script_kit_gpui::set_main_window_visible(false);
                                                        ctx.hide();
                                                    }
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ScriptList", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::PathPrompt { entity, .. } => {
                                        // Path prompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to PathPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        entity_clone.update(ctx, |path_prompt: &mut PathPrompt, path_cx| {
                                            if has_cmd && key_lower == "k" {
                                                path_prompt.toggle_actions(path_cx);
                                            } else {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => path_prompt.move_up(path_cx),
                                                    "down" | "arrowdown" => path_prompt.move_down(path_cx),
                                                    "enter" => path_prompt.handle_enter(path_cx),
                                                    "escape" => path_prompt.submit_cancel(),
                                                    "left" | "arrowleft" => path_prompt.navigate_to_parent(path_cx),
                                                    "right" | "arrowright" => path_prompt.navigate_into_selected(path_cx),
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in PathPrompt", key_lower));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    AppView::ArgPrompt { id, .. } => {
                                        // Arg prompt key handling via SimulateKey
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ArgPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        // Check for Cmd+K to toggle actions popup
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
                                            view.toggle_arg_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.show_actions_popup = false;
                                                                view.actions_dialog = None;
                                                                view.focused_input = FocusedInput::ArgPrompt;
                                                                window.focus(&view.focus_handle, ctx);
                                                            }
                                                            view.trigger_action_by_name(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
                                                        view.show_actions_popup = false;
                                                        view.actions_dialog = None;
                                                        view.focused_input = FocusedInput::ArgPrompt;
                                                        window.focus(&view.focus_handle, ctx);
                                                    }
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
                                                    }
                                                }
                                            }
                                        } else {
                                            // Normal arg prompt key handling
                                            let prompt_id = id.clone();
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    if view.arg_selected_index > 0 {
                                                        view.arg_selected_index -= 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    let filtered = view.filtered_arg_choices();
                                                    if view.arg_selected_index < filtered.len().saturating_sub(1) {
                                                        view.arg_selected_index += 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - submit selection");
                                                    let filtered = view.filtered_arg_choices();
                                                    if let Some((_, choice)) = filtered.get(view.arg_selected_index) {
                                                        let value = choice.value.clone();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    } else if !view.arg_input.is_empty() {
                                                        let value = view.arg_input.text().to_string();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    }
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - cancel script");
                                                    view.submit_prompt_response(prompt_id, None, ctx);
                                                    view.cancel_script_execution(ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::EditorPrompt { entity, id, .. } => {
                                        // Editor prompt key handling for template/snippet navigation and choice popup
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to EditorPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        let prompt_id_clone = id.clone();

                                        // Check if choice popup is visible
                                        let has_choice_popup = entity_clone.update(ctx, |editor: &mut EditorPrompt, _| {
                                            editor.is_choice_popup_visible()
                                        });

                                        if has_choice_popup {
                                            // Handle choice popup navigation
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    logging::log("STDIN", "SimulateKey: Up in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_up_public(cx);
                                                    });
                                                }
                                                "down" | "arrowdown" => {
                                                    logging::log("STDIN", "SimulateKey: Down in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_down_public(cx);
                                                    });
                                                }
                                                "enter" if !has_cmd => {
                                                    logging::log("STDIN", "SimulateKey: Enter in choice popup - confirming");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                    });
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape in choice popup - cancelling");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_cancel_public(cx);
                                                    });
                                                }
                                                "tab" if !has_shift => {
                                                    logging::log("STDIN", "SimulateKey: Tab in choice popup - confirm and next");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                        editor.next_tabstop_public(window, cx);
                                                    });
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in choice popup", key_lower));
                                                }
                                            }
                                        } else if key_lower == "tab" && !has_cmd {
                                            // Handle Tab key for snippet navigation
                                            entity_clone.update(ctx, |editor: &mut EditorPrompt, editor_cx| {
                                                logging::log("STDIN", "SimulateKey: Tab in EditorPrompt - calling next_tabstop");
                                                if editor.in_snippet_mode() {
                                                    editor.next_tabstop_public(window, editor_cx);
                                                } else {
                                                    logging::log("STDIN", "SimulateKey: Tab - not in snippet mode");
                                                }
                                            });
                                        } else if key_lower == "enter" && has_cmd {
                                            // Cmd+Enter submits - get content from editor
                                            logging::log("STDIN", "SimulateKey: Cmd+Enter in EditorPrompt - submitting");
                                            let content = entity_clone.update(ctx, |editor, editor_cx| {
                                                editor.content(editor_cx)
                                            });
                                            view.submit_prompt_response(prompt_id_clone.clone(), Some(content), ctx);
                                        } else if key_lower == "escape" && !has_cmd {
                                            logging::log("STDIN", "SimulateKey: Escape in EditorPrompt - cancelling");
                                            view.submit_prompt_response(prompt_id_clone.clone(), None, ctx);
                                            view.cancel_script_execution(ctx);
                                        } else {
                                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EditorPrompt", key_lower));
                                        }
                                    }
                                    AppView::ChatPrompt { entity, .. } => {
                                        // ChatPrompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ChatPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle chat actions");
                                            view.toggle_chat_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in chat actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in chat actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in chat actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing chat action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                                            }
                                                            view.execute_chat_action(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close chat actions dialog");
                                                        view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                                    }
                                                    _ => {
                                                        // Handle printable characters for search
                                                        if let Some(ch) = key_lower.chars().next() {
                                                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                                                                logging::log("STDIN", &format!("SimulateKey: Char '{}' in chat actions dialog", ch));
                                                                dialog.update(ctx, |d, cx| d.handle_char(ch, cx));
                                                            } else {
                                                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ChatPrompt actions dialog", key_lower));
                                                            }
                                                        }
                                                    }
                                                }
                                                // Notify the actions window to re-render
                                                crate::actions::notify_actions_window(ctx);
                                            }
                                        } else {
                                            // Route setup keys (tab, arrows, enter, escape) to ChatPrompt
                                            entity.update(ctx, |chat, cx| {
                                                if chat.handle_setup_key(&key_lower, has_shift, cx) {
                                                    logging::log("STDIN", &format!("SimulateKey: Setup handled '{}'", key_lower));
                                                } else {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled '{}' in ChatPrompt", key_lower));
                                                }
                                            });
                                        }
                                    }
                                    AppView::EmojiPickerView { filter, selected_index, selected_category } => {
                                        let filter_clone = filter.clone();
                                        let cat = *selected_category;
                                        let old_idx = *selected_index;
                                        let ordered = crate::emoji::filtered_ordered_emojis(&filter_clone, cat);
                                        let filtered_len = ordered.len();
                                        if filtered_len == 0 {
                                            return;
                                        }
                                        let cols = crate::emoji::GRID_COLS;
                                        let new_idx = match key_lower.as_str() {
                                            "up" | "arrowup" => old_idx.saturating_sub(cols),
                                            "down" | "arrowdown" => (old_idx + cols).min(filtered_len.saturating_sub(1)),
                                            "left" | "arrowleft" => old_idx.saturating_sub(1),
                                            "right" | "arrowright" => (old_idx + 1).min(filtered_len.saturating_sub(1)),
                                            "enter" => {
                                                if let Some(emoji) = ordered.get(old_idx) {
                                                    ctx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.emoji.to_string()));
                                                    view.close_and_reset_window(ctx);
                                                }
                                                return;
                                            }
                                            "escape" => {
                                                view.close_and_reset_window(ctx);
                                                return;
                                            }
                                            _ => {
                                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EmojiPicker", key_lower));
                                                return;
                                            }
                                        };
                                        // Apply new index
                                        if let AppView::EmojiPickerView { selected_index, .. } = &mut view.current_view {
                                            *selected_index = new_idx;
                                        }
                                        let row = crate::emoji::compute_scroll_row(new_idx, &ordered);
                                        view.emoji_scroll_handle.scroll_to_item(row, gpui::ScrollStrategy::Nearest);
                                        view.input_mode = InputMode::Keyboard;
                                        view.hovered_index = None;
                                        ctx.notify();
                                    }
                                    _ => {
                                        logging::log("STDIN", &format!("SimulateKey: View {:?} not supported for key simulation", std::mem::discriminant(&view.current_view)));
                                    }
                                }
                            }

                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening AI window via stdin command");
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log("STDIN", "Opening AI window with mock data via stdin command");
                                // First insert mock data
                                if let Err(e) = ai::insert_mock_data() {
                                    logging::log("STDIN", &format!("Failed to insert mock data: {}", e));
                                } else {
                                    logging::log("STDIN", "Mock data inserted successfully");
                                }
                                // Then open the window
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::ShowAiCommandBar => {
                                logging::log("STDIN", "Showing AI command bar via stdin command");
                                ai::show_ai_command_bar(ctx);
                            }
                            ExternalCommand::SimulateAiKey { key, modifiers } => {
                                logging::log(
                                    "STDIN",
                                    &format!("Simulating AI key: '{}' with modifiers: {:?}", key, modifiers),
                                );
                                ai::simulate_ai_key(&key, modifiers);
                            }
                            ExternalCommand::CaptureWindow { title, path } => {
                                logging::log("STDIN", &format!("Capturing window with title '{}' to '{}'", title, path));
                                match validate_capture_window_output_path(&path) {
                                    Ok(validated_path) => {
                                        match capture_window_by_title(&title, false) {
                                            Ok((png_data, width, height)) => {
                                                let mut can_write = true;
                                                if let Some(parent) = validated_path.parent() {
                                                    if let Err(e) = std::fs::create_dir_all(parent) {
                                                        can_write = false;
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Failed to create screenshot directory '{}': {}",
                                                                parent.display(),
                                                                e
                                                            ),
                                                        );
                                                    }
                                                }

                                                if can_write {
                                                    if let Err(e) = std::fs::write(&validated_path, &png_data) {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!("Failed to write screenshot: {}", e),
                                                        );
                                                    } else {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Screenshot saved: {} ({}x{})",
                                                                validated_path.display(),
                                                                width,
                                                                height
                                                            ),
                                                        );
                                                    }
                                                } else {
                                                    tracing::warn!(
                                                        category = "STDIN",
                                                        event_type = "stdin_capture_window_dir_create_failed",
                                                        requested_path = %path,
                                                        resolved_path = %validated_path.display(),
                                                        correlation_id = %logging::current_correlation_id(),
                                                        "Skipping screenshot write due to directory creation failure"
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                logging::log("STDIN", &format!("Failed to capture window: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let correlation_id = logging::current_correlation_id();
                                        tracing::warn!(
                                            category = "STDIN",
                                            event_type = "stdin_capture_window_path_rejected",
                                            requested_path = %path,
                                            reason = %e,
                                            correlation_id = %correlation_id,
                                            "Rejected captureWindow output path"
                                        );
                                        logging::log(
                                            "STDIN",
                                            &format!("Rejected captureWindow path '{}': {}", path, e),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAiSearch { text } => {
                                logging::log("STDIN", &format!("Setting AI search filter to: {}", text));
                                ai::set_ai_search(ctx, &text);
                            }
                            ExternalCommand::SetAiInput { text, submit } => {
                                logging::log("STDIN", &format!("Setting AI input to: {} (submit={})", text, submit));
                                ai::set_ai_input(ctx, &text, submit);
                            }
                            ExternalCommand::ShowGrid { grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, ref depth } => {
                                logging::log("STDIN", &format!(
                                    "ShowGrid: size={}, bounds={}, box_model={}, guides={}, dimensions={}, depth={:?}",
                                    grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, depth
                                ));
                                let options = protocol::GridOptions {
                                    grid_size,
                                    show_bounds,
                                    show_box_model,
                                    show_alignment_guides,
                                    show_dimensions,
                                    depth: depth.clone(),
                                    color_scheme: None,
                                };
                                view.show_grid(options, ctx);
                            }
                            ExternalCommand::HideGrid => {
                                logging::log("STDIN", "HideGrid: hiding debug grid overlay");
                                view.hide_grid(ctx);
                            }
                            ExternalCommand::ExecuteFallback { ref fallback_id, ref input } => {
                                logging::log("STDIN", &format!("ExecuteFallback: id='{}', input='{}'", fallback_id, input));
                                execute_fallback_action(view, fallback_id, input, window, ctx);
                            }
                            ExternalCommand::ShowShortcutRecorder { ref command_id, ref command_name } => {
                                logging::log("STDIN", &format!("ShowShortcutRecorder: command_id='{}', command_name='{}'", command_id, command_name));
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), ctx);
                            }
                        }
                    ctx.notify();
                }); // close app_entity_inner.update
            }); // close window_for_stdin.update
        }); // close cx.update
    }

    logging::log("STDIN", "Async stdin command handler exiting");
})
.detach();

        // Shutdown monitor task - checks SHUTDOWN_REQUESTED flag set by signal handler
        // Performs all cleanup on the main thread where it's safe to call logging,
        // mutexes, and other non-async-signal-safe functions.
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            loop {
                // Check every 500ms for shutdown signal
                // 500ms is acceptable latency for graceful shutdown while reducing CPU wakeups
                Timer::after(std::time::Duration::from_millis(500)).await;

                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                    logging::log("SHUTDOWN", "Shutdown signal detected, performing graceful cleanup");

                    // Kill all tracked child processes
                    logging::log("SHUTDOWN", "Killing all child processes");
                    PROCESS_MANAGER.kill_all_processes();

                    // Remove main PID file
                    PROCESS_MANAGER.remove_main_pid();

                    logging::log("SHUTDOWN", "Cleanup complete, quitting application");

                    // Quit the GPUI application
                    let _ = cx.update(|cx| {
                        cx.quit();
                    });

                    break;
                }
            }
        }).detach();

        logging::log("APP", "Application ready - Cmd+; to show, Esc to hide, Cmd+K for actions");

});
}
