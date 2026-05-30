use super::*;

pub(super) fn calculate_fallback_error_message(expression: &str) -> String {
    format!(
        "Could not evaluate expression \"{}\". Check the syntax and try again.",
        expression
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MainWindowGlobalKeyIntent {
    OpenAcpWithCurrentContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MainWindowActionsKeyIntent {
    ToggleActions,
    CloseEmbeddedAcpWindow,
}

fn main_window_global_key_intent(
    event: &gpui::KeystrokeEvent,
) -> Option<MainWindowGlobalKeyIntent> {
    let key = event.keystroke.key.as_str();
    let has_shift = event.keystroke.modifiers.shift;

    if crate::ui_foundation::is_key_enter(key)
        && event.keystroke.modifiers.platform
        && !has_shift
        && !event.keystroke.modifiers.alt
        && !event.keystroke.modifiers.control
    {
        return Some(MainWindowGlobalKeyIntent::OpenAcpWithCurrentContext);
    }

    None
}

#[inline]
fn is_plain_platform_cmd_w(event: &gpui::KeystrokeEvent) -> bool {
    let key = event.keystroke.key.as_str();
    let modifiers = &event.keystroke.modifiers;
    modifiers.platform
        && !modifiers.shift
        && !modifiers.alt
        && !modifiers.control
        && key.eq_ignore_ascii_case("w")
}

fn main_window_actions_key_intent(
    current_view: &AppView,
    event: &gpui::KeystrokeEvent,
) -> Option<MainWindowActionsKeyIntent> {
    let key = event.keystroke.key.as_str();
    let has_cmd = event.keystroke.modifiers.platform;
    let has_shift = event.keystroke.modifiers.shift;

    if has_cmd && key.eq_ignore_ascii_case("k") && !has_shift {
        return Some(MainWindowActionsKeyIntent::ToggleActions);
    }

    if has_cmd
        && key.eq_ignore_ascii_case("w")
        && !has_shift
        && matches!(current_view, AppView::AcpChatView { .. })
    {
        return Some(MainWindowActionsKeyIntent::CloseEmbeddedAcpWindow);
    }

    None
}

impl ScriptListApp {
    fn handle_main_window_global_key_intent(
        &mut self,
        intent: MainWindowGlobalKeyIntent,
        cx: &mut Context<Self>,
    ) -> bool {
        match intent {
            MainWindowGlobalKeyIntent::OpenAcpWithCurrentContext => {
                self.try_route_global_cmd_enter_to_acp_context_capture(cx)
            }
        }
    }

    /// Exit cwd-pick mode (Tab → FileSearchView) and return to the main menu
    /// without setting a cwd. Used by Escape and by the second Backspace at
    /// the disk root.
    fn exit_cwd_pick_to_main_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        tracing::info!(
            target: "script_kit::spine",
            event = "cwd_pick_exit_to_main_menu",
            "Left cwd-pick FileSearchView without selecting a cwd"
        );
        self.cwd_pick_mode = false;
        self.reset_to_script_list(cx);
        self.clear_filter(window, cx);
    }

    /// Re-seed the cwd-pick FileSearchView with `query` (e.g. "/"), keeping the
    /// shared gpui input in sync without re-triggering the filter change
    /// handler.
    fn reseed_cwd_pick_query(
        &mut self,
        query: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_file_search_view(query.to_string(), FileSearchPresentation::Full, cx);
        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(query.to_string(), window, cx);
            let len = query.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
    }

    /// Handle Escape / Backspace inside the cwd-pick FileSearchView.
    ///
    /// - Escape: return to the main menu in one press.
    /// - Backspace from "~/": collapse to "/" (disk root).
    /// - Backspace from "/": return to the main menu (so two deletes from the
    ///   initial "~/" land back on the launcher).
    ///
    /// Returns `true` when the key was consumed. Any other state (deeper paths,
    /// modified keys) is left to normal input editing.
    fn try_handle_cwd_pick_nav_key(
        &mut self,
        event: &gpui::KeystrokeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.cwd_pick_mode || !matches!(self.current_view, AppView::FileSearchView { .. }) {
            return false;
        }

        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;
        let has_modifier =
            modifiers.platform || modifiers.alt || modifiers.control || modifiers.shift;

        if crate::ui_foundation::is_key_escape(key) && !has_modifier {
            self.exit_cwd_pick_to_main_menu(window, cx);
            return true;
        }

        let is_backspace =
            key.eq_ignore_ascii_case("backspace") || key.eq_ignore_ascii_case("delete");
        if is_backspace && !has_modifier {
            let query = match &self.current_view {
                AppView::FileSearchView { query, .. } => query.clone(),
                _ => return false,
            };
            match query.as_str() {
                "~/" => {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "cwd_pick_backspace_to_disk_root",
                        "Backspace collapsed cwd-pick query from ~/ to /"
                    );
                    self.reseed_cwd_pick_query("/", window, cx);
                    return true;
                }
                "/" => {
                    self.exit_cwd_pick_to_main_menu(window, cx);
                    return true;
                }
                _ => return false,
            }
        }

        false
    }

    fn handle_main_window_actions_key_intent(
        &mut self,
        intent: MainWindowActionsKeyIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match intent {
            MainWindowActionsKeyIntent::ToggleActions => {
                self.handle_cmd_k_actions_toggle(window, cx)
            }
            MainWindowActionsKeyIntent::CloseEmbeddedAcpWindow => {
                tracing::info!(
                    target: "script_kit::keyboard",
                    event = "embedded_acp_cmd_w_close_window",
                );
                logging::log("KEY", "Interceptor: Cmd+W -> close window from Agent Chat");
                self.close_tab_ai_harness_terminal_with_window(window, cx);
                self.close_and_reset_window(cx);
                true
            }
        }
    }

    fn close_main_window_from_top_level_cmd_w(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::keyboard",
            event = "top_level_cmd_w_close_main_window",
            current_view = %self.app_view_name(),
            actions_open = crate::actions::is_actions_window_open(),
            confirm_open = crate::confirm::is_confirm_window_open(),
            show_actions_popup = self.show_actions_popup,
        );

        if crate::confirm::is_confirm_window_open() {
            crate::confirm::route_key_to_confirm_popup("escape", cx);
        }

        if crate::actions::is_actions_window_open() || self.show_actions_popup {
            self.close_actions_popup_for_current_view(window, cx);
        }

        match &self.current_view {
            AppView::QuickTerminalView { .. } => {
                self.close_quick_terminal_main_window_state_first(cx);
            }
            AppView::AcpChatView { .. } => {
                self.close_tab_ai_harness_terminal_with_window(window, cx);
                self.close_and_reset_window(cx);
            }
            AppView::ThemeChooserView { .. } => {
                if let Some(original) = self.theme_before_chooser.take() {
                    self.restore_theme_chooser_theme(
                        original,
                        "theme_chooser_top_level_cmd_w_undo",
                        cx,
                    );
                    let _ = crate::theme::service::persist_theme_and_sync_all_windows(
                        cx,
                        self.theme.as_ref(),
                        "theme_chooser_top_level_cmd_w_undo_persist",
                    );
                }
                self.clear_theme_chooser_controls();
                self.close_and_reset_window(cx);
            }
            _ => {
                self.close_and_reset_window(cx);
            }
        }
    }

    pub(crate) fn new(
        config: config::Config,
        bun_available: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // PERF: Parallelize script + scriptlet loading to reduce startup wall time.
        let load_start = std::time::Instant::now();
        let (script_report, scriptlets, scripts_elapsed, scriptlets_elapsed) = std::thread::scope(
            |scope| {
                let scripts_handle = scope.spawn(|| {
                    let start = std::time::Instant::now();
                    let loaded = scripts::read_scripts_report();
                    (loaded, start.elapsed())
                });

                let scriptlets_handle = scope.spawn(|| {
                    let start = std::time::Instant::now();
                    // Use load_scriptlets() to load from all plugins (plugins/*/scriptlets/*.md)
                    // This includes built-in extensions like CleanShot and user extensions
                    let loaded = scripts::load_scriptlets();
                    (loaded, start.elapsed())
                });

                let (script_report, scripts_elapsed) = match scripts_handle.join() {
                    Ok(result) => result,
                    Err(_) => {
                        logging::log(
                            "PERF",
                            "Script loading thread panicked; retrying read_scripts_report synchronously",
                        );
                        let retry_start = std::time::Instant::now();
                        (scripts::read_scripts_report(), retry_start.elapsed())
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

                (
                    script_report,
                    scriptlets,
                    scripts_elapsed,
                    scriptlets_elapsed,
                )
            },
        );

        let scripts: Vec<std::sync::Arc<scripts::Script>> =
            script_report.scripts.iter().cloned().collect();
        let script_validation_report = Some(script_report.validation.clone());

        // Theme cache was initialized earlier in app startup before window creation.
        // Reuse it here so ScriptListApp construction does not re-read theme files
        // or re-run system appearance detection.
        let theme_load_started = std::time::Instant::now();
        let theme = std::sync::Arc::new(theme::get_cached_theme());
        logging::log(
            "PERF",
            &format!(
                "Startup theme reuse: source=cached elapsed_ms={:.2}",
                theme_load_started.elapsed().as_secs_f64() * 1000.0
            ),
        );
        // Config is now passed in from main() to avoid duplicate load (~100-300ms savings)

        // Load frecency data for suggested section tracking
        let suggested_config = config.get_suggested();
        let mut frecency_store = FrecencyStore::with_config(&suggested_config);
        frecency_store.load().ok(); // Ignore errors - starts fresh if file doesn't exist

        // Load built-in entries based on config, filtering out commands hidden via
        // `hiddenCommands` or per-command `commands.*.hidden` overrides.
        let builtin_entries: Vec<_> = builtins::get_builtin_entries(&config.get_builtins())
            .into_iter()
            .filter(|entry| !config.is_command_hidden(&entry.id))
            .collect();

        // Apps are loaded in the background to avoid blocking startup
        // Start with empty list, will be populated asynchronously
        let apps = Vec::new();

        let total_elapsed = load_start.elapsed();
        logging::log(
            "PERF",
            &format!(
                "Startup loading: {:.2}ms total ({} scripts in {:.2}ms, {} scriptlets in {:.2}ms, apps loading in background)",
                total_elapsed.as_secs_f64() * 1000.0,
                scripts.len(),
                scripts_elapsed.as_secs_f64() * 1000.0,
                scriptlets.len(),
                scriptlets_elapsed.as_secs_f64() * 1000.0
            ),
        );
        logging::log(
            "APP",
            &format!(
                "Loaded {} scripts from ~/.scriptkit/plugins/*/scripts",
                scripts.len()
            ),
        );
        logging::log(
            "APP",
            &format!(
                "Loaded {} scriptlets from ~/.scriptkit/plugins/*/scriptlets",
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
                        app.main_menu_result_caches.mark_apps_loaded();
                        app.rebuild_root_windows_after_app_icon_cache_update(
                            "apps_loaded_root_windows_icons",
                            cx,
                        );
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

        #[cfg(not(test))]
        {
            let share_rx = crate::script_sharing::spawn_clipboard_share_watcher();
            cx.spawn(async move |this, cx| {
                while let Ok(import) = share_rx.recv().await {
                    tracing::info!(
                        share_uri = %import.uri,
                        title = %import.bundle.title,
                        kind = ?import.bundle.kind,
                        "clipboard_share_bundle_detected"
                    );
                    script_kit_gpui::request_show_main_window();
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(180))
                        .await;

                    let options = crate::confirm::ParentConfirmOptions {
                        title: import.bundle.prompt_title().into(),
                        body: import.bundle.prompt_body().into(),
                        confirm_text: "Install".into(),
                        cancel_text: "Ignore".into(),
                        ..Default::default()
                    };
                    let trace_id = format!(
                        "share-import-{}-{}",
                        import.bundle.kind.display_name().to_lowercase(),
                        import.bundle.title.to_lowercase().replace(' ', "-")
                    );

                    let confirmed =
                        match crate::confirm::confirm_with_parent_dialog(cx, options, &trace_id)
                            .await
                        {
                            Ok(confirmed) => confirmed,
                            Err(error) => {
                                tracing::error!(
                                    ?error,
                                    title = %import.bundle.title,
                                    "clipboard_share_confirm_failed"
                                );
                                continue;
                            }
                        };
                    if !confirmed {
                        continue;
                    }

                    let install_result =
                        crate::script_sharing::install_share_bundle(&import.bundle);
                    let title = import.bundle.title.clone();
                    let kind = import.bundle.kind.display_name().to_lowercase();
                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| match install_result {
                            Ok(outcome) => {
                                app.refresh_scripts(cx);
                                app.refresh_skills(cx);
                                app.current_view = AppView::ScriptList;
                                app.show_hud(
                                    format!("Installed shared {} into {}", kind, outcome.plugin_id),
                                    Some(2000),
                                    cx,
                                );
                            }
                            Err(error) => {
                                app.show_error_toast(
                                    format!(
                                        "Failed to install shared {} '{}': {}",
                                        kind, title, error
                                    ),
                                    cx,
                                );
                            }
                        })
                    });
                }
            })
            .detach();
        }
        logging::log("UI", "Script Kit logo SVG loaded for header rendering");

        // Start cursor blink timer - updates all inputs that track cursor visibility
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(530))
                    .await;

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

        let gpui_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                // Placeholder identifies the active accent-exploration variation.
                .placeholder(crate::designs::AccentVariation::default().placeholder())
                .inline_completion_visible_without_focus(true)
        });
        let gpui_input_subscription = cx.subscribe_in(&gpui_input_state, window, {
            move |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Focus => {
                    this.gpui_input_focused = true;
                    // Set focused_input based on current view
                    if matches!(
                        this.current_view,
                        AppView::MiniPrompt { .. } | AppView::ArgPrompt { .. }
                    ) {
                        this.focused_input = FocusedInput::ArgPrompt;
                    } else {
                        this.focused_input = FocusedInput::MainFilter;
                    }

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
                    let current_value = this.gpui_input_state.read(cx).value().to_string();

                    if matches!(
                        this.current_view,
                        AppView::MiniPrompt { .. } | AppView::ArgPrompt { .. }
                    ) {
                        // Sync text from Input component to arg_input for choice filtering
                        let prev_original_idx = this
                            .filtered_arg_choices()
                            .get(this.arg_selected_index)
                            .map(|(orig_idx, _)| *orig_idx);
                        this.arg_input.set_text(&current_value);
                        this.sync_arg_prompt_after_text_change(
                            prev_original_idx,
                            window,
                            cx,
                        );
                        cx.notify();
                    } else {
                        let input_received_at = std::time::Instant::now();
                        if logging::filter_perf_trace_enabled() {
                            logging::log(
                                "FILTER_PERF",
                                &format!(
                                    "[1/5] INPUT_CHANGE value='{}' len={} at {:?}",
                                    current_value,
                                    current_value.len(),
                                    input_received_at
                                ),
                            );
                        }
                        this.main_menu_render_diagnostics.filter_perf_start =
                            Some(input_received_at);
                        this.handle_filter_input_change(window, cx);
                    }
                }
                InputEvent::PressEnter { .. } => {
                    // Block Enter when confirm popup is open — the confirm
                    // popup's key routing already handled this Enter via
                    // capture_key_down in render_impl.
                    if confirm::is_confirm_window_open() {
                        logging::log("KEY", "Ignoring PressEnter: confirm popup is open");
                        return;
                    }

                    if matches!(this.current_view, AppView::ThemeChooserView { .. }) {
                        if !this.show_actions_popup && !actions::is_actions_window_open() {
                            this.submit_theme_chooser_from_input_enter(window, cx);
                        } else {
                            logging::log(
                                "KEY",
                                "Ignoring ThemeChooser PressEnter: actions popup is open",
                            );
                        }
                        return;
                    }

                    // Handle Enter for mini/arg prompts — submit the arg value
                    let prompt_id = match &this.current_view {
                        AppView::MiniPrompt { id, .. } | AppView::ArgPrompt { id, .. } => {
                            Some(id.clone())
                        }
                        _ => None,
                    };
                    if let Some(prompt_id) = prompt_id {
                        this.submit_arg_prompt_from_current_state(&prompt_id, cx);
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
                            this.main_menu_fallback_state.is_active(),
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
                        if this.should_consume_script_list_enter_after_submit(
                            "input_press_enter_script_list",
                        ) {
                            logging::log(
                                "KEY",
                                "Ignoring PressEnter: prompt submit already consumed this Enter",
                            );
                            return;
                        }
                        // Check if we're in fallback mode first
                        if this.main_menu_fallback_state.is_active() {
                            this.execute_selected_fallback(cx);
                        } else {
                            this.execute_selected(cx);
                        }
                    }
                }
                InputEvent::PressTab { secondary } => {
                    if matches!(this.current_view, AppView::ScriptList)
                        && this.menu_syntax_capture_form_owns_input()
                    {
                        if *secondary {
                            this.focus_previous_menu_syntax_form_field(window, cx);
                        } else {
                            this.focus_next_menu_syntax_form_field(window, cx);
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
        let (inline_chat_actions_tx, inline_chat_actions_rx) = mpsc::sync_channel(4);
        let (inline_chat_continue_tx, inline_chat_continue_rx) = mpsc::sync_channel(4);
        let (inline_chat_configure_tx, inline_chat_configure_rx) = mpsc::sync_channel(4);
        let (inline_chat_claude_code_tx, inline_chat_claude_code_rx) = mpsc::sync_channel(4);
        // Create channel for naming dialog completion signals
        let (naming_submit_tx, naming_submit_rx) = mpsc::sync_channel(4);
        let default_response_sender = create_stdout_response_sender();
        // Discover plugin skills for main-menu search
        let plugin_skills: Vec<std::sync::Arc<crate::plugins::PluginSkill>> = {
            let skills = crate::plugins::discover_plugins()
                .and_then(|index| crate::plugins::discover_plugin_skills(&index))
                .unwrap_or_default();
            skills.into_iter().map(std::sync::Arc::new).collect()
        };
        crate::dictation::hydrate_dictation_resource_from_history();
        let window_search_test_provider =
            std::env::var_os("SCRIPT_KIT_WINDOW_SEARCH_TEST_PROVIDER").is_some();
        let initial_cached_windows = if window_search_test_provider {
            crate::window_control::list_windows().unwrap_or_default()
        } else {
            Vec::new()
        };
        let initial_root_windows_provider_status = if window_search_test_provider {
            crate::window_control::RootWindowsProviderStatus::Ready {
                count: initial_cached_windows.len(),
            }
        } else {
            crate::window_control::RootWindowsProviderStatus::Unknown
        };
        let initial_cached_root_windows = Self::build_root_window_entries(
            &initial_cached_windows,
            &apps,
            &std::collections::HashMap::new(),
        );

        // Restore the persisted global working directory (footer cwd chip) so it
        // survives app restarts; the user tends to stay in one directory. A
        // restored cwd counts as an explicit pick (revision 1) so the agent
        // launches there. Falls back to ~/.scriptkit (revision 0) when no valid
        // persisted directory exists.
        let (initial_spine_cwd, initial_spine_cwd_label, initial_spine_cwd_revision) = {
            let persisted = crate::config::load_user_preferences()
                .ai
                .cwd
                .map(std::path::PathBuf::from)
                .filter(|path| path.is_dir());
            match persisted {
                Some(path) => {
                    let label = crate::file_search::shorten_path(&path.to_string_lossy())
                        .trim_end_matches('/')
                        .to_string();
                    (Some(path), Some(label), 1_u64)
                }
                None => (
                    dirs::home_dir().map(|h| h.join(".scriptkit")),
                    Some("~/.scriptkit".to_string()),
                    0_u64,
                ),
            }
        };

        // Resolve the persisted agent/model into footer display labels so the
        // selection (Shift+Tab Agent & Model picker) is visible on first paint.
        let (initial_spine_agent_label, initial_spine_model_label) =
            Self::resolve_agent_model_footer_labels();

        let mut app = ScriptListApp {
            scripts,
            scriptlets,
            skills: plugin_skills,
            script_validation_report,
            builtin_entries,
            apps,
            // P0 FIX: Cached data for builtin views (avoids cloning per frame)
            cached_clipboard_entries: Vec::new(),
            paste_sequential_state: None,
            focused_clipboard_entry_id: None,
            cached_windows: initial_cached_windows,
            cached_root_windows: initial_cached_root_windows,
            root_windows_provider_status: initial_root_windows_provider_status,
            root_windows_refresh_generation: 0,
            root_windows_refresh_token: 0,
            root_windows_refreshing: false,
            root_windows_last_completed_at: None,
            root_window_focus_recency: std::collections::HashMap::new(),
            root_window_focus_seq: 0,
            cached_browser_tabs: Vec::new(),
            cached_browser_history: Vec::new(),
            browser_history_loading: false,
            cached_file_results: Vec::new(),
            root_file_results: Vec::new(),
            root_file_result_cache: std::collections::VecDeque::new(),
            root_file_search_mode: None,
            root_recent_file_results: Vec::new(),
            root_recent_file_revision: u64::MAX,
            root_file_search_query: String::new(),
            root_file_search_generation: 0,
            root_file_search_cancel: None,
            root_file_search_loading: false,
            root_file_provider_loading: false,
            root_file_frame: None,
            root_file_source_chip_page_key: None,
            root_file_source_chip_visible_limit:
                crate::file_search::ROOT_FILE_SOURCE_CHIP_INITIAL_VISIBLE_ROWS,
            root_passive_frame: None,
            spine_file_search_query: String::new(),
            spine_file_search_generation: 0,
            spine_file_search_loading: false,
            spine_file_search_results: Vec::new(),
            spine_file_search_cancel: None,
            pending_root_file_actions_file: None,
            pending_root_unified_actions_subject: None,
            cached_processes: Vec::new(),
            process_manager_refresh_task: None,
            cached_current_app_entries: Vec::new(),
            current_app_commands_session: None,
            selected_index: 0,
            filter_text: String::new(),
            inline_calculator: None,
            gpui_input_state,
            gpui_input_focused: false,
            ghost_prediction: None,
            prediction_revision: Default::default(),
            ghost_llm_generation: 0,
            ghost_llm_cancel: None,
            ghost_llm_cache: std::collections::VecDeque::new(),
            launcher_context: Default::default(),
            launcher_context_generation: 0,
            gpui_input_subscriptions: vec![gpui_input_subscription],
            bounds_subscription: None,     // Set later after window setup
            appearance_subscription: None, // Set later after window setup
            suppress_filter_events: false,
            pending_programmatic_filter_echo: None,
            pending_filter_sync: false,
            history_filter_render_pending: None,
            pending_placeholder: None,
            last_output: None,
            focus_handle: cx.focus_handle(),
            show_logs: false,
            show_info_panel: false,
            quick_terminal_warm_pty: None,
            quick_terminal_warm_inflight: false,
            quick_terminal_warm_created_at: None,
            theme,
            config,
            // Scroll activity tracking: start with scrollbar hidden
            scrollbar_visibility: crate::transitions::Opacity::INVISIBLE,
            scrollbar_fade_gen: 0,
            last_scroll_time: None,
            builtin_wheel_owned_selected_index: None,
            current_view: AppView::ScriptList,
            submit_diagnostics: SubmitDiagnosticsState::default(),
            main_window_mode: MainWindowMode::Mini,
            script_session: Arc::new(ParkingMutex::new(None)),
            arg_input: TextInputState::new(),
            arg_selected_index: 0,
            prompt_receiver: None,
            response_sender: Some(default_response_sender.clone()),
            default_response_sender: Some(default_response_sender),
            // Variable-height list state for main menu (section headers at 24px, items at 48px)
            // Start with 0 items; filter replacement installs a fresh state without measuring all rows.
            main_list_state: ListState::new(
                0,
                ListAlignment::Top,
                px(crate::list_item::effective_average_item_height_for_scroll()),
            ),
            main_list_row_generation: 0,
            menu_syntax_main_hint_scroll_handle: ScrollHandle::new(),
            list_scroll_handle: UniformListScrollHandle::new(),
            arg_list_scroll_handle: UniformListScrollHandle::new(),
            clipboard_list_scroll_handle: UniformListScrollHandle::new(),
            emoji_scroll_handle: UniformListScrollHandle::new(),
            emoji_frequent_snapshot: Vec::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            browser_tabs_scroll_handle: UniformListScrollHandle::new(),
            process_list_scroll_handle: UniformListScrollHandle::new(),
            current_app_commands_scroll_handle: UniformListScrollHandle::new(),
            acp_history_scroll_handle: ScrollHandle::new(),
            browser_history_scroll_handle: ScrollHandle::new(),
            dictation_history_scroll_handle: ScrollHandle::new(),
            notes_browse_scroll_handle: ScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            footer_gallery_scroll_handle: UniformListScrollHandle::new(),
            file_search_scroll_handle: UniformListScrollHandle::new(),
            theme_chooser_list_state: ListState::new(0, ListAlignment::Top, px(100.)).measure_all(),
            file_search_loading: false,
            file_search_debounce_task: None,
            file_search_current_dir: None,
            file_search_current_dir_show_hidden: false,
            file_search_frozen_filter: None,
            file_search_actions_path: None,
            file_search_sort_mode: crate::actions::FileSearchSortMode::default(),
            file_search_gen: 0,
            file_search_cancel: None,
            file_search_display_indices: Vec::new(),
            file_search_selection_mode: FileSearchSelectionMode::AutoFirst,
            file_search_preview_thumbnail: FileSearchThumbnailPreviewState::Idle,
            show_actions_popup: false,
            actions_closed_at: None,
            actions_dialog: None,
            cursor_visible: true,
            focused_input: FocusedInput::MainFilter,
            current_script_pid: None,
            main_menu_result_caches: MainMenuResultCacheState::default(),
            // P3: Two-stage filter coalescing
            computed_filter_text: String::new(),
            filter_coalescer: FilterCoalescer::new(),
            menu_syntax_mode: crate::menu_syntax::MenuSyntaxMode::default(),
            spine_enabled: true,
            spine_parse: crate::spine::SpineParse {
                segments: vec![],
                input: String::new(),
            },
            spine_projection: None,
            spine_cwd: initial_spine_cwd,
            spine_cwd_label: initial_spine_cwd_label,
            spine_cwd_revision: initial_spine_cwd_revision,
            ghost_context_cache: crate::scripts::search::ghost::GhostContextCache::default(),
            cwd_pick_mode: false,
            agent_model_picker_active: false,
            spine_agent_label: initial_spine_agent_label,
            spine_model_label: initial_spine_model_label,
            spine_live_preview_cache: Default::default(),
            menu_syntax_trigger_popup_state:
                crate::menu_syntax_trigger_popup::MenuSyntaxTriggerPopupState::default(),
            menu_syntax_object_selector_state:
                crate::menu_syntax::MenuSyntaxObjectSelectorState::default(),
            menu_syntax_form_focused_index: 0,
            menu_syntax_form_signature: None,
            menu_syntax_form_inputs: Vec::new(),
            menu_syntax_form_input_subscriptions: Vec::new(),
            menu_syntax_form_syncing_from_input: false,
            menu_syntax_form_input_active: false,
            menu_syntax_form_draft_field_id: None,
            menu_syntax_form_draft_value: String::new(),
            menu_syntax_form_suggestion_field_id: None,
            menu_syntax_form_suggestion_selected_index: None,
            pending_menu_syntax_ai_proposal: None,
            menu_syntax_trigger_popup_suppressed_filter: None,
            // Scroll stabilization: start with no last scrolled index
            last_scrolled_index: None,
            // Preview cache: start empty, will populate on first render
            preview_cache_path: None,
            preview_cache_match_signature: None,
            preview_cache_lines: Vec::new(),
            // Scriptlet preview cache: avoid re-highlighting on every render
            scriptlet_preview_cache_key: None,
            scriptlet_preview_cache_lines: Vec::new(),
            // Design system: start with default design
            current_design: DesignVariant::default(),
            // Accent-color exploration: start on the first variation (Row Tint)
            current_accent_variation: crate::designs::AccentVariation::default(),
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
            main_menu_fallback_state: MainMenuFallbackState::default(),
            theme_before_chooser: None,
            theme_chooser_management: None,
            theme_chooser_controls: None,
            main_menu_render_diagnostics: MainMenuRenderDiagnosticsState::default(),
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
            main_list_suppress_hover_until_mouse_move: false,
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
            tab_ai_harness_script_list_trigger: None,
            tab_ai_harness_apply_back_route: None,
            embedded_acp_chat: None,
            embedded_acp_focus_handle: None,
            prewarmed_acp_chat: None,
            active_agent_chat_warm_lease: None,
            acp_ready_script_path: None,
            acp_footer_dot_status: None,
            acp_footer_model_display: None,
            acp_footer_snapshot: None,
            attachment_portal_host_snapshot: None,
            attachment_portal_return_view: None,
            attachment_portal_return_focus_target: None,
            attachment_portal_return_width: None,
            active_attachment_portal_kind: None,
            acp_surface_state: crate::ai::acp::surface_state::AcpSurfaceState::Hidden,
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
            inline_chat_actions_sender: inline_chat_actions_tx,
            inline_chat_actions_receiver: inline_chat_actions_rx,
            mini_ai_last_close_snapshot: None,
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

        let app_entity_for_cmd_w = cx.entity().downgrade();
        let cmd_w_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_cmd_w;
            move |event, window, cx| {
                if !is_plain_platform_cmd_w(event) {
                    return;
                }

                let is_notes = crate::notes::is_notes_window(window);
                let is_ai = crate::ai::is_ai_window(window);
                let is_detached_acp = crate::ai::acp::chat_window::is_chat_window(window);
                let is_actions = crate::actions::is_actions_window(window);

                if is_notes || is_ai || is_detached_acp {
                    return;
                }

                if !script_kit_gpui::is_main_window_visible() && !is_actions {
                    return;
                }

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        this.close_main_window_from_top_level_cmd_w(window, cx);
                        cx.stop_propagation();
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(cmd_w_interceptor);

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

                // Skip keystrokes from secondary windows — interceptors are
                // GLOBAL and fire for ALL windows.  Secondary windows own
                // their own Tab/Cmd+Enter handling.
                let is_notes = crate::notes::is_notes_window(window);
                let is_ai = crate::ai::is_ai_window(window);
                let is_detached_acp = crate::ai::acp::chat_window::is_chat_window(window);
                let is_actions = crate::actions::is_actions_window(window);
                if is_notes || is_ai || is_detached_acp || is_actions {
                    tracing::debug!(
                        target: "script_kit::keyboard",
                        event = "tab_interceptor_skipped_secondary_window",
                        is_notes,
                        is_ai,
                        is_detached_acp,
                        is_actions,
                    );
                    return;
                }

                let key = event.keystroke.key.as_str();
                let is_tab_key = key.eq_ignore_ascii_case("tab");
                let has_shift = event.keystroke.modifiers.shift;
                let is_plain_enter = crate::ui_foundation::is_key_enter(key)
                    && !event.keystroke.modifiers.platform
                    && !has_shift
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control;

                if confirm::consume_main_window_key_while_confirm_open(
                    key,
                    &event.keystroke.modifiers,
                    cx,
                ) {
                    cx.stop_propagation();
                    return;
                }

                // cwd-pick mode (Tab → FileSearchView) owns Escape/Backspace so
                // the launcher's progressive-escape semantics apply: one Escape
                // returns to the main menu, Backspace from "~/" collapses to the
                // disk root, and Backspace from "/" returns to the main menu.
                // Must fire before the Input component eats Backspace.
                {
                    let mut consumed = false;
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            consumed = this.try_handle_cwd_pick_nav_key(event, window, cx);
                        });
                    }
                    if consumed {
                        cx.stop_propagation();
                        return;
                    }
                }

                let global_key_intent = main_window_global_key_intent(event);
                if let Some(intent) = global_key_intent {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            if this.handle_main_window_global_key_intent(intent, cx) {
                                cx.stop_propagation();
                            }
                        });
                    }
                    return;
                }
                // Check for Tab key (no cmd/alt/ctrl modifiers, but shift is allowed)
                if is_tab_key
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let owner = match &this.current_view {
                                AppView::ScriptList => "script_list",
                                AppView::AcpChatView { .. } => "embedded_acp",
                                AppView::QuickTerminalView { .. } => "quick_terminal",
                                AppView::FileSearchView { .. } => "file_search",
                                AppView::ChatPrompt { .. } => "chat_prompt",
                                _ => "main_window_other",
                            };
                            tracing::debug!(
                                target: "script_kit::keyboard",
                                event = "tab_interceptor_owner_path",
                                owner,
                                has_shift,
                                show_actions_popup = this.show_actions_popup,
                                save_offer_open = this.tab_ai_save_offer_state.is_some(),
                            );

                            // File search owns Tab locally: plain Tab browses
                            // into the selected directory and Shift+Tab goes up.
                            if matches!(this.current_view, AppView::FileSearchView { .. }) {
                                cx.stop_propagation();
                                if this.show_actions_popup {
                                    return;
                                }

                                if has_shift {
                                    let current_query = match &this.current_view {
                                        AppView::FileSearchView { query, .. } => query.clone(),
                                        _ => String::new(),
                                    };

                                    let parent_path_opt = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(&current_query)
                                    {
                                        if parsed.filter.is_some() {
                                            Some(parsed.directory)
                                        } else {
                                            crate::file_search::parent_dir_display(
                                                &parsed.directory,
                                            )
                                        }
                                    } else {
                                        None
                                    };

                                    if let Some(parent_path) = parent_path_opt {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Navigating up from '{}' to '{}'",
                                                current_query, parent_path
                                            ),
                                        );
                                        this.gpui_input_state.update(cx, |state, cx| {
                                            state.set_value(parent_path.clone(), window, cx);
                                            let len = parent_path.len();
                                            state.set_selection(len, len, window, cx);
                                        });
                                        cx.notify();
                                    } else {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Already at root '{}', no-op",
                                                current_query
                                            ),
                                        );
                                    }
                                } else if !this.navigate_file_search_into_selected_directory(cx) {
                                    crate::logging::log(
                                        "KEY",
                                        "Tab: no selected directory to navigate into",
                                    );
                                }
                                return;
                            }

                            // ChatPrompt keeps Shift+Tab for local setup
                            // navigation and leaves plain Tab local.
                            if matches!(this.current_view, AppView::ChatPrompt { .. }) {
                                if has_shift {
                                    if let AppView::ChatPrompt { entity, .. } = &this.current_view {
                                        let handled = entity.update(cx, |chat, cx| {
                                            chat.handle_setup_key("tab", true, cx)
                                        });
                                        if handled {
                                            cx.stop_propagation();
                                            return;
                                        }
                                    }
                                }
                            }

                            // Menu-syntax trigger popup owns Tab when it is
                            // visible — Tab applies the selected row (keep-open
                            // for open-value qualifiers like `source:`,
                            // close-after-apply for bare qualifiers or capture
                            // targets). Runs BEFORE the ACP plain-Tab routing
                            // branch so menu-syntax keyboard stays consistent
                            // with the ACP slash / @ pickers.
                            if matches!(this.current_view, AppView::ScriptList)
                                && crate::menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open()
                            {
                                let intent = if has_shift {
                                    crate::menu_syntax::InlinePickerKeyIntent::MoveUp
                                } else {
                                    crate::menu_syntax::InlinePickerKeyIntent::Apply
                                };
                                if this.apply_menu_syntax_object_selector_intent(
                                    intent, window, cx,
                                ) {
                                    cx.stop_propagation();
                                    return;
                                }
                            }

                            if matches!(this.current_view, AppView::ScriptList)
                                && crate::menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open()
                            {
                                let intent = if has_shift {
                                    crate::menu_syntax::InlinePickerKeyIntent::MoveUp
                                } else {
                                    crate::menu_syntax::InlinePickerKeyIntent::Apply
                                };
                                if this.apply_menu_syntax_trigger_popup_intent(
                                    intent, window, cx,
                                ) {
                                    cx.stop_propagation();
                                    return;
                                }
                            }

                            if matches!(this.current_view, AppView::ScriptList)
                                && this.handle_menu_syntax_form_key_input(
                                    key,
                                    event.keystroke.key_char.as_deref(),
                                    &event.keystroke.modifiers,
                                    window,
                                    cx,
                                )
                            {
                                cx.stop_propagation();
                                return;
                            }

                            if this.menu_syntax_capture_form_owns_input() {
                                if has_shift {
                                    this.focus_previous_menu_syntax_form_field(window, cx);
                                } else {
                                    this.focus_next_menu_syntax_form_field(window, cx);
                                }
                                cx.stop_propagation();
                                return;
                            }

                            if matches!(this.current_view, AppView::ScriptList)
                                && !has_shift
                                && this.accept_ghost_prediction(window, cx)
                            {
                                cx.stop_propagation();
                                return;
                            }

                            // Tab on ScriptList opens the cwd picker — the
                            // chip-as-button affordance. Fires only when
                            // nothing else above (menu-syntax popups, ghost
                            // prediction, capture form, ACP/terminal locals)
                            // claimed the keystroke. The picker is the same
                            // FileSearchView that `>` used to open; the
                            // user's first typed char inside it transitions
                            // into ordinary file navigation.
                            if matches!(this.current_view, AppView::ScriptList)
                                && !has_shift
                                && this.spine_enabled
                                && !this.show_actions_popup
                            {
                                tracing::info!(
                                    target: "script_kit::spine",
                                    event = "cwd_pick_enter_file_search_tab",
                                    "Tab → FileSearchView (cwd pick)"
                                );
                                this.cwd_pick_mode = true;
                                this.open_file_search_view(
                                    "~/".to_string(),
                                    FileSearchPresentation::Full,
                                    cx,
                                );
                                this.suppress_filter_events = true;
                                this.gpui_input_state.update(cx, |state, cx| {
                                    state.set_value("~/".to_string(), window, cx);
                                    let len = "~/".len();
                                    state.set_selection(len, len, window, cx);
                                });
                                this.suppress_filter_events = false;
                                cx.stop_propagation();
                                return;
                            }

                            // Shift+Tab on the main menu opens the global
                            // Agent & Model picker so the user can pre-configure
                            // (and persist) which agent/model the next Cmd+Enter
                            // launch uses, without leaving the launcher.
                            if matches!(this.current_view, AppView::ScriptList)
                                && has_shift
                                && this.spine_enabled
                                && !this.show_actions_popup
                                && !this.menu_syntax_capture_form_owns_input()
                            {
                                tracing::info!(
                                    target: "script_kit::spine",
                                    event = "agent_model_picker_open_shift_tab",
                                    "Shift+Tab → Agent & Model picker"
                                );
                                this.open_agent_model_picker_window(window, cx);
                                cx.stop_propagation();
                                return;
                            }

                            if matches!(this.current_view, AppView::ScriptList)
                                && this.try_navigate_root_file_directory_with_tab(
                                    has_shift, window, cx,
                                )
                            {
                                cx.stop_propagation();
                                return;
                            }

                            // Tab-to-Agent deprecated: Cmd+Enter is the AI entry.
                            // Ghost text acceptance (above) now owns plain Tab.

                            // Agent Chat owns Tab locally. Plain Tab stays
                            // swallowed so the global interceptor cannot re-open a
                            // fresh chat. Shift+Tab is the documented Agent·Model
                            // shortcut, so route it to the in-chat profile/model
                            // picker via the window-aware entry point (the same
                            // method the footer Agent·Model chip uses). We do NOT
                            // reuse open_agent_model_picker_window here: that path
                            // forces non-ScriptList views back to ScriptList,
                            // which would leave Agent Chat mid-conversation.
                            if let AppView::AcpChatView { entity, .. } = &this.current_view {
                                if has_shift {
                                    cx.stop_propagation();
                                    if this.show_actions_popup
                                        || crate::actions::is_actions_window_open()
                                    {
                                        return;
                                    }
                                    tracing::info!(
                                        target: "script_kit::keyboard",
                                        event = "acp_shift_tab_profile_picker",
                                        "Opening Agent Chat profile/model picker from Shift+Tab"
                                    );
                                    let entity = entity.clone();
                                    entity.update(cx, |chat, cx| {
                                        chat.open_profile_trigger_picker_in_window(window, cx);
                                    });
                                    return;
                                }
                                let handled = entity
                                    .update(cx, |chat, cx| chat.handle_tab_key(false, cx));
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
                        });
                    }
                }

                // Keep plain Enter routed to ACP mention acceptance in the
                // embedded main-window host when the picker is open.
                if is_plain_enter {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            if let AppView::ScriptIssuesView { report } = &this.current_view {
                                let report = report.clone();
                                this.fix_script_issues_in_agent(&report, cx);
                                cx.stop_propagation();
                                return;
                            }
                            // Menu-syntax trigger popup owns Enter when it is
                            // visible on ScriptList — Accept the selected row
                            // the same way the ACP / and @ pickers do.
                            if matches!(this.current_view, AppView::ScriptList)
                                && crate::menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open()
                            {
                                if this.apply_menu_syntax_object_selector_intent(
                                    crate::menu_syntax::InlinePickerKeyIntent::Accept,
                                    window,
                                    cx,
                                ) {
                                    cx.stop_propagation();
                                    return;
                                }
                            }
                            if matches!(this.current_view, AppView::ScriptList)
                                && crate::menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open()
                            {
                                if this.apply_menu_syntax_trigger_popup_intent(
                                    crate::menu_syntax::InlinePickerKeyIntent::Accept,
                                    window,
                                    cx,
                                ) {
                                    cx.stop_propagation();
                                    return;
                                }
                            }
                            if let AppView::AcpChatView { entity, .. } = &this.current_view {
                                let handled =
                                    entity.update(cx, |chat, cx| chat.handle_enter_key(cx));
                                if handled {
                                    cx.stop_propagation();
                                }
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(tab_interceptor);

        // Prewarm ACP config and the hidden Agent Chat connection so the first
        // compatible ACP submit can reuse an initialized runtime/session.
        crate::ai::acp::prewarm_agent_config();

        // Prewarm Agent Chat and the Tab AI harness asynchronously so AI-entry
        // shortcuts do not pay subprocess/session startup cost on submit.
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
                    this.warm_acp_chat_on_startup(cx);
                    this.warm_tab_ai_harness_on_startup(cx);
                    this.warm_quick_terminal_pty(cx);
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

                if confirm::consume_main_window_key_while_confirm_open(
                    key,
                    &event.keystroke.modifiers,
                    cx,
                ) {
                    cx.stop_propagation();
                    return;
                }
                let no_direction_modifiers = !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control;

                // Alt+Left / Alt+Right cycle the live accent-color exploration
                // variation on the main menu. The emoji-grid branch below requires
                // no_direction_modifiers (which excludes alt), so there is no clash.
                let alt_direction = event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.shift;
                if (is_left || is_right) && alt_direction {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Scope to the main menu; leave other surfaces untouched.
                            if !matches!(this.current_view, AppView::ScriptList) {
                                return;
                            }
                            if this.show_actions_popup
                                || crate::actions::is_actions_window_open()
                            {
                                return;
                            }
                            this.cycle_accent_variation(is_right, window, cx);
                        });
                    }
                    cx.stop_propagation();
                    return;
                }

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

                            let frequent_snapshot = this.emoji_frequent_snapshot.clone();
                            if let AppView::EmojiPickerView {
                                selected_index,
                                filter,
                                selected_category,
                            } = &mut this.current_view
                            {
                                let display = crate::emoji::display_ordered_emojis(
                                    filter,
                                    *selected_category,
                                    &frequent_snapshot,
                                );
                                let filtered_len = display.emojis.len();
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

                                let row = crate::emoji::compute_display_scroll_row(
                                    *selected_index,
                                    &display,
                                );
                                this.emoji_scroll_handle
                                    .scroll_to_item(row, ScrollStrategy::Nearest);

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

                            let emoji_frequent_snapshot = this.emoji_frequent_snapshot.clone();
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

                                    let mut moved_selection = false;
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        moved_selection = true;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        moved_selection = true;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    if moved_selection {
                                        this.lock_file_search_selection_to_user_choice();
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
                                AppView::AcpHistoryView {
                                    selected_index,
                                    filter,
                                } => {
                                    let filtered_len = if filter.is_empty() {
                                        crate::ai::acp::history::load_history().len()
                                    } else {
                                        let filter_lower = filter.to_lowercase();
                                        crate::ai::acp::history::load_history()
                                            .into_iter()
                                            .filter(|entry| {
                                                entry
                                                    .first_message
                                                    .to_lowercase()
                                                    .contains(&filter_lower)
                                                    || entry
                                                        .timestamp
                                                        .to_lowercase()
                                                        .contains(&filter_lower)
                                            })
                                            .count()
                                    };
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.acp_history_scroll_handle
                                            .scroll_to_item(*selected_index);
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        this.acp_history_scroll_handle
                                            .scroll_to_item(*selected_index);
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::BrowserHistoryView {
                                    selected_index,
                                    filter,
                                } => {
                                    let filtered_len =
                                        crate::browser_history::fuzzy_search_browser_history(
                                            &this.cached_browser_history,
                                            filter,
                                        )
                                        .len();
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.browser_history_scroll_handle
                                            .scroll_to_item(*selected_index);
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        this.browser_history_scroll_handle
                                            .scroll_to_item(*selected_index);
                                        cx.notify();
                                    }
                                    cx.stop_propagation();
                                }
                                AppView::DictationHistoryView {
                                    selected_index,
                                    filter,
                                } => {
                                    let filtered_len =
                                        crate::dictation::search_history(filter, 100).len();
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.dictation_history_scroll_handle
                                            .scroll_to_item(*selected_index);
                                        cx.notify();
                                    } else if is_down && *selected_index + 1 < filtered_len {
                                        *selected_index += 1;
                                        this.dictation_history_scroll_handle
                                            .scroll_to_item(*selected_index);
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
                                AppView::EmojiPickerView {
                                    filter,
                                    selected_index,
                                    selected_category,
                                } => {
                                    let display = crate::emoji::display_ordered_emojis(
                                        filter,
                                        *selected_category,
                                        &emoji_frequent_snapshot,
                                    );
                                    let filtered_len = display.emojis.len();
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

                                    let layout = crate::emoji::build_display_grid_layout(
                                        &display,
                                        crate::emoji::GRID_COLS,
                                    );
                                    let direction = if is_up {
                                        crate::emoji::EmojiNavDirection::Up
                                    } else {
                                        crate::emoji::EmojiNavDirection::Down
                                    };
                                    *selected_index = layout.move_index(*selected_index, direction);
                                    let row = layout.scroll_row_for_index(*selected_index);
                                    this.emoji_scroll_handle
                                        .scroll_to_item(row, ScrollStrategy::Nearest);

                                    this.input_mode = InputMode::Keyboard;
                                    this.hovered_index = None;
                                    cx.notify();
                                    cx.stop_propagation();
                                }
                                AppView::ScriptList => {
                                    // Menu-syntax trigger popup owns Up/Down when it is
                                    // visible — route there BEFORE actions-popup and the
                                    // main-list navigation so the popup's selection
                                    // highlight tracks the arrow keys instead of the
                                    // launcher list underneath.
                                    if crate::menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open()
                                        && (is_up || is_down)
                                    {
                                        let intent = if is_up {
                                            crate::menu_syntax::InlinePickerKeyIntent::MoveUp
                                        } else {
                                            crate::menu_syntax::InlinePickerKeyIntent::MoveDown
                                        };
                                        if this.apply_menu_syntax_object_selector_intent(
                                            intent, window, cx,
                                        ) {
                                            cx.stop_propagation();
                                            return;
                                        }
                                    }
                                    if crate::menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open()
                                        && (is_up || is_down)
                                    {
                                        let intent = if is_up {
                                            crate::menu_syntax::InlinePickerKeyIntent::MoveUp
                                        } else {
                                            crate::menu_syntax::InlinePickerKeyIntent::MoveDown
                                        };
                                        if this.apply_menu_syntax_trigger_popup_intent(
                                            intent, window, cx,
                                        ) {
                                            cx.stop_propagation();
                                            return;
                                        }
                                    }

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
                                        if let Some(pending_filter) =
                                            this.history_filter_render_pending.as_ref()
                                        {
                                            tracing::info!(
                                                target: "script_kit::input_history",
                                                event = "history_key_repeat_coalesced_until_render",
                                                key = %key,
                                                pending_filter_len = pending_filter.len(),
                                                history_index = ?this.input_history.current_index(),
                                                selected_index = this.selected_index,
                                            );
                                            cx.stop_propagation();
                                            return;
                                        }
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
                                        let source_filter_mode =
                                            this.source_filter_mode_blocks_input_history_recall();
                                        let filter_has_text = !this.filter_text.is_empty()
                                            || !this.computed_filter_text.is_empty();

                                        tracing::info!(
                                            target: "script_kit::input_history",
                                            event = "main_menu_arrow_history_decision",
                                            key = %key,
                                            action_resolved = event.action.is_some(),
                                            context_depth = event.context_stack.len(),
                                            selected_index = this.selected_index,
                                            first_item_position = ?first_item_position,
                                            at_top_of_list,
                                            in_history,
                                            source_filter_mode,
                                            filter_has_text,
                                            history_index = ?this.input_history.current_index(),
                                            grouped_item_count = grouped_items.len(),
                                            route = if source_filter_mode {
                                                "source_filter_list_up"
                                            } else if filter_has_text {
                                                "filter_text_up_noop"
                                            } else if in_history || at_top_of_list {
                                                "history_up"
                                            } else {
                                                "list_up"
                                            },
                                        );
                                        if !source_filter_mode && filter_has_text && at_top_of_list {
                                            cx.stop_propagation();
                                            return;
                                        }

                                        if !source_filter_mode && (in_history || at_top_of_list) {
                                            if let Some(text) = this.input_history.navigate_up() {
                                                let safe = logging::log_user_value(&text);
                                                tracing::info!(
                                                    target: "script_kit::input_history",
                                                    event = "history_recalled",
                                                    direction = "up",
                                                    filter_preview = %safe,
                                                    filter_bytes = safe.raw_bytes,
                                                    filter_safe_bytes = safe.safe_bytes,
                                                    filter_truncated = safe.truncated,
                                                    history_index = ?this.input_history.current_index(),
                                                );
                                                logging::log(
                                                    HISTORY,
                                                    &format!("Recalled len={}", text.len()),
                                                );
                                                if text != this.filter_text
                                                    || text != this.computed_filter_text
                                                {
                                                    this.history_filter_render_pending =
                                                        Some(text.clone());
                                                    this.set_filter_text_immediate(
                                                        text, window, cx,
                                                    );
                                                } else {
                                                    tracing::info!(
                                                        target: "script_kit::input_history",
                                                        event = "history_recall_noop_already_rendered",
                                                        direction = "up",
                                                        filter_len = text.len(),
                                                        history_index = ?this.input_history.current_index(),
                                                    );
                                                }
                                            }
                                            cx.stop_propagation();
                                            return;
                                        }

                                        this.move_selection_up(cx);
                                    } else if is_down {
                                        if let Some(pending_filter) =
                                            this.history_filter_render_pending.as_ref()
                                        {
                                            tracing::info!(
                                                target: "script_kit::input_history",
                                                event = "history_key_repeat_coalesced_until_render",
                                                key = %key,
                                                pending_filter_len = pending_filter.len(),
                                                history_index = ?this.input_history.current_index(),
                                                selected_index = this.selected_index,
                                            );
                                            cx.stop_propagation();
                                            return;
                                        }
                                        let in_history =
                                            this.input_history.current_index().is_some();
                                        let source_filter_mode =
                                            this.source_filter_mode_blocks_input_history_recall();
                                        tracing::info!(
                                            target: "script_kit::input_history",
                                            event = "main_menu_arrow_history_decision",
                                            key = %key,
                                            action_resolved = event.action.is_some(),
                                            context_depth = event.context_stack.len(),
                                            selected_index = this.selected_index,
                                            in_history,
                                            source_filter_mode,
                                            history_index = ?this.input_history.current_index(),
                                            route = if source_filter_mode {
                                                "source_filter_list_down"
                                            } else if in_history {
                                                "history_down"
                                            } else {
                                                "list_down"
                                            },
                                        );
                                        if !source_filter_mode && in_history {
                                            if let Some(text) = this.input_history.navigate_down() {
                                                let safe = logging::log_user_value(&text);
                                                tracing::info!(
                                                    target: "script_kit::input_history",
                                                    event = "history_recalled",
                                                    direction = "down",
                                                    filter_preview = %safe,
                                                    filter_bytes = safe.raw_bytes,
                                                    filter_safe_bytes = safe.safe_bytes,
                                                    filter_truncated = safe.truncated,
                                                    history_index = ?this.input_history.current_index(),
                                                );
                                                logging::log(
                                                    HISTORY,
                                                    &format!("Recalled len={}", text.len()),
                                                );
                                                if text != this.filter_text
                                                    || text != this.computed_filter_text
                                                {
                                                    this.history_filter_render_pending =
                                                        Some(text.clone());
                                                    this.set_filter_text_immediate(
                                                        text, window, cx,
                                                    );
                                                } else {
                                                    tracing::info!(
                                                        target: "script_kit::input_history",
                                                        event = "history_recall_noop_already_rendered",
                                                        direction = "down",
                                                        filter_len = text.len(),
                                                        history_index = ?this.input_history.current_index(),
                                                    );
                                                }
                                            } else {
                                                this.input_history.reset_navigation();
                                                if !this.filter_text.is_empty()
                                                    || !this.computed_filter_text.is_empty()
                                                {
                                                    this.history_filter_render_pending =
                                                        Some(String::new());
                                                }
                                                this.clear_filter(window, cx);
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

                if confirm::consume_main_window_key_while_confirm_open(
                    key,
                    &event.keystroke.modifiers,
                    cx,
                ) {
                    cx.stop_propagation();
                    return;
                }

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
                let is_notes = crate::notes::is_notes_window(window);
                let is_ai = crate::ai::is_ai_window(window);
                let is_detached_acp = crate::ai::acp::chat_window::is_chat_window(window);
                let is_actions = crate::actions::is_actions_window(window);

                let key = event.keystroke.key.as_str();
                let key_lower = key.to_ascii_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;
                let has_alt = event.keystroke.modifiers.alt;
                let has_ctrl = event.keystroke.modifiers.control;
                let key_char = event.keystroke.key_char.as_deref();
                let is_actions_close_key = crate::ui_foundation::is_key_escape(key)
                    || (has_cmd && key.eq_ignore_ascii_case("k") && !has_shift);

                // ACP can open the shared actions dialog from its own focused
                // composer even when the launcher visibility flag is false.
                // Close keys still need to reach the shared dialog before the
                // hidden-window guard below has a chance to skip them.
                if is_actions_close_key {
                    let mut close_key_routed = false;
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            if !is_actions
                                && !this.show_actions_popup
                                && !crate::actions::is_actions_window_open()
                            {
                                return;
                            }
                            let Some(host) = this.current_actions_host() else {
                                return;
                            };
                            match this.route_key_to_actions_dialog(
                                key,
                                key_char,
                                &event.keystroke.modifiers,
                                host,
                                window,
                                cx,
                            ) {
                                ActionsRoute::NotHandled => {}
                                ActionsRoute::Handled | ActionsRoute::Execute { .. } => {
                                    tracing::info!(
                                        target: "script_kit::actions",
                                        event = if is_actions {
                                            "actions_interceptor_routed_from_actions_window"
                                        } else {
                                            "actions_interceptor_routed_close_before_visibility_guard"
                                        },
                                        host = ?host,
                                        key = %key,
                                    );
                                    cx.stop_propagation();
                                    close_key_routed = true;
                                }
                            }
                        });
                    }
                    if close_key_routed {
                        return;
                    }
                }

                // When the main window is hidden (e.g. Notes/AI open), main-menu
                // key interceptors must not consume keystrokes from secondary windows.
                if !script_kit_gpui::is_main_window_visible() {
                    tracing::debug!(
                        target: "script_kit::keyboard",
                        event = "actions_interceptor_main_window_hidden",
                        is_notes,
                        is_ai,
                        is_detached_acp,
                        is_actions,
                    );
                    return;
                }

                if is_actions {
                    return;
                }

                // CRITICAL: Skip processing if this keystroke is from a secondary window.
                // intercept_keystrokes is GLOBAL and fires for ALL windows in the app.
                // We only want to handle keystrokes for the main window.
                if is_notes || is_ai || is_detached_acp {
                    tracing::debug!(
                        target: "script_kit::keyboard",
                        event = "actions_interceptor_skipped_secondary_window",
                        is_notes,
                        is_ai,
                        is_detached_acp,
                        is_actions,
                    );
                    return; // Let the secondary window handle its own keystrokes
                }

                if confirm::consume_main_window_key_while_confirm_open(
                    key,
                    &event.keystroke.modifiers,
                    cx,
                ) {
                    cx.stop_propagation();
                    return;
                }

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Route shared actions-dialog keys first; local actions
                        // key intents run only after the dialog declines the key.
                        let host = this.current_actions_host();

                        // Arrow keys are handled by arrow_interceptor to avoid double-processing
                        // (which can skip 2 items per keypress when both interceptors handle arrows).
                        if this.show_actions_popup
                            && (crate::ui_foundation::is_key_up(key)
                                || crate::ui_foundation::is_key_down(key))
                        {
                            return;
                        }

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
                                    tracing::debug!(
                                        target: "script_kit::actions",
                                        event = "actions_interceptor_routed",
                                        host = ?host,
                                        key = %key,
                                    );
                                    cx.stop_propagation();
                                    return;
                                }
                                ActionsRoute::Execute {
                                    action_id,
                                    should_close,
                                } => {
                                    this.execute_actions_route_action(
                                        host,
                                        action_id,
                                        should_close,
                                        window,
                                        cx,
                                    );
                                    cx.stop_propagation();
                                    return;
                                }
                            }
                        }

                        if this.try_execute_root_file_action_shortcut(
                            &key_lower, has_cmd, has_shift, has_alt, has_ctrl, window, cx,
                        ) {
                            cx.stop_propagation();
                            return;
                        }

                        if let Some(intent) =
                            main_window_actions_key_intent(&this.current_view, event)
                        {
                            if this.handle_main_window_actions_key_intent(intent, window, cx) {
                                if matches!(intent, MainWindowActionsKeyIntent::ToggleActions) {
                                    tracing::info!(
                                        target: "script_kit::actions",
                                        event = "actions_interceptor_toggled",
                                        host = ?host,
                                    );
                                }
                                cx.stop_propagation();
                                return;
                            }
                        }

                        let acp_escape_popup_open = match &this.current_view {
                            AppView::AcpChatView { entity, .. } => {
                                entity.read(cx).has_escape_dismissible_popup()
                            }
                            _ => false,
                        };
                        let acp_escape_focused_text_origin = match &this.current_view {
                            AppView::AcpChatView { entity, .. } => {
                                let chat = entity.read(cx);
                                chat.is_focused_text_mini()
                                    || chat.focused_text_originated_from_quick_prompt()
                            }
                            _ => false,
                        };

                        let acp_escape_cancelled_streaming = if crate::ui_foundation::is_key_escape(key)
                            && !has_cmd
                            && !has_shift
                            && !acp_escape_focused_text_origin
                        {
                            match &this.current_view {
                                AppView::AcpChatView { entity, .. } => entity.update(cx, |chat, cx| {
                                    chat.cancel_streaming_from_escape(cx)
                                }),
                                _ => false,
                            }
                        } else {
                            false
                        };
                        if acp_escape_cancelled_streaming {
                            logging::log(
                                "KEY",
                                "Interceptor: Escape -> cancel Agent Chat streaming",
                            );
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Escape for AcpChatView.
                        if crate::ui_foundation::is_key_escape(key)
                            && !has_cmd
                            && !has_shift
                            && !this.show_actions_popup
                            && !acp_escape_popup_open
                            && matches!(this.current_view, AppView::AcpChatView { .. })
                        {
                            if acp_escape_focused_text_origin {
                                tracing::info!(
                                    target: "script_kit::keyboard",
                                    event = "focused_text_quick_prompt_escape_hide_requested",
                                );
                                this.close_acp_chat_main_window_state_first(cx);
                                logging::log(
                                    "KEY",
                                    "Interceptor: Escape -> hide focused-text quick prompt Agent Chat",
                                );
                                cx.stop_propagation();
                                return;
                            }
                            if this.opened_from_main_menu {
                                tracing::info!(
                                    target: "script_kit::keyboard",
                                    event = "embedded_acp_escape_return_to_origin",
                                );
                                this.close_tab_ai_harness_terminal_with_window(window, cx);
                                logging::log(
                                    "KEY",
                                    "Interceptor: Escape -> return to main menu from Agent Chat",
                                );
                            } else {
                                tracing::info!(
                                    target: "script_kit::keyboard",
                                    event = "embedded_acp_escape_close_window",
                                );
                                this.close_acp_chat_main_window_state_first(cx);
                                logging::log(
                                    "KEY",
                                    "Interceptor: Escape -> close Agent Chat window",
                                );
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Cmd+Shift+K for add_shortcut in ScriptList
                        if has_cmd
                            && key.eq_ignore_ascii_case("k")
                            && has_shift
                            && matches!(this.current_view, AppView::ScriptList)
                        {
                            logging::log(
                                "KEY",
                                "Interceptor: Cmd+Shift+K -> add_shortcut (ScriptList)",
                            );
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
                                logging::log(
                                    "KEY",
                                    &format!(
                                        "Interceptor: Cmd+- (key={}) -> decrease light opacity",
                                        key
                                    ),
                                );
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
                                logging::log(
                                    "KEY",
                                    &format!(
                                        "Interceptor: Cmd+= (key={}) -> increase light opacity",
                                        key
                                    ),
                                );
                                this.adjust_light_opacity(0.05, cx);
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+M to cycle vibrancy material (blur effect)
                            if has_cmd && !has_shift && key.eq_ignore_ascii_case("m") {
                                logging::log(
                                    "KEY",
                                    "Interceptor: Cmd+M -> cycle vibrancy material",
                                );
                                let description = platform::cycle_vibrancy_material();
                                this.toast_manager.push(
                                    components::toast::Toast::info(description, &this.theme)
                                        .duration_ms(Some(TOAST_INFO_MS)),
                                );
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }

                            // Handle Cmd+Shift+A to cycle vibrancy appearance (VibrantLight, VibrantDark, etc.)
                            if has_cmd && has_shift && key.eq_ignore_ascii_case("a") {
                                logging::log(
                                    "KEY",
                                    "Interceptor: Cmd+Shift+A -> cycle vibrancy appearance",
                                );
                                let description = platform::cycle_appearance();
                                this.toast_manager.push(
                                    components::toast::Toast::info(description, &this.theme)
                                        .duration_ms(Some(TOAST_INFO_MS)),
                                );
                                cx.notify();
                                cx.stop_propagation();
                                return;
                            }
                        }

                        // Only handle remaining keys if in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            return;
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
