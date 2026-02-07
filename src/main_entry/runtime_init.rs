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
