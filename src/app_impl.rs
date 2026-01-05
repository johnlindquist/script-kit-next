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
        let scriptlets = scripts::read_scriptlets();
        let scriptlets_elapsed = scriptlets_start.elapsed();

        let theme = theme::load_theme();
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
            &format!("Loaded {} scripts from ~/.scriptkit/scripts", scripts.len()),
        );
        logging::log(
            "APP",
            &format!(
                "Loaded {} scriptlets from ~/.scriptkit/scriptlets/scriptlets.md",
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
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        // Skip cursor blink when:
                        // 1. Window is hidden (no visual feedback needed)
                        // 2. Window is not focused (prevents wasted work + incorrect UX)
                        // 3. No input is focused (no cursor to blink)
                        if !script_kit_gpui::is_main_window_visible()
                            || !platform::is_main_window_focused()
                            || app.focused_input == FocusedInput::None
                        {
                            return;
                        }

                        app.cursor_visible = !app.cursor_visible;
                        // Also update ActionsDialog cursor if it exists
                        if let Some(ref dialog) = app.actions_dialog {
                            dialog.update(cx, |d, _cx| {
                                d.set_cursor_visible(app.cursor_visible);
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
                    this.handle_filter_input_change(window, cx);
                }
                InputEvent::PressEnter { .. } => {
                    if matches!(this.current_view, AppView::ScriptList) && !this.show_actions_popup
                    {
                        this.execute_selected(cx);
                    }
                }
            }
        });

        let mut app = ScriptListApp {
            scripts,
            scriptlets,
            builtin_entries,
            apps,
            // P0 FIX: Cached data for builtin views (avoids cloning per frame)
            cached_clipboard_entries: Vec::new(),
            cached_windows: Vec::new(),
            selected_index: 0,
            filter_text: String::new(),
            gpui_input_state,
            gpui_input_focused: false,
            gpui_input_subscriptions: vec![gpui_input_subscription],
            suppress_filter_events: false,
            pending_filter_sync: false,
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
            // P0-2: Initialize hover debounce timer
            last_hover_notify: std::time::Instant::now(),
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
            // Pending focus: start with MainFilter since that's what we want focused initially
            pending_focus: Some(FocusTarget::MainFilter),
            // Scroll stabilization: track last scrolled index for each handle
            last_scrolled_main: None,
            last_scrolled_arg: None,
            last_scrolled_clipboard: None,
            last_scrolled_window: None,
            last_scrolled_design_gallery: None,
            // Show warning banner when bun is not available
            show_bun_warning: !bun_available,
            // Pending confirmation for dangerous actions
            pending_confirmation: None,
            // Menu bar integration: Now handled by frontmost_app_tracker module
            // which pre-fetches menu items in background when apps activate
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
        self.theme = theme::load_theme();
        logging::log("APP", "Theme reloaded based on system appearance");

        // Propagate theme to open ActionsDialog (if any) for hot-reload support
        if let Some(ref dialog) = self.actions_dialog {
            let theme_arc = std::sync::Arc::new(self.theme.clone());
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

    /// Apply pending focus if set. Called at the start of render() when window
    /// is focused. This applies focus exactly once, then clears pending_focus.
    ///
    /// Returns true if focus was applied (for logging/debugging).
    fn apply_pending_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        // Only apply if window is actually focused (avoid focus thrash)
        if !platform::is_main_window_focused() {
            return false;
        }

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
            FocusTarget::AppRoot => {
                window.focus(&self.focus_handle, cx);
                self.focused_input = FocusedInput::None;
            }
        }

        true
    }

    fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        self.scripts = scripts::read_scripts();
        self.scriptlets = scripts::read_scriptlets();
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

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
    /// 3. Updates hotkeys/expand triggers incrementally
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
                    s.expand.clone(),
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
                    s.expand.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Compute diff
        let diff = diff_scriptlets(&old_scriptlets, &new_scriptlets);

        if diff.is_empty() {
            logging::log("APP", &format!("No changes detected in {}", path.display()));
            return;
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet diff: {} added, {} removed, {} shortcut changes, {} expand changes, {} alias changes",
                diff.added.len(),
                diff.removed.len(),
                diff.shortcut_changes.len(),
                diff.expand_changes.len(),
                diff.alias_changes.len()
            ),
        );

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

        // Apply expand manager changes (macOS only)
        #[cfg(target_os = "macos")]
        {
            // For removed scriptlets, clear their triggers
            for removed in &diff.removed {
                if removed.expand.is_some() {
                    // We'd need access to the expand manager here
                    // For now, log that we would clear triggers
                    logging::log(
                        "EXPAND",
                        &format!("Would clear expand trigger for removed: {}", removed.name),
                    );
                }
            }

            // For added scriptlets with expand, register them
            for added in &diff.added {
                if added.expand.is_some() {
                    logging::log(
                        "EXPAND",
                        &format!("Would register expand trigger for added: {}", added.name),
                    );
                }
            }

            // For changed expand triggers, update them
            for change in &diff.expand_changes {
                logging::log(
                    "EXPAND",
                    &format!(
                        "Would update expand trigger for {}: {:?} -> {:?}",
                        change.name, change.old, change.new
                    ),
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
            logging::log_debug(
                "CACHE",
                &format!("Filter cache MISS - recomputing for '{}'", self.filter_text),
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

            if !self.filter_text.is_empty() {
                logging::log(
                    "PERF",
                    &format!(
                        "Search '{}' took {:.2}ms ({} results from {} total)",
                        self.filter_text,
                        search_elapsed.as_secs_f64() * 1000.0,
                        self.cached_filtered_results.len(),
                        self.scripts.len()
                            + self.scriptlets.len()
                            + self.builtin_entries.len()
                            + self.apps.len()
                    ),
                );
            }
        } else {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache HIT for '{}'", self.filter_text),
            );
        }
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
            logging::log_debug(
                "CACHE",
                &format!("Grouped cache HIT for '{}'", self.computed_filter_text),
            );
            return (
                self.cached_grouped_items.clone(),
                self.cached_grouped_flat_results.clone(),
            );
        }

        // Cache miss - need to recompute
        logging::log_debug(
            "CACHE",
            &format!(
                "Grouped cache MISS - recomputing for '{}'",
                self.computed_filter_text
            ),
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

        if !self.computed_filter_text.is_empty() {
            logging::log_debug(
                "CACHE",
                &format!(
                    "Grouped results computed in {:.2}ms for '{}' ({} items)",
                    elapsed.as_secs_f64() * 1000.0,
                    self.computed_filter_text,
                    self.cached_grouped_items.len()
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
        self.grouped_cache_key = String::from("\0_INVALIDATED_\0");
        // Also reset computed_filter_text to force recompute
        self.computed_filter_text = String::from("\0_INVALIDATED_\0");
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
    ) -> &[syntax::HighlightedLine] {
        // Check if cache is valid for this path
        if self.preview_cache_path.as_deref() == Some(script_path)
            && !self.preview_cache_lines.is_empty()
        {
            logging::log_debug("CACHE", &format!("Preview cache HIT for '{}'", script_path));
            return &self.preview_cache_lines;
        }

        // Cache miss - need to re-read and re-highlight
        logging::log_debug(
            "CACHE",
            &format!("Preview cache MISS - loading '{}'", script_path),
        );

        self.preview_cache_path = Some(script_path.to_string());
        self.preview_cache_lines = match std::fs::read_to_string(script_path) {
            Ok(content) => {
                // Only take first 15 lines for preview
                let preview: String = content.lines().take(15).collect::<Vec<_>>().join("\n");
                syntax::highlight_code_lines(&preview, lang)
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to read preview: {}", e));
                Vec::new()
            }
        };

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
        if let Some(path) = self.alias_registry.get(&alias_lower) {
            // Find the script/scriptlet by path
            for script in &self.scripts {
                if script.path.to_string_lossy() == *path {
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
                if scriptlet_path == path {
                    logging::log(
                        "ALIAS",
                        &format!("Found scriptlet match: '{}' -> '{}'", alias, scriptlet.name),
                    );
                    return Some(AliasMatch::Scriptlet(scriptlet.clone()));
                }
            }

            // Path in registry but not found in current scripts (stale entry)
            logging::log(
                "ALIAS",
                &format!(
                    "Stale registry entry: '{}' -> '{}' (not found)",
                    alias, path
                ),
            );
        }

        None
    }

    fn execute_selected(&mut self, cx: &mut Context<Self>) {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the grouped item at selected_index and extract the result index
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            Some(GroupedListItem::SectionHeader(_)) => None, // Section headers are not selectable
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
                };
                if let Some(path) = frecency_path {
                    self.frecency_store.record_use(&path);
                    self.frecency_store.save().ok(); // Best-effort save
                    self.invalidate_grouped_cache(); // Invalidate cache so next show reflects frecency
                }

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing script: {}", script_match.script.name),
                        );
                        self.execute_interactive(&script_match.script, cx);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing scriptlet: {}", scriptlet_match.scriptlet.name),
                        );
                        self.execute_scriptlet(&scriptlet_match.scriptlet, cx);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing built-in: {}", builtin_match.entry.name),
                        );
                        self.execute_builtin(&builtin_match.entry, cx);
                    }
                    scripts::SearchResult::App(app_match) => {
                        logging::log("EXEC", &format!("Launching app: {}", app_match.app.name));
                        self.execute_app(&app_match.app, cx);
                    }
                    scripts::SearchResult::Window(window_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Focusing window: {}", window_match.window.title),
                        );
                        self.execute_window_focus(&window_match.window, cx);
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Agent selected: {}", agent_match.agent.name),
                        );
                        // TODO: Implement agent execution via mdflow
                        self.last_output = Some(SharedString::from(format!(
                            "Agent execution not yet implemented: {}",
                            agent_match.agent.name
                        )));
                    }
                }
            }
        }
    }

    fn handle_filter_input_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.suppress_filter_events {
            return;
        }

        // Skip filter updates when actions popup is open
        // (text input should go to actions dialog search, not main filter)
        if self.show_actions_popup {
            return;
        }

        let new_text = self.gpui_input_state.read(cx).value().to_string();
        if new_text == self.filter_text {
            return;
        }

        // Clear pending confirmation when typing (user is changing context)
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        let previous_text = std::mem::replace(&mut self.filter_text, new_text.clone());
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

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

        self.queue_filter_compute(new_text, cx);
    }

    fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        // P3: Debounce expensive search/window resize work.
        // Use 8ms debounce (half a frame) to batch rapid keystrokes.
        if self.filter_coalescer.queue(value) {
            cx.spawn(async move |this, cx| {
                // Wait 8ms for coalescing window (half frame at 60fps)
                Timer::after(std::time::Duration::from_millis(8)).await;

                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        if let Some(latest) = app.filter_coalescer.take_latest() {
                            if app.computed_filter_text != latest {
                                app.computed_filter_text = latest;
                                // This will trigger cache recompute on next get_grouped_results_cached()
                                app.update_window_size();
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
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;

        self.selected_index = 0;
        self.last_scrolled_index = None;
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.computed_filter_text = text;
        self.filter_coalescer.reset();
        self.update_window_size();
        cx.notify();
    }

    fn clear_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_filter_text_immediate(String::new(), window, cx);
    }

    fn sync_filter_input_if_needed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;
    }

    fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.show_logs = !self.show_logs;
        cx.notify();
    }

    /// Update window size based on current view and item count.
    /// This implements dynamic window resizing:
    /// - Script list: resize based on filtered results (including section headers)
    /// - Arg prompt: resize based on filtered choices
    /// - Div/Editor/Term: use full height
    fn update_window_size(&mut self) {
        let (view_type, item_count) = match &self.current_view {
            AppView::ScriptList => {
                // Get grouped results which includes section headers (cached)
                let (grouped_items, _) = self.get_grouped_results_cached();
                let count = grouped_items.len();
                (ViewType::ScriptList, count)
            }
            AppView::ArgPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    (ViewType::ArgPromptNoChoices, 0)
                } else {
                    (ViewType::ArgPromptWithChoices, filtered.len())
                }
            }
            AppView::DivPrompt { .. } => (ViewType::DivPrompt, 0),
            AppView::FormPrompt { .. } => (ViewType::DivPrompt, 0), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => (ViewType::EditorPrompt, 0),
            AppView::SelectPrompt { .. } => (ViewType::ArgPromptWithChoices, 0),
            AppView::PathPrompt { .. } => (ViewType::DivPrompt, 0),
            AppView::EnvPrompt { .. } => (ViewType::ArgPromptNoChoices, 0), // Env prompt is a simple input
            AppView::DropPrompt { .. } => (ViewType::DivPrompt, 0), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => (ViewType::DivPrompt, 0), // Template prompt uses div size
            AppView::TermPrompt { .. } => (ViewType::TermPrompt, 0),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                return;
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
                (ViewType::ScriptList, filtered_count)
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
                (ViewType::ScriptList, filtered_count)
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
                (ViewType::ScriptList, filtered_count)
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
                (ViewType::ScriptList, filtered_count)
            }
            AppView::ScratchPadView { .. } => (ViewType::EditorPrompt, 0),
            AppView::QuickTerminalView { .. } => (ViewType::TermPrompt, 0),
        };

        let target_height = height_for_view(view_type, item_count);
        resize_first_window_to_height(target_height);
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
        logging::log("KEY", "Toggling actions popup");
        if self.show_actions_popup || is_actions_window_open() {
            // Close - return focus to main filter
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.focused_input = FocusedInput::MainFilter;
            self.pending_focus = Some(FocusTarget::MainFilter);

            // Close the separate actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Refocus main filter
            self.focus_main_filter(window, cx);
            logging::log("FOCUS", "Actions closed, focus returned to MainFilter");
        } else {
            // Open actions as a separate window with vibrancy blur
            self.show_actions_popup = true;

            // CRITICAL: Transfer focus from Input to main focus_handle
            // This prevents the Input from receiving text (which would go to main filter)
            // while keeping keyboard focus in main window for routing to actions dialog
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;
            self.focused_input = FocusedInput::ActionsSearch;

            let script_info = self.get_focused_script_info();

            // Create the dialog entity HERE in main app (for keyboard routing)
            let theme_arc = std::sync::Arc::new(self.theme.clone());
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut d = ActionsDialog::with_script(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    script_info.clone(),
                    theme_arc,
                );
                // Hide the search input - main header already has the search box
                // Text input routes to dialog.handle_char() for filtering
                d.set_hide_search(true);
                d
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Get main window bounds for positioning the actions popup
            // Use the canonical coordinate system (top-left origin, Y increases downward)
            // which matches what our main window positioning uses
            let main_bounds = if let Some((x, y, w, h)) = crate::platform::get_main_window_bounds()
            {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "Main window bounds (canonical): origin=({}, {}), size={}x{}",
                        x, y, w, h
                    ),
                );
                gpui::Bounds {
                    origin: gpui::Point {
                        x: px(x as f32),
                        y: px(y as f32),
                    },
                    size: gpui::Size {
                        width: px(w as f32),
                        height: px(h as f32),
                    },
                }
            } else {
                // Fallback to GPUI bounds if platform API unavailable
                let bounds = window.bounds();
                logging::log(
                    "ACTIONS",
                    &format!(
                        "Main window bounds (GPUI fallback): origin=({:?}, {:?}), size={:?}x{:?}",
                        bounds.origin.x, bounds.origin.y, bounds.size.width, bounds.size.height
                    ),
                );
                bounds
            };

            // Open the actions window via spawn, passing the shared dialog entity
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| match open_actions_window(cx, main_bounds, dialog) {
                    Ok(_handle) => {
                        logging::log("ACTIONS", "Actions popup window opened");
                    }
                    Err(e) => {
                        logging::log("ACTIONS", &format!("Failed to open actions window: {}", e));
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
            // Close - return focus to arg prompt
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.focused_input = FocusedInput::ArgPrompt;
            self.pending_focus = Some(FocusTarget::AppRoot); // ArgPrompt uses parent focus
            window.focus(&self.focus_handle, cx);
            logging::log("FOCUS", "Arg actions closed, focus returned to ArgPrompt");
        } else {
            // Check if we have SDK actions
            if let Some(ref sdk_actions) = self.sdk_actions {
                logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
                if !sdk_actions.is_empty() {
                    // Open - create dialog entity with SDK actions
                    self.show_actions_popup = true;
                    self.focused_input = FocusedInput::ActionsSearch;

                    let theme_arc = std::sync::Arc::new(self.theme.clone());
                    let sdk_actions_clone = sdk_actions.clone();
                    let dialog = cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        let mut dialog = ActionsDialog::with_script(
                            focus_handle,
                            std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                            None,                                 // No script info for arg prompts
                            theme_arc,
                        );
                        // Set SDK actions to replace built-in actions
                        dialog.set_sdk_actions(sdk_actions_clone);
                        dialog
                    });

                    // Hide the dialog's built-in search input since header already has search
                    dialog.update(cx, |d, _| d.set_hide_search(true));

                    // Focus the dialog's internal focus handle
                    self.actions_dialog = Some(dialog.clone());
                    self.pending_focus = Some(FocusTarget::ActionsDialog);
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
        cx.notify();
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
                            // Hide window and set reset flag after opening external app
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
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
                        // Hide window and set reset flag after opening external app
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        cx.hide();
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
                            // Hide window and set reset flag after opening external app
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
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
            };

            self.execute_interactive(&script, cx);
            return;
        }

        // For non-TypeScript tools (bash, python, etc.), run synchronously
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
                expand: scriptlet.expand.clone(),
                description: scriptlet.description.clone(),
                ..Default::default()
            },
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: scriptlet.file_path.clone(),
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
            };

            self.execute_interactive(&script, cx);
        } else {
            logging::log("ERROR", &format!("Script file not found: {}", path));
        }
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
        for toast in pending {
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

        // Close actions window FIRST if open (it's a child of main window)
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
    fn handle_global_shortcut_with_options(
        &mut self,
        event: &gpui::KeyDownEvent,
        is_dismissable: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let key_str = event.keystroke.key.to_lowercase();
        let has_cmd = event.keystroke.modifiers.platform;

        // Cmd+W always closes window
        if has_cmd && key_str == "w" {
            logging::log("KEY", "Cmd+W - closing window");
            self.close_and_reset_window(cx);
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
    /// - ArgPrompt, DivPrompt, FormPrompt, SelectPrompt, PathPrompt, EnvPrompt, DropPrompt, TemplatePrompt
    /// - Built-in views (ClipboardHistory, AppLauncher, WindowSwitcher, DesignGallery)
    /// - ScriptList
    ///
    /// Non-dismissable prompts:
    /// - TermPrompt, EditorPrompt (these require explicit Cmd+W to close)
    #[allow(dead_code)]
    fn is_dismissable_view(&self) -> bool {
        !matches!(
            self.current_view,
            AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ScratchPadView { .. }
                | AppView::QuickTerminalView { .. }
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
            AppView::ClipboardHistoryView { .. } => "ClipboardHistoryView",
            AppView::AppLauncherView { .. } => "AppLauncherView",
            AppView::WindowSwitcherView { .. } => "WindowSwitcherView",
            AppView::DesignGalleryView { .. } => "DesignGalleryView",
            AppView::ScratchPadView { .. } => "ScratchPadView",
            AppView::QuickTerminalView { .. } => "QuickTerminalView",
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
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        // Resize window for script list content
        let count = self.scripts.len() + self.scriptlets.len();
        resize_first_window_to_height(height_for_view(ViewType::ScriptList, count));

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

    /// Convert hex color to rgba with opacity from theme
    fn hex_to_rgba_with_opacity(&self, hex: u32, opacity: f32) -> u32 {
        // Convert opacity (0.0-1.0) to alpha byte (0-255)
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
        (hex << 8) | alpha
    }

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
}

// Note: convert_menu_bar_items/convert_menu_bar_item functions were removed
// because frontmost_app_tracker is now compiled as part of the binary crate
// (via `mod frontmost_app_tracker` in main.rs) so it returns binary types directly.
