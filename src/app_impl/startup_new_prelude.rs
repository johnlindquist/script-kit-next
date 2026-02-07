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
                                old_count,
                                new_count,
                                app.computed_filter_text
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

