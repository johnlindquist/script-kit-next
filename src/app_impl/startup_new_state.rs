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
            bounds_subscription: None,     // Set later after window setup
            appearance_subscription: None, // Set later after window setup
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
            emoji_scroll_handle: UniformListScrollHandle::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            file_search_scroll_handle: UniformListScrollHandle::new(),
            theme_chooser_scroll_handle: UniformListScrollHandle::new(),
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
            // Cached provider registry - built in background, None until ready
            cached_provider_registry: None,
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
