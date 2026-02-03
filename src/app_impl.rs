impl ScriptListApp {
    fn new(
        config: config::Config,
        bun_available: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // PERF: Measure script loading time
        let load_start = std::time::Instant::now();
        let scripts = scripts::read_scripts();
        let scripts_elapsed = load_start.elapsed();

        let scriptlets_start = std::time::Instant::now();
        // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
        // This includes built-in extensions like CleanShot and user extensions
        let scriptlets = scripts::load_scriptlets();
        let scriptlets_elapsed = scriptlets_start.elapsed();

        let theme = std::sync::Arc::new(theme::load_theme());
        // Config is now passed in from main() to avoid duplicate load (~100-300ms savings)

        // Load frecency data for suggested section tracking
        let suggested_config = config.get_suggested();
        let mut frecency_store = FrecencyStore::with_config(&suggested_config);
        frecency_store.load().ok(); // Ignore errors - starts fresh if file doesn't exist

        // Load built-in entries based on config
        let builtin_entries = builtins::get_builtin_entries(&config.get_builtins());

        // Apps are loaded in the background to avoid blocking startup
        // Start with empty list, will be populated asynchronously
        let apps = Vec::new();

        let total_elapsed = load_start.elapsed();
        logging::log("PERF", &format!(
            "Startup loading: {:.2}ms total ({} scripts in {:.2}ms, {} scriptlets in {:.2}ms, apps loading in background)",
            total_elapsed.as_secs_f64() * 1000.0,
            scripts.len(),
            scripts_elapsed.as_secs_f64() * 1000.0,
            scriptlets.len(),
            scriptlets_elapsed.as_secs_f64() * 1000.0
        ));
        logging::log(
            "APP",
            &format!(
                "Loaded {} scripts from ~/.scriptkit/kit/*/scripts",
                scripts.len()
            ),
        );
        logging::log(
            "APP",
            &format!(
                "Loaded {} scriptlets from ~/.scriptkit/kit/*/extensions",
                scriptlets.len()
            ),
        );
        logging::log(
            "APP",
            &format!("Loaded {} built-in features", builtin_entries.len()),
        );
        logging::log("APP", "Applications loading in background...");
        logging::log("APP", "Loaded theme with system appearance detection");
        logging::log(
            "APP",
            &format!(
                "Loaded config: hotkey={:?}+{}, bun_path={:?}",
                config.hotkey.modifiers, config.hotkey.key, config.bun_path
            ),
        );

        // Load apps in background thread to avoid blocking startup
        let app_launcher_enabled = config.get_builtins().app_launcher;
        if app_launcher_enabled {
            // Use a channel to send loaded apps back to main thread
            let (tx, rx) =
                std::sync::mpsc::channel::<(Vec<app_launcher::AppInfo>, std::time::Duration)>();

            // Spawn background thread for app scanning
            std::thread::spawn(move || {
                let start = std::time::Instant::now();
                let apps = app_launcher::scan_applications().clone();
                let elapsed = start.elapsed();
                let _ = tx.send((apps, elapsed));
            });

            // Poll for results using a spawned task
            cx.spawn(async move |this, cx| {
                // Poll the channel periodically
                loop {
                    Timer::after(std::time::Duration::from_millis(50)).await;
                    match rx.try_recv() {
                        Ok((apps, elapsed)) => {
                            let app_count = apps.len();
                            let _ = cx.update(|cx| {
                                this.update(cx, |app, cx| {
                                    app.apps = apps;
                                    // Invalidate caches since apps changed
                                    app.filter_cache_key = String::from("\0_APPS_LOADED_\0");
                                    app.grouped_cache_key = String::from("\0_APPS_LOADED_\0");
                                    logging::log(
                                        "APP",
                                        &format!(
                                            "Background app loading complete: {} apps in {:.2}ms",
                                            app_count,
                                            elapsed.as_secs_f64() * 1000.0
                                        ),
                                    );
                                    // CRITICAL: Sync list state after cache invalidation
                                    // Without this, the GPUI list component doesn't know
                                    // about the new apps and may render stale item counts
                                    let old_count = app.main_list_state.item_count();
                                    app.sync_list_state();
                                    let new_count = app.main_list_state.item_count();
                                    app.validate_selection_bounds(cx);
                                    logging::log(
                                        "APP",
                                        &format!(
                                            "List state synced after app load: {} -> {} items (filter='{}')",
                                            old_count,
                                            new_count,
                                            app.computed_filter_text
                                        ),
                                    );
                                    cx.notify();
                                })
                            });
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    }
                }
            })
            .detach();
        }
        logging::log("UI", "Script Kit logo SVG loaded for header rendering");

        // Start cursor blink timer - updates all inputs that track cursor visibility
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(std::time::Duration::from_millis(530)).await;

                // CRITICAL: Check window visibility BEFORE cx.update() to avoid
                // unnecessary GPUI context access when window is hidden.
                // This reduces CPU usage at idle significantly.
                if !script_kit_gpui::is_main_window_visible() {
                    continue;
                }

                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        // Additional checks for focused state
                        // (window visibility already checked above)
                        let actions_popup_open = is_actions_window_open();
                        let any_window_focused =
                            platform::is_main_window_focused() || actions_popup_open;
                        if !any_window_focused || app.focused_input == FocusedInput::None {
                            return;
                        }

                        app.cursor_visible = !app.cursor_visible;
                        // Also update ActionsDialog cursor if it exists
                        if let Some(ref dialog) = app.actions_dialog {
                            dialog.update(cx, |d, _cx| {
                                d.set_cursor_visible(app.cursor_visible);
                            });
                            // Notify the actions window to repaint with new cursor state
                            notify_actions_window(cx);
                        }
                        // Also update AliasInput cursor if it exists
                        if let Some(ref alias_input) = app.alias_input_entity {
                            alias_input.update(cx, |input, _cx| {
                                input.set_cursor_visible(app.cursor_visible);
                            });
                        }
                        cx.notify();
                    })
                });
            }
        })
        .detach();

        let gpui_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder(DEFAULT_PLACEHOLDER));
        let gpui_input_subscription = cx.subscribe_in(&gpui_input_state, window, {
            move |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Focus => {
                    this.gpui_input_focused = true;
                    this.focused_input = FocusedInput::MainFilter;

                    // Close actions popup when main input receives focus
                    // This ensures consistent behavior: clicking the input closes actions
                    // just like pressing Cmd+K would
                    if this.show_actions_popup || is_actions_window_open() {
                        logging::log(
                            "FOCUS",
                            "Main input focused while actions open - closing actions (same as Cmd+K)",
                        );
                        this.show_actions_popup = false;
                        this.actions_dialog = None;
                        // Close the actions window
                        cx.spawn(async move |_this, cx| {
                            cx.update(|cx| {
                                close_actions_window(cx);
                            })
                            .ok();
                        })
                        .detach();
                    }

                    cx.notify();
                }
                InputEvent::Blur => {
                    this.gpui_input_focused = false;
                    if this.focused_input == FocusedInput::MainFilter {
                        this.focused_input = FocusedInput::None;
                    }
                    cx.notify();
                }
                InputEvent::Change => {
                    let input_received_at = std::time::Instant::now();
                    // Read the current input value to see what we're processing
                    let current_value = this.gpui_input_state.read(cx).value().to_string();
                    logging::log(
                        "FILTER_PERF",
                        &format!(
                            "[1/5] INPUT_CHANGE value='{}' len={} at {:?}",
                            current_value,
                            current_value.len(),
                            input_received_at
                        ),
                    );
                    this.filter_perf_start = Some(input_received_at);
                    this.handle_filter_input_change(window, cx);
                }
                InputEvent::PressEnter { .. } => {
                    if matches!(this.current_view, AppView::ScriptList) && !this.show_actions_popup
                    {
                        // Check if we're in fallback mode first
                        if this.fallback_mode && !this.cached_fallbacks.is_empty() {
                            this.execute_selected_fallback(cx);
                        } else {
                            this.execute_selected(cx);
                        }
                    }
                }
            }
        });

        // Create channel for API key configuration completion signals
        // Small buffer (4) prevents blocking, more than enough for normal use
        let (api_key_tx, api_key_rx) = mpsc::sync_channel(4);

        // Create channel for builtin confirmation modal signals
        // When a dangerous action (Quit, Shut Down, etc.) requires confirmation,
        // the modal callback sends (entry_id, confirmed) through this channel
        let (builtin_confirm_tx, builtin_confirm_rx) = async_channel::bounded(4);

        // Create channel for inline chat escape signals
        let (inline_chat_escape_tx, inline_chat_escape_rx) = mpsc::sync_channel(4);
        // Create channel for inline chat configure signals (when user wants to set up API key)
        let (inline_chat_configure_tx, inline_chat_configure_rx) = mpsc::sync_channel(4);
        // Create channel for inline chat Claude Code signals (when user wants to enable Claude Code)
        let (inline_chat_claude_code_tx, inline_chat_claude_code_rx) = mpsc::sync_channel(4);
        let mut app = ScriptListApp {
            scripts,
            scriptlets,
            builtin_entries,
            apps,
            // P0 FIX: Cached data for builtin views (avoids cloning per frame)
            cached_clipboard_entries: Vec::new(),
            focused_clipboard_entry_id: None,
            cached_windows: Vec::new(),
            cached_file_results: Vec::new(),
            selected_index: 0,
            filter_text: String::new(),
            gpui_input_state,
            gpui_input_focused: false,
            gpui_input_subscriptions: vec![gpui_input_subscription],
            bounds_subscription: None, // Set later after window setup
            suppress_filter_events: false,
            pending_filter_sync: false,
            pending_placeholder: None,
            last_output: None,
            focus_handle: cx.focus_handle(),
            show_logs: false,
            theme,
            config,
            // Scroll activity tracking: start with scrollbar hidden
            is_scrolling: false,
            last_scroll_time: None,
            current_view: AppView::ScriptList,
            script_session: Arc::new(ParkingMutex::new(None)),
            arg_input: TextInputState::new(),
            arg_selected_index: 0,
            prompt_receiver: None,
            response_sender: None,
            // Variable-height list state for main menu (section headers at 24px, items at 48px)
            // Start with 0 items, will be reset when grouped_items changes
            // .measure_all() ensures all items are measured upfront for correct scroll height
            main_list_state: ListState::new(0, ListAlignment::Top, px(100.)).measure_all(),
            list_scroll_handle: UniformListScrollHandle::new(),
            arg_list_scroll_handle: UniformListScrollHandle::new(),
            clipboard_list_scroll_handle: UniformListScrollHandle::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            file_search_scroll_handle: UniformListScrollHandle::new(),
            file_search_loading: false,
            file_search_debounce_task: None,
            file_search_current_dir: None,
            file_search_frozen_filter: None,
            file_search_actions_path: None,
            file_search_gen: 0,
            file_search_cancel: None,
            file_search_display_indices: Vec::new(),
            show_actions_popup: false,
            actions_dialog: None,
            cursor_visible: true,
            focused_input: FocusedInput::MainFilter,
            current_script_pid: None,
            // P1: Initialize filter cache
            cached_filtered_results: Vec::new(),
            filter_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
            // P1: Initialize grouped results cache (Arc for cheap clone)
            cached_grouped_items: Arc::from([]),
            cached_grouped_flat_results: Arc::from([]),
            grouped_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
            // P3: Two-stage filter coalescing
            computed_filter_text: String::new(),
            filter_coalescer: FilterCoalescer::new(),
            // Scroll stabilization: start with no last scrolled index
            last_scrolled_index: None,
            // Preview cache: start empty, will populate on first render
            preview_cache_path: None,
            preview_cache_lines: Vec::new(),
            // Scriptlet preview cache: avoid re-highlighting on every render
            scriptlet_preview_cache_key: None,
            scriptlet_preview_cache_lines: Vec::new(),
            // Design system: start with default design
            current_design: DesignVariant::default(),
            // Toast manager: initialize for error notifications
            toast_manager: ToastManager::new(),
            // Clipboard image cache: decoded RenderImages for thumbnails/preview
            clipboard_image_cache: std::collections::HashMap::new(),
            // Frecency store for tracking script usage
            frecency_store,
            // Mouse hover tracking - starts as None (no item hovered)
            hovered_index: None,
            // Input mode: starts as Mouse (default), switches to Keyboard on arrow keys
            input_mode: InputMode::Mouse,
            // Fallback mode state - starts as false (showing scripts, not fallbacks)
            fallback_mode: false,
            fallback_selected_index: 0,
            cached_fallbacks: Vec::new(),
            // P0-2: Initialize hover debounce timer
            last_hover_notify: std::time::Instant::now(),
            // Render log deduplication: track last logged state to skip cursor-blink spam
            last_render_log_filter: String::new(),
            last_render_log_selection: usize::MAX, // Use MAX to ensure first render logs
            last_render_log_item_count: usize::MAX,
            log_this_render: true, // Default to true for first render
            // Filter performance tracking - None until first filter change
            filter_perf_start: None,
            // Pending path action - starts as None (Arc<Mutex<>> for callback access)
            pending_path_action: Arc::new(Mutex::new(None)),
            // Signal to close path actions dialog
            close_path_actions: Arc::new(Mutex::new(false)),
            // Shared state: path actions dialog visibility (for toggle behavior)
            path_actions_showing: Arc::new(Mutex::new(false)),
            // Shared state: path actions search text (for header display)
            path_actions_search_text: Arc::new(Mutex::new(String::new())),
            // Pending path action result - action_id + path_info to execute
            pending_path_action_result: Arc::new(Mutex::new(None)),
            // Alias/shortcut registries - populated below
            alias_registry: std::collections::HashMap::new(),
            shortcut_registry: std::collections::HashMap::new(),
            // SDK actions - starts empty, populated by setActions() from scripts
            sdk_actions: None,
            action_shortcuts: std::collections::HashMap::new(),
            // Debug grid overlay - check env var at startup
            grid_config: if std::env::var("SCRIPT_KIT_DEBUG_GRID").is_ok() {
                logging::log(
                    "DEBUG_GRID",
                    "SCRIPT_KIT_DEBUG_GRID env var set - enabling grid overlay",
                );
                Some(debug_grid::GridConfig::default())
            } else {
                None
            },
            // Navigation coalescing for rapid arrow key events
            nav_coalescer: NavCoalescer::new(),
            // Wheel scroll accumulator starts at 0
            wheel_accum: 0.0,
            // Window focus tracking - for detecting focus lost and auto-dismissing prompts
            was_window_focused: false,
            // Pin state - when true, window stays open on blur
            is_pinned: false,
            // Pending focus: start with MainFilter since that's what we want focused initially
            // DEPRECATED: Use focus_coordinator instead. This remains for gradual migration.
            pending_focus: Some(FocusTarget::MainFilter),
            // Focus coordinator: unified focus management with push/pop overlay semantics
            focus_coordinator: focus_coordinator::FocusCoordinator::with_main_filter_focus(),
            // Scroll stabilization: track last scrolled index for each handle
            last_scrolled_main: None,
            last_scrolled_arg: None,
            last_scrolled_clipboard: None,
            last_scrolled_window: None,
            last_scrolled_design_gallery: None,
            // Show warning banner when bun is not available
            show_bun_warning: !bun_available,
            // Builtin confirmation channel
            builtin_confirm_sender: builtin_confirm_tx,
            builtin_confirm_receiver: builtin_confirm_rx,
            // Menu bar integration: Now handled by frontmost_app_tracker module
            // which pre-fetches menu items in background when apps activate
            // Shortcut recorder state - starts as None (no recorder showing)
            shortcut_recorder_state: None,
            // Shortcut recorder entity - persisted to maintain focus
            shortcut_recorder_entity: None,
            // Alias input state - starts as None (no alias input showing)
            alias_input_state: None,
            // Alias input entity - persisted to maintain focus
            alias_input_entity: None,
            // Input history for shell-like up/down navigation
            input_history: {
                let mut history = input_history::InputHistory::new();
                if let Err(e) = history.load() {
                    tracing::warn!("Failed to load input history: {}", e);
                }
                history
            },
            // API key configuration state
            pending_api_key_config: None,
            // API key completion channel - for EnvPrompt callback to signal completion
            // The channel is created here and both ends are stored
            api_key_completion_sender: api_key_tx,
            api_key_completion_receiver: api_key_rx,
            // Navigation tracking: starts false, set to true when opening built-in views from main menu
            opened_from_main_menu: false,
            // Inline chat escape channel - for ChatPrompt escape callback to signal return to main menu
            inline_chat_escape_sender: inline_chat_escape_tx,
            inline_chat_escape_receiver: inline_chat_escape_rx,
            // Inline chat configure channel - for ChatPrompt configure callback to trigger API key setup
            inline_chat_configure_sender: inline_chat_configure_tx,
            inline_chat_configure_receiver: inline_chat_configure_rx,
            // Inline chat Claude Code channel - for ChatPrompt Claude Code callback to enable Claude Code
            inline_chat_claude_code_sender: inline_chat_claude_code_tx,
            inline_chat_claude_code_receiver: inline_chat_claude_code_rx,
            // Light theme opacity adjustment offset (Cmd+Shift+[/])
            light_opacity_offset: 0.0,
            // Mouse cursor hidden state - hidden while typing, shown on mouse move
            mouse_cursor_hidden: false,
        };

        // Build initial alias/shortcut registries (conflicts logged, not shown via HUD on startup)
        let conflicts = app.rebuild_registries();
        if !conflicts.is_empty() {
            logging::log(
                "STARTUP",
                &format!(
                    "Found {} alias/shortcut conflicts on startup",
                    conflicts.len()
                ),
            );
        }

        // Add Tab key interceptor for "Ask AI" feature and file search directory navigation
        // This fires BEFORE normal key handling, allowing us to intercept Tab
        // even when the Input component has focus
        let app_entity_for_tab = cx.entity().downgrade();
        let tab_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_tab;
            move |event, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let has_shift = event.keystroke.modifiers.shift;
                // Check for Tab key (no cmd/alt/ctrl modifiers, but shift is allowed)
                if key == "tab"
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // FIRST: If confirm dialog is open, route Tab to it for button switching
                            let confirm_open = crate::confirm::is_confirm_window_open();
                            crate::logging::log(
                                "KEY",
                                &format!("Tab intercepted, confirm_open={}", confirm_open),
                            );
                            if confirm_open && crate::confirm::dispatch_confirm_key(&key, cx) {
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Tab/Shift+Tab in FileSearchView for directory/file navigation
                            // CRITICAL: ALWAYS consume Tab/Shift+Tab to prevent focus traversal
                            if let AppView::FileSearchView {
                                query,
                                selected_index,
                            } = &mut this.current_view
                            {
                                // ALWAYS stop propagation for Tab/Shift+Tab in FileSearchView
                                // This prevents Tab from falling through to focus traversal
                                cx.stop_propagation();

                                if has_shift {
                                    // Shift+Tab: Go up one directory level using parent_dir_display helper
                                    // This handles ~/, /, ./, ../ and regular paths correctly
                                    if let Some(parent_path) =
                                        crate::file_search::parent_dir_display(query)
                                    {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Navigating up from '{}' to '{}'",
                                                query, parent_path
                                            ),
                                        );

                                        // Update the input - handle_filter_input_change will:
                                        // - Update query
                                        // - Reset selected_index to 0
                                        // - Detect directory change
                                        // - Trigger async directory load
                                        this.gpui_input_state.update(cx, |state, cx| {
                                            state.set_value(parent_path.clone(), window, cx);
                                            // Ensure cursor is at end with no selection after programmatic set_value
                                            // This prevents issues where GPUI might leave caret at wrong position
                                            let len = parent_path.len();
                                            state.set_selection(len, len, window, cx);
                                        });

                                        cx.notify();
                                    } else {
                                        // At root (/ or ~/) - no parent to navigate to
                                        // Key is consumed (stop_propagation called above) but no action taken
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Already at root '{}', no-op",
                                                query
                                            ),
                                        );
                                    }
                                } else {
                                    // Tab: Enter directory OR autocomplete file name
                                    // Get filtered results to find selected item
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    let filtered_results: Vec<_> =
                                        if let Some(ref pattern) = filter_pattern {
                                            crate::file_search::filter_results_nucleo_simple(
                                                &this.cached_file_results,
                                                pattern,
                                            )
                                        } else {
                                            this.cached_file_results.iter().enumerate().collect()
                                        };

                                    // Defensive bounds check: clamp selected_index if out of bounds
                                    let filtered_len = filtered_results.len();
                                    if filtered_len > 0 && *selected_index >= filtered_len {
                                        *selected_index = filtered_len - 1;
                                    }

                                    if let Some((_, file)) = filtered_results.get(*selected_index) {
                                        if file.file_type == crate::file_search::FileType::Directory
                                        {
                                            // Directory: Enter it (append /)
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            let new_path = format!("{}/", shortened);
                                            crate::logging::log(
                                                "KEY",
                                                &format!("Tab: Entering directory: {}", new_path),
                                            );

                                            // Update the input - handle_filter_input_change handles the rest
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(new_path.clone(), window, cx);
                                                // Ensure cursor is at end with no selection after programmatic set_value
                                                let len = new_path.len();
                                                state.set_selection(len, len, window, cx);
                                            });

                                            cx.notify();
                                        } else {
                                            // File: Autocomplete the full path (terminal-style tab completion)
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            crate::logging::log(
                                                "KEY",
                                                &format!(
                                                    "Tab: Autocompleting file path: {}",
                                                    shortened
                                                ),
                                            );

                                            // Set the input to the file's full path
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(shortened.clone(), window, cx);
                                                // Ensure cursor is at end with no selection after programmatic set_value
                                                let len = shortened.len();
                                                state.set_selection(len, len, window, cx);
                                            });

                                            cx.notify();
                                        }
                                    } else {
                                        // No selection - just consume the key
                                        crate::logging::log(
                                            "KEY",
                                            "Tab: No selection to autocomplete, no-op",
                                        );
                                    }
                                }
                                return;
                            }

                            // Handle Tab in ScriptList view for Ask AI feature
                            // Shows inline ChatPrompt with built-in AI (prefers Vercel AI Gateway)
                            if matches!(this.current_view, AppView::ScriptList)
                                && !this.filter_text.is_empty()
                                && !this.show_actions_popup
                                && !has_shift
                            {
                                let query = this.filter_text.clone();

                                // Clear filter text before switching view
                                this.filter_text.clear();

                                // Show inline AI chat with the query as initial input
                                this.show_inline_ai_chat(Some(query), cx);

                                // Stop propagation so Input doesn't handle it
                                cx.stop_propagation();
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(tab_interceptor);

        // Add arrow key interceptor for builtin views with Input components
        // This fires BEFORE Input component handles arrow keys, allowing list navigation
        let app_entity_for_arrows = cx.entity().downgrade();
        let arrow_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_arrows;
            move |event, _window, cx| {
                let key = event.keystroke.key.to_lowercase();
                // Check for Up/Down arrow keys (no modifiers except shift for selection)
                if (key == "up" || key == "arrowup" || key == "down" || key == "arrowdown")
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // FIRST: If confirm dialog is open, route all arrow keys to it
                            if crate::confirm::is_confirm_window_open()
                                && crate::confirm::dispatch_confirm_key(&key, cx)
                            {
                                cx.stop_propagation();
                                return;
                            }

                            // Only intercept in views that use Input + list navigation
                            match &mut this.current_view {
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Compute filtered length using same logic as render
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    // Use Nucleo fuzzy matching for consistent filtering with render
                                    let filtered_len = if let Some(ref pattern) = filter_pattern {
                                        crate::file_search::filter_results_nucleo_simple(
                                            &this.cached_file_results,
                                            pattern,
                                        )
                                        .len()
                                    } else {
                                        this.cached_file_results.len()
                                    };

                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    // Stop propagation so Input doesn't handle it
                                    cx.stop_propagation();
                                }
                                AppView::ClipboardHistoryView {
                                    selected_index,
                                    filter,
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    let filtered_entries: Vec<_> = if filter.is_empty() {
                                        this.cached_clipboard_entries.iter().enumerate().collect()
                                    } else {
                                        let filter_lower = filter.to_lowercase();
                                        this.cached_clipboard_entries
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, e)| {
                                                e.text_preview
                                                    .to_lowercase()
                                                    .contains(&filter_lower)
                                            })
                                            .collect()
                                    };
                                    let filtered_len = filtered_entries.len();
                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.clipboard_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.clipboard_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                    }
                                    this.focused_clipboard_entry_id = filtered_entries
                                        .get(*selected_index)
                                        .map(|(_, entry)| entry.id.clone());
                                    cx.notify();
                                    cx.stop_propagation();
                                }
                                AppView::AppLauncherView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    // Filter apps to get correct count
                                    let filtered_len = this.apps.len();
                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::WindowSwitcherView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    let filtered_len = this.cached_windows.len();
                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.window_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.window_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::ScriptList => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Main menu: handle list navigation + input history
                                    if key == "up" || key == "arrowup" {
                                        // Input history: only when filter empty AND at top of list
                                        if this.filter_text.is_empty() && this.selected_index == 0 {
                                            if let Some(text) = this.input_history.navigate_up() {
                                                logging::log(
                                                    "HISTORY",
                                                    &format!("Recalled: {}", text),
                                                );
                                                this.filter_text = text.clone();
                                                let text_len = text.len();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            text.clone(),
                                                            _window,
                                                            input_cx,
                                                        );
                                                        state.set_selection(
                                                            text_len, text_len, _window, input_cx,
                                                        );
                                                    },
                                                );
                                                this.queue_filter_compute(text, cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                                return;
                                            }
                                        }
                                        // Normal up navigation - use move_selection_up to skip section headers
                                        this.move_selection_up(cx);
                                    } else if key == "down" || key == "arrowdown" {
                                        // Down during history navigation returns to newer entries
                                        if this.input_history.current_index().is_some() {
                                            if let Some(text) = this.input_history.navigate_down() {
                                                logging::log(
                                                    "HISTORY",
                                                    &format!("Recalled: {}", text),
                                                );
                                                this.filter_text = text.clone();
                                                let text_len = text.len();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            text.clone(),
                                                            _window,
                                                            input_cx,
                                                        );
                                                        state.set_selection(
                                                            text_len, text_len, _window, input_cx,
                                                        );
                                                    },
                                                );
                                                this.queue_filter_compute(text, cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                                return;
                                            } else {
                                                // Past newest - clear to empty
                                                this.input_history.reset_navigation();
                                                this.filter_text.clear();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            String::new(),
                                                            _window,
                                                            input_cx,
                                                        );
                                                        state
                                                            .set_selection(0, 0, _window, input_cx);
                                                    },
                                                );
                                                this.queue_filter_compute(String::new(), cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                                return;
                                            }
                                        }
                                        // Normal down navigation - use move_selection_down to skip section headers
                                        this.move_selection_down(cx);
                                    }
                                    cx.stop_propagation();
                                }
                                _ => {
                                    // Don't intercept arrows for other views (let normal handling work)
                                }
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(arrow_interceptor);

        // Add interceptor for actions popup in FileSearchView and ScriptList
        // This handles Cmd+K (toggle), Escape (close), Enter (submit), and typing
        let app_entity_for_actions = cx.entity().downgrade();
        let actions_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_actions;
            move |event, window, cx| {
                // CRITICAL: Skip processing if this keystroke is from Notes or AI window
                // intercept_keystrokes is GLOBAL and fires for ALL windows in the app
                // We only want to handle keystrokes for the main window
                if crate::notes::is_notes_window(window) || crate::ai::is_ai_window(window) {
                    return; // Let the secondary window handle its own keystrokes
                }

                let key = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;
                let key_char = event.keystroke.key_char.as_deref();

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // FIRST: If confirm dialog is open, route Enter/Escape to it
                        // NOTE: Tab is handled by the dedicated Tab interceptor above, so
                        // we exclude it here to avoid double-dispatching toggle_focus()
                        if key != "tab"
                            && crate::confirm::is_confirm_window_open()
                            && crate::confirm::dispatch_confirm_key(&key, cx)
                        {
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Cmd+K to toggle actions popup (works in ScriptList, FileSearchView, ArgPrompt)
                        // This MUST be intercepted here because the Input component has focus and
                        // normal on_key_down handlers won't receive the event
                        if has_cmd && key == "k" && !has_shift {
                            match &mut this.current_view {
                                AppView::ScriptList => {
                                    // Toggle actions for the main script list
                                    if this.has_actions() {
                                        logging::log(
                                            "KEY",
                                            "Interceptor: Cmd+K -> toggle_actions (ScriptList)",
                                        );
                                        this.toggle_actions(cx, window);
                                    }
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                } => {
                                    // Get the filter pattern for directory path parsing
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    let filtered_results: Vec<_> =
                                        if let Some(ref pattern) = filter_pattern {
                                            crate::file_search::filter_results_nucleo_simple(
                                                &this.cached_file_results,
                                                pattern,
                                            )
                                        } else {
                                            this.cached_file_results.iter().enumerate().collect()
                                        };

                                    // Defensive bounds check: clamp selected_index if out of bounds
                                    let filtered_len = filtered_results.len();
                                    if filtered_len > 0 && *selected_index >= filtered_len {
                                        *selected_index = filtered_len - 1;
                                    }

                                    if let Some((_, file)) = filtered_results.get(*selected_index) {
                                        let file_clone = (*file).clone();
                                        this.toggle_file_search_actions(&file_clone, window, cx);
                                    }
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::ArgPrompt { .. } => {
                                    // Toggle actions for arg prompts (SDK setActions)
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_arg_actions (ArgPrompt)");
                                    this.toggle_arg_actions(cx, window);
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::ChatPrompt { .. } => {
                                    // Toggle actions for chat prompts
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_chat_actions (ChatPrompt)");
                                    this.toggle_chat_actions(cx, window);
                                    cx.stop_propagation();
                                    return;
                                }
                                AppView::ClipboardHistoryView { .. } => {
                                    // Toggle actions for selected clipboard entry
                                    if let Some(entry) = this.selected_clipboard_entry() {
                                        logging::log(
                                            "KEY",
                                            "Interceptor: Cmd+K -> toggle_clipboard_actions (ClipboardHistoryView)",
                                        );
                                        this.toggle_clipboard_actions(entry, window, cx);
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                                _ => {
                                    // Other views don't support Cmd+K actions
                                }
                            }
                        }

                        // Handle Cmd+Shift+K for add_shortcut in ScriptList
                        if has_cmd && key == "k" && has_shift
                            && matches!(this.current_view, AppView::ScriptList)
                        {
                            logging::log("KEY", "Interceptor: Cmd+Shift+K -> add_shortcut (ScriptList)");
                            this.handle_action("add_shortcut".to_string(), cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Window tweaker shortcuts (only enabled with SCRIPT_KIT_WINDOW_TWEAKER=1)
                        let window_tweaker_enabled = std::env::var("SCRIPT_KIT_WINDOW_TWEAKER")
                            .map(|v| v == "1")
                            .unwrap_or(false);

                        if window_tweaker_enabled {
                            // Handle Cmd+- to decrease light theme opacity
                            if has_cmd && !has_shift && (key == "-" || key == "minus") {
                                logging::log("KEY", &format!("Interceptor: Cmd+- (key={}) -> decrease light opacity", key));
                                this.adjust_light_opacity(-0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+= (or Cmd+Shift+=) to increase light theme opacity
                            if has_cmd && (key == "=" || key == "equal" || key == "plus") {
                                logging::log("KEY", &format!("Interceptor: Cmd+= (key={}) -> increase light opacity", key));
                                this.adjust_light_opacity(0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+M to cycle vibrancy material (blur effect)
                            if has_cmd && !has_shift && key == "m" {
                                logging::log("KEY", "Interceptor: Cmd+M -> cycle vibrancy material");
                                let description = platform::cycle_vibrancy_material();
                                this.toast_manager.push(components::toast::Toast::info(
                                    description,
                                    &this.theme,
                                ));
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+Shift+A to cycle vibrancy appearance (VibrantLight, VibrantDark, etc.)
                            if has_cmd && has_shift && key == "a" {
                                logging::log("KEY", "Interceptor: Cmd+Shift+A -> cycle vibrancy appearance");
                                let description = platform::cycle_appearance();
                                this.toast_manager.push(components::toast::Toast::info(
                                    description,
                                    &this.theme,
                                ));
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }
                        }

                        // Only handle remaining keys if in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            // Arrow keys are handled by arrow_interceptor to avoid double-processing
                            // (which can skip 2 items per keypress when both interceptors handle arrows).
                            if key == "up"
                                || key == "arrowup"
                                || key == "down"
                                || key == "arrowdown"
                            {
                                return;
                            }

                            // Route modal actions keys for ScriptList and ClipboardHistoryView.
                            let host = if matches!(this.current_view, AppView::ScriptList) {
                                Some(ActionsDialogHost::MainList)
                            } else if matches!(this.current_view, AppView::ClipboardHistoryView { .. }) {
                                Some(ActionsDialogHost::ClipboardHistory)
                            } else {
                                None
                            };

                            if let Some(host) = host {
                                match this.route_key_to_actions_dialog(
                                    &key,
                                    key_char,
                                    &event.keystroke.modifiers,
                                    host,
                                    window,
                                    cx,
                                ) {
                                    ActionsRoute::NotHandled => {}
                                    ActionsRoute::Handled => {
                                        cx.stop_propagation();
                                        return;
                                    }
                                    ActionsRoute::Execute { action_id } => {
                                        this.handle_action(action_id, cx);
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                            }
                            return;
                        }

                        // Only handle remaining keys if actions popup is open (FileSearchView)
                        if !this.show_actions_popup {
                            return;
                        }

                        // Handle Escape to close actions popup
                        if key == "escape" {
                            this.close_actions_popup(ActionsDialogHost::FileSearch, window, cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Enter to submit selected action
                        if key == "enter" {
                            if let Some(ref dialog) = this.actions_dialog {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();

                                if let Some(action_id) = action_id {
                                    crate::logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "FileSearch actions executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );

                                    if should_close {
                                        this.close_actions_popup(
                                            ActionsDialogHost::FileSearch,
                                            window,
                                            cx,
                                        );
                                    }

                                    // Use handle_action instead of trigger_action_by_name
                                    // handle_action supports both built-in actions (open_file, quick_look, etc.)
                                    // and SDK actions
                                    this.handle_action(action_id, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Backspace for actions search
                        if key == "backspace" {
                            if let Some(ref dialog) = this.actions_dialog {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                crate::actions::notify_actions_window(cx);
                                crate::actions::resize_actions_window(cx, dialog);
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle printable character input for actions search
                        if let Some(chars) = key_char {
                            if let Some(ch) = chars.chars().next() {
                                if ch.is_ascii_graphic() || ch == ' ' {
                                    if let Some(ref dialog) = this.actions_dialog {
                                        dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        crate::actions::notify_actions_window(cx);
                                        crate::actions::resize_actions_window(cx, dialog);
                                    }
                                    cx.stop_propagation();
                                }
                            }
                        }
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(actions_interceptor);

        // CRITICAL FIX: Sync list state on initialization
        // This was removed when state mutations were moved out of render(),
        // but we still need to sync once during initialization so the list
        // knows about the scripts that were loaded.
        // Without this, the first render shows "No scripts or snippets found"
        // because main_list_state starts with 0 items.
        app.sync_list_state();
        app.validate_selection_bounds(cx);

        app
    }

    /// Switch to a different design variant
    ///
    /// Cycle to the next design variant.
    /// Use Cmd+1 to cycle through all designs.
    fn cycle_design(&mut self, cx: &mut Context<Self>) {
        let old_design = self.current_design;
        let new_design = old_design.next();
        let all_designs = DesignVariant::all();
        let old_idx = all_designs
            .iter()
            .position(|&v| v == old_design)
            .unwrap_or(0);
        let new_idx = all_designs
            .iter()
            .position(|&v| v == new_design)
            .unwrap_or(0);

        logging::log(
            "DESIGN",
            &format!(
                "Cycling design: {} ({}) -> {} ({}) [total: {}]",
                old_design.name(),
                old_idx,
                new_design.name(),
                new_idx,
                all_designs.len()
            ),
        );
        logging::log(
            "DESIGN",
            &format!(
                "Design '{}': {}",
                new_design.name(),
                new_design.description()
            ),
        );

        self.current_design = new_design;
        logging::log(
            "DESIGN",
            &format!("self.current_design is now: {:?}", self.current_design),
        );
        cx.notify();
    }

    fn update_theme(&mut self, cx: &mut Context<Self>) {
        let base_theme = theme::load_theme();

        // Preserve opacity offset in light mode, reset in dark mode
        if base_theme.is_dark_mode() {
            self.light_opacity_offset = 0.0;
            self.theme = std::sync::Arc::new(base_theme);
        } else if self.light_opacity_offset != 0.0 {
            // Apply the opacity offset if set
            self.theme = std::sync::Arc::new(base_theme.with_opacity_offset(self.light_opacity_offset));
        } else {
            self.theme = std::sync::Arc::new(base_theme);
        }

        logging::log("APP", "Theme reloaded based on system appearance");

        // Propagate theme to open ActionsDialog (if any) for hot-reload support
        if let Some(ref dialog) = self.actions_dialog {
            let theme_arc = std::sync::Arc::clone(&self.theme);
            dialog.update(cx, |d, _| {
                d.update_theme(theme_arc);
            });
            logging::log("APP", "Theme propagated to ActionsDialog");
        }

        cx.notify();
    }

    fn update_config(&mut self, cx: &mut Context<Self>) {
        self.config = config::load_config();
        clipboard_history::set_max_text_content_len(
            self.config.get_clipboard_history_max_text_length(),
        );
        // Hot-reload hotkeys from updated config
        hotkeys::update_hotkeys(&self.config);
        logging::log(
            "APP",
            &format!("Config reloaded: padding={:?}", self.config.get_padding()),
        );
        cx.notify();
    }

    /// Adjust the light theme opacity by a delta amount
    ///
    /// Use Cmd+Shift+[ to decrease and Cmd+Shift+] to increase.
    /// The offset is clamped to the range -0.5 to +0.5.
    fn adjust_light_opacity(&mut self, delta: f32, cx: &mut Context<Self>) {
        // Only adjust if we're in light mode
        let base_theme = theme::load_theme();
        if base_theme.is_dark_mode() {
            logging::log("APP", "Opacity adjustment only works in light mode");
            return;
        }

        // Adjust the offset
        self.light_opacity_offset = (self.light_opacity_offset + delta).clamp(-0.5, 0.5);

        // Create new theme with adjusted opacity
        let adjusted_theme = base_theme.with_opacity_offset(self.light_opacity_offset);
        self.theme = std::sync::Arc::new(adjusted_theme);

        let new_opacity = self.theme.get_opacity().main;
        logging::log(
            "APP",
            &format!(
                "Light opacity adjusted: offset={:.2}, main={:.2}",
                self.light_opacity_offset, new_opacity
            ),
        );

        // Show toast with current opacity level
        let percent = (new_opacity * 100.0).round() as i32;
        self.toast_manager.push(components::toast::Toast::info(
            format!("Opacity: {}%", percent),
            &self.theme,
        ));

        cx.notify();
    }

    /// Request focus for a specific target. Focus will be applied once on the
    /// next render when window access is available, then cleared.
    ///
    /// This avoids the "perpetually enforce focus in render()" anti-pattern.
    /// Use this instead of directly calling window.focus() from non-render code.
    #[allow(dead_code)] // Public API for external callers without direct pending_focus access
    pub fn request_focus(&mut self, target: FocusTarget, cx: &mut Context<Self>) {
        self.pending_focus = Some(target);
        cx.notify();
    }

    // === FocusCoordinator Integration Methods ===
    // These methods provide a unified focus management API using the new FocusCoordinator.
    // They exist alongside the old system for gradual migration.

    /// Request focus using the new coordinator system.
    ///
    /// This sets both the coordinator's pending request AND syncs to the old system
    /// for backward compatibility during migration.
    #[allow(dead_code)]
    pub fn focus_via_coordinator(
        &mut self,
        request: focus_coordinator::FocusRequest,
        cx: &mut Context<Self>,
    ) {
        self.focus_coordinator.request(request);
        // Sync to old system for backward compatibility
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Push an overlay (like actions dialog) with automatic restore on pop.
    ///
    /// Saves current focus state and requests focus to the overlay.
    /// Call `pop_focus_overlay()` when the overlay closes to restore.
    pub fn push_focus_overlay(
        &mut self,
        overlay_request: focus_coordinator::FocusRequest,
        cx: &mut Context<Self>,
    ) {
        self.focus_coordinator.push_overlay(overlay_request);
        // Sync to old system
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Pop an overlay and restore previous focus state.
    ///
    /// Called when an overlay (actions dialog, shortcut recorder, etc.) closes.
    /// Restores focus to whatever was focused before the overlay opened.
    pub fn pop_focus_overlay(&mut self, cx: &mut Context<Self>) {
        self.focus_coordinator.pop_overlay();
        // Sync to old system
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Clear all overlays and return to main filter focus.
    ///
    /// Useful for "escape all" or error recovery scenarios.
    #[allow(dead_code)]
    pub fn clear_focus_overlays(&mut self, cx: &mut Context<Self>) {
        self.focus_coordinator.clear_overlays();
        // Sync to old system
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Get the current cursor owner from the coordinator.
    #[allow(dead_code)]
    pub fn current_cursor_owner(&self) -> focus_coordinator::CursorOwner {
        self.focus_coordinator.cursor_owner()
    }

    /// Sync coordinator state to legacy focused_input/pending_focus fields.
    ///
    /// This bridges the new and old systems during migration.
    fn sync_coordinator_to_legacy(&mut self) {
        // Sync cursor owner to focused_input
        self.focused_input = match self.focus_coordinator.cursor_owner() {
            focus_coordinator::CursorOwner::MainFilter => FocusedInput::MainFilter,
            focus_coordinator::CursorOwner::ActionsSearch => FocusedInput::ActionsSearch,
            focus_coordinator::CursorOwner::ArgPrompt => FocusedInput::ArgPrompt,
            focus_coordinator::CursorOwner::ChatPrompt => FocusedInput::None, // ChatPrompt not in old enum
            focus_coordinator::CursorOwner::None => FocusedInput::None,
        };

        // Sync pending target to pending_focus
        if let Some(request) = self.focus_coordinator.peek_pending() {
            self.pending_focus = Some(match request.target {
                focus_coordinator::FocusTarget::MainFilter => FocusTarget::MainFilter,
                focus_coordinator::FocusTarget::ActionsDialog => FocusTarget::ActionsDialog,
                focus_coordinator::FocusTarget::ArgPrompt => FocusTarget::AppRoot, // ArgPrompt uses AppRoot
                focus_coordinator::FocusTarget::PathPrompt => FocusTarget::PathPrompt,
                focus_coordinator::FocusTarget::FormPrompt => FocusTarget::FormPrompt,
                focus_coordinator::FocusTarget::EditorPrompt => FocusTarget::EditorPrompt,
                focus_coordinator::FocusTarget::SelectPrompt => FocusTarget::SelectPrompt,
                focus_coordinator::FocusTarget::EnvPrompt => FocusTarget::EnvPrompt,
                focus_coordinator::FocusTarget::DropPrompt => FocusTarget::DropPrompt,
                focus_coordinator::FocusTarget::TemplatePrompt => FocusTarget::TemplatePrompt,
                focus_coordinator::FocusTarget::TermPrompt => FocusTarget::TermPrompt,
                focus_coordinator::FocusTarget::ChatPrompt => FocusTarget::ChatPrompt,
                focus_coordinator::FocusTarget::DivPrompt => FocusTarget::AppRoot, // DivPrompt uses AppRoot
                focus_coordinator::FocusTarget::ScratchPad => FocusTarget::EditorPrompt,
                focus_coordinator::FocusTarget::QuickTerminal => FocusTarget::TermPrompt,
            });
        }
    }

    /// Apply pending focus if set. Called at the start of render() when window
    /// is focused. This applies focus exactly once, then clears pending_focus.
    ///
    /// Returns true if focus was applied (for logging/debugging).
    fn apply_pending_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(target) = self.pending_focus.take() else {
            return false;
        };

        logging::log("FOCUS", &format!("Applying pending focus: {:?}", target));

        match target {
            FocusTarget::MainFilter => {
                let input_state = self.gpui_input_state.clone();
                input_state.update(cx, |state, cx| {
                    state.focus(window, cx);
                });
                self.focused_input = FocusedInput::MainFilter;
            }
            FocusTarget::ActionsDialog => {
                if let Some(ref dialog) = self.actions_dialog {
                    let fh = dialog.read(cx).focus_handle.clone();
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::ActionsSearch;
                }
            }
            FocusTarget::EditorPrompt => {
                let entity = match &self.current_view {
                    AppView::EditorPrompt { entity, .. } => Some(entity),
                    AppView::ScratchPadView { entity, .. } => Some(entity),
                    _ => None,
                };
                if let Some(entity) = entity {
                    entity.update(cx, |editor, cx| {
                        editor.focus(window, cx);
                    });
                    // EditorPrompt has its own cursor management
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::PathPrompt => {
                if let AppView::PathPrompt { focus_handle, .. } = &self.current_view {
                    let fh = focus_handle.clone();
                    window.focus(&fh, cx);
                    // PathPrompt has its own cursor management
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::FormPrompt => {
                if let AppView::FormPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    // FormPrompt has its own focus handling
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::SelectPrompt => {
                if let AppView::SelectPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::EnvPrompt => {
                if let AppView::EnvPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::DropPrompt => {
                if let AppView::DropPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::TemplatePrompt => {
                if let AppView::TemplatePrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::TermPrompt => {
                let entity = match &self.current_view {
                    AppView::TermPrompt { entity, .. } => Some(entity),
                    AppView::QuickTerminalView { entity, .. } => Some(entity),
                    _ => None,
                };
                if let Some(entity) = entity {
                    let fh = entity.read(cx).focus_handle.clone();
                    window.focus(&fh, cx);
                    // Terminal handles its own cursor
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::ChatPrompt => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::AppRoot => {
                window.focus(&self.focus_handle, cx);
                // Don't reset focused_input here - the caller already set it appropriately.
                // For example, ArgPrompt sets focused_input = ArgPrompt before setting
                // pending_focus = AppRoot, and we want to preserve that so the cursor blinks.
            }
        }

        true
    }

    fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        self.scripts = scripts::read_scripts();
        // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
        self.scriptlets = scripts::load_scriptlets();
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Sync list component state and validate selection
        // This moves state mutation OUT of render() (anti-pattern fix)
        self.sync_list_state();
        self.selected_index = 0;
        self.validate_selection_bounds(cx);
        self.main_list_state
            .scroll_to_reveal_item(self.selected_index);
        self.last_scrolled_index = Some(self.selected_index);

        // Rebuild alias/shortcut registries and show HUD for any conflicts
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx); // 4s for conflict messages
        }

        logging::log(
            "APP",
            &format!(
                "Scripts refreshed: {} scripts, {} scriptlets loaded",
                self.scripts.len(),
                self.scriptlets.len()
            ),
        );
        cx.notify();
    }

    /// Refresh app launcher cache and invalidate search caches.
    ///
    /// Called by AppWatcher when applications are added/removed/updated.
    /// This properly invalidates filter/grouped caches so the main search
    /// immediately reflects new apps without requiring user to type.
    ///
    /// NOTE: cx.notify() is efficient - GPUI batches notifications and only
    /// re-renders when the event loop runs. We always call it because:
    /// 1. If user is in ScriptList, cached search results need updating
    /// 2. If user is in AppLauncherView, the list needs updating
    /// 3. The cost of an "unnecessary" notify is near-zero (just marks dirty)
    pub fn refresh_apps(&mut self, cx: &mut Context<Self>) {
        self.apps = crate::app_launcher::get_cached_apps();
        // Invalidate caches so main search includes new apps
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Sync list component state and validate selection
        // This ensures the GPUI list component knows about the new app count
        self.sync_list_state();
        self.validate_selection_bounds(cx);

        logging::log(
            "APP",
            &format!("Apps refreshed: {} applications loaded", self.apps.len()),
        );
        cx.notify();
    }

    /// Dismiss the bun warning banner
    fn dismiss_bun_warning(&mut self, cx: &mut Context<Self>) {
        logging::log("APP", "Bun warning banner dismissed by user");
        self.show_bun_warning = false;
        cx.notify();
    }

    /// Open bun.sh in the default browser
    fn open_bun_website(&self) {
        logging::log("APP", "Opening https://bun.sh in default browser");
        if let Err(e) = std::process::Command::new("open")
            .arg("https://bun.sh")
            .spawn()
        {
            logging::log("APP", &format!("Failed to open bun.sh: {}", e));
        }
    }

    /// Handle incremental scriptlet file change
    ///
    /// Instead of reloading all scriptlets, this method:
    /// 1. Parses only the changed file
    /// 2. Diffs against cached state to find what changed
    /// 3. Updates hotkeys/keyword triggers incrementally
    /// 4. Updates the scriptlets list
    ///
    /// # Arguments
    /// * `path` - Path to the changed/deleted scriptlet file
    /// * `is_deleted` - Whether the file was deleted (vs created/modified)
    /// * `cx` - The context for UI updates
    fn handle_scriptlet_file_change(
        &mut self,
        path: &std::path::Path,
        is_deleted: bool,
        cx: &mut Context<Self>,
    ) {
        use script_kit_gpui::scriptlet_cache::{diff_scriptlets, CachedScriptlet};

        logging::log(
            "APP",
            &format!(
                "Incremental scriptlet change: {} (deleted={})",
                path.display(),
                is_deleted
            ),
        );

        // Get old cached scriptlets for this file (if any)
        // Note: We're using a simple approach here - comparing name+shortcut+expand+alias
        let old_scriptlets: Vec<CachedScriptlet> = self
            .scriptlets
            .iter()
            .filter(|s| {
                s.file_path
                    .as_ref()
                    .map(|fp| fp.starts_with(&path.to_string_lossy().to_string()))
                    .unwrap_or(false)
            })
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.keyword.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Parse new scriptlets from file (empty if deleted)
        let new_scripts_scriptlets = if is_deleted {
            vec![]
        } else {
            scripts::read_scriptlets_from_file(path)
        };

        let new_scriptlets: Vec<CachedScriptlet> = new_scripts_scriptlets
            .iter()
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.keyword.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // ALWAYS update keyword triggers when a file changes
        // This is needed because the diff only tracks registration metadata (name, shortcut, keyword, alias)
        // but NOT the actual content. So content changes like "success three" -> "success four"
        // would be missed if we only update on diff changes.
        #[cfg(target_os = "macos")]
        {
            let (added, removed, updated) =
                crate::keyword_manager::update_keyword_triggers_for_file(
                    path,
                    &new_scripts_scriptlets,
                );
            if added > 0 || removed > 0 || updated > 0 {
                logging::log(
                    "KEYWORD",
                    &format!(
                        "Updated keyword triggers for {}: {} added, {} removed, {} updated",
                        path.display(),
                        added,
                        removed,
                        updated
                    ),
                );
            }
        }

        // Compute diff for registration metadata changes (shortcuts, aliases)
        let diff = diff_scriptlets(&old_scriptlets, &new_scriptlets);

        if diff.is_empty() {
            logging::log(
                "APP",
                &format!("No registration metadata changes in {}", path.display()),
            );
            // Still need to update the scriptlets list even if no registration changes
            // because the content might have changed
        } else {
            logging::log(
                "APP",
                &format!(
                    "Scriptlet diff: {} added, {} removed, {} shortcut changes, {} keyword changes, {} alias changes",
                    diff.added.len(),
                    diff.removed.len(),
                    diff.shortcut_changes.len(),
                    diff.keyword_changes.len(),
                    diff.alias_changes.len()
                ),
            );
        }

        // Apply hotkey changes
        for removed in &diff.removed {
            if removed.shortcut.is_some() {
                if let Err(e) = hotkeys::unregister_script_hotkey(&removed.file_path) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister hotkey for {}: {}", removed.name, e),
                    );
                }
            }
        }

        for added in &diff.added {
            if let Some(ref shortcut) = added.shortcut {
                if let Err(e) = hotkeys::register_script_hotkey(&added.file_path, shortcut) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register hotkey for {}: {}", added.name, e),
                    );
                }
            }
        }

        for change in &diff.shortcut_changes {
            if let Err(e) = hotkeys::update_script_hotkey(
                &change.file_path,
                change.old.as_deref(),
                change.new.as_deref(),
            ) {
                logging::log(
                    "HOTKEY",
                    &format!("Failed to update hotkey for {}: {}", change.name, e),
                );
            }
        }

        // Update the scriptlets list
        // Remove old scriptlets from this file
        let path_str = path.to_string_lossy().to_string();
        self.scriptlets.retain(|s| {
            !s.file_path
                .as_ref()
                .map(|fp| fp.starts_with(&path_str))
                .unwrap_or(false)
        });

        // Add new scriptlets from this file
        self.scriptlets.extend(new_scripts_scriptlets);

        // Sort by name to maintain consistent ordering
        self.scriptlets.sort_by(|a, b| a.name.cmp(&b.name));

        // Invalidate caches
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Sync list component state so GPUI renders the correct item count
        self.sync_list_state();
        self.validate_selection_bounds(cx);

        // Rebuild alias/shortcut registries for this file's scriptlets
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx);
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet file updated incrementally: {} now has {} total scriptlets",
                path.display(),
                self.scriptlets.len()
            ),
        );

        cx.notify();
    }

    /// Get unified filtered results combining scripts and scriptlets
    /// Helper to get filter text as string (for compatibility with existing code)
    fn filter_text(&self) -> &str {
        self.filter_text.as_str()
    }

    /// P1: Now uses caching - invalidates only when filter_text changes
    fn filtered_results(&self) -> Vec<scripts::SearchResult> {
        let filter_text = self.filter_text();
        // P1: Return cached results if filter hasn't changed
        if filter_text == self.filter_cache_key {
            logging::log_debug("CACHE", &format!("Filter cache HIT for '{}'", filter_text));
            return self.cached_filtered_results.clone();
        }

        // P1: Cache miss - need to recompute (will be done by get_filtered_results_mut)
        logging::log_debug(
            "CACHE",
            &format!(
                "Filter cache MISS - need recompute for '{}' (cached key: '{}')",
                filter_text, self.filter_cache_key
            ),
        );

        // PERF: Measure search time (only log when actually filtering)
        let search_start = std::time::Instant::now();
        let results = scripts::fuzzy_search_unified(&self.scripts, &self.scriptlets, filter_text);
        let search_elapsed = search_start.elapsed();

        // Only log search performance when there's an active filter
        if !filter_text.is_empty() {
            logging::log(
                "PERF",
                &format!(
                    "Search '{}' took {:.2}ms ({} results from {} total)",
                    filter_text,
                    search_elapsed.as_secs_f64() * 1000.0,
                    results.len(),
                    self.scripts.len() + self.scriptlets.len()
                ),
            );
        }
        results
    }

    /// P1: Get filtered results with cache update (mutable version)
    /// Call this when you need to ensure cache is updated
    fn get_filtered_results_cached(&mut self) -> &Vec<scripts::SearchResult> {
        if self.filter_text != self.filter_cache_key {
            logging::log(
                "FILTER_PERF",
                &format!(
                    "[4a/5] SEARCH_START for '{}' (scripts={}, scriptlets={}, builtins={}, apps={})",
                    self.filter_text,
                    self.scripts.len(),
                    self.scriptlets.len(),
                    self.builtin_entries.len(),
                    self.apps.len()
                ),
            );
            let search_start = std::time::Instant::now();
            self.cached_filtered_results = scripts::fuzzy_search_unified_all(
                &self.scripts,
                &self.scriptlets,
                &self.builtin_entries,
                &self.apps,
                &self.filter_text,
            );
            self.filter_cache_key = self.filter_text.clone();
            let search_elapsed = search_start.elapsed();

            logging::log(
                "FILTER_PERF",
                &format!(
                    "[4a/5] SEARCH_DONE '{}' in {:.2}ms -> {} results",
                    self.filter_text,
                    search_elapsed.as_secs_f64() * 1000.0,
                    self.cached_filtered_results.len(),
                ),
            );
        }
        // NOTE: Removed cache HIT log - fires every render, only log MISS for diagnostics
        &self.cached_filtered_results
    }

    /// P1: Invalidate filter cache (call when scripts/scriptlets change)
    #[allow(dead_code)]
    fn invalidate_filter_cache(&mut self) {
        logging::log_debug("CACHE", "Filter cache INVALIDATED");
        self.filter_cache_key = String::from("\0_INVALIDATED_\0");
    }

    /// P1: Get grouped results with caching - avoids recomputing 9+ times per keystroke
    ///
    /// This is the ONLY place that should call scripts::get_grouped_results().
    /// P3: Cache is keyed off computed_filter_text (not filter_text) for two-stage filtering.
    ///
    /// P1-Arc: Returns Arc clones for cheap sharing with render closures.
    fn get_grouped_results_cached(
        &mut self,
    ) -> (Arc<[GroupedListItem]>, Arc<[scripts::SearchResult]>) {
        // P3: Key off computed_filter_text for two-stage filtering
        if self.computed_filter_text == self.grouped_cache_key {
            // NOTE: Removed cache HIT log - fires every render frame, causing log spam.
            // Cache hits are normal operation. Only log cache MISS (below) for diagnostics.
            return (
                self.cached_grouped_items.clone(),
                self.cached_grouped_flat_results.clone(),
            );
        }

        // Cache miss - need to recompute
        logging::log(
            "FILTER_PERF",
            &format!("[4b/5] GROUP_START for '{}'", self.computed_filter_text),
        );

        let start = std::time::Instant::now();
        let suggested_config = self.config.get_suggested();

        // Get menu bar items from the background tracker (pre-fetched when apps activate)
        #[cfg(target_os = "macos")]
        let (menu_bar_items, menu_bar_bundle_id): (
            Vec<menu_bar::MenuBarItem>,
            Option<String>,
        ) = {
            let cached = frontmost_app_tracker::get_cached_menu_items();
            let bundle_id = frontmost_app_tracker::get_last_real_app().map(|a| a.bundle_id);
            // No conversion needed - tracker is compiled as part of binary crate
            // so it already returns binary crate types
            (cached, bundle_id)
        };
        #[cfg(not(target_os = "macos"))]
        let (menu_bar_items, menu_bar_bundle_id): (
            Vec<menu_bar::MenuBarItem>,
            Option<String>,
        ) = (Vec::new(), None);

        logging::log(
            "APP",
            &format!(
                "get_grouped_results: filter='{}', menu_bar_items={}, bundle_id={:?}",
                self.computed_filter_text,
                menu_bar_items.len(),
                menu_bar_bundle_id
            ),
        );
        let (grouped_items, flat_results) = get_grouped_results(
            &self.scripts,
            &self.scriptlets,
            &self.builtin_entries,
            &self.apps,
            &self.frecency_store,
            &self.computed_filter_text,
            &suggested_config,
            &menu_bar_items,
            menu_bar_bundle_id.as_deref(),
        );
        let elapsed = start.elapsed();

        // P1-Arc: Convert to Arc<[T]> for cheap clone
        self.cached_grouped_items = grouped_items.into();
        self.cached_grouped_flat_results = flat_results.into();
        self.grouped_cache_key = self.computed_filter_text.clone();

        logging::log(
            "FILTER_PERF",
            &format!(
                "[4b/5] GROUP_DONE '{}' in {:.2}ms -> {} items (from {} results)",
                self.computed_filter_text,
                elapsed.as_secs_f64() * 1000.0,
                self.cached_grouped_items.len(),
                self.cached_grouped_flat_results.len()
            ),
        );

        // Log total time from input to grouped results if we have the start time
        if let Some(perf_start) = self.filter_perf_start {
            let total_elapsed = perf_start.elapsed();
            logging::log(
                "FILTER_PERF",
                &format!(
                    "[5/5] TOTAL_TIME '{}': {:.2}ms (input->grouped)",
                    self.computed_filter_text,
                    total_elapsed.as_secs_f64() * 1000.0
                ),
            );
        }

        (
            self.cached_grouped_items.clone(),
            self.cached_grouped_flat_results.clone(),
        )
    }

    /// P1: Invalidate grouped results cache (call when scripts/scriptlets/apps change)
    fn invalidate_grouped_cache(&mut self) {
        logging::log_debug("CACHE", "Grouped cache INVALIDATED");
        // Set grouped_cache_key to a sentinel that won't match computed_filter_text.
        // This ensures the cache check (computed_filter_text == grouped_cache_key) fails,
        // forcing a recompute on the next get_grouped_results_cached() call.
        // DO NOT set computed_filter_text here - that would cause both to match (false cache HIT).
        self.grouped_cache_key = String::from("\0_INVALIDATED_\0");
    }

    /// Get the currently selected search result, correctly mapping from grouped index.
    ///
    /// This function handles the mapping from `selected_index` (which is the visual
    /// position in the grouped list including section headers) to the actual
    /// `SearchResult` in the flat results array.
    ///
    /// Returns `None` if:
    /// - The selected index points to a section header (headers aren't selectable)
    /// - The selected index is out of bounds
    /// - No results exist
    fn get_selected_result(&mut self) -> Option<scripts::SearchResult> {
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();

        match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        }
    }

    /// Get or update the preview cache for syntax-highlighted code lines.
    /// Only re-reads and re-highlights when the script path actually changes.
    /// Returns cached lines if path matches, otherwise updates cache and returns new lines.
    fn get_or_update_preview_cache(
        &mut self,
        script_path: &str,
        lang: &str,
        is_dark: bool,
    ) -> &[syntax::HighlightedLine] {
        // Check if cache is valid for this path
        if self.preview_cache_path.as_deref() == Some(script_path)
            && !self.preview_cache_lines.is_empty()
        {
            // NOTE: Removed cache HIT log - fires every render, only log MISS for diagnostics
            return &self.preview_cache_lines;
        }

        // Cache miss - need to re-read and re-highlight
        let cache_miss_start = std::time::Instant::now();
        logging::log(
            "FILTER_PERF",
            &format!("[PREVIEW_CACHE_MISS] Loading '{}'", script_path),
        );

        self.preview_cache_path = Some(script_path.to_string());

        let read_start = std::time::Instant::now();
        self.preview_cache_lines = match std::fs::read_to_string(script_path) {
            Ok(content) => {
                let read_elapsed = read_start.elapsed();

                // Only take first 15 lines for preview
                let highlight_start = std::time::Instant::now();
                let preview: String = content.lines().take(15).collect::<Vec<_>>().join("\n");
                let lines = syntax::highlight_code_lines(&preview, lang, is_dark);
                let highlight_elapsed = highlight_start.elapsed();

                logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[PREVIEW_CACHE_MISS] read={:.2}ms highlight={:.2}ms ({} bytes, {} lines)",
                        read_elapsed.as_secs_f64() * 1000.0,
                        highlight_elapsed.as_secs_f64() * 1000.0,
                        content.len(),
                        lines.len()
                    ),
                );

                lines
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to read preview: {}", e));
                Vec::new()
            }
        };

        let cache_miss_elapsed = cache_miss_start.elapsed();
        logging::log(
            "FILTER_PERF",
            &format!(
                "[PREVIEW_CACHE_MISS] Total={:.2}ms for '{}'",
                cache_miss_elapsed.as_secs_f64() * 1000.0,
                script_path
            ),
        );

        &self.preview_cache_lines
    }

    /// Invalidate the preview cache (call when selection might change to different script)
    #[allow(dead_code)]
    fn invalidate_preview_cache(&mut self) {
        self.preview_cache_path = None;
        self.preview_cache_lines.clear();
    }

    #[allow(dead_code)]
    fn filtered_scripts(&self) -> Vec<Arc<scripts::Script>> {
        let filter_text = self.filter_text();
        if filter_text.is_empty() {
            self.scripts.clone()
        } else {
            let filter_lower = filter_text.to_lowercase();
            self.scripts
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        }
    }

    /// Find a script or scriptlet by alias (case-insensitive exact match)
    /// Uses O(1) registry lookup instead of O(n) iteration
    fn find_alias_match(&self, alias: &str) -> Option<AliasMatch> {
        let alias_lower = alias.to_lowercase();

        // O(1) lookup in registry
        if let Some(command_id) = self.alias_registry.get(&alias_lower) {
            // Check for builtin/{id} command IDs
            if let Some(builtin_id) = command_id.strip_prefix("builtin/") {
                let config = crate::config::BuiltInConfig::default();
                if let Some(entry) = builtins::get_builtin_entries(&config)
                    .into_iter()
                    .find(|e| e.id == builtin_id)
                {
                    logging::log(
                        "ALIAS",
                        &format!("Found builtin match: '{}' -> '{}'", alias, entry.name),
                    );
                    return Some(AliasMatch::BuiltIn(std::sync::Arc::new(entry)));
                }
            }

            // Check for app/{bundle_id} command IDs
            if let Some(bundle_id) = command_id.strip_prefix("app/") {
                if let Some(app) = self
                    .apps
                    .iter()
                    .find(|a| a.bundle_id.as_deref() == Some(bundle_id))
                {
                    logging::log(
                        "ALIAS",
                        &format!("Found app match: '{}' -> '{}'", alias, app.name),
                    );
                    return Some(AliasMatch::App(std::sync::Arc::new(app.clone())));
                }
            }

            // Find the script/scriptlet by path
            for script in &self.scripts {
                if script.path.to_string_lossy() == *command_id {
                    logging::log(
                        "ALIAS",
                        &format!("Found script match: '{}' -> '{}'", alias, script.name),
                    );
                    return Some(AliasMatch::Script(script.clone()));
                }
            }

            // Check scriptlets by file_path or name
            for scriptlet in &self.scriptlets {
                let scriptlet_path = scriptlet.file_path.as_ref().unwrap_or(&scriptlet.name);
                if scriptlet_path == command_id {
                    logging::log(
                        "ALIAS",
                        &format!("Found scriptlet match: '{}' -> '{}'", alias, scriptlet.name),
                    );
                    return Some(AliasMatch::Scriptlet(scriptlet.clone()));
                }
            }

            // Command ID in registry but not found (stale entry)
            logging::log(
                "ALIAS",
                &format!(
                    "Stale registry entry: '{}' -> '{}' (not found)",
                    alias, command_id
                ),
            );
        }

        None
    }

    fn execute_selected(&mut self, cx: &mut Context<Self>) {
        // Record input to history if filter has meaningful text
        if !self.filter_text.trim().is_empty() {
            self.input_history.add_entry(&self.filter_text);
            if let Err(e) = self.input_history.save() {
                tracing::warn!("Failed to save input history: {}", e);
            }
        }

        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the grouped item at selected_index and extract the result index
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            Some(GroupedListItem::SectionHeader(..)) => None, // Section headers are not selectable
            None => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx).cloned() {
                // Record frecency usage before executing (unless excluded)
                let frecency_path: Option<String> = match &result {
                    scripts::SearchResult::Script(sm) => {
                        Some(sm.script.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::App(am) => {
                        Some(am.app.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::BuiltIn(bm) => {
                        // Skip frecency tracking for excluded builtins (e.g., "Quit Script Kit")
                        let excluded = &self.config.get_suggested().excluded_commands;
                        if bm.entry.should_exclude_from_frecency(excluded) {
                            None
                        } else {
                            Some(format!("builtin:{}", bm.entry.name))
                        }
                    }
                    scripts::SearchResult::Scriptlet(sm) => {
                        Some(format!("scriptlet:{}", sm.scriptlet.name))
                    }
                    scripts::SearchResult::Window(wm) => {
                        Some(format!("window:{}:{}", wm.window.app, wm.window.title))
                    }
                    scripts::SearchResult::Agent(am) => {
                        Some(format!("agent:{}", am.agent.path.to_string_lossy()))
                    }
                    // Fallbacks don't track frecency - they're utility commands
                    scripts::SearchResult::Fallback(_) => None,
                };
                if let Some(path) = frecency_path {
                    self.frecency_store.record_use(&path);
                    self.frecency_store.save().ok(); // Best-effort save
                    self.invalidate_grouped_cache(); // Invalidate cache so next show reflects frecency
                }

                // Log the action being performed (matches button text from get_default_action_text())
                let action_text = result.get_default_action_text();
                logging::log(
                    "EXEC",
                    &format!(
                        "Action: '{}' on '{}' (type: {})",
                        action_text,
                        result.name(),
                        result.type_label()
                    ),
                );

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        self.execute_interactive(&script_match.script, cx);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        self.execute_scriptlet(&scriptlet_match.scriptlet, cx);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        self.execute_builtin(&builtin_match.entry, cx);
                    }
                    scripts::SearchResult::App(app_match) => {
                        self.execute_app(&app_match.app, cx);
                    }
                    scripts::SearchResult::Window(window_match) => {
                        self.execute_window_focus(&window_match.window, cx);
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        // TODO: Implement agent execution via mdflow
                        self.last_output = Some(SharedString::from(format!(
                            "Agent execution not yet implemented: {}",
                            agent_match.agent.name
                        )));
                    }
                    scripts::SearchResult::Fallback(fallback_match) => {
                        // Execute the fallback with the current filter text as input
                        self.execute_fallback_item(&fallback_match.fallback, cx);
                    }
                }
            }
        }
    }

    /// Execute a fallback item (from the "Use with..." section in search results)
    /// This is called when a fallback is selected from the grouped list
    pub fn execute_fallback_item(
        &mut self,
        fallback: &crate::fallbacks::FallbackItem,
        cx: &mut Context<Self>,
    ) {
        let input = self.filter_text.clone();

        logging::log(
            "EXEC",
            &format!(
                "Executing fallback item: {} with input: '{}'",
                fallback.name(),
                input
            ),
        );

        // Check if this is a "stay open" action (like run-in-terminal which opens a view)
        // Check if this is a "stay open" action (opens its own view)
        let should_close = match fallback {
            crate::fallbacks::FallbackItem::Builtin(builtin) => {
                !matches!(builtin.id, "run-in-terminal" | "search-files")
            }
            crate::fallbacks::FallbackItem::Script(_) => false,
        };

        // Execute the fallback action
        match fallback {
            crate::fallbacks::FallbackItem::Builtin(builtin) => {
                let fallback_id = builtin.id.to_string();
                self.execute_builtin_fallback_inline(&fallback_id, &input, cx);
            }
            crate::fallbacks::FallbackItem::Script(config) => {
                self.execute_interactive(&config.script, cx);
            }
        }

        // Close the window after executing (unless it's a stay-open action)
        if should_close {
            self.close_and_reset_window(cx);
        }
    }

    /// Execute the currently selected fallback command
    /// This is called from keyboard handler, so we need to defer window access
    pub fn execute_selected_fallback(&mut self, cx: &mut Context<Self>) {
        if !self.fallback_mode || self.cached_fallbacks.is_empty() {
            return;
        }

        let input = self.filter_text.clone();
        if let Some(fallback) = self
            .cached_fallbacks
            .get(self.fallback_selected_index)
            .cloned()
        {
            logging::log("EXEC", &format!("Executing fallback: {}", fallback.name()));

            // Check if this is a "stay open" action (opens its own view)
            let should_close = match &fallback {
                crate::fallbacks::FallbackItem::Builtin(builtin) => {
                    !matches!(builtin.id, "run-in-terminal" | "search-files")
                }
                crate::fallbacks::FallbackItem::Script(_) => false,
            };

            // Execute the fallback action
            match &fallback {
                crate::fallbacks::FallbackItem::Builtin(builtin) => {
                    let fallback_id = builtin.id.to_string();
                    self.execute_builtin_fallback_inline(&fallback_id, &input, cx);
                }
                crate::fallbacks::FallbackItem::Script(config) => {
                    self.execute_interactive(&config.script, cx);
                }
            }

            // Close the window after executing (unless it's a stay-open action)
            if should_close {
                self.close_and_reset_window(cx);
            }
        }
    }

    /// Execute a built-in fallback action without window reference
    fn execute_builtin_fallback_inline(
        &mut self,
        fallback_id: &str,
        input: &str,
        cx: &mut Context<Self>,
    ) {
        use crate::fallbacks::builtins::{get_builtin_fallbacks, FallbackResult};

        logging::log(
            "FALLBACK",
            &format!("Executing fallback '{}' with input: {}", fallback_id, input),
        );

        // Find the fallback by ID
        let fallbacks = get_builtin_fallbacks();
        let fallback = fallbacks.iter().find(|f| f.id == fallback_id);

        let Some(fallback) = fallback else {
            logging::log("FALLBACK", &format!("Unknown fallback ID: {}", fallback_id));
            return;
        };

        // Execute the fallback and get the result
        match fallback.execute(input) {
            Ok(result) => match result {
                FallbackResult::RunTerminal { command } => {
                    logging::log("FALLBACK", &format!("RunTerminal: {}", command));
                    // Open the built-in terminal with the command
                    self.open_terminal_with_command(command, cx);
                }
                FallbackResult::AddNote { content } => {
                    logging::log("FALLBACK", &format!("AddNote: {}", content));
                    let item = gpui::ClipboardItem::new_string(content);
                    cx.write_to_clipboard(item);
                    if let Err(e) = crate::notes::open_notes_window(cx) {
                        logging::log("FALLBACK", &format!("Failed to open Notes: {}", e));
                    }
                }
                FallbackResult::Copy { text } => {
                    logging::log("FALLBACK", &format!("Copy: {} chars", text.len()));
                    let item = gpui::ClipboardItem::new_string(text);
                    cx.write_to_clipboard(item);
                    crate::hud_manager::show_hud("Copied to clipboard".to_string(), Some(1500), cx);
                }
                FallbackResult::OpenUrl { url } => {
                    logging::log("FALLBACK", &format!("OpenUrl: {}", url));
                    let _ = open::that(&url);
                }
                FallbackResult::Calculate { expression } => {
                    // Evaluate the expression using meval
                    logging::log("FALLBACK", &format!("Calculate: {}", expression));
                    match meval::eval_str(&expression) {
                        Ok(result) => {
                            let item = gpui::ClipboardItem::new_string(result.to_string());
                            cx.write_to_clipboard(item);
                            crate::hud_manager::show_hud(
                                format!("{} = {}", expression, result),
                                Some(3000),
                                cx,
                            );
                        }
                        Err(e) => {
                            logging::log("FALLBACK", &format!("Calculate error: {}", e));
                            crate::hud_manager::show_hud(format!("Error: {}", e), Some(3000), cx);
                        }
                    }
                }
                FallbackResult::OpenFile { path } => {
                    logging::log("FALLBACK", &format!("OpenFile: {}", path));
                    let expanded = if path.starts_with("~") {
                        if let Some(home) = dirs::home_dir() {
                            path.replacen("~", &home.to_string_lossy(), 1)
                        } else {
                            path.clone()
                        }
                    } else {
                        path.clone()
                    };
                    let _ = open::that(&expanded);
                }
                FallbackResult::SearchFiles { query } => {
                    logging::log("FALLBACK", &format!("SearchFiles: {}", query));
                    self.open_file_search(query, cx);
                }
            },
            Err(e) => {
                logging::log("FALLBACK", &format!("Fallback execution error: {}", e));
            }
        }
    }

    fn handle_filter_input_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let handler_start = std::time::Instant::now();

        if self.suppress_filter_events {
            return;
        }

        // Skip filter updates when actions popup is open
        // (text input should go to actions dialog search, not main filter)
        if self.show_actions_popup {
            return;
        }

        let new_text = self.gpui_input_state.read(cx).value().to_string();

        // Sync filter to builtin views that use the shared input
        match &mut self.current_view {
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => {
                if *filter != new_text {
                    *filter = new_text.clone();
                    *selected_index = 0;
                    self.clipboard_list_scroll_handle
                        .scroll_to_item(0, ScrollStrategy::Top);
                }
                let filtered_entries: Vec<_> = if filter.is_empty() {
                    self.cached_clipboard_entries.iter().enumerate().collect()
                } else {
                    let filter_lower = filter.to_lowercase();
                    self.cached_clipboard_entries
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.text_preview.to_lowercase().contains(&filter_lower))
                        .collect()
                };
                self.focused_clipboard_entry_id = filtered_entries
                    .get(*selected_index)
                    .map(|(_, entry)| entry.id.clone());
                cx.notify();
                return; // Don't run main menu filter logic
            }
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => {
                if *filter != new_text {
                    *filter = new_text.clone();
                    *selected_index = 0;
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => {
                if *filter != new_text {
                    *filter = new_text.clone();
                    *selected_index = 0;
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::FileSearchView {
                query,
                selected_index,
            } => {
                if *query != new_text {
                    logging::log(
                        "SEARCH",
                        &format!(
                            "Query changed: '{}' -> '{}' (cached_results={}, display_indices={})",
                            query,
                            new_text,
                            self.cached_file_results.len(),
                            self.file_search_display_indices.len()
                        ),
                    );

                    // Get old filter BEFORE updating query (for frozen filter during transitions)
                    let old_filter =
                        if let Some(old_parsed) = crate::file_search::parse_directory_path(query) {
                            old_parsed.filter
                        } else if !query.is_empty() {
                            Some(query.clone())
                        } else {
                            None
                        };

                    // Update query immediately for responsive UI
                    *query = new_text.clone();
                    *selected_index = 0;

                    // CRITICAL: Increment generation and cancel previous search
                    // This ensures stale results are ignored AND mdfind process is killed
                    self.file_search_gen += 1;
                    let gen = self.file_search_gen;
                    logging::log("SEARCH", &format!("Generation incremented to {}", gen));

                    // Cancel any in-flight search by setting the cancel token
                    if let Some(cancel) = self.file_search_cancel.take() {
                        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    // Cancel existing debounce task (drops the Task, stopping the async work)
                    self.file_search_debounce_task = None;

                    // Check if this is a directory path with potential filter
                    // e.g., ~/dev/fin -> list ~/dev/ and filter by "fin"
                    if let Some(parsed) = crate::file_search::parse_directory_path(&new_text) {
                        // Directory path mode - check if we need to reload directory
                        let dir_changed =
                            self.file_search_current_dir.as_ref() != Some(&parsed.directory);

                        if dir_changed {
                            // Directory changed - need to load new directory contents
                            // DON'T clear results - keep old results with frozen filter
                            // This prevents visual flash during directory transitions
                            // Freeze the OLD filter so old results display correctly
                            self.file_search_frozen_filter = Some(old_filter);
                            self.file_search_current_dir = Some(parsed.directory.clone());
                            self.file_search_loading = true;
                            // Don't reset scroll - keep position stable during transition
                            cx.notify();

                            // Create new cancel token for this search
                            let cancel = crate::file_search::new_cancel_token();
                            self.file_search_cancel = Some(cancel.clone());

                            let dir_to_list = parsed.directory.clone();
                            let task = cx.spawn(async move |this, cx| {
                                // Small debounce for directory listing (reduced from 50ms to 30ms)
                                Timer::after(std::time::Duration::from_millis(30)).await;

                                // Use channel for streaming results
                                let (tx, rx) = std::sync::mpsc::channel();

                                // Start streaming directory listing in background thread
                                std::thread::spawn({
                                    let cancel = cancel.clone();
                                    let dir = dir_to_list.clone();
                                    move || {
                                        crate::file_search::list_directory_streaming(
                                            &dir,
                                            cancel,
                                            false, // include metadata
                                            |event| {
                                                let _ = tx.send(event);
                                            },
                                        );
                                    }
                                });

                                let mut pending: Vec<crate::file_search::FileResult> = Vec::new();
                                let mut done = false;
                                let mut first_batch = true; // Track if we need to clear old results

                                // Batch UI updates at ~60fps (16ms intervals)
                                while !done {
                                    Timer::after(std::time::Duration::from_millis(16)).await;

                                    // Drain all available results
                                    while let Ok(event) = rx.try_recv() {
                                        match event {
                                            crate::file_search::SearchEvent::Result(r) => {
                                                pending.push(r);
                                            }
                                            crate::file_search::SearchEvent::Done => {
                                                done = true;
                                                break;
                                            }
                                        }
                                    }

                                    // Update UI with batched results
                                    if !pending.is_empty() || done {
                                        let batch = std::mem::take(&mut pending);
                                        let is_done = done;
                                        let is_first = first_batch;
                                        first_batch = false;
                                        let _ = cx.update(|cx| {
                                            this.update(cx, |app, cx| {
                                                // Ignore stale generations
                                                if app.file_search_gen != gen {
                                                    return;
                                                }

                                                // Clear old results on first batch to prevent accumulation
                                                // This happens AFTER debounce so frozen filter had time to display
                                                if is_first {
                                                    app.cached_file_results.clear();
                                                }

                                                // Append batch
                                                for r in batch {
                                                    app.cached_file_results.push(r);
                                                }

                                                if is_done {
                                                    app.file_search_loading = false;
                                                    // Clear frozen filter - now using real results
                                                    app.file_search_frozen_filter = None;
                                                    // Sort by directories first, then alphabetically
                                                    app.sort_directory_results();
                                                    // Recompute display indices after loading completes
                                                    app.recompute_file_search_display_indices();
                                                    // Reset selected_index when results finish loading
                                                    if let AppView::FileSearchView {
                                                        selected_index,
                                                        ..
                                                    } = &mut app.current_view
                                                    {
                                                        *selected_index = 0;
                                                    }
                                                    app.file_search_scroll_handle
                                                        .scroll_to_item(0, ScrollStrategy::Top);
                                                }

                                                cx.notify();
                                            })
                                        });
                                    }
                                }
                            });
                            self.file_search_debounce_task = Some(task);
                        } else {
                            // Same directory - just filter existing results (instant!)
                            // Clear any frozen filter since we're not in transition
                            self.file_search_frozen_filter = None;
                            self.file_search_loading = false;
                            // Recompute display indices for new filter
                            self.recompute_file_search_display_indices();
                            cx.notify();
                        }
                        return; // Don't run main menu filter logic
                    }

                    // Not a directory path - do regular file search with streaming
                    logging::log(
                        "SEARCH",
                        &format!("Starting mdfind search for query: '{}'", new_text),
                    );
                    self.file_search_current_dir = None;
                    self.file_search_loading = true;
                    // Clear cached results for new search
                    self.cached_file_results.clear();
                    self.file_search_display_indices.clear();
                    cx.notify();

                    // Create new cancel token for this search
                    let cancel = crate::file_search::new_cancel_token();
                    self.file_search_cancel = Some(cancel.clone());

                    // Shorter debounce for streaming (75ms instead of 200ms)
                    let search_query = new_text.clone();
                    let task = cx.spawn(async move |this, cx| {
                        // Wait for debounce period
                        Timer::after(std::time::Duration::from_millis(75)).await;

                        // Use channel for streaming results
                        let (tx, rx) = std::sync::mpsc::channel();

                        // Start streaming search in background thread
                        std::thread::spawn({
                            let cancel = cancel.clone();
                            let q = search_query.clone();
                            move || {
                                crate::file_search::search_files_streaming(
                                    &q,
                                    None,
                                    crate::file_search::DEFAULT_SEARCH_LIMIT,
                                    cancel,
                                    false, // include metadata (can set true for faster first results)
                                    |event| {
                                        let _ = tx.send(event);
                                    },
                                );
                            }
                        });

                        let mut pending: Vec<crate::file_search::FileResult> = Vec::new();
                        let mut done = false;

                        // Batch UI updates at ~60fps (16ms intervals)
                        while !done {
                            Timer::after(std::time::Duration::from_millis(16)).await;

                            // Drain all available results
                            while let Ok(event) = rx.try_recv() {
                                match event {
                                    crate::file_search::SearchEvent::Result(r) => {
                                        pending.push(r);
                                    }
                                    crate::file_search::SearchEvent::Done => {
                                        done = true;
                                        break;
                                    }
                                }
                            }

                            // Update UI with batched results
                            if !pending.is_empty() || done {
                                let batch = std::mem::take(&mut pending);
                                let batch_count = batch.len();
                                let is_done = done;
                                let query_for_log = search_query.clone();
                                let _ = cx.update(|cx| {
                                    this.update(cx, |app, cx| {
                                        // Ignore stale generations
                                        if app.file_search_gen != gen {
                                            return;
                                        }

                                        // Verify query still matches (extra safety)
                                        if let AppView::FileSearchView { query, .. } =
                                            &app.current_view
                                        {
                                            if *query != query_for_log {
                                                return;
                                            }
                                        }

                                        // Append batch
                                        let old_count = app.cached_file_results.len();
                                        for r in batch {
                                            app.cached_file_results.push(r);
                                        }

                                        if is_done {
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "File search for '{}' found {} results (streaming)",
                                                    query_for_log,
                                                    app.cached_file_results.len()
                                                ),
                                            );
                                            app.file_search_loading = false;
                                            // Recompute display indices now that all results are in
                                            app.recompute_file_search_display_indices();
                                            // Reset selected_index when search completes
                                            if let AppView::FileSearchView {
                                                selected_index,
                                                ..
                                            } = &mut app.current_view
                                            {
                                                *selected_index = 0;
                                            }
                                            app.file_search_scroll_handle
                                                .scroll_to_item(0, ScrollStrategy::Top);
                                        } else if batch_count > 0 && old_count == 0 {
                                            // First batch arrived - log it
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "File search first batch: {} results",
                                                    batch_count
                                                ),
                                            );
                                        }

                                        cx.notify();
                                    })
                                });
                            }
                        }
                    });

                    // Store task so it can be cancelled if user types more
                    self.file_search_debounce_task = Some(task);
                }
                return; // Don't run main menu filter logic
            }
            _ => {} // Continue with main menu logic
        }
        if new_text == self.filter_text {
            return;
        }

        let previous_text = std::mem::replace(&mut self.filter_text, new_text.clone());

        // Reset input history navigation when user types (they're no longer navigating history)
        self.input_history.reset_navigation();

        // FIX: Don't reset selected_index here - do it in queue_filter_compute() callback
        // AFTER computed_filter_text is updated. This prevents a race condition where:
        // 1. We set selected_index=0 immediately
        // 2. Render runs before async cache update
        // 3. Stale grouped_items has SectionHeader at index 0
        // 4. coerce_selection moves selection to index 1
        // Instead, we'll reset selection when the cache actually updates.
        self.last_scrolled_index = None;

        if new_text.ends_with(' ') {
            let trimmed = new_text.trim_end_matches(' ');
            if !trimmed.is_empty() && trimmed == previous_text {
                if let Some(alias_match) = self.find_alias_match(trimmed) {
                    logging::log("ALIAS", &format!("Alias '{}' triggered execution", trimmed));
                    match alias_match {
                        AliasMatch::Script(script) => {
                            self.execute_interactive(&script, cx);
                        }
                        AliasMatch::Scriptlet(scriptlet) => {
                            self.execute_scriptlet(&scriptlet, cx);
                        }
                        AliasMatch::BuiltIn(entry) => {
                            self.execute_builtin(&entry, cx);
                        }
                        AliasMatch::App(app) => {
                            self.execute_app(&app, cx);
                        }
                    }
                    self.clear_filter(window, cx);
                    return;
                }
            }
        }

        // P3: Notify immediately so UI updates (responsive typing)
        cx.notify();

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.queue_filter_compute(new_text.clone(), cx);

        // Log handler timing
        let handler_elapsed = handler_start.elapsed();
        if handler_elapsed.as_millis() > 5 {
            logging::log(
                "FILTER_PERF",
                &format!(
                    "[HANDLER_SLOW] handle_filter_input_change took {:.2}ms for '{}'",
                    handler_elapsed.as_secs_f64() * 1000.0,
                    new_text
                ),
            );
        }
    }

    fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        // P3: Debounce expensive search/window resize work.
        // Use 8ms debounce (half a frame) to batch rapid keystrokes.
        logging::log(
            "FILTER_PERF",
            &format!("[2/5] QUEUE_FILTER value='{}' len={}", value, value.len()),
        );
        if self.filter_coalescer.queue(value) {
            cx.spawn(async move |this, cx| {
                // Wait 8ms for coalescing window (half frame at 60fps)
                Timer::after(std::time::Duration::from_millis(8)).await;

                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        if let Some(latest) = app.filter_coalescer.take_latest() {
                            if app.computed_filter_text != latest {
                                let coalesce_start = std::time::Instant::now();
                                logging::log(
                                    "FILTER_PERF",
                                    &format!(
                                        "[3/5] COALESCE_PROCESS value='{}' (after 8ms debounce)",
                                        latest
                                    ),
                                );
                                app.computed_filter_text = latest.clone();
                                // Sync list component state and validate selection
                                // This moves state mutation OUT of render() (anti-pattern fix)
                                app.sync_list_state();
                                app.selected_index = 0;
                                app.validate_selection_bounds(cx);
                                app.main_list_state
                                    .scroll_to_reveal_item(app.selected_index);
                                app.last_scrolled_index = Some(app.selected_index);
                                // This will trigger window resize
                                app.update_window_size();
                                let coalesce_elapsed = coalesce_start.elapsed();
                                logging::log(
                                    "FILTER_PERF",
                                    &format!(
                                        "[3/5] COALESCE_DONE in {:.2}ms for '{}'",
                                        coalesce_elapsed.as_secs_f64() * 1000.0,
                                        latest
                                    ),
                                );
                                cx.notify();
                            }
                        }
                    })
                });
            })
            .detach();
        }
    }

    fn set_filter_text_immediate(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.suppress_filter_events = true;
        self.filter_text = text.clone();
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
            // Ensure cursor is at end with no selection after programmatic set_value
            let len = text.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.computed_filter_text = text.clone();
        self.filter_coalescer.reset();

        // Sync list component state and validate selection
        // This moves state mutation OUT of render() (anti-pattern fix)
        self.sync_list_state();
        self.selected_index = 0;
        self.validate_selection_bounds(cx);
        self.main_list_state
            .scroll_to_reveal_item(self.selected_index);
        self.last_scrolled_index = Some(self.selected_index);

        // Update fallback mode immediately based on filter results
        // This ensures SimulateKey commands can check fallback_mode correctly
        // NOTE: validate_selection_bounds already clears fallback_mode and cached_fallbacks,
        // but we need special handling for legacy SimulateKey compatibility
        if !text.is_empty() {
            let results = self.get_filtered_results_cached();
            if results.is_empty() {
                // No matches - check if we should enter fallback mode
                use crate::fallbacks::collect_fallbacks;
                let fallbacks = collect_fallbacks(&text, self.scripts.as_slice());
                if !fallbacks.is_empty() {
                    self.fallback_mode = true;
                    self.cached_fallbacks = fallbacks;
                    self.fallback_selected_index = 0;
                }
            }
        }

        self.update_window_size_deferred(window, cx);
        cx.notify();
    }

    fn clear_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_filter_text_immediate(String::new(), window, cx);
    }

    fn sync_filter_input_if_needed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Sync placeholder if pending
        if let Some(placeholder) = self.pending_placeholder.take() {
            self.gpui_input_state.update(cx, |state, cx| {
                state.set_placeholder(placeholder, window, cx);
            });
        }

        if !self.pending_filter_sync {
            return;
        }

        let desired = self.filter_text.clone();
        let current = self.gpui_input_state.read(cx).value().to_string();
        if current == desired {
            self.pending_filter_sync = false;
            return;
        }

        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(desired.clone(), window, cx);
            // Ensure cursor is at end with no selection after programmatic set_value
            let len = desired.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;
    }

    fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.show_logs = !self.show_logs;
        cx.notify();
    }

    /// Hide the mouse cursor while typing.
    /// The cursor will be shown again when the mouse moves.
    fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = true;
            crate::platform::hide_cursor_until_mouse_moves();
            cx.notify();
        }
    }

    /// Show the mouse cursor (called when mouse moves).
    /// Also switches to Mouse input mode to re-enable hover effects.
    fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        // Switch to mouse mode to re-enable hover effects
        self.input_mode = InputMode::Mouse;

        if self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = false;
            cx.notify();
        }
    }

    /// Calculate view type and item count for window sizing.
    /// Extracted from update_window_size for reuse.
    fn calculate_window_size_params(&mut self) -> Option<(ViewType, usize)> {
        match &self.current_view {
            AppView::ScriptList => {
                // Get grouped results which includes section headers (cached)
                let (grouped_items, _) = self.get_grouped_results_cached();
                let count = grouped_items.len();
                Some((ViewType::ScriptList, count))
            }
            AppView::ArgPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    Some((ViewType::ArgPromptNoChoices, 0))
                } else {
                    Some((ViewType::ArgPromptWithChoices, filtered.len()))
                }
            }
            AppView::DivPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FormPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::SelectPrompt { .. } => Some((ViewType::ArgPromptWithChoices, 0)),
            AppView::PathPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::EnvPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)), // Compact: header + footer only
            AppView::DropPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => Some((ViewType::DivPrompt, 0)), // Template prompt uses div size
            AppView::ChatPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Chat prompt uses div size
            AppView::TermPrompt { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                None
            }
            // P0 FIX: Clipboard history and app launcher use standard height (same as script list)
            // View state only - data comes from self fields
            AppView::ClipboardHistoryView { filter, .. } => {
                let entries = &self.cached_clipboard_entries;
                let filtered_count = if filter.is_empty() {
                    entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries
                        .iter()
                        .filter(|e| e.text_preview.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::AppLauncherView { filter, .. } => {
                let apps = &self.apps;
                let filtered_count = if filter.is_empty() {
                    apps.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    apps.iter()
                        .filter(|a| a.name.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::WindowSwitcherView { filter, .. } => {
                let windows = &self.cached_windows;
                let filtered_count = if filter.is_empty() {
                    windows.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    windows
                        .iter()
                        .filter(|w| {
                            w.title.to_lowercase().contains(&filter_lower)
                                || w.app.to_lowercase().contains(&filter_lower)
                        })
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::DesignGalleryView { filter, .. } => {
                // Calculate total gallery items (separators + icons)
                let total_items = designs::separator_variations::SeparatorStyle::count()
                    + designs::icon_variations::total_icon_count();
                let filtered_count = if filter.is_empty() {
                    total_items
                } else {
                    // For now, return total - filtering can be added later
                    total_items
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::ScratchPadView { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::QuickTerminalView { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::FileSearchView { ref query, .. } => {
                let results = &self.cached_file_results;
                let filtered_count = if query.is_empty() {
                    results.len()
                } else {
                    let query_lower = query.to_lowercase();
                    results
                        .iter()
                        .filter(|r| r.name.to_lowercase().contains(&query_lower))
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
        }
    }

    /// Update window size using deferred execution (SAFE during render/event cycles).
    ///
    /// Uses Window::defer to schedule the resize at the end of the current effect cycle,
    /// preventing RefCell borrow conflicts that can occur when calling platform APIs
    /// during GPUI's render or event processing.
    ///
    /// Use this version when you have access to `window` and `cx`.
    fn update_window_size_deferred(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            crate::window_resize::defer_resize_to_view(view_type, item_count, window, &mut *cx);
        }
    }

    /// Update window size synchronously.
    ///
    /// SAFETY: Only call from async handlers (cx.spawn closures, message handlers)
    /// that run OUTSIDE the GPUI render cycle. Calling during render will cause
    /// RefCell borrow panics.
    ///
    /// Prefer `update_window_size_deferred` when you have window/cx access.
    fn update_window_size(&mut self) {
        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            let target_height = height_for_view(view_type, item_count);
            resize_first_window_to_height(target_height);
        }
    }

    fn set_prompt_input(&mut self, text: String, cx: &mut Context<Self>) {
        match &mut self.current_view {
            AppView::ArgPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                self.arg_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.update_window_size();
                cx.notify();
            }
            AppView::PathPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::SelectPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::EnvPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::TemplatePrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::FormPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::FileSearchView {
                query,
                selected_index,
            } => {
                // Check if query looks like a directory path
                // If so, list directory contents instead of searching
                let results = if crate::file_search::is_directory_path(&text) {
                    crate::file_search::list_directory(
                        &text,
                        crate::file_search::DEFAULT_CACHE_LIMIT,
                    )
                } else {
                    crate::file_search::search_files(
                        &text,
                        None,
                        crate::file_search::DEFAULT_SEARCH_LIMIT,
                    )
                };
                logging::log(
                    "EXEC",
                    &format!(
                        "File search setInput '{}' found {} results",
                        text,
                        results.len()
                    ),
                );
                self.cached_file_results = results;
                *query = text.clone();
                *selected_index = 0;
                self.file_search_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                // Mark that we need to sync the input text on next render
                self.filter_text = text;
                self.pending_filter_sync = true;
                cx.notify();
            }
            _ => {}
        }
    }

    /// Helper to get filtered arg choices without cloning
    fn get_filtered_arg_choices<'a>(&self, choices: &'a [Choice]) -> Vec<&'a Choice> {
        if self.arg_input.is_empty() {
            choices.iter().collect()
        } else {
            let filter = self.arg_input.text().to_lowercase();
            choices
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&filter))
                .collect()
        }
    }

    fn focus_main_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_input = FocusedInput::MainFilter;
        let input_state = self.gpui_input_state.clone();
        input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        let popup_state = self.show_actions_popup;
        let window_open = is_actions_window_open();
        logging::log(
            "KEY",
            &format!(
                "Toggling actions popup (show_actions_popup={}, is_actions_window_open={})",
                popup_state, window_open
            ),
        );
        if self.show_actions_popup || is_actions_window_open() {
            // Close - use coordinator to restore previous focus
            self.show_actions_popup = false;
            self.actions_dialog = None;

            // Close the separate actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Use coordinator to restore focus (will pop the overlay and set pending_focus)
            self.pop_focus_overlay(cx);

            // Also directly focus main filter for immediate feedback
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "Actions closed via toggle, focus restored via coordinator",
            );
        } else {
            if !self.has_actions() {
                return;
            }
            // Open actions as a separate window with vibrancy blur
            self.show_actions_popup = true;

            // Use coordinator to push overlay - saves current focus state for restore
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // CRITICAL: Transfer focus from Input to main focus_handle
            // This prevents the Input from receiving text (which would go to main filter)
            // while keeping keyboard focus in main window for routing to actions dialog
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;

            let script_info = self.get_focused_script_info();

            // Get the full scriptlet with actions if focused item is a scriptlet
            let focused_scriptlet = self.get_focused_scriptlet_with_actions();

            // Create the dialog entity HERE in main app (for keyboard routing)
            let theme_arc = std::sync::Arc::clone(&self.theme);
            // Create the dialog entity (search input shown at bottom, Raycast-style)
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_script(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    script_info.clone(),
                    theme_arc,
                );

                // If we have a scriptlet with actions, pass it to the dialog
                if let Some(ref scriptlet) = focused_scriptlet {
                    dialog.set_focused_scriptlet(script_info.clone(), Some(scriptlet.clone()));
                }

                dialog
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            // This ensures the same cleanup happens whether closing via Cmd+K toggle or Escape
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    app_entity.update(cx, |app, cx| {
                        app.show_actions_popup = false;
                        app.actions_dialog = None;
                        // Use coordinator to pop overlay and restore previous focus
                        app.pop_focus_overlay(cx);
                        logging::log(
                            "FOCUS",
                            "Actions closed via escape, focus restored via coordinator",
                        );
                    });
                }));
            });

            // Get main window bounds and display_id for positioning the actions popup
            //
            // CRITICAL: We use GPUI's window.bounds() which returns SCREEN-RELATIVE coordinates
            // (top-left origin, relative to the window's current screen). We also capture the
            // display_id so the actions window is created on the SAME screen as the main window.
            //
            // This fixes multi-monitor issues where the actions popup would appear on the wrong
            // screen or at wrong coordinates when the main window was on a secondary display.
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Main window bounds (GPUI screen-relative): origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
                    main_bounds.origin.x, main_bounds.origin.y,
                    main_bounds.size.width, main_bounds.size.height,
                    display_id
                ),
            );

            // Open the actions window via spawn, passing the shared dialog entity and display_id
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::BottomRight,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "Actions popup window opened");
                        }
                        Err(e) => {
                            logging::log(
                                "ACTIONS",
                                &format!("Failed to open actions window: {}", e),
                            );
                        }
                    }
                })
                .ok();
            })
            .detach();

            logging::log("FOCUS", "Actions opened, keyboard routing active");
        }
        cx.notify();
    }

    /// Toggle actions dialog for arg prompts with SDK-defined actions
    fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log(
            "KEY",
            &format!(
                "toggle_arg_actions called: show_actions_popup={}, actions_dialog.is_some={}, sdk_actions.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some(),
                self.sdk_actions.is_some()
            ),
        );
        if self.show_actions_popup {
            // Close - use coordinator to restore to arg prompt
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.pop_focus_overlay(cx);
            window.focus(&self.focus_handle, cx);
            logging::log(
                "FOCUS",
                "Arg actions closed, focus restored via coordinator",
            );
        } else {
            // Clone SDK actions early to avoid borrow conflicts
            let sdk_actions_opt = self.sdk_actions.clone();

            // Check if we have SDK actions
            if let Some(sdk_actions) = sdk_actions_opt {
                logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
                if !sdk_actions.is_empty() {
                    // Open - push overlay to save arg prompt focus state
                    self.show_actions_popup = true;
                    self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

                    let theme_arc = std::sync::Arc::clone(&self.theme);
                    let dialog = cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        let mut dialog = ActionsDialog::with_script(
                            focus_handle,
                            std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                            None,                                 // No script info for arg prompts
                            theme_arc,
                        );
                        // Set SDK actions to replace built-in actions
                        dialog.set_sdk_actions(sdk_actions);
                        dialog
                    });

                    // Show search input at bottom (Raycast-style)

                    // Focus the dialog's internal focus handle
                    self.actions_dialog = Some(dialog.clone());
                    let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                    window.focus(&dialog_focus_handle, cx);
                    logging::log(
                        "FOCUS",
                        &format!(
                            "Arg actions OPENED: show_actions_popup={}, actions_dialog.is_some={}",
                            self.show_actions_popup,
                            self.actions_dialog.is_some()
                        ),
                    );
                } else {
                    logging::log("KEY", "No SDK actions available to show (empty list)");
                }
            } else {
                logging::log("KEY", "No SDK actions defined for this arg prompt (None)");
            }
        }
    }
    /// Toggle terminal command bar for built-in terminal
    /// Shows common terminal actions (Clear, Copy, Paste, Scroll, etc.)
    #[allow(dead_code)]
    pub fn toggle_terminal_commands(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{Action, ActionCategory, ActionsDialog, ActionsDialogConfig, SearchPosition, SectionStyle, AnchorPosition};
        use crate::terminal::get_terminal_commands;

        logging::log(
            "KEY",
            &format!(
                "toggle_terminal_commands called: show_actions_popup={}, actions_dialog.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some()
            ),
        );

        if self.show_actions_popup {
            // Close - use coordinator to restore focus
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.pop_focus_overlay(cx);
            window.focus(&self.focus_handle, cx);
            logging::log("FOCUS", "Terminal commands closed, focus restored");
        } else {
            // Open - create actions from terminal commands
            self.show_actions_popup = true;
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let terminal_commands = get_terminal_commands();

            // Convert terminal commands to Actions
            let actions: Vec<Action> = terminal_commands
                .into_iter()
                .map(|cmd| {
                    Action::new(
                        cmd.action.id(),
                        cmd.name.clone(),
                        Some(cmd.description.clone()),
                        ActionCategory::Terminal,
                    )
                    .with_shortcut_opt(cmd.shortcut.clone())
                })
                .collect();

            // Create dialog with terminal-style config
            let config = ActionsDialogConfig {
                search_position: SearchPosition::Bottom,
                section_style: SectionStyle::None,
                anchor: AnchorPosition::Top,
                show_icons: false,
                show_footer: false,
            };

            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_config(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}),
                    actions,
                    theme_arc,
                    config,
                )
            });

            self.actions_dialog = Some(dialog.clone());
            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
            window.focus(&dialog_focus_handle, cx);
            logging::log("FOCUS", "Terminal commands opened");
        }
    }


    /// Toggle actions dialog for chat prompts
    /// Opens ActionsDialog with model selection and chat-specific actions
    pub fn toggle_chat_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{ChatModelInfo, ChatPromptInfo};

        logging::log(
            "KEY",
            &format!(
                "toggle_chat_actions called: show_actions_popup={}, actions_dialog.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            // Close - use coordinator to restore to chat prompt
            self.show_actions_popup = false;
            self.actions_dialog = None;

            // Close the separate actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Use coordinator to pop overlay and restore previous focus
            self.pop_focus_overlay(cx);
            window.focus(&self.focus_handle, cx);
            logging::log(
                "FOCUS",
                "Chat actions closed, focus restored via coordinator",
            );
        } else {
            // Get chat info from current ChatPrompt entity
            let chat_info = if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                let chat = entity.read(cx);
                ChatPromptInfo {
                    current_model: chat.model.clone(),
                    available_models: chat
                        .models
                        .iter()
                        .map(|m| ChatModelInfo {
                            id: m.id.clone(),
                            display_name: m.name.clone(),
                            provider: m.provider.clone(),
                        })
                        .collect(),
                    has_messages: !chat.messages.is_empty(),
                    has_response: chat
                        .messages
                        .iter()
                        .any(|m| m.position == crate::protocol::ChatMessagePosition::Left),
                }
            } else {
                logging::log(
                    "KEY",
                    "toggle_chat_actions called but current view is not ChatPrompt",
                );
                return;
            };

            // Open actions as a separate window with vibrancy blur
            self.show_actions_popup = true;
            // Push overlay to save chat prompt focus state
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_chat(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &chat_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    app_entity.update(cx, |app, cx| {
                        app.show_actions_popup = false;
                        app.actions_dialog = None;
                        // Use coordinator to pop overlay and restore previous focus
                        app.pop_focus_overlay(cx);
                        logging::log(
                            "FOCUS",
                            "Chat actions closed via escape, focus restored via coordinator",
                        );
                    });
                }));
            });

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Chat actions: Main window bounds origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
                    main_bounds.origin.x, main_bounds.origin.y,
                    main_bounds.size.width, main_bounds.size.height,
                    display_id
                ),
            );

            // Open the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::BottomRight,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "Chat actions popup window opened");
                        }
                        Err(e) => {
                            logging::log(
                                "ACTIONS",
                                &format!("Failed to open chat actions window: {}", e),
                            );
                        }
                    }
                })
                .ok();
            })
            .detach();

            logging::log("FOCUS", "Chat actions opened, keyboard routing active");
        }
        cx.notify();
    }

    /// Execute an action selected from the chat actions dialog
    pub fn execute_chat_action(&mut self, action_id: &str, cx: &mut Context<Self>) {
        logging::log("ACTIONS", &format!("execute_chat_action: {}", action_id));

        // Handle model selection (action_id starts with "select_model_")
        if let Some(model_id) = action_id.strip_prefix("select_model_") {
            if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                let model_id_owned = model_id.to_string();
                entity.update(cx, |chat, cx| {
                    // Find model by ID and set it
                    if let Some(model) = chat.models.iter().find(|m| m.id == model_id_owned) {
                        chat.model = Some(model.name.clone());
                        logging::log("CHAT", &format!("Model changed to: {}", model.name));
                        cx.notify();
                    }
                });
            }
            return;
        }

        // Handle other chat actions
        match action_id {
            "continue_in_chat" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.handle_continue_in_chat(cx);
                    });
                }
            }
            "copy_response" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.handle_copy_last_response(cx);
                    });
                }
            }
            "clear_conversation" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.clear_messages(cx);
                    });
                }
            }
            _ => {
                logging::log("ACTIONS", &format!("Unknown chat action: {}", action_id));
            }
        }
    }

    // ========================================================================
    // Actions Dialog Routing - Shared key routing for all prompt types
    // ========================================================================

    /// Route keyboard events to the actions dialog when open.
    ///
    /// This centralizes the duplicated key routing logic from all render_prompts/*.rs
    /// files into a single location, eliminating ~80 lines of duplicated code per prompt.
    ///
    /// # Arguments
    /// * `key` - The key string from the KeyDownEvent (case-insensitive)
    /// * `key_char` - Optional key_char from the event for printable character input
    /// * `host` - Which type of host is routing (determines focus restoration behavior)
    /// * `window` - Window reference for focus operations
    /// * `cx` - Context for entity updates and notifications
    ///
    /// # Returns
    /// * `ActionsRoute::NotHandled` - Actions popup not open, route to normal handlers
    /// * `ActionsRoute::Handled` - Key was consumed by the actions dialog
    /// * `ActionsRoute::Execute { action_id }` - User selected an action, caller should execute it
    fn route_key_to_actions_dialog(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        modifiers: &gpui::Modifiers,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ActionsRoute {
        // Not open - let caller handle the key
        if !self.show_actions_popup {
            return ActionsRoute::NotHandled;
        }

        // Defensive: if UI says it's open but dialog is None, don't leak keys
        let Some(ref dialog) = self.actions_dialog else {
            return ActionsRoute::Handled;
        };

        // Use allocation-free key helpers from ui_foundation
        use crate::ui_foundation::{
            is_key_backspace, is_key_down, is_key_enter, is_key_escape, is_key_up, printable_char,
        };

        if is_key_up(key) {
            dialog.update(cx, |d, cx| d.move_up(cx));
            return ActionsRoute::Handled;
        }

        if is_key_down(key) {
            dialog.update(cx, |d, cx| d.move_down(cx));
            return ActionsRoute::Handled;
        }

        if is_key_enter(key) {
            let action_id = dialog.read(cx).get_selected_action_id();
            let should_close = dialog.read(cx).selected_action_should_close();

            if let Some(action_id) = action_id {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "Actions dialog executing action: {} (close={}, host={:?})",
                        action_id, should_close, host
                    ),
                );

                if should_close {
                    self.close_actions_popup(host, window, cx);
                }

                return ActionsRoute::Execute { action_id };
            }
            return ActionsRoute::Handled;
        }

        if is_key_escape(key) {
            self.close_actions_popup(host, window, cx);
            return ActionsRoute::Handled;
        }

        if is_key_backspace(key) {
            dialog.update(cx, |d, cx| d.handle_backspace(cx));
            crate::actions::notify_actions_window(cx);
            crate::actions::resize_actions_window(cx, dialog);
            return ActionsRoute::Handled;
        }

        // Check for printable character input (only when no modifiers are held)
        // This prevents Cmd+E from being treated as typing 'e' into the search
        if !modifiers.platform && !modifiers.control && !modifiers.alt {
            if let Some(ch) = printable_char(key_char) {
                dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                crate::actions::notify_actions_window(cx);
                crate::actions::resize_actions_window(cx, dialog);
                return ActionsRoute::Handled;
            }
        }

        // Check if keystroke matches any action shortcut in the dialog
        // This allows Cmd+E, Cmd+L, etc. to execute the corresponding action
        let key_lower = key.to_lowercase();
        let keystroke_shortcut = shortcuts::keystroke_to_shortcut(&key_lower, modifiers);

        // Read dialog actions and look for matching shortcut
        // First pass: find the match (if any) while holding the borrow
        let matched_action_id: Option<String> = {
            let dialog_ref = dialog.read(cx);
            dialog_ref.actions.iter().find_map(|action| {
                action.shortcut.as_ref().and_then(|display_shortcut| {
                    let normalized = Self::normalize_display_shortcut(display_shortcut);
                    if normalized == keystroke_shortcut {
                        Some(action.id.clone())
                    } else {
                        None
                    }
                })
            })
        }; // dialog_ref borrow released here

        // Second pass: execute the action if found (borrow released)
        if let Some(action_id) = matched_action_id {
            logging::log(
                "ACTIONS",
                &format!(
                    "Actions dialog shortcut matched: {} -> {} (host={:?})",
                    keystroke_shortcut, action_id, host
                ),
            );

            // Built-in actions always close the dialog
            self.close_actions_popup(host, window, cx);

            return ActionsRoute::Execute { action_id };
        }

        // Modal behavior: swallow all other keys while popup is open
        ActionsRoute::Handled
    }

    /// Convert a display shortcut (⌘⇧E) to normalized form (cmd+shift+e)
    fn normalize_display_shortcut(display: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        let mut key_char: Option<char> = None;

        for ch in display.chars() {
            match ch {
                '⌘' => parts.push("cmd"),
                '⌃' => parts.push("ctrl"),
                '⌥' => parts.push("alt"),
                '⇧' => parts.push("shift"),
                '↵' => key_char = Some('e'), // Enter - map to 'enter' below
                '⎋' => key_char = Some('`'), // Escape placeholder
                '⇥' => key_char = Some('t'), // Tab placeholder
                '⌫' => key_char = Some('b'), // Backspace placeholder
                _ => key_char = Some(ch),
            }
        }

        // Sort modifiers alphabetically (matches keystroke_to_shortcut order)
        parts.sort();

        let mut result = parts.join("+");
        if let Some(k) = key_char {
            if !result.is_empty() {
                result.push('+');
            }
            // Handle special keys
            match k {
                'e' if display.contains('↵') => result.push_str("enter"),
                '`' if display.contains('⎋') => result.push_str("escape"),
                't' if display.contains('⇥') => result.push_str("tab"),
                'b' if display.contains('⌫') => result.push_str("backspace"),
                _ => result.push_str(&k.to_lowercase().to_string()),
            }
        }

        result
    }

    /// Close the actions popup and restore focus based on host type.
    ///
    /// This centralizes close behavior, ensuring cx.notify() is always called
    /// and focus is correctly restored based on which prompt hosted the dialog.
    ///
    /// NOTE: The `host` parameter is now deprecated. Focus restoration is handled
    /// automatically by the FocusCoordinator's overlay stack. The host is kept
    /// for logging purposes only.
    fn close_actions_popup(
        &mut self,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Close the separate actions window if open
        // This ensures consistent behavior whether closing via Cmd+K, Escape, backdrop click,
        // or any other close mechanism
        if is_actions_window_open() {
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();
        }

        // Use coordinator to pop overlay and restore previous focus
        // The coordinator's stack tracks where we came from, so no need
        // to manually switch on host type anymore.
        self.pop_focus_overlay(cx);

        // Also directly focus the app root for immediate feedback
        window.focus(&self.focus_handle, cx);
        logging::log(
            "FOCUS",
            &format!(
                "Actions popup closed (host={:?}), focus restored via coordinator",
                host
            ),
        );
        // cx.notify() already called by pop_focus_overlay
    }

    /// Edit a script in configured editor (config.editor > $EDITOR > "code")
    #[allow(dead_code)]
    fn edit_script(&mut self, path: &std::path::Path) {
        let editor = self.config.get_editor();
        logging::log(
            "UI",
            &format!("Opening script in editor '{}': {}", editor, path.display()),
        );
        let path_str = path.to_string_lossy().to_string();

        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new(&editor).arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully spawned editor: {}", editor)),
                Err(e) => logging::log(
                    "ERROR",
                    &format!("Failed to spawn editor '{}': {}", editor, e),
                ),
            }
        });
    }

    /// Open config.ts for configuring a keyboard shortcut
    /// Creates the file with documentation if it doesn't exist
    ///
    /// NOTE: This is the legacy approach. For new code, use `show_shortcut_recorder()` instead
    /// which provides an inline modal UI for recording shortcuts.
    #[allow(dead_code)]
    fn open_config_for_shortcut(&mut self, command_id: &str) {
        let config_path = shellexpand::tilde("~/.scriptkit/kit/config.ts").to_string();
        let editor = self.config.get_editor();

        logging::log(
            "UI",
            &format!(
                "Opening config.ts for shortcut configuration: {} (command: {})",
                config_path, command_id
            ),
        );

        // Ensure config.ts exists with documentation
        let config_path_buf = std::path::PathBuf::from(&config_path);
        if !config_path_buf.exists() {
            if let Err(e) = Self::create_config_template(&config_path_buf) {
                logging::log("ERROR", &format!("Failed to create config.ts: {}", e));
            }
        }

        // Copy command_id to clipboard as a hint
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = self.pbcopy(command_id) {
                logging::log("ERROR", &format!("Failed to copy command ID: {}", e));
            } else {
                self.last_output = Some(gpui::SharedString::from(format!(
                    "Copied '{}' to clipboard - paste in config.ts commands section",
                    command_id
                )));
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(command_id).is_ok() {
                    self.last_output = Some(gpui::SharedString::from(format!(
                        "Copied '{}' to clipboard - paste in config.ts commands section",
                        command_id
                    )));
                }
            }
        }

        let config_path_clone = config_path.clone();
        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new(&editor).arg(&config_path_clone).spawn() {
                Ok(_) => logging::log("UI", &format!("Opened config.ts in {}", editor)),
                Err(e) => logging::log("ERROR", &format!("Failed to open config.ts: {}", e)),
            }
        });
    }

    /// Create config.ts template with keyboard shortcut documentation
    #[allow(dead_code)]
    fn create_config_template(path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let template = r#"// Script Kit Configuration
// https://scriptkit.com/docs/config

import type { Config } from "@scriptkit/sdk";

export default {
  // ============================================
  // MAIN HOTKEY
  // ============================================
  // The keyboard shortcut to open Script Kit
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // ============================================
  // KEYBOARD SHORTCUTS
  // ============================================
  // Configure shortcuts for any command (scripts, built-ins, apps, snippets)
  //
  // Command ID formats:
  //   - "script/my-script"           - User scripts (by filename without extension)
  //   - "builtin/clipboard-history"  - Built-in features
  //   - "app/com.apple.Safari"       - Apps (by bundle ID)
  //   - "scriptlet/my-snippet"       - Scriptlets/snippets
  //
  // Modifier keys: "meta" (⌘), "ctrl", "alt" (⌥), "shift"
  // Key names: "KeyA"-"KeyZ", "Digit0"-"Digit9", "Space", "Enter", etc.
  //
  // Example:
  //   commands: {
  //     "builtin/clipboard-history": {
  //       shortcut: { modifiers: ["meta", "shift"], key: "KeyV" }
  //     },
  //     "app/com.apple.Safari": {
  //       shortcut: { modifiers: ["meta", "alt"], key: "KeyS" }
  //     }
  //   }
  commands: {
    // Add your shortcuts here
  },

  // ============================================
  // WINDOW HOTKEYS
  // ============================================
  // notesHotkey: { modifiers: ["meta", "shift"], key: "KeyN" },
  // aiHotkey: { modifiers: ["meta", "shift"], key: "Space" },

  // ============================================
  // APPEARANCE
  // ============================================
  // editorFontSize: 14,
  // terminalFontSize: 14,
  // uiScale: 1.0,

  // ============================================
  // PATHS
  // ============================================
  // bun_path: "/opt/homebrew/bin/bun",
  // editor: "code",
} satisfies Config;
"#;

        let mut file = std::fs::File::create(path)?;
        file.write_all(template.as_bytes())?;
        logging::log(
            "UI",
            &format!("Created config.ts template: {}", path.display()),
        );
        Ok(())
    }

    /// Show the inline shortcut recorder for a command.
    ///
    /// This replaces `open_config_for_shortcut` for non-script commands.
    /// For scripts, we still open the script file directly to edit the // Shortcut: comment.
    ///
    /// # Arguments
    /// * `command_id` - The unique identifier for the command (e.g., "builtin/clipboard-history")
    /// * `command_name` - Human-readable name of the command
    /// * `cx` - The context for UI updates
    fn show_shortcut_recorder(
        &mut self,
        command_id: String,
        command_name: String,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "SHORTCUT",
            &format!(
                "Showing shortcut recorder for '{}' (id: {})",
                command_name, command_id
            ),
        );

        // Store state - the entity will be created in render_shortcut_recorder_overlay
        // when we have window access
        self.shortcut_recorder_state = Some(ShortcutRecorderState {
            command_id,
            command_name,
        });

        // Clear any existing entity so a new one is created with correct focus
        self.shortcut_recorder_entity = None;

        // Close actions popup if open
        self.show_actions_popup = false;
        self.actions_dialog = None;

        cx.notify();
    }

    /// Close the shortcut recorder and clear state.
    /// Returns focus to the main filter input.
    pub fn close_shortcut_recorder(&mut self, cx: &mut Context<Self>) {
        if self.shortcut_recorder_state.is_some() || self.shortcut_recorder_entity.is_some() {
            logging::log(
                "SHORTCUT",
                "Closing shortcut recorder, returning focus to main filter",
            );
            self.shortcut_recorder_state = None;
            self.shortcut_recorder_entity = None;
            // Return focus to the main filter input
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    /// Render the shortcut recorder overlay if state is set.
    ///
    /// Returns None if no recorder is active.
    ///
    /// The recorder is created once and persisted to maintain keyboard focus.
    /// Callbacks use cx.entity() to communicate back to the parent app.
    fn render_shortcut_recorder_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        use crate::components::shortcut_recorder::ShortcutRecorder;

        // Check if we have state but no entity yet - need to create the recorder
        let state = self.shortcut_recorder_state.as_ref()?;

        // Create entity if needed (only once per show)
        if self.shortcut_recorder_entity.is_none() {
            let command_id = state.command_id.clone();
            let command_name = state.command_name.clone();
            let theme = std::sync::Arc::clone(&self.theme);

            // Get a weak reference to the app for callbacks
            let app_entity = cx.entity().downgrade();
            let app_entity_for_cancel = app_entity.clone();

            let recorder = cx.new(move |cx| {
                // Create the recorder with its own focus handle from its own context
                // This is CRITICAL for keyboard events to work
                let mut r = ShortcutRecorder::new(cx, theme);
                r.set_command_name(Some(command_name.clone()));
                r.set_command_description(Some(format!("ID: {}", command_id)));

                // Set save callback - directly updates the app via entity reference
                let app_for_save = app_entity.clone();
                r.on_save = Some(Box::new(move |recorded| {
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "Recorder on_save triggered: {}",
                            recorded.to_config_string()
                        ),
                    );
                    // Schedule the save on the app - this will be picked up by the app
                    if app_for_save.upgrade().is_some() {
                        // We can't call update() from here directly, so we'll use a different approach
                        // Store the result in the recorder and check it in render
                        logging::log("SHORTCUT", "Save callback - app entity available");
                    }
                }));

                // Set cancel callback
                let app_for_cancel = app_entity_for_cancel.clone();
                r.on_cancel = Some(Box::new(move || {
                    logging::log("SHORTCUT", "Recorder on_cancel triggered");
                    if let Some(_app) = app_for_cancel.upgrade() {
                        logging::log("SHORTCUT", "Cancel callback - app entity available");
                    }
                }));

                r
            });

            self.shortcut_recorder_entity = Some(recorder);
            logging::log("SHORTCUT", "Created new shortcut recorder entity");
        }

        // Get the existing entity
        let recorder = self.shortcut_recorder_entity.as_ref()?;

        // ALWAYS focus the recorder to ensure it captures keyboard input
        // This is critical for modal behavior - the recorder must have focus
        let recorder_fh = recorder.read(cx).focus_handle.clone();
        let was_focused = recorder_fh.is_focused(window);
        window.focus(&recorder_fh, cx);
        if !was_focused {
            logging::log("SHORTCUT", "Focused shortcut recorder (was not focused)");
        }

        // Check for pending actions from the recorder (Save or Cancel)
        // We need to update() the recorder entity to take the pending action
        let pending_action = recorder.update(cx, |r, _cx| r.take_pending_action());

        if let Some(action) = pending_action {
            use crate::components::shortcut_recorder::RecorderAction;
            match action {
                RecorderAction::Save(recorded) => {
                    logging::log(
                        "SHORTCUT",
                        &format!("Handling save action: {}", recorded.to_config_string()),
                    );
                    // Handle the save - need to defer to avoid borrow issues
                    let app_entity = cx.entity().downgrade();
                    cx.spawn(async move |_this, cx| {
                        gpui::Timer::after(std::time::Duration::from_millis(1)).await;
                        let _ = cx.update(|cx| {
                            if let Some(app) = app_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.handle_shortcut_save(&recorded, cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
                RecorderAction::Cancel => {
                    logging::log("SHORTCUT", "Handling cancel action");
                    // Handle the cancel - need to defer to avoid borrow issues
                    let app_entity = cx.entity().downgrade();
                    cx.spawn(async move |_this, cx| {
                        gpui::Timer::after(std::time::Duration::from_millis(1)).await;
                        let _ = cx.update(|cx| {
                            if let Some(app) = app_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.close_shortcut_recorder(cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
            }
        }

        // Clone the entity for rendering
        let recorder_clone = recorder.clone();

        // Render the recorder as a child element
        Some(
            div()
                .id("shortcut-recorder-wrapper")
                .absolute()
                .inset_0()
                .child(recorder_clone)
                .into_any_element(),
        )
    }

    /// Handle saving a shortcut from the recorder.
    ///
    /// This saves the shortcut to ~/.scriptkit/shortcuts.json and updates the registry.
    fn handle_shortcut_save(
        &mut self,
        recorded: &crate::components::shortcut_recorder::RecordedShortcut,
        cx: &mut Context<Self>,
    ) {
        let Some(ref state) = self.shortcut_recorder_state else {
            logging::log("SHORTCUT", "No recorder state when trying to save");
            return;
        };

        let command_id = state.command_id.clone();
        let command_name = state.command_name.clone();

        // Convert RecordedShortcut to the persistence Shortcut type
        let shortcut = crate::shortcuts::Shortcut {
            key: recorded.key.clone().unwrap_or_default().to_lowercase(),
            modifiers: crate::shortcuts::Modifiers {
                cmd: recorded.cmd,
                ctrl: recorded.ctrl,
                alt: recorded.alt,
                shift: recorded.shift,
            },
        };

        logging::log(
            "SHORTCUT",
            &format!(
                "Saving shortcut for '{}' ({}): {}",
                command_name,
                command_id,
                shortcut.to_canonical_string()
            ),
        );

        // Save to persistence
        match crate::shortcuts::save_shortcut_override(&command_id, &shortcut) {
            Ok(()) => {
                logging::log("SHORTCUT", "Shortcut saved to shortcuts.json");

                // Register the hotkey immediately so it works without restart
                let shortcut_str = shortcut.to_canonical_string();
                match crate::hotkeys::register_dynamic_shortcut(
                    &command_id,
                    &shortcut_str,
                    &command_name,
                ) {
                    Ok(id) => {
                        logging::log(
                            "SHORTCUT",
                            &format!("Registered hotkey immediately (id: {})", id),
                        );
                        self.show_hud(
                            format!("Shortcut set: {} (active now)", shortcut.display()),
                            Some(2000),
                            cx,
                        );
                    }
                    Err(e) => {
                        // Shortcut saved but couldn't register - will work after restart
                        logging::log(
                            "SHORTCUT",
                            &format!("Shortcut saved but registration failed: {} - will work after restart", e),
                        );
                        self.show_hud(
                            format!("Shortcut set: {} (restart to activate)", shortcut.display()),
                            Some(3000),
                            cx,
                        );
                    }
                }
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to save shortcut: {}", e));
                self.show_hud(format!("Failed to save shortcut: {}", e), Some(4000), cx);
            }
        }

        // Close the recorder and restore focus
        self.close_shortcut_recorder(cx);
    }

    /// Show the alias input overlay for configuring a command alias.
    ///
    /// The alias input allows users to set a text alias that can be typed
    /// in the main menu to quickly run a command.
    fn show_alias_input(
        &mut self,
        command_id: String,
        command_name: String,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "ALIAS",
            &format!(
                "Showing alias input for '{}' (id: {})",
                command_name, command_id
            ),
        );

        // Load existing alias if any
        let existing_alias = crate::aliases::load_alias_overrides()
            .ok()
            .and_then(|overrides| overrides.get(&command_id).cloned())
            .unwrap_or_default();

        // Store state
        self.alias_input_state = Some(AliasInputState {
            command_id,
            command_name,
            alias_text: existing_alias,
        });

        // Close actions popup if open
        self.show_actions_popup = false;
        self.actions_dialog = None;

        cx.notify();
    }

    /// Close the alias input and clear state.
    /// Returns focus to the main filter input.
    pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
        if self.alias_input_state.is_some() || self.alias_input_entity.is_some() {
            logging::log(
                "ALIAS",
                "Closing alias input, returning focus to main filter",
            );
            self.alias_input_state = None;
            self.alias_input_entity = None; // Clear entity to reset for next open
                                            // Return focus to the main filter input (like close_shortcut_recorder does)
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    /// Update the alias text in the input state.
    /// Currently unused - will be connected when real text input is added.
    #[allow(dead_code)]
    fn update_alias_text(&mut self, text: String, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.alias_input_state {
            state.alias_text = text;
            cx.notify();
        }
    }

    /// Save the current alias and close the input.
    /// If alias_from_entity is provided, use that; otherwise fall back to state.alias_text.
    fn save_alias_with_text(&mut self, alias_from_entity: Option<String>, cx: &mut Context<Self>) {
        let Some(ref state) = self.alias_input_state else {
            logging::log("ALIAS", "No alias input state when trying to save");
            return;
        };

        let command_id = state.command_id.clone();
        let command_name = state.command_name.clone();
        // Prefer alias from entity if provided, else use state
        let alias_text = alias_from_entity
            .unwrap_or_else(|| state.alias_text.clone())
            .trim()
            .to_string();

        if alias_text.is_empty() {
            // Empty alias means remove it
            match crate::aliases::remove_alias_override(&command_id) {
                Ok(()) => {
                    logging::log("ALIAS", &format!("Removed alias for: {}", command_id));
                    self.show_hud("Alias removed".to_string(), Some(2000), cx);
                }
                Err(e) => {
                    logging::log("ERROR", &format!("Failed to remove alias: {}", e));
                    self.show_hud(format!("Failed to remove alias: {}", e), Some(4000), cx);
                }
            }
        } else {
            // Validate alias: should be alphanumeric with optional hyphens/underscores
            if !alias_text
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                self.show_hud(
                    "Alias must contain only letters, numbers, hyphens, or underscores".to_string(),
                    Some(4000),
                    cx,
                );
                return;
            }

            logging::log(
                "ALIAS",
                &format!(
                    "Saving alias for '{}' ({}): {}",
                    command_name, command_id, alias_text
                ),
            );

            match crate::aliases::save_alias_override(&command_id, &alias_text) {
                Ok(()) => {
                    logging::log("ALIAS", "Alias saved to aliases.json");
                    self.show_hud(
                        format!("Alias set: {} → {}", alias_text, command_name),
                        Some(2000),
                        cx,
                    );
                    // Refresh scripts to update the alias registry
                    self.refresh_scripts(cx);
                }
                Err(e) => {
                    logging::log("ERROR", &format!("Failed to save alias: {}", e));
                    self.show_hud(format!("Failed to save alias: {}", e), Some(4000), cx);
                }
            }
        }

        // Close the input and restore focus
        self.close_alias_input(cx);
    }

    /// Render the alias input overlay if state is set.
    ///
    /// Returns None if no alias input is active.
    ///
    /// The alias input entity is created once and persisted to maintain keyboard focus.
    /// This follows the same pattern as render_shortcut_recorder_overlay.
    fn render_alias_input_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        use crate::components::alias_input::{AliasInput, AliasInputAction};

        // Check if we have state but no entity yet - need to create the input
        let state = self.alias_input_state.as_ref()?;

        // Create entity if needed (only once per show)
        if self.alias_input_entity.is_none() {
            let command_id = state.command_id.clone();
            let command_name = state.command_name.clone();
            let current_alias = if state.alias_text.is_empty() {
                None
            } else {
                Some(state.alias_text.clone())
            };
            let theme = std::sync::Arc::clone(&self.theme);

            let input_entity = cx.new(move |cx| {
                // Create the alias input with its own focus handle from its own context
                // This is CRITICAL for keyboard events to work
                AliasInput::new(cx, theme)
                    .with_command_name(command_name)
                    .with_command_id(command_id)
                    .with_current_alias(current_alias)
            });

            self.alias_input_entity = Some(input_entity);
            logging::log("ALIAS", "Created new alias input entity");
        }

        // Get the existing entity - clone it early to avoid borrow conflicts
        let input_entity = self.alias_input_entity.clone()?;

        // ALWAYS focus the input entity to ensure it captures keyboard input
        // This is critical for modal behavior - the input must have focus
        let input_fh = input_entity.read(cx).focus_handle.clone();
        let was_focused = input_fh.is_focused(window);
        window.focus(&input_fh, cx);
        if !was_focused {
            logging::log("ALIAS", "Focused alias input (was not focused)");
        }

        // Check for pending actions from the input entity (Save, Cancel, or Clear)
        // We need to update() the entity to take the pending action
        let pending_action = input_entity.update(cx, |input, _cx| input.take_pending_action());

        if let Some(action) = pending_action {
            match action {
                AliasInputAction::Save(alias) => {
                    logging::log("ALIAS", &format!("Handling save action: {}", alias));
                    // Handle the save - need to defer to avoid borrow issues
                    let app_entity = cx.entity().downgrade();
                    cx.spawn(async move |_this, cx| {
                        gpui::Timer::after(std::time::Duration::from_millis(1)).await;
                        let _ = cx.update(|cx| {
                            if let Some(app) = app_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.save_alias_with_text(Some(alias), cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
                AliasInputAction::Cancel => {
                    logging::log("ALIAS", "Handling cancel action");
                    self.close_alias_input(cx);
                }
                AliasInputAction::Clear => {
                    logging::log("ALIAS", "Handling clear action (remove alias)");
                    // Clear means remove the alias - save with empty string
                    let app_entity = cx.entity().downgrade();
                    cx.spawn(async move |_this, cx| {
                        gpui::Timer::after(std::time::Duration::from_millis(1)).await;
                        let _ = cx.update(|cx| {
                            if let Some(app) = app_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.save_alias_with_text(Some(String::new()), cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
            }
        }

        // Return the entity's view as an element
        Some(input_entity.into_any_element())
    }

    /// Execute a path action from the actions dialog
    /// Handles actions like copy_path, open_in_finder, open_in_editor, etc.
    fn execute_path_action(
        &mut self,
        action_id: &str,
        path_info: &PathInfo,
        path_prompt_entity: &Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!(
                "Executing path action '{}' for: {} (is_dir={})",
                action_id, path_info.path, path_info.is_dir
            ),
        );

        match action_id {
            "select_file" | "open_directory" => {
                // For select/open, trigger submission through the path prompt
                // We need to trigger the submit callback with this path
                path_prompt_entity.update(cx, |prompt, cx| {
                    // Find the index of this path in filtered_entries and submit it
                    if let Some(idx) = prompt
                        .filtered_entries
                        .iter()
                        .position(|e| e.path == path_info.path)
                    {
                        prompt.selected_index = idx;
                    }
                    // For directories, navigate into them; for files, submit
                    if path_info.is_dir && action_id == "open_directory" {
                        prompt.navigate_to(&path_info.path, cx);
                    } else {
                        // Submit the selected path
                        let id = prompt.id.clone();
                        let path = path_info.path.clone();
                        (prompt.on_submit)(id, Some(path));
                    }
                });
            }
            "copy_path" => {
                // Copy full path to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.path.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_info.path),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.path
                                    )));
                                } else {
                                    logging::log("ERROR", "Failed to write to pbcopy stdin");
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                            self.last_output = Some(SharedString::from("Failed to copy path"));
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    use arboard::Clipboard;
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&path_info.path) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_info.path),
                                );
                                self.last_output =
                                    Some(SharedString::from(format!("Copied: {}", path_info.path)));
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                self.last_output = Some(SharedString::from("Failed to copy path"));
                            }
                        },
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to access clipboard: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to access clipboard"));
                        }
                    }
                }
            }
            "copy_filename" => {
                // Copy just the filename to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.name.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!(
                                            "Copied filename to clipboard: {}",
                                            path_info.name
                                        ),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.name
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                        }
                    }
                }
            }
            "open_in_finder" => {
                // Reveal in Finder (macOS)
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let path_to_reveal = if path_info.is_dir {
                        path_info.path.clone()
                    } else {
                        // For files, reveal the containing folder with the file selected
                        path_info.path.clone()
                    };

                    match Command::new("open").args(["-R", &path_to_reveal]).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Revealed in Finder: {}", path_info.path));
                            // Hide main window only (not entire app) to keep HUD visible
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            platform::hide_main_window();
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to reveal in Finder: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to reveal in Finder"));
                        }
                    }
                }
            }
            "open_in_editor" => {
                // Open in configured editor
                let editor = self.config.get_editor();
                let path_str = path_info.path.clone();
                logging::log(
                    "UI",
                    &format!("Opening in editor '{}': {}", editor, path_str),
                );

                match std::process::Command::new(&editor).arg(&path_str).spawn() {
                    Ok(_) => {
                        logging::log("UI", &format!("Opened in editor: {}", path_str));
                        // Hide main window only (not entire app) to keep HUD visible
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        platform::hide_main_window();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                        self.last_output = Some(SharedString::from("Failed to open in editor"));
                    }
                }
            }
            "open_in_terminal" => {
                // Open terminal at this location
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    // Get the directory (if file, use parent directory)
                    let dir_path = if path_info.is_dir {
                        path_info.path.clone()
                    } else {
                        std::path::Path::new(&path_info.path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_info.path.clone())
                    };

                    // Try iTerm first, fall back to Terminal.app
                    let script = format!(
                        r#"tell application "Terminal"
                            do script "cd '{}'"
                            activate
                        end tell"#,
                        dir_path
                    );

                    match Command::new("osascript").args(["-e", &script]).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened terminal at: {}", dir_path));
                            // Hide main window only (not entire app) to keep HUD visible
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            platform::hide_main_window();
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open terminal: {}", e));
                            self.last_output = Some(SharedString::from("Failed to open terminal"));
                        }
                    }
                }
            }
            "move_to_trash" => {
                // Move to trash (macOS)
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let path_str = path_info.path.clone();
                    let name = path_info.name.clone();

                    // Use AppleScript to move to trash (preserves undo capability)
                    let script = format!(
                        r#"tell application "Finder"
                            delete POSIX file "{}"
                        end tell"#,
                        path_str
                    );

                    match Command::new("osascript").args(["-e", &script]).spawn() {
                        Ok(mut child) => {
                            // Wait for completion and check result
                            match child.wait() {
                                Ok(status) if status.success() => {
                                    logging::log("UI", &format!("Moved to trash: {}", path_str));
                                    self.last_output = Some(SharedString::from(format!(
                                        "Moved to Trash: {}",
                                        name
                                    )));
                                    // Refresh the path prompt to show the file is gone
                                    path_prompt_entity.update(cx, |prompt, cx| {
                                        let current = prompt.current_path.clone();
                                        prompt.navigate_to(&current, cx);
                                    });
                                }
                                _ => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to move to trash: {}", path_str),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to move to Trash"));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn trash command: {}", e));
                            self.last_output = Some(SharedString::from("Failed to move to Trash"));
                        }
                    }
                }
            }
            _ => {
                logging::log("UI", &format!("Unknown path action: {}", action_id));
            }
        }

        cx.notify();
    }

    /// Execute a scriptlet (simple code snippet from .md file)
    fn execute_scriptlet(&mut self, scriptlet: &scripts::Scriptlet, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!(
                "Executing scriptlet: {} (tool: {})",
                scriptlet.name, scriptlet.tool
            ),
        );

        let tool = scriptlet.tool.to_lowercase();

        // TypeScript/Kit scriptlets need to run interactively (they may use SDK prompts)
        // These should be spawned like regular scripts, not run synchronously
        if matches!(tool.as_str(), "kit" | "ts" | "bun" | "deno" | "js") {
            logging::log(
                "EXEC",
                &format!(
                    "TypeScript scriptlet '{}' - running interactively",
                    scriptlet.name
                ),
            );

            // Write scriptlet content to a temp file
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!(
                "scriptlet-{}-{}.ts",
                scriptlet.name.to_lowercase().replace(' ', "-"),
                std::process::id()
            ));

            if let Err(e) = std::fs::write(&temp_file, &scriptlet.code) {
                logging::log(
                    "ERROR",
                    &format!("Failed to write temp scriptlet file: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to write scriptlet: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }

            // Create a Script struct and run it interactively
            let script = scripts::Script {
                name: scriptlet.name.clone(),
                description: scriptlet.description.clone(),
                path: temp_file,
                extension: "ts".to_string(),
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
                kit_name: None,
            };

            self.execute_interactive(&script, cx);
            return;
        }

        // Shell tools (bash, zsh, sh, fish, etc.) run in the built-in terminal
        // so users can see output interactively
        if scriptlets::SHELL_TOOLS.contains(&tool.as_str()) {
            logging::log(
                "EXEC",
                &format!(
                    "Shell scriptlet '{}' (tool: {}) - running in terminal",
                    scriptlet.name, tool
                ),
            );

            // Write scriptlet code to a temp file and execute it
            let temp_dir = std::env::temp_dir();
            let extension = match tool.as_str() {
                "bash" | "zsh" | "sh" => "sh",
                "fish" => "fish",
                "powershell" | "pwsh" => "ps1",
                "cmd" => "bat",
                _ => "sh",
            };
            let temp_file = temp_dir.join(format!(
                "extension-{}-{}.{}",
                scriptlet.name.to_lowercase().replace(' ', "-"),
                std::process::id(),
                extension
            ));

            if let Err(e) = std::fs::write(&temp_file, &scriptlet.code) {
                logging::log(
                    "ERROR",
                    &format!("Failed to write temp extension file: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to write extension: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }

            // Build the command to execute the script file
            let shell_command = format!("{} {}", tool, temp_file.display());

            self.open_terminal_with_command(shell_command, cx);
            return;
        }

        // For other tools (python, ruby, template, etc.), run synchronously
        // These don't use the SDK and won't block waiting for input

        // Convert scripts::Scriptlet to scriptlets::Scriptlet for executor
        let exec_scriptlet = scriptlets::Scriptlet {
            name: scriptlet.name.clone(),
            command: scriptlet.command.clone().unwrap_or_else(|| {
                // Generate command slug from name if not present
                scriptlet.name.to_lowercase().replace(' ', "-")
            }),
            tool: scriptlet.tool.clone(),
            scriptlet_content: scriptlet.code.clone(),
            inputs: vec![], // TODO: Parse inputs from code if needed
            group: scriptlet.group.clone().unwrap_or_default(),
            preview: None,
            metadata: scriptlets::ScriptletMetadata {
                shortcut: scriptlet.shortcut.clone(),
                keyword: scriptlet.keyword.clone(),
                description: scriptlet.description.clone(),
                ..Default::default()
            },
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: scriptlet.file_path.clone(),
            actions: vec![], // Scriptlet actions parsed from H3 headers
        };

        // Execute with default options (no inputs for now)
        let options = executor::ScriptletExecOptions::default();

        match executor::run_scriptlet(&exec_scriptlet, options) {
            Ok(result) => {
                if result.success {
                    logging::log(
                        "EXEC",
                        &format!(
                            "Scriptlet '{}' succeeded: exit={}",
                            scriptlet.name, result.exit_code
                        ),
                    );

                    // Handle special tool types that need interactive prompts
                    if tool == "template" && !result.stdout.is_empty() {
                        // Template tool: show template prompt with the content
                        let id = format!("scriptlet-template-{}", uuid::Uuid::new_v4());
                        logging::log(
                            "EXEC",
                            &format!(
                                "Template scriptlet '{}' - showing template prompt",
                                scriptlet.name
                            ),
                        );
                        self.handle_prompt_message(
                            PromptMessage::ShowTemplate {
                                id,
                                template: result.stdout.clone(),
                            },
                            cx,
                        );
                        return;
                    }

                    // Store output if any
                    if !result.stdout.is_empty() {
                        self.last_output = Some(SharedString::from(result.stdout.clone()));
                    }

                    // Hide window after successful execution
                    script_kit_gpui::set_main_window_visible(false);
                    cx.hide();
                } else {
                    // Execution failed (non-zero exit code)
                    let error_msg = if !result.stderr.is_empty() {
                        result.stderr.clone()
                    } else {
                        format!("Exit code: {}", result.exit_code)
                    };

                    logging::log(
                        "ERROR",
                        &format!("Scriptlet '{}' failed: {}", scriptlet.name, error_msg),
                    );

                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Scriptlet failed: {}", error_msg),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            Err(e) => {
                logging::log(
                    "ERROR",
                    &format!("Failed to execute scriptlet '{}': {}", scriptlet.name, e),
                );

                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to execute: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    /// Execute a script or scriptlet by its file path
    /// Used by global shortcuts to directly invoke scripts
    #[allow(dead_code)]
    fn execute_script_by_path(&mut self, path: &str, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Executing script by path: {}", path));

        // Check if it's a scriptlet (contains #)
        if path.contains('#') {
            // It's a scriptlet path like "/path/to/file.md#command"
            if let Some(scriptlet) = self
                .scriptlets
                .iter()
                .find(|s| s.file_path.as_ref().map(|p| p == path).unwrap_or(false))
            {
                let scriptlet_clone = scriptlet.clone();
                self.execute_scriptlet(&scriptlet_clone, cx);
                return;
            }
            logging::log("ERROR", &format!("Scriptlet not found: {}", path));
            return;
        }

        // It's a regular script - find by path
        if let Some(script) = self
            .scripts
            .iter()
            .find(|s| s.path.to_string_lossy() == path)
        {
            let script_clone = script.clone();
            self.execute_interactive(&script_clone, cx);
            return;
        }

        // Not found in loaded scripts - try to execute directly as a file
        let script_path = std::path::PathBuf::from(path);
        if script_path.exists() {
            let name = script_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("script")
                .to_string();

            let script = scripts::Script {
                name,
                path: script_path.clone(),
                extension: script_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("ts")
                    .to_string(),
                description: None,
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
                kit_name: None,
            };

            self.execute_interactive(&script, cx);
        } else {
            logging::log("ERROR", &format!("Script file not found: {}", path));
        }
    }

    /// Execute by command ID or legacy file path.
    ///
    /// Command IDs have formats like:
    /// - "scriptlet/my-scriptlet" - execute a scriptlet
    /// - "builtin/ai-chat" - execute a builtin
    /// - "app/com.apple.Finder" - launch an app
    /// - Otherwise: treated as a file path (legacy behavior)
    ///
    /// Returns `true` if the main window should be shown, `false` if not.
    /// Apps and certain builtins (AI Chat, Notes) open their own windows
    /// and don't need the main window.
    pub fn execute_by_command_id_or_path(
        &mut self,
        command_id: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        logging::log(
            "EXEC",
            &format!("Executing by command ID or path: {}", command_id),
        );

        // Builtins that open their own windows and don't need main window
        const NO_MAIN_WINDOW_BUILTINS: &[&str] = &[
            "builtin-ai-chat",
            "builtin-notes",
            "builtin-new-note",
            "builtin-search-notes",
            "builtin-quick-capture",
            "builtin-new-conversation",
        ];

        // Parse command ID format: "type/identifier"
        if let Some((cmd_type, identifier)) = command_id.split_once('/') {
            match cmd_type {
                "scriptlet" => {
                    // Find scriptlet by name
                    logging::bench_log("scriptlet_lookup_start");
                    if let Some(scriptlet) = self.scriptlets.iter().find(|s| s.name == identifier) {
                        logging::bench_log("scriptlet_found");
                        let scriptlet_clone = scriptlet.clone();
                        logging::log("EXEC", &format!("Executing scriptlet: {}", identifier));
                        self.execute_scriptlet(&scriptlet_clone, cx);
                        // Don't show window immediately - scriptlets that need it (like getSelectedText)
                        // will call hide() first, then their prompts (chat, arg, etc.) will show the window.
                        // This prevents the flash of main menu before the scriptlet UI appears.
                        return false;
                    }
                    logging::log("ERROR", &format!("Scriptlet not found: {}", identifier));
                    return false;
                }
                "builtin" => {
                    // Execute builtin by ID
                    let config = crate::config::BuiltInConfig::default();
                    if let Some(entry) = builtins::get_builtin_entries(&config)
                        .iter()
                        .find(|e| e.id == identifier)
                    {
                        logging::log("EXEC", &format!("Executing builtin: {}", identifier));
                        self.execute_builtin(entry, cx);
                        // Check if this builtin opens its own window
                        let needs_main_window = !NO_MAIN_WINDOW_BUILTINS.contains(&identifier);
                        logging::log(
                            "EXEC",
                            &format!(
                                "Builtin {} needs_main_window: {}",
                                identifier, needs_main_window
                            ),
                        );
                        return needs_main_window;
                    }
                    logging::log("ERROR", &format!("Builtin not found: {}", identifier));
                    return false;
                }
                "app" => {
                    // Launch app by bundle ID - find app in cached apps and launch
                    // Apps NEVER need the main window - they open externally
                    logging::log(
                        "EXEC",
                        &format!("Launching app by bundle ID: {}", identifier),
                    );
                    let apps = crate::app_launcher::get_cached_apps();
                    if let Some(app) = apps
                        .iter()
                        .find(|a| a.bundle_id.as_deref() == Some(identifier))
                    {
                        if let Err(e) = crate::app_launcher::launch_application(app) {
                            logging::log("ERROR", &format!("Failed to launch app: {}", e));
                        }
                    } else {
                        logging::log("ERROR", &format!("App not found: {}", identifier));
                    }
                    return false; // Apps never need main window
                }
                _ => {
                    // Unknown type - fall through to path-based execution
                    logging::log(
                        "EXEC",
                        &format!("Unknown command type '{}', trying as path", cmd_type),
                    );
                }
            }
        }

        // Check if command_id matches a scriptlet by name or file_path
        // Scriptlets don't need immediate window show - they control their own visibility
        if let Some(scriptlet) = self.scriptlets.iter().find(|s| {
            s.name == command_id
                || s.file_path
                    .as_ref()
                    .map(|p| p == command_id)
                    .unwrap_or(false)
        }) {
            logging::log(
                "EXEC",
                &format!("Found scriptlet by name/path: {}", scriptlet.name),
            );
            let scriptlet_clone = scriptlet.clone();
            self.execute_scriptlet(&scriptlet_clone, cx);
            return false; // Scriptlets don't need immediate window show
        }

        // Fall back to path-based execution (legacy behavior)
        // Scripts typically need the main window for prompts
        self.execute_script_by_path(command_id, cx);
        true
    }

    /// Cancel the currently running script and clean up all state
    fn cancel_script_execution(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "=== Canceling script execution ===");

        // Send cancel message to script (Exit with cancel code)
        // Use try_send to avoid blocking UI thread during cancellation
        if let Some(ref sender) = self.response_sender {
            // Try to send Exit message to terminate the script cleanly
            let exit_msg = Message::Exit {
                code: Some(1), // Non-zero code indicates cancellation
                message: Some("Cancelled by user".to_string()),
            };
            match sender.try_send(exit_msg) {
                Ok(()) => logging::log("EXEC", "Sent Exit message to script"),
                Err(std::sync::mpsc::TrySendError::Full(_)) => logging::log(
                    "EXEC",
                    "Exit message dropped - channel full (script may be stuck)",
                ),
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    logging::log("EXEC", "Exit message dropped - script already exited")
                }
            }
        } else {
            logging::log("EXEC", "No response_sender - script may not be running");
        }

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This ensures cleanup even if Drop doesn't fire properly
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {}", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Abort script session if it exists
        {
            let mut session_guard = self.script_session.lock();
            if let Some(_session) = session_guard.take() {
                logging::log("EXEC", "Cleared script session");
            }
        }

        // Reset to script list view
        self.reset_to_script_list(cx);
        logging::log("EXEC", "=== Script cancellation complete ===");
    }

    /// Flush pending toasts from ToastManager to gpui-component's NotificationList
    ///
    /// This should be called at the start of render() where we have window access.
    /// The ToastManager acts as a staging queue for toasts pushed from callbacks
    /// that don't have window access.
    fn flush_pending_toasts(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) {
        use gpui_component::WindowExt;

        let pending = self.toast_manager.drain_pending();
        let count = pending.len();
        if count > 0 {
            logging::log(
                "UI",
                &format!("Flushing {} pending toast(s) to NotificationList", count),
            );
        }
        for toast in pending {
            logging::log("UI", &format!("Pushing notification: {}", toast.message));
            let notification = pending_toast_to_notification(&toast);
            window.push_notification(notification, cx);
        }
    }

    /// Close window and reset to default state (Cmd+W global handler)
    ///
    /// This method handles the global Cmd+W shortcut which should work
    /// regardless of what prompt or view is currently active. It:
    /// 1. Cancels any running script
    /// 2. Resets state to the default script list
    /// 3. Hides the window
    fn close_and_reset_window(&mut self, cx: &mut Context<Self>) {
        logging::log("VISIBILITY", "=== Close and reset window ===");

        // Reset pin state when window is closed
        self.is_pinned = false;

        // Close child windows FIRST if open (they are children of main window)
        // Actions window
        if self.show_actions_popup || is_actions_window_open() {
            self.show_actions_popup = false;
            self.actions_dialog = None;
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();
            logging::log("VISIBILITY", "Closed actions window before hiding main");
        }

        // Confirm window (modal)
        if crate::confirm::is_confirm_window_open() {
            crate::confirm::close_confirm_window(cx);
            logging::log("VISIBILITY", "Closed confirm window before hiding main");
        }

        // Save window position BEFORE hiding (main window is hidden, not closed)
        if let Some((x, y, w, h)) = crate::platform::get_main_window_bounds() {
            crate::window_state::save_window_bounds(
                crate::window_state::WindowRole::Main,
                crate::window_state::PersistedWindowBounds::new(x, y, w, h),
            );
        }

        // Update visibility state FIRST to prevent race conditions
        script_kit_gpui::set_main_window_visible(false);
        logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

        // If in a prompt, cancel the script execution
        if self.is_in_prompt() {
            logging::log(
                "VISIBILITY",
                "In prompt mode - canceling script before hiding",
            );
            self.cancel_script_execution(cx);
        } else {
            // Just reset to script list (clears filter, selection, scroll)
            self.reset_to_script_list(cx);
        }

        // Check if Notes or AI windows are open BEFORE hiding
        let notes_open = notes::is_notes_window_open();
        let ai_open = ai::is_ai_window_open();
        logging::log(
            "VISIBILITY",
            &format!(
                "Secondary windows: notes_open={}, ai_open={}",
                notes_open, ai_open
            ),
        );

        // CRITICAL: Only hide main window if Notes/AI are open
        // cx.hide() hides the ENTIRE app (all windows), so we use
        // platform::hide_main_window() to hide only the main window
        if notes_open || ai_open {
            logging::log(
                "VISIBILITY",
                "Using hide_main_window() - secondary windows are open",
            );
            platform::hide_main_window();
        } else {
            logging::log("VISIBILITY", "Using cx.hide() - no secondary windows");
            cx.hide();
        }
        logging::log("VISIBILITY", "=== Window closed ===");
    }

    /// Go back to main menu or close window depending on how the view was opened.
    ///
    /// If the current built-in view was opened from the main menu, this returns to the
    /// main menu (ScriptList). If it was opened directly via hotkey or protocol command,
    /// this closes the window entirely.
    ///
    /// This provides consistent UX: pressing ESC always "goes back" one step.
    fn go_back_or_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.opened_from_main_menu {
            logging::log(
                "KEY",
                "ESC - returning to main menu (opened from main menu)",
            );
            // Return to main menu
            self.current_view = AppView::ScriptList;
            self.filter_text.clear();
            self.selected_index = 0;
            // Reset the flag since we're now in main menu
            self.opened_from_main_menu = false;
            // Sync input and reset placeholder to default
            self.gpui_input_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
                state.set_selection(0, 0, window, cx);
                state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
            });
            self.update_window_size_deferred(window, cx);
            self.pending_focus = Some(FocusTarget::MainFilter);
            self.focused_input = FocusedInput::MainFilter;
            cx.notify();
        } else {
            logging::log(
                "KEY",
                "ESC - closing window (opened directly via hotkey/protocol)",
            );
            self.close_and_reset_window(cx);
        }
    }

    /// Handle global keyboard shortcuts with configurable dismissability
    ///
    /// Returns `true` if the shortcut was handled (caller should return early)
    ///
    /// # Arguments
    /// * `event` - The key down event to check
    /// * `is_dismissable` - If true, ESC key will also close the window (for prompts like arg, div, form, etc.)
    ///   If false, only Cmd+W closes the window (for prompts like term, editor)
    /// * `cx` - The context
    ///
    /// # Handled shortcuts
    /// - Cmd+W: Always closes window and resets to default state
    /// - Escape: Only closes window if `is_dismissable` is true AND actions popup is not showing
    /// - Cmd+Shift+M: Cycle vibrancy material (for debugging)
    #[tracing::instrument(skip(self, event, cx), fields(key = %event.keystroke.key, modifiers = ?event.keystroke.modifiers, is_dismissable))]
    fn handle_global_shortcut_with_options(
        &mut self,
        event: &gpui::KeyDownEvent,
        is_dismissable: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        // If the shortcut recorder is active, don't process any shortcuts here.
        // The recorder has its own key handlers and should receive all key events.
        if self.shortcut_recorder_state.is_some() {
            return false;
        }

        let key_str = event.keystroke.key.to_lowercase();
        let has_cmd = event.keystroke.modifiers.platform;
        let has_shift = event.keystroke.modifiers.shift;

        // Cmd+W always closes window
        if has_cmd && key_str == "w" {
            logging::log("KEY", "Cmd+W - closing window");
            self.close_and_reset_window(cx);
            return true;
        }

        // Cmd+Shift+M cycles vibrancy material (for debugging)
        if has_cmd && has_shift && key_str == "m" {
            let result = crate::platform::cycle_vibrancy_material();
            logging::log("KEY", &format!("Cmd+Shift+M - {}", result));
            // Show HUD with the material name
            self.show_hud(result, None, cx);
            return true;
        }

        // Cmd+Shift+P toggles pin mode (window stays open on blur)
        if has_cmd && has_shift && key_str == "p" {
            self.is_pinned = !self.is_pinned;
            let status = if self.is_pinned {
                "📌 Window Pinned"
            } else {
                "📌 Window Unpinned"
            };
            logging::log("KEY", &format!("Cmd+Shift+P - {}", status));
            self.show_hud(status.to_string(), None, cx);
            cx.notify();
            return true;
        }

        // ESC closes dismissable prompts (when actions popup is not showing)
        if is_dismissable && key_str == "escape" && !self.show_actions_popup {
            logging::log("KEY", "ESC in dismissable prompt - closing window");
            self.close_and_reset_window(cx);
            return true;
        }

        false
    }

    /// Check if the current view is a dismissable prompt
    ///
    /// Dismissable prompts are those that feel "closeable" with escape:
    /// - ArgPrompt, DivPrompt, FormPrompt, SelectPrompt, PathPrompt, DropPrompt, TemplatePrompt
    /// - Built-in views (ClipboardHistory, AppLauncher, WindowSwitcher, DesignGallery)
    /// - ScriptList
    ///
    /// Non-dismissable prompts:
    /// - TermPrompt, EditorPrompt (these require explicit Cmd+W to close)
    /// - EnvPrompt (stays open on blur so user can copy API keys from other windows)
    #[allow(dead_code)]
    fn is_dismissable_view(&self) -> bool {
        !matches!(
            self.current_view,
            AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ScratchPadView { .. }
                | AppView::QuickTerminalView { .. }
                | AppView::EnvPrompt { .. }
        )
    }

    /// Show a HUD (heads-up display) overlay message
    ///
    /// This creates a separate floating window positioned at bottom-center of the
    /// screen containing the mouse cursor. The HUD is independent of the main
    /// Script Kit window and will remain visible even when the main window is hidden.
    ///
    /// Position: Bottom-center (85% down screen)
    /// Duration: 2000ms default, configurable
    /// Shape: Pill (40px tall, variable width)
    fn show_hud(&mut self, text: String, duration_ms: Option<u64>, cx: &mut Context<Self>) {
        // Delegate to the HUD manager which creates a separate floating window
        // This ensures the HUD is visible even when the main app window is hidden
        hud_manager::show_hud(text, duration_ms, cx);
    }

    /// Show the debug grid overlay with specified options
    ///
    /// This method converts protocol::GridOptions to debug_grid::GridConfig
    /// and enables the grid overlay rendering.
    fn show_grid(&mut self, options: protocol::GridOptions, cx: &mut Context<Self>) {
        use debug_grid::{GridColorScheme, GridConfig, GridDepth};
        use protocol::GridDepthOption;

        // Convert protocol depth to debug_grid depth
        let depth = match &options.depth {
            GridDepthOption::Preset(s) if s == "all" => GridDepth::All,
            GridDepthOption::Preset(_) => GridDepth::Prompts,
            GridDepthOption::Components(names) => GridDepth::Components(names.clone()),
        };

        self.grid_config = Some(GridConfig {
            grid_size: options.grid_size,
            show_bounds: options.show_bounds,
            show_box_model: options.show_box_model,
            show_alignment_guides: options.show_alignment_guides,
            show_dimensions: options.show_dimensions,
            depth,
            color_scheme: GridColorScheme::default(),
        });

        logging::log(
            "DEBUG_GRID",
            &format!(
                "Grid overlay enabled: size={}, bounds={}, box_model={}, guides={}, dimensions={}",
                options.grid_size,
                options.show_bounds,
                options.show_box_model,
                options.show_alignment_guides,
                options.show_dimensions
            ),
        );

        cx.notify();
    }

    /// Hide the debug grid overlay
    fn hide_grid(&mut self, cx: &mut Context<Self>) {
        self.grid_config = None;
        logging::log("DEBUG_GRID", "Grid overlay hidden");
        cx.notify();
    }

    /// Rebuild alias and shortcut registries from current scripts/scriptlets.
    /// Returns a list of conflict messages (if any) for HUD display.
    /// Conflict rule: first-registered wins - duplicates are blocked.
    fn rebuild_registries(&mut self) -> Vec<String> {
        let mut conflicts = Vec::new();
        self.alias_registry.clear();
        self.shortcut_registry.clear();

        // Register script aliases
        for script in &self.scripts {
            if let Some(ref alias) = script.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias,
                            script.path.display(),
                            existing_path
                        ),
                    );
                } else {
                    self.alias_registry
                        .insert(alias_lower, script.path.to_string_lossy().to_string());
                }
            }
        }

        // Register scriptlet aliases
        for scriptlet in &self.scriptlets {
            if let Some(ref alias) = scriptlet.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.alias_registry.insert(alias_lower, path);
                }
            }

            // Register scriptlet shortcuts
            if let Some(ref shortcut) = scriptlet.shortcut {
                let shortcut_lower = shortcut.to_lowercase();
                if let Some(existing_path) = self.shortcut_registry.get(&shortcut_lower) {
                    conflicts.push(format!(
                        "Shortcut conflict: '{}' already used by {}",
                        shortcut,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "Conflict: shortcut '{}' in {} blocked (already used by {})",
                            shortcut, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.shortcut_registry.insert(shortcut_lower, path);
                }
            }
        }

        // Load alias overrides from ~/.scriptkit/aliases.json
        // These provide aliases for built-ins, apps, and other commands
        // that don't have their own alias metadata
        if let Ok(alias_overrides) = crate::aliases::load_alias_overrides() {
            for (command_id, alias) in alias_overrides {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' for {} blocked (already used by {})",
                            alias, command_id, existing_path
                        ),
                    );
                } else {
                    // Use the command_id as the path identifier
                    // This allows find_alias_match to find built-ins and apps
                    self.alias_registry.insert(alias_lower, command_id);
                }
            }
        }

        logging::log(
            "REGISTRY",
            &format!(
                "Rebuilt registries: {} aliases, {} shortcuts, {} conflicts",
                self.alias_registry.len(),
                self.shortcut_registry.len(),
                conflicts.len()
            ),
        );

        conflicts
    }

    /// Reset all state and return to the script list view.
    /// This clears all prompt state and resizes the window appropriately.
    fn reset_to_script_list(&mut self, cx: &mut Context<Self>) {
        let old_view = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::ActionsDialog => "ActionsDialog",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::ChatPrompt { .. } => "ChatPrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistoryView",
            AppView::AppLauncherView { .. } => "AppLauncherView",
            AppView::WindowSwitcherView { .. } => "WindowSwitcherView",
            AppView::DesignGalleryView { .. } => "DesignGalleryView",
            AppView::ScratchPadView { .. } => "ScratchPadView",
            AppView::QuickTerminalView { .. } => "QuickTerminalView",
            AppView::FileSearchView { .. } => "FileSearchView",
        };

        let old_focused_input = self.focused_input;
        logging::log(
            "UI",
            &format!(
                "Resetting to script list (was: {}, focused_input: {:?})",
                old_view, old_focused_input
            ),
        );

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This runs BEFORE clearing channels to ensure cleanup even if Drop doesn't fire
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {} during reset", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Reset view
        self.current_view = AppView::ScriptList;

        // CRITICAL: Reset focused_input to MainFilter so the cursor appears
        // This was a bug where focused_input could remain as ArgPrompt/None after
        // script exit, causing the cursor to not show in the main filter.
        self.focused_input = FocusedInput::MainFilter;
        self.gpui_input_focused = false;
        self.pending_focus = Some(FocusTarget::MainFilter);
        // Reset placeholder back to default for main menu
        self.pending_placeholder = Some(DEFAULT_PLACEHOLDER.to_string());
        logging::log(
            "FOCUS",
            "Reset focused_input to MainFilter for cursor display",
        );

        // Clear arg prompt state
        self.arg_input.clear();
        self.arg_selected_index = 0;
        // P0: Reset arg scroll handle
        self.arg_list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);

        // Clear filter and selection state for fresh menu
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.filter_coalescer.reset();
        self.pending_filter_sync = true;

        // Sync list component state and validate selection
        // This moves state mutation OUT of render() (anti-pattern fix)
        self.invalidate_grouped_cache(); // Ensure cache is fresh
        self.sync_list_state();
        self.selected_index = 0;
        self.hovered_index = None; // Reset hover state to prevent stale highlight on reopen
        self.validate_selection_bounds(cx);
        // Scroll to the very top of the list (not just reveal the item)
        // This ensures the first item is at the top, not just visible somewhere in the viewport
        self.main_list_state.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        self.last_scrolled_index = Some(self.selected_index);

        // NOTE: Window resize is NOT done here to avoid RefCell borrow conflicts.
        // Callers that need resize should use deferred resize via window_ops::queue_resize
        // after the update closure completes. The show_main_window_helper handles this
        // for the visibility flow. Other callers rely on next render to resize.

        // Clear output
        self.last_output = None;

        // Clear channels (they will be dropped, closing the connections)
        self.prompt_receiver = None;
        self.response_sender = None;

        // Clear script session (parking_lot mutex never poisons)
        *self.script_session.lock() = None;

        // Clear actions popup state (prevents stale actions dialog from persisting)
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Clear pending path action and close signal
        if let Ok(mut guard) = self.pending_path_action.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.close_path_actions.lock() {
            *guard = false;
        }

        logging::log(
            "UI",
            "State reset complete - view is now ScriptList (filter, selection, scroll cleared)",
        );
        cx.notify();
    }

    /// Ensure the selection is at the first selectable item.
    ///
    /// This is a lightweight method that only resets the selection position,
    /// without clearing the filter or other state. Call this when showing
    /// the main menu to ensure the user always starts at the top.
    ///
    /// FIX: Resolves bug where main menu sometimes opened with a random item
    /// selected instead of the first item (e.g., "Reset Window Positions"
    /// instead of "AI Chat").
    pub fn ensure_selection_at_first_item(&mut self, cx: &mut Context<Self>) {
        // Only reset selection if we're in the script list view
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }

        // Invalidate cache to ensure fresh data
        self.invalidate_grouped_cache();
        self.sync_list_state();

        // Reset selection to first item
        self.selected_index = 0;
        self.hovered_index = None; // Reset hover state to prevent stale highlight on reopen
        self.validate_selection_bounds(cx);

        // Scroll to top
        self.main_list_state.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        self.last_scrolled_index = Some(self.selected_index);

        logging::log(
            "UI",
            &format!(
                "Selection reset to first item: selected_index={}, hovered_index=None",
                self.selected_index
            ),
        );
        cx.notify();
    }

    /// Check if we're currently in a prompt view (script is running)
    fn is_in_prompt(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ArgPrompt { .. }
                | AppView::DivPrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::ScratchPadView { .. }
                | AppView::QuickTerminalView { .. }
        )
    }

    /// Submit a response to the current prompt
    ///
    /// Uses try_send() to avoid blocking the UI thread if the script's input
    /// channel is full. User-initiated actions should never freeze the UI.
    fn submit_prompt_response(
        &mut self,
        id: String,
        value: Option<String>,
        _cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!("Submitting response for {}: {:?}", id, value),
        );

        let response = Message::Submit { id, value };

        if let Some(ref sender) = self.response_sender {
            // Use try_send to avoid blocking UI thread
            // If channel is full, the script isn't reading - log warning but don't freeze UI
            match sender.try_send(response) {
                Ok(()) => {
                    logging::log("UI", "Response queued for script");
                }
                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                    // Channel is full - script isn't reading stdin fast enough
                    // This shouldn't happen in normal operation, log as warning
                    logging::log(
                        "WARN",
                        "Response channel full - script may be stuck. Response dropped.",
                    );
                }
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    // Channel disconnected - script has exited
                    logging::log("UI", "Response channel disconnected - script exited");
                }
            }
        } else {
            logging::log("UI", "No response sender available");
        }

        // Return to waiting state (script will send next prompt or exit)
        // Don't change view here - wait for next message from script
    }

    /// Get filtered choices for arg prompt
    fn filtered_arg_choices(&self) -> Vec<(usize, &Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices.iter().enumerate().collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// P0: Get filtered choices as owned data for uniform_list closure
    fn get_filtered_arg_choices_owned(&self) -> Vec<(usize, Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Create box shadows from theme configuration
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        let shadow_config = self.theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        // For black (0x000000), we use h=0, s=0, l=0
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion for shadow color
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0) // achromatic
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Show inline AI chat prompt with built-in AI provider support.
    /// This switches to the ChatPrompt view with direct AI integration (no SDK needed).
    /// Prefers Vercel AI Gateway if configured, otherwise uses the first available provider.
    pub fn show_inline_ai_chat(&mut self, initial_query: Option<String>, cx: &mut Context<Self>) {
        use crate::ai::ProviderRegistry;
        use crate::prompts::{ChatEscapeCallback, ChatPrompt, ChatSubmitCallback};

        // Mark as opened from main menu so ESC returns to main menu
        self.opened_from_main_menu = true;

        // Create escape callback that signals via channel
        let escape_sender = self.inline_chat_escape_sender.clone();
        let escape_callback: ChatEscapeCallback = std::sync::Arc::new(move |_id| {
            let _ = escape_sender.try_send(());
        });

        // Initialize provider registry from environment with config
        let registry = ProviderRegistry::from_environment_with_config(Some(&self.config));

        if !registry.has_any_provider() {
            crate::logging::log("CHAT", "No AI providers configured - showing setup card");

            // Create configure callback that signals via channel
            let configure_sender = self.inline_chat_configure_sender.clone();
            let configure_callback: crate::prompts::ChatConfigureCallback =
                std::sync::Arc::new(move || {
                    crate::logging::log("CHAT", "Configure callback triggered - sending signal");
                    let _ = configure_sender.try_send(());
                });

            // Create Claude Code callback that signals via channel
            let claude_code_sender = self.inline_chat_claude_code_sender.clone();
            let claude_code_callback: crate::prompts::ChatClaudeCodeCallback =
                std::sync::Arc::new(move || {
                    crate::logging::log(
                        "CHAT",
                        "Claude Code callback triggered - sending signal",
                    );
                    let _ = claude_code_sender.try_send(());
                });

            // Create a no-op submit callback since we're in setup mode
            let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
                crate::logging::log("CHAT", "No providers - submission ignored (setup mode)");
            });

            let chat_prompt = ChatPrompt::new(
                "inline-ai-setup".to_string(),
                Some("Configure API key to continue...".to_string()),
                vec![],
                None, // No hint needed - setup card is the UI
                None,
                self.focus_handle.clone(),
                noop_callback,
                std::sync::Arc::clone(&self.theme),
            )
            .with_title("Ask AI")
            .with_save_history(false) // Don't save setup state to history
            .with_escape_callback(escape_callback.clone())
            .with_needs_setup(true)
            .with_configure_callback(configure_callback)
            .with_claude_code_callback(claude_code_callback);

            let entity = cx.new(|_| chat_prompt);
            self.current_view = AppView::ChatPrompt {
                id: "inline-ai-setup".to_string(),
                entity,
            };
            self.focused_input = FocusedInput::None;
            self.pending_focus = Some(FocusTarget::ChatPrompt);
            resize_to_view_sync(ViewType::DivPrompt, 0);
            cx.notify();
            return;
        }

        crate::logging::log(
            "CHAT",
            &format!(
                "Showing inline AI chat with {} providers",
                registry.provider_ids().len()
            ),
        );

        // Create a no-op callback since built-in AI handles submissions internally
        let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
            // Built-in AI mode handles this internally
        });

        let placeholder = Some("Ask anything...".to_string());

        let mut chat_prompt = ChatPrompt::new(
            "inline-ai".to_string(),
            placeholder,
            vec![],
            None,
            None,
            self.focus_handle.clone(),
            noop_callback,
            std::sync::Arc::clone(&self.theme),
        )
        .with_title("Ask AI")
        .with_save_history(true)
        .with_escape_callback(escape_callback)
        .with_builtin_ai(registry, true); // true = prefer Vercel AI Gateway

        // If there's an initial query, set it in the input and auto-submit
        if let Some(query) = initial_query {
            chat_prompt.input.set_text(&query);
            chat_prompt = chat_prompt.with_pending_submit(true);
        }

        let entity = cx.new(|_| chat_prompt);
        self.current_view = AppView::ChatPrompt {
            id: "inline-ai".to_string(),
            entity,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::ChatPrompt);
        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }
}

// Note: convert_menu_bar_items/convert_menu_bar_item functions were removed
// because frontmost_app_tracker is now compiled as part of the binary crate
// (via `mod frontmost_app_tracker` in main.rs) so it returns binary types directly.
