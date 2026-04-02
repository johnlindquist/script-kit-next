use super::*;

pub(super) fn calculate_fallback_error_message(expression: &str) -> String {
    format!(
        "Could not evaluate expression \"{}\". Check the syntax and try again.",
        expression
    )
}

impl ScriptListApp {
    pub(crate) fn new(
        config: config::Config,
        bun_available: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // PERF: Parallelize script + scriptlet loading to reduce startup wall time.
        let load_start = std::time::Instant::now();
        let (scripts, scriptlets, scripts_elapsed, scriptlets_elapsed) = std::thread::scope(
            |scope| {
                let scripts_handle = scope.spawn(|| {
                    let start = std::time::Instant::now();
                    let loaded = scripts::read_scripts();
                    (loaded, start.elapsed())
                });

                let scriptlets_handle = scope.spawn(|| {
                    let start = std::time::Instant::now();
                    // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
                    // This includes built-in extensions like CleanShot and user extensions
                    let loaded = scripts::load_scriptlets();
                    (loaded, start.elapsed())
                });

                let (scripts, scripts_elapsed) = match scripts_handle.join() {
                    Ok(result) => result,
                    Err(_) => {
                        logging::log(
                            "PERF",
                            "Script loading thread panicked; retrying read_scripts synchronously",
                        );
                        let retry_start = std::time::Instant::now();
                        (scripts::read_scripts(), retry_start.elapsed())
                    }
                };

                let (scriptlets, scriptlets_elapsed) = match scriptlets_handle.join() {
                    Ok(result) => result,
                    Err(_) => {
                        logging::log(
                            "PERF",
                            "Scriptlet loading thread panicked; retrying load_scriptlets synchronously",
                        );
                        let retry_start = std::time::Instant::now();
                        (scripts::load_scriptlets(), retry_start.elapsed())
                    }
                };

                (scripts, scriptlets, scripts_elapsed, scriptlets_elapsed)
            },
        );

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
            // Use an async channel so the UI task can await completion without polling.
            let (tx, rx) =
                async_channel::bounded::<(Vec<app_launcher::AppInfo>, std::time::Duration)>(1);

            // Spawn background thread for app scanning
            std::thread::spawn(move || {
                let start = std::time::Instant::now();
                let apps = app_launcher::scan_applications().clone();
                let elapsed = start.elapsed();
                if tx.send_blocking((apps, elapsed)).is_err() {
                    logging::log(
                        "APP",
                        "Background app loading result dropped: receiver unavailable",
                    );
                }
            });

            // Event-driven receive: no timer wakeups while waiting for app scan completion.
            cx.spawn(async move |this, cx| {
                let Ok((apps, elapsed)) = rx.recv().await else {
                    logging::log(
                        "APP",
                        "Background app loading failed to deliver result: channel closed",
                    );
                    return;
                };

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
                                old_count, new_count, app.computed_filter_text
                            ),
                        );
                        cx.notify();
                    })
                });
            })
            .detach();
        }
        logging::log("UI", "Script Kit logo SVG loaded for header rendering");

        // Start cursor blink timer - updates all inputs that track cursor visibility
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_millis(530)).await;

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
                            "Main input focused while actions open - closing actions via shared close path",
                        );
                        this.close_actions_popup(ActionsDialogHost::MainList, window, cx);
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
                    // Block Enter when confirm popup is open — the confirm
                    // popup's key routing already handled this Enter via
                    // capture_key_down in render_impl.
                    if confirm::is_confirm_window_open() {
                        logging::log("KEY", "Ignoring PressEnter: confirm popup is open");
                        return;
                    }

                    logging::log(
                        "KEY",
                        &format!(
                            "PressEnter event: visible={}, grace={}, view={:?}, actions_popup={}, fallback_mode={}, selected_index={}, filter_len={}",
                            script_kit_gpui::is_main_window_visible(),
                            script_kit_gpui::is_within_focus_grace_period(),
                            std::mem::discriminant(&this.current_view),
                            this.show_actions_popup,
                            this.fallback_mode,
                            this.selected_index,
                            this.filter_text.len()
                        ),
                    );

                    if !script_kit_gpui::is_main_window_visible() {
                        logging::log("KEY", "Ignoring PressEnter: main window not visible");
                        return;
                    }

                    if script_kit_gpui::is_within_focus_grace_period() {
                        logging::log("KEY", "Ignoring PressEnter: within focus grace period");
                        return;
                    }

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

        // Legacy chat channels (retained for inline chat compatibility — not the primary Tab AI surface)
        let (inline_chat_escape_tx, inline_chat_escape_rx) = mpsc::sync_channel(4);
        let (inline_chat_continue_tx, inline_chat_continue_rx) = mpsc::sync_channel(4);
        let (inline_chat_configure_tx, inline_chat_configure_rx) = mpsc::sync_channel(4);
        let (inline_chat_claude_code_tx, inline_chat_claude_code_rx) = mpsc::sync_channel(4);
        // Create channel for naming dialog completion signals
        let (naming_submit_tx, naming_submit_rx) = mpsc::sync_channel(4);
        let mut app = ScriptListApp {
            scripts,
            scriptlets,
            builtin_entries,
            apps,
            // P0 FIX: Cached data for builtin views (avoids cloning per frame)
            cached_clipboard_entries: Vec::new(),
            paste_sequential_state: None,
            focused_clipboard_entry_id: None,
            cached_windows: Vec::new(),
            cached_file_results: Vec::new(),
            cached_processes: Vec::new(),
            process_manager_refresh_task: None,
            cached_current_app_entries: Vec::new(),
            selected_index: 0,
            filter_text: String::new(),
            inline_calculator: None,
            gpui_input_state,
            gpui_input_focused: false,
            gpui_input_subscriptions: vec![gpui_input_subscription],
            bounds_subscription: None,     // Set later after window setup
            appearance_subscription: None, // Set later after window setup
            suppress_filter_events: false,
            pending_filter_sync: false,
            pending_placeholder: None,
            last_output: None,
            focus_handle: cx.focus_handle(),
            show_logs: false,
            show_info_panel: false,
            theme,
            config,
            // Scroll activity tracking: start with scrollbar hidden
            scrollbar_visibility: crate::transitions::Opacity::INVISIBLE,
            scrollbar_fade_gen: 0,
            last_scroll_time: None,
            current_view: AppView::ScriptList,
            main_window_mode: MainWindowMode::Mini,
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
            emoji_scroll_handle: UniformListScrollHandle::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            process_list_scroll_handle: UniformListScrollHandle::new(),
            current_app_commands_scroll_handle: UniformListScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            file_search_scroll_handle: UniformListScrollHandle::new(),
            theme_chooser_scroll_handle: UniformListScrollHandle::new(),
            file_search_loading: false,
            file_search_debounce_task: None,
            file_search_current_dir: None,
            file_search_frozen_filter: None,
            file_search_actions_path: None,
            file_search_sort_mode: crate::actions::FileSearchSortMode::default(),
            file_search_gen: 0,
            file_search_cancel: None,
            file_search_display_indices: Vec::new(),
            file_search_preview_thumbnail: FileSearchThumbnailPreviewState::Idle,
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
            cached_grouped_first_selectable_index: None,
            cached_grouped_last_selectable_index: None,
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
            theme_before_chooser: None,
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
            pending_tab_ai_execution: None,
            tab_ai_save_offer_state: None,
            tab_ai_harness: None,
            tab_ai_harness_capture_generation: 0,
            tab_ai_harness_return_view: None,
            tab_ai_harness_return_focus_target: None,
            tab_ai_harness_apply_back_route: None,
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
            active_favorites: None,
            // Inline chat escape channel - for ChatPrompt escape callback to signal return to main menu
            inline_chat_escape_sender: inline_chat_escape_tx,
            inline_chat_escape_receiver: inline_chat_escape_rx,
            // Inline chat continue channel - for ChatPrompt continue callback to hide main window
            inline_chat_continue_sender: inline_chat_continue_tx,
            inline_chat_continue_receiver: inline_chat_continue_rx,
            // Inline chat configure channel - for ChatPrompt configure callback to trigger API key setup
            inline_chat_configure_sender: inline_chat_configure_tx,
            inline_chat_configure_receiver: inline_chat_configure_rx,
            // Inline chat Claude Code channel - for ChatPrompt Claude Code callback to enable Claude Code
            inline_chat_claude_code_sender: inline_chat_claude_code_tx,
            inline_chat_claude_code_receiver: inline_chat_claude_code_rx,
            // Naming dialog completion channel - for NamingPrompt callback to signal submit/cancel
            naming_submit_sender: naming_submit_tx,
            naming_submit_receiver: naming_submit_rx,
            // Light theme opacity adjustment offset (Cmd+Shift+[/])
            light_opacity_offset: 0.0,
            // Mouse cursor hidden state - hidden while typing, shown on mouse move
            mouse_cursor_hidden: false,
            // Cached provider registry - built in background, None until ready
            cached_provider_registry: None,
            cached_main_window_preflight: None,
            main_window_preflight_cache_key: String::from("\0_UNINITIALIZED_\0"),
            window_orchestrator: crate::window_orchestrator::OrchestratorState::default(),
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
        // Build provider registry in background to avoid blocking UI when opening AI chat
        {
            let config_clone = app.config.clone();
            let (tx, rx) = async_channel::bounded::<crate::ai::ProviderRegistry>(1);

            std::thread::spawn(move || {
                let registry =
                    crate::ai::ProviderRegistry::from_environment_with_config(Some(&config_clone));
                if tx.send_blocking(registry).is_err() {
                    logging::log(
                        "APP",
                        "Provider registry build result dropped: receiver unavailable",
                    );
                }
            });

            cx.spawn(async move |this, cx| {
                let Ok(registry) = rx.recv().await else {
                    logging::log(
                        "APP",
                        "Background provider registry build failed: channel closed",
                    );
                    return;
                };

                let provider_count = registry.provider_ids().len();
                let _ = cx.update(|cx| {
                    this.update(cx, |app, _cx| {
                        app.cached_provider_registry = Some(registry);
                        logging::log(
                            "APP",
                            &format!(
                                "Background provider registry ready: {} providers",
                                provider_count
                            ),
                        );
                    })
                });
            })
            .detach();
        }

        // Add Tab key interceptor for "Ask AI" feature and file search directory navigation
        // This fires BEFORE normal key handling, allowing us to intercept Tab
        // even when the Input component has focus
        let app_entity_for_tab = cx.entity().downgrade();
        let tab_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_tab;
            move |event, window, cx| {
                // When the main window is hidden (e.g. Notes/AI open), main-menu
                // key interceptors must not consume keystrokes from secondary windows.
                if !script_kit_gpui::is_main_window_visible() {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let is_tab_key = key.eq_ignore_ascii_case("tab");
                let has_shift = event.keystroke.modifiers.shift;
                // Check for Tab key (no cmd/alt/ctrl modifiers, but shift is allowed)
                if is_tab_key
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Handle Tab/Shift+Tab in FileSearchView for directory/file navigation
                            // CRITICAL: ALWAYS consume Tab/Shift+Tab to prevent focus traversal
                            if let AppView::FileSearchView {
                                query,
                                selected_index,
                                ..
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
                                    // Reuse the precomputed display order instead of
                                    // re-running Nucleo on every Tab keypress.
                                    // Use direct field access to avoid borrow conflict
                                    // with the &mut selected_index from the pattern match.
                                    let display_len =
                                        this.file_search_display_indices.len();
                                    if display_len > 0 {
                                        let clamped =
                                            (*selected_index).min(display_len - 1);
                                        *selected_index = clamped;

                                        let file_info = this
                                            .file_search_display_indices
                                            .get(clamped)
                                            .and_then(|&ri| {
                                                this.cached_file_results.get(ri)
                                            })
                                            .map(|f| (f.file_type.clone(), f.path.clone()));

                                        if let Some((file_type, path)) = file_info {
                                            if file_type
                                                == crate::file_search::FileType::Directory
                                            {
                                                let shortened =
                                                    crate::file_search::shorten_path(&path);
                                                let new_path = format!(
                                                    "{}/",
                                                    shortened.trim_end_matches('/')
                                                );
                                                crate::logging::log(
                                                    "KEY",
                                                    &format!(
                                                        "Tab: Entering directory: {}",
                                                        new_path
                                                    ),
                                                );

                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, cx| {
                                                        state.set_value(
                                                            new_path.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                        let len = new_path.len();
                                                        state.set_selection(
                                                            len, len, window, cx,
                                                        );
                                                    },
                                                );

                                                cx.notify();
                                            } else {
                                                let shortened =
                                                    crate::file_search::shorten_path(&path);
                                                crate::logging::log(
                                                    "KEY",
                                                    &format!(
                                                        "Tab: Autocompleting file path: {}",
                                                        shortened
                                                    ),
                                                );

                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, cx| {
                                                        state.set_value(
                                                            shortened.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                        let len = shortened.len();
                                                        state.set_selection(
                                                            len, len, window, cx,
                                                        );
                                                    },
                                                );

                                                cx.notify();
                                            }
                                        }
                                    } else {
                                        crate::logging::log(
                                            "KEY",
                                            "Tab: No selection to autocomplete, no-op",
                                        );
                                    }
                                }
                                return;
                            }

                            // Handle Tab/Shift+Tab in ChatPrompt setup mode
                            // Must intercept here to prevent GPUI focus traversal from consuming Tab
                            if let AppView::ChatPrompt { entity, .. } = &this.current_view {
                                let handled = entity.update(cx, |chat, cx| {
                                    chat.handle_setup_key("tab", has_shift, cx)
                                });
                                if handled {
                                    cx.stop_propagation();
                                    return;
                                }
                            }

                            // Shift+Tab in ScriptList: route typed query through the
                            // quick-submit planner so the harness gets intelligent
                            // classification, synthesized intent, and the right
                            // capture kind — not just a raw string paste.
                            if has_shift
                                && matches!(this.current_view, AppView::ScriptList)
                                && !this.filter_text.is_empty()
                                && !this.show_actions_popup
                            {
                                let query = this.filter_text.clone();
                                this.submit_to_current_or_new_tab_ai_harness_from_text(
                                    query,
                                    crate::ai::TabAiQuickSubmitSource::ShiftTab,
                                    cx,
                                );
                                cx.stop_propagation();
                                return;
                            }

                            // Consume Tab/Shift+Tab while the ACP chat is
                            // open so the universal Tab AI fallback does not
                            // spawn a second ACP session on top of this one.
                            if let AppView::AcpChatView { entity, .. } = &this.current_view {
                                let handled = entity.update(cx, |chat, cx| {
                                    chat.handle_tab_key(has_shift, cx)
                                });
                                if handled {
                                    cx.stop_propagation();
                                    return;
                                }
                            }

                            // Forward Tab/Shift+Tab directly to the harness
                            // terminal PTY.  We must NOT call cx.propagate()
                            // here because GPUI's built-in focus-traversal
                            // would consume the Tab keystroke before it reaches
                            // the TermPrompt key handler.  Instead, write the
                            // raw byte to the PTY and stop propagation.
                            if let AppView::QuickTerminalView { entity, .. } = &this.current_view {
                                entity.update(cx, |term, _cx| {
                                    let running = term.terminal.is_running();
                                    let bytes: &[u8] = if has_shift {
                                        b"\x1b[Z" // Shift+Tab (backtab)
                                    } else {
                                        b"\t" // Tab
                                    };
                                    if !running {
                                        tracing::warn!(
                                            event = "quick_terminal_tab_pty_dead",
                                            has_shift,
                                            "Tab intercepted but PTY is not running"
                                        );
                                        return;
                                    }
                                    match term.terminal.input(bytes) {
                                        Ok(()) => tracing::debug!(
                                            event = "quick_terminal_tab_sent",
                                            has_shift,
                                            "Tab byte written to PTY"
                                        ),
                                        Err(e) => tracing::warn!(
                                            event = "quick_terminal_tab_write_failed",
                                            error = %e,
                                            has_shift,
                                            "Failed to write Tab to PTY"
                                        ),
                                    }
                                });
                                cx.stop_propagation();
                                return;
                            }

                            // Block Tab while the save-offer overlay is visible
                            if this.tab_ai_save_offer_state.is_some() {
                                cx.stop_propagation();
                                return;
                            }

                            // Universal Tab AI: open the harness terminal surface
                            // from any non-special surface (not FileSearch, not ChatPrompt setup)
                            if !has_shift && !this.show_actions_popup {
                                this.open_tab_ai_chat(cx);
                                cx.stop_propagation();
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(tab_interceptor);

        // Prewarm the Tab AI harness asynchronously so the first Tab press
        // reuses a live PTY instead of paying spawn cost. Runs once, silently.
        let app_entity_for_tab_ai_warm = cx.entity().downgrade();
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;
            let _ = cx.update(|cx| {
                let Some(app) = app_entity_for_tab_ai_warm.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| {
                    this.warm_tab_ai_harness_on_startup(cx);
                });
            });
        })
        .detach();

        // Add arrow key interceptor for builtin views with Input components
        // This fires BEFORE Input component handles arrow keys, allowing list navigation
        let app_entity_for_arrows = cx.entity().downgrade();
        let arrow_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_arrows;
            move |event, window, cx| {
                // When the main window is hidden (e.g. Notes/AI open), main-menu
                // key interceptors must not consume keystrokes from secondary windows.
                if !script_kit_gpui::is_main_window_visible() {
                    return;
                }

                // intercept_keystrokes is GLOBAL and fires for ALL windows in the app.
                // Keep main list arrow routing scoped to the main window so notes/AI/actions
                // windows receive their own navigation key events.
                if crate::notes::is_notes_window(window)
                    || crate::ai::is_ai_window(window)
                    || crate::ai::acp::chat_window::is_chat_window(window)
                    || crate::actions::is_actions_window(window)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let is_up = crate::ui_foundation::is_key_up(key);
                let is_down = crate::ui_foundation::is_key_down(key);
                let is_left = crate::ui_foundation::is_key_left(key);
                let is_right = crate::ui_foundation::is_key_right(key);
                let no_direction_modifiers = !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control;

                // Emoji picker uses Left/Right to navigate the grid and must consume
                // those keys before the search input moves its text cursor.
                if (is_left || is_right) && no_direction_modifiers {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Keep left/right from moving any focused input while actions popup is open.
                            if this.show_actions_popup {
                                cx.stop_propagation();
                                return;
                            }

                            if let AppView::EmojiPickerView {
                                selected_index,
                                filter,
                                selected_category,
                            } = &mut this.current_view
                            {
                                let ordered = crate::emoji::filtered_ordered_emojis(
                                    filter,
                                    *selected_category,
                                );
                                let filtered_len = ordered.len();
                                if filtered_len == 0 {
                                    *selected_index = 0;
                                    this.hovered_index = None;
                                    cx.notify();
                                    cx.stop_propagation();
                                    return;
                                }

                                if *selected_index >= filtered_len {
                                    *selected_index = filtered_len - 1;
                                }

                                let old_idx = *selected_index;
                                *selected_index = if is_left {
                                    old_idx.saturating_sub(1)
                                } else {
                                    (old_idx + 1).min(filtered_len.saturating_sub(1))
                                };

                                let row =
                                    crate::emoji::compute_scroll_row(*selected_index, &ordered);
                                this.emoji_scroll_handle.scroll_to_item(row, ScrollStrategy::Nearest);

                                this.input_mode = InputMode::Keyboard;
                                this.hovered_index = None;
                                cx.notify();
                                cx.stop_propagation();
                            }
                        });
                    }
                }
                // Check for Up/Down arrow keys (no modifiers except shift for selection)
                if (is_up || is_down) && no_direction_modifiers {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Universal: Route arrow keys to actions dialog when popup is open
                            // This ensures ALL views (ChatPrompt, ArgPrompt, etc.) route
                            // arrows to the dialog, not just the few views with explicit cases below.
                            if this.show_actions_popup {
                                if let Some(ref dialog) = this.actions_dialog {
                                    if is_up {
                                        dialog.update(cx, |d, cx| d.move_up(cx));
                                    } else if is_down {
                                        dialog.update(cx, |d, cx| d.move_down(cx));
                                    }
                                    crate::actions::notify_actions_window(cx);
                                }
                                cx.stop_propagation();
                                return;
                            }

                            // Only intercept in views that use Input + list navigation
                            match &mut this.current_view {
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                    ..
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if is_up {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if is_down {
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

                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
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
                                            if is_up {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if is_down {
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
                                                    || e.ocr_text
                                                        .as_deref()
                                                        .unwrap_or("")
                                                        .to_lowercase()
                                                        .contains(&filter_lower)
                                            })
                                            .collect()
                                    };
                                    let filtered_len = filtered_entries.len();
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.clipboard_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                    } else if is_down && *selected_index + 1 < filtered_len {
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
                                    let old_index = *selected_index;

                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                    }

                                    if *selected_index != old_index {
                                        tracing::debug!(
                                            target: "script_kit::scroll",
                                            event = "builtin_selection_nav",
                                            view = "app_launcher",
                                            old_index,
                                            new_index = *selected_index,
                                            total_items = filtered_len,
                                            strategy = "nearest",
                                        );

                                        this.list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        this.input_mode = InputMode::Keyboard;
                                        this.hovered_index = None;
                                        cx.notify();
                                    }

                                    cx.stop_propagation();
                                }
                                AppView::WindowSwitcherView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    let filtered_len = this.cached_windows.len();
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.window_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        this.window_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::ProcessManagerView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    let filtered_len = this.cached_processes.len();
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.process_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        this.process_list_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::CurrentAppCommandsView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    let filtered_len = this.cached_current_app_entries.len();
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.current_app_commands_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        this.current_app_commands_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::SearchAiPresetsView {
                                    selected_index,
                                    filter: _,
                                } => {
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        cx.notify();
                                    } else if is_down {
                                        *selected_index += 1;
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::ScriptList => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if is_up {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if is_down {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Main menu: handle list navigation + input history
                                    const HISTORY: &str = "HISTORY";
                                    if is_up {
                                        let (grouped_items, _) = this.get_grouped_results_cached();
                                        let first_item_position =
                                            grouped_items.iter().position(|item| {
                                                matches!(
                                                    item,
                                                    crate::list_item::GroupedListItem::Item(_)
                                                )
                                            });
                                        let at_top_of_list = first_item_position
                                            .map(|position| this.selected_index <= position)
                                            .unwrap_or(true);
                                        let in_history =
                                            this.input_history.current_index().is_some();

                                        if in_history || at_top_of_list {
                                            if let Some(text) = this.input_history.navigate_up() {
                                                logging::log(
                                                    HISTORY,
                                                    &format!("Recalled: {}", text),
                                                );
                                                this.filter_text = text.clone();
                                                let text_len = text.len();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            text.clone(),
                                                            window,
                                                            input_cx,
                                                        );
                                                        state.set_selection(
                                                            text_len, text_len, window, input_cx,
                                                        );
                                                    },
                                                );
                                                this.queue_filter_compute(text, cx);
                                                cx.notify();
                                            }
                                            cx.stop_propagation();
                                            return;
                                        }

                                        this.move_selection_up(cx);
                                    } else if is_down {
                                        if this.input_history.current_index().is_some() {
                                            if let Some(text) = this.input_history.navigate_down() {
                                                logging::log(
                                                    HISTORY,
                                                    &format!("Recalled: {}", text),
                                                );
                                                this.filter_text = text.clone();
                                                let text_len = text.len();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            text.clone(),
                                                            window,
                                                            input_cx,
                                                        );
                                                        state.set_selection(
                                                            text_len, text_len, window, input_cx,
                                                        );
                                                    },
                                                );
                                                this.queue_filter_compute(text, cx);
                                                cx.notify();
                                            } else {
                                                this.input_history.reset_navigation();
                                                this.filter_text.clear();
                                                this.gpui_input_state.update(
                                                    cx,
                                                    |state, input_cx| {
                                                        state.set_value(
                                                            String::new(),
                                                            window,
                                                            input_cx,
                                                        );
                                                        state
                                                            .set_selection(0, 0, window, input_cx);
                                                    },
                                                );
                                                this.queue_filter_compute(String::new(), cx);
                                                cx.notify();
                                            }
                                            cx.stop_propagation();
                                            return;
                                        }

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

        // Add Home/End/PageUp/PageDown key interceptor for jump navigation
        let app_entity_for_home_end = cx.entity().downgrade();
        let home_end_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_home_end;
            move |event, window, cx| {
                // When the main window is hidden (e.g. Notes/AI open), main-menu
                // key interceptors must not consume keystrokes from secondary windows.
                if !script_kit_gpui::is_main_window_visible() {
                    return;
                }

                // Skip processing if this keystroke is from a secondary window
                if crate::notes::is_notes_window(window)
                    || crate::ai::is_ai_window(window)
                    || crate::ai::acp::chat_window::is_chat_window(window)
                    || crate::actions::is_actions_window(window)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_platform_mod = event.keystroke.modifiers.platform; // Cmd on macOS

                // Home key or Cmd+Up → jump to first item
                // End key or Cmd+Down → jump to last item
                let is_home = key.eq_ignore_ascii_case("home")
                    || (has_platform_mod && crate::ui_foundation::is_key_up(key));
                let is_end = key.eq_ignore_ascii_case("end")
                    || (has_platform_mod && crate::ui_foundation::is_key_down(key));
                // Page Up/Down → move by ~10 selectable items
                let is_page_up = key.eq_ignore_ascii_case("pageup");
                let is_page_down = key.eq_ignore_ascii_case("pagedown");

                if !is_home && !is_end && !is_page_up && !is_page_down {
                    return;
                }

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Only handle in ScriptList view
                        if !matches!(this.current_view, AppView::ScriptList) {
                            return;
                        }

                        // Don't handle if actions popup is open
                        if this.show_actions_popup {
                            return;
                        }

                        if is_home {
                            this.move_selection_to_first(cx);
                        } else if is_end {
                            this.move_selection_to_last(cx);
                        } else if is_page_up {
                            this.move_selection_page_up(cx);
                        } else if is_page_down {
                            this.move_selection_page_down(cx);
                        }

                        cx.stop_propagation();
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(home_end_interceptor);

        // Add interceptor for actions popup in FileSearchView and ScriptList
        // This handles Cmd+K (toggle), Escape (close), Enter (submit), and typing
        let app_entity_for_actions = cx.entity().downgrade();
        let actions_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_actions;
            move |event, window, cx| {
                // When the main window is hidden (e.g. Notes/AI open), main-menu
                // key interceptors must not consume keystrokes from secondary windows.
                if !script_kit_gpui::is_main_window_visible() {
                    return;
                }

                // CRITICAL: Skip processing if this keystroke is from a secondary window.
                // intercept_keystrokes is GLOBAL and fires for ALL windows in the app.
                // We only want to handle keystrokes for the main window.
                if crate::notes::is_notes_window(window)
                    || crate::ai::is_ai_window(window)
                    || crate::ai::acp::chat_window::is_chat_window(window)
                    || crate::actions::is_actions_window(window)
                {
                    return; // Let the secondary window handle its own keystrokes
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;
                let key_char = event.keystroke.key_char.as_deref();

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Handle Cmd+K to toggle actions popup (works in ScriptList, FileSearchView, ArgPrompt)
                        // This MUST be intercepted here because the Input component has focus and
                        // normal on_key_down handlers won't receive the event
                        if has_cmd && key.eq_ignore_ascii_case("k") && !has_shift {
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
                                    ..
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

                                    let selected_file = filtered_results
                                        .get(*selected_index)
                                        .map(|(_, file)| (*file).clone());
                                    this.toggle_file_search_actions(
                                        selected_file.as_ref(),
                                        window,
                                        cx,
                                    );
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
                                AppView::WebcamView { .. } => {
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_webcam_actions (WebcamView)");
                                    this.toggle_webcam_actions(cx, window);
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
                                AppView::AcpChatView { .. } => {
                                    logging::log("KEY", "Interceptor: Cmd+K -> toggle_actions (AcpChatView)");
                                    this.toggle_actions(cx, window);
                                    cx.stop_propagation();
                                    return;
                                }
                                _ => {
                                    // Other views don't support Cmd+K actions
                                }
                            }
                        }

                        // Handle Cmd+W for AcpChatView (restores previous view, not full window close)
                        if has_cmd && key.eq_ignore_ascii_case("w") && !has_shift
                            && matches!(this.current_view, AppView::AcpChatView { .. })
                        {
                            logging::log("KEY", "Interceptor: Cmd+W -> close ACP chat");
                            this.close_tab_ai_harness_terminal(cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Cmd+Shift+K for add_shortcut in ScriptList
                        if has_cmd && key.eq_ignore_ascii_case("k") && has_shift
                            && matches!(this.current_view, AppView::ScriptList)
                        {
                            logging::log("KEY", "Interceptor: Cmd+Shift+K -> add_shortcut (ScriptList)");
                            this.handle_action("add_shortcut".to_string(), window, cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Window tweaker shortcuts (only enabled with SCRIPT_KIT_WINDOW_TWEAKER=1)
                        let window_tweaker_enabled = std::env::var("SCRIPT_KIT_WINDOW_TWEAKER")
                            .map(|v| v == "1")
                            .unwrap_or(false);

                        if window_tweaker_enabled {
                            // Handle Cmd+- to decrease light theme opacity
                            if has_cmd
                                && !has_shift
                                && (key == "-" || key.eq_ignore_ascii_case("minus"))
                            {
                                logging::log("KEY", &format!("Interceptor: Cmd+- (key={}) -> decrease light opacity", key));
                                this.adjust_light_opacity(-0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+= (or Cmd+Shift+=) to increase light theme opacity
                            if has_cmd
                                && (key == "="
                                    || key.eq_ignore_ascii_case("equal")
                                    || key.eq_ignore_ascii_case("plus"))
                            {
                                logging::log("KEY", &format!("Interceptor: Cmd+= (key={}) -> increase light opacity", key));
                                this.adjust_light_opacity(0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+M to cycle vibrancy material (blur effect)
                            if has_cmd && !has_shift && key.eq_ignore_ascii_case("m") {
                                logging::log("KEY", "Interceptor: Cmd+M -> cycle vibrancy material");
                                let description = platform::cycle_vibrancy_material();
                                this.toast_manager.push(
                                    components::toast::Toast::info(
                                        description,
                                        &this.theme,
                                    )
                                    .duration_ms(Some(TOAST_INFO_MS)),
                                );
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+Shift+A to cycle vibrancy appearance (VibrantLight, VibrantDark, etc.)
                            if has_cmd && has_shift && key.eq_ignore_ascii_case("a") {
                                logging::log("KEY", "Interceptor: Cmd+Shift+A -> cycle vibrancy appearance");
                                let description = platform::cycle_appearance();
                                this.toast_manager.push(
                                    components::toast::Toast::info(
                                        description,
                                        &this.theme,
                                    )
                                    .duration_ms(Some(TOAST_INFO_MS)),
                                );
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }
                        }

                        // Only handle remaining keys if in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            // Arrow keys are handled by arrow_interceptor to avoid double-processing
                            // (which can skip 2 items per keypress when both interceptors handle arrows).
                            if crate::ui_foundation::is_key_up(key)
                                || crate::ui_foundation::is_key_down(key)
                            {
                                return;
                            }

                            // Route modal actions keys for all views that support actions dialogs.
                            // This ensures enter, escape, backspace, and character keys are
                            // routed to the actions dialog when it's open, regardless of view type.
                            let host = match &this.current_view {
                                AppView::ScriptList => Some(ActionsDialogHost::MainList),
                                AppView::ClipboardHistoryView { .. } => Some(ActionsDialogHost::ClipboardHistory),
                                AppView::EmojiPickerView { .. } => Some(ActionsDialogHost::EmojiPicker),
                                AppView::ChatPrompt { .. } => Some(ActionsDialogHost::ChatPrompt),
                                AppView::ArgPrompt { .. } => Some(ActionsDialogHost::ArgPrompt),
                                AppView::DivPrompt { .. } => Some(ActionsDialogHost::DivPrompt),
                                AppView::EditorPrompt { .. } => Some(ActionsDialogHost::EditorPrompt),
                                AppView::TermPrompt { .. } => Some(ActionsDialogHost::TermPrompt),
                                AppView::FormPrompt { .. } => Some(ActionsDialogHost::FormPrompt),
                                AppView::WebcamView { .. } => Some(ActionsDialogHost::WebcamPrompt),
                                AppView::AcpChatView { .. } => Some(ActionsDialogHost::AcpChat),
                                _ => None,
                            };

                            if let Some(host) = host {
                                match this.route_key_to_actions_dialog(
                                    key,
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
                                        match host {
                                            ActionsDialogHost::ChatPrompt => {
                                                this.execute_chat_action(&action_id, cx);
                                            }
                                            ActionsDialogHost::ArgPrompt => {
                                                this.trigger_action_by_name(&action_id, cx);
                                            }
                                            ActionsDialogHost::WebcamPrompt => {
                                                let start = std::time::Instant::now();
                                                let dctx = crate::action_helpers::DispatchContext::for_builtin("builtin-webcam");
                                                let outcome = this.execute_webcam_action(&action_id, &dctx, cx);
                                                Self::log_builtin_outcome("builtin-webcam", &dctx, "webcam_action", &outcome, &start);
                                            }
                                            _ => {
                                                this.handle_action(action_id, window, cx);
                                            }
                                        }
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
                        if crate::ui_foundation::is_key_escape(key) {
                            this.close_actions_popup(ActionsDialogHost::FileSearch, window, cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Enter to submit selected action
                        if crate::ui_foundation::is_key_enter(key) {
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
                                    this.handle_action(action_id, window, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Backspace for actions search
                        if key.eq_ignore_ascii_case("backspace") {
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
        app.rebuild_main_window_preflight_if_needed();

        app
    }
}
