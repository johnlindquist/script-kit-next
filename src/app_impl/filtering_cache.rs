use super::*;

const INLINE_CALCULATOR_SECTION_LABEL: &str = "Calculator";
const INLINE_CALCULATOR_RESULT_INDEX: usize = usize::MAX;

fn timed_root_passive_source<T>(
    source: &'static str,
    query: &str,
    explicit: bool,
    f: impl FnOnce() -> Vec<T>,
) -> Vec<T> {
    let start = std::time::Instant::now();
    let rows = f();
    let elapsed = start.elapsed();
    if logging::filter_perf_trace_enabled() || elapsed >= std::time::Duration::from_millis(8) {
        logging::log(
            "FILTER_PERF",
            &format!(
                "[PASSIVE_SOURCE_DONE] source={} query_len={} explicit={} in {:.2}ms -> {} hits",
                source,
                query.chars().count(),
                explicit,
                elapsed.as_secs_f64() * 1000.0,
                rows.len()
            ),
        );
    }
    rows
}

fn prepend_inline_calculator_group(
    grouped_items: Vec<GroupedListItem>,
    flat_results: Vec<scripts::SearchResult>,
    calculator: Option<&crate::calculator::CalculatorInlineResult>,
) -> (Vec<GroupedListItem>, Vec<scripts::SearchResult>) {
    let Some(_calculator) = calculator else {
        return (grouped_items, flat_results);
    };

    let mut merged_grouped_items = Vec::with_capacity(grouped_items.len() + 2);
    merged_grouped_items.push(GroupedListItem::SectionHeader(
        INLINE_CALCULATOR_SECTION_LABEL.to_string(),
        None,
    ));
    merged_grouped_items.push(GroupedListItem::Item(INLINE_CALCULATOR_RESULT_INDEX));
    merged_grouped_items.extend(grouped_items);

    (merged_grouped_items, flat_results)
}

fn root_window_duplicate_key(window: &crate::window_control::WindowInfo) -> (String, String) {
    (
        window
            .bundle_id
            .clone()
            .unwrap_or_else(|| window.app.to_lowercase()),
        window.title.to_lowercase(),
    )
}

fn root_window_duplicate_counts(
    windows: &[crate::window_control::WindowInfo],
) -> std::collections::HashMap<(String, String), usize> {
    let mut counts = std::collections::HashMap::new();
    for window in windows {
        *counts.entry(root_window_duplicate_key(window)).or_insert(0) += 1;
    }
    counts
}

impl ScriptListApp {
    pub(crate) fn filter_text(&self) -> &str {
        self.filter_text.as_str()
    }

    pub(crate) fn build_root_window_entries(
        windows: &[crate::window_control::WindowInfo],
        apps: &[crate::app_launcher::AppInfo],
        recency: &std::collections::HashMap<String, u64>,
    ) -> Vec<crate::scripts::RootWindowEntry> {
        let lookup = crate::app_launcher::AppIconLookup::from_apps(apps);
        let duplicate_counts = root_window_duplicate_counts(windows);
        let mut duplicate_seen = std::collections::HashMap::<(String, String), usize>::new();

        let mut entries = windows
            .iter()
            .cloned()
            .map(|window| {
                let duplicate_key = root_window_duplicate_key(&window);
                let duplicate_count = duplicate_counts.get(&duplicate_key).copied().unwrap_or(1);
                let duplicate_rank = if duplicate_count > 1 {
                    let rank = duplicate_seen.entry(duplicate_key).or_insert(0);
                    *rank += 1;
                    Some(*rank)
                } else {
                    None
                };
                let duplicate_label =
                    duplicate_rank.map(|rank| format!("Window {rank} of {duplicate_count}"));
                let subtitle = crate::window_control::build_window_descriptor(
                    &window.app,
                    window.pid,
                    window.bounds,
                    window.is_frontmost_app,
                    window.is_focused,
                    window.is_main,
                    window.is_minimized,
                    window.is_on_current_space,
                    duplicate_label.as_deref(),
                );
                let local_recency_seq = recency.get(&window.selection_key()).copied();
                crate::scripts::RootWindowEntry {
                    app_icon: lookup.icon_for_window(&window),
                    subtitle,
                    duplicate_rank,
                    duplicate_count,
                    local_recency_seq,
                    window,
                }
            })
            .collect::<Vec<_>>();

        entries.sort_by(|a, b| {
            b.window
                .is_frontmost_app
                .cmp(&a.window.is_frontmost_app)
                .then_with(|| b.window.is_focused.cmp(&a.window.is_focused))
                .then_with(|| b.window.is_main.cmp(&a.window.is_main))
                .then_with(|| b.local_recency_seq.cmp(&a.local_recency_seq))
                .then_with(|| a.window.is_minimized.cmp(&b.window.is_minimized))
                .then_with(|| a.window.app_order.cmp(&b.window.app_order))
                .then_with(|| a.window.window_index.cmp(&b.window.window_index))
                .then_with(|| a.window.title.cmp(&b.window.title))
                .then_with(|| a.window.id.cmp(&b.window.id))
        });

        entries
    }

    pub(crate) fn install_root_windows(
        &mut self,
        windows: Vec<crate::window_control::WindowInfo>,
        cx: &mut Context<Self>,
    ) {
        self.cached_windows = windows;
        self.cached_root_windows = Self::build_root_window_entries(
            &self.cached_windows,
            &self.apps,
            &self.root_window_focus_recency,
        );
        let count = self.cached_root_windows.len();
        self.root_windows_refresh_generation = self.root_windows_refresh_generation.wrapping_add(1);
        self.root_windows_provider_status =
            crate::window_control::RootWindowsProviderStatus::Ready { count };
        self.root_windows_last_completed_at = Some(std::time::Instant::now());
        self.invalidate_grouped_cache();
        self.reconcile_script_list_after_filter_change("root_windows_refresh_complete", cx);
        cx.notify();
    }

    pub(crate) fn rebuild_root_windows_after_app_icon_cache_update(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        if self.cached_windows.is_empty() {
            return;
        }

        self.cached_root_windows = Self::build_root_window_entries(
            &self.cached_windows,
            &self.apps,
            &self.root_window_focus_recency,
        );
        self.root_windows_refresh_generation = self.root_windows_refresh_generation.wrapping_add(1);
        self.invalidate_grouped_cache();
        self.reconcile_script_list_after_filter_change(reason, cx);
        self.rebuild_main_window_preflight_if_needed();
    }

    pub(crate) fn maybe_start_root_windows_refresh_for_query(
        &mut self,
        query_text: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(advanced_query) = self.menu_syntax_mode.advanced_query_for(query_text) else {
            return;
        };
        let windows_explicit = advanced_query
            .source_filters
            .includes(crate::menu_syntax::RootUnifiedSourceFilter::Windows)
            && advanced_query
                .source_filters
                .allows(crate::menu_syntax::RootUnifiedSourceFilter::Windows);
        if !windows_explicit || self.root_windows_refreshing {
            return;
        }

        let stale = self
            .root_windows_last_completed_at
            .map(|completed_at| completed_at.elapsed() >= std::time::Duration::from_secs(3))
            .unwrap_or(true);
        if !self.cached_root_windows.is_empty() && !stale {
            return;
        }

        self.root_windows_refreshing = true;
        self.root_windows_refresh_token = self.root_windows_refresh_token.wrapping_add(1);
        let token = self.root_windows_refresh_token;
        let stale_count = self.cached_root_windows.len();
        self.root_windows_provider_status =
            crate::window_control::RootWindowsProviderStatus::Refreshing { count: stale_count };
        self.root_windows_refresh_generation = self.root_windows_refresh_generation.wrapping_add(1);
        self.invalidate_grouped_cache();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { crate::window_control::list_windows() })
                .await;

            let _ = this.update(cx, |app, cx| {
                if app.root_windows_refresh_token != token {
                    return;
                }
                app.root_windows_refreshing = false;
                match result {
                    Ok(windows) => app.install_root_windows(windows, cx),
                    Err(error) => {
                        let message = error.to_string();
                        let lower = message.to_ascii_lowercase();
                        app.root_windows_provider_status =
                            if lower.contains("accessibility") || lower.contains("permission") {
                                crate::window_control::RootWindowsProviderStatus::PermissionRequired
                            } else {
                                crate::window_control::RootWindowsProviderStatus::ProviderError {
                                    message: message
                                        .lines()
                                        .next()
                                        .unwrap_or("unknown error")
                                        .to_string(),
                                }
                            };
                        app.root_windows_refresh_generation =
                            app.root_windows_refresh_generation.wrapping_add(1);
                        app.invalidate_grouped_cache();
                        app.reconcile_script_list_after_filter_change(
                            "root_windows_refresh_error",
                            cx,
                        );
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub(crate) fn current_query_includes_root_source(
        &self,
        query_text: &str,
        source: crate::menu_syntax::RootUnifiedSourceFilter,
    ) -> bool {
        self.menu_syntax_mode
            .advanced_query_for(query_text)
            .is_some_and(|advanced_query| {
                advanced_query.source_filters.includes(source)
                    && advanced_query.source_filters.allows(source)
            })
    }

    pub(crate) fn invalidate_root_passive_and_grouped_cache(&mut self) {
        self.root_passive_frame = None;
        self.invalidate_grouped_cache();
        self.invalidate_main_window_preflight();
    }

    fn root_browser_tabs_refresh_options_for_query(
        &self,
        query_text: &str,
    ) -> Option<(crate::browser_tabs::RootBrowserTabsSectionOptions, bool)> {
        let source = crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs;
        let unified_search = self.config.get_unified_search();
        let mut options = unified_search.browser_tabs_section_options();
        let advanced_query = self.menu_syntax_mode.advanced_query_for(query_text);
        let source_filters = advanced_query
            .map(|query| query.source_filters.clone())
            .unwrap_or_default();
        let explicit_tabs = source_filters.includes(source) && source_filters.allows(source);

        if explicit_tabs {
            options.enabled = true;
            options.min_query_chars = 0;
            options.max_results = options
                .max_results
                .max(unified_search.passive_result_limits().max_total_results);
            return Some((options, true));
        }

        if !source_filters.allows(source) {
            return None;
        }

        if advanced_query.is_some_and(|query| query.has_predicates()) {
            return None;
        }

        if self.menu_syntax_object_selector_state.owns_main_list()
            || self.menu_syntax_trigger_popup_state.owns_main_list()
            || self
                .menu_syntax_mode
                .capture_composer_owns_input_for(query_text)
            || self.menu_syntax_mode.command_owns_input_for(query_text)
        {
            return None;
        }

        let search_text =
            crate::menu_syntax::free_text_for_search(&self.menu_syntax_mode, query_text);
        if !crate::browser_tabs::root_browser_tabs_query_is_eligible(search_text, options.clone()) {
            return None;
        }

        Some((options, false))
    }

    fn current_query_can_show_root_browser_tabs(&self, query_text: &str) -> bool {
        self.root_browser_tabs_refresh_options_for_query(query_text)
            .is_some()
    }

    pub(crate) fn maybe_start_root_browser_tabs_refresh_for_query(
        &mut self,
        query_text: &str,
        cx: &mut Context<Self>,
    ) {
        let Some((options, explicit_tabs)) =
            self.root_browser_tabs_refresh_options_for_query(query_text)
        else {
            return;
        };

        let providers = options.providers.clone();
        let reason = if explicit_tabs {
            "explicit_tabs_query"
        } else {
            "implicit_tabs_query"
        };
        let Some(refresh) = crate::browser_tabs::try_begin_root_browser_tabs_refresh(
            options.cache_ttl_ms,
            providers.len(),
            reason,
        ) else {
            return;
        };

        self.invalidate_root_passive_and_grouped_cache();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result =
                cx.background_executor()
                    .spawn(async move {
                        crate::browser_tabs::refresh_root_browser_tabs_snapshot(providers)
                    })
                    .await;

            let _ = this.update(cx, |app, cx| {
                let changed =
                    crate::browser_tabs::finish_root_browser_tabs_refresh(refresh, result);
                if !changed {
                    return;
                }
                app.invalidate_root_passive_and_grouped_cache();
                if app.current_query_can_show_root_browser_tabs(&app.computed_filter_text) {
                    app.reconcile_script_list_after_filter_change(
                        "browser_tabs_refresh_complete",
                        cx,
                    );
                }
                app.rebuild_main_window_preflight_if_needed();
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn maybe_start_root_browser_history_refresh_for_query(
        &mut self,
        query_text: &str,
        cx: &mut Context<Self>,
    ) {
        let source = crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory;
        if !self.current_query_includes_root_source(query_text, source) {
            return;
        }

        let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
            return;
        };
        let unified_search = self.config.get_unified_search();
        let mut options = unified_search.browser_history_section_options();
        options.enabled = true;
        options.min_query_chars = 0;
        options.max_age_days = 365;
        options.max_results = options
            .max_results
            .max(unified_search.passive_result_limits().max_total_results);
        let Some(refresh) = crate::browser_history::try_begin_root_browser_history_refresh(
            &options,
            "explicit_history_query",
        ) else {
            return;
        };

        self.invalidate_root_passive_and_grouped_cache();
        cx.notify();

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = crate::browser_history::refresh_root_browser_history_snapshot_from_home(
                &home, &options,
            );
            let _ = tx.send(result);
        });

        cx.spawn(async move |this, cx| {
            let result = loop {
                match rx.try_recv() {
                    Ok(result) => break result,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(16))
                            .await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break Err(anyhow::anyhow!(
                            "browser history refresh worker disconnected"
                        ));
                    }
                }
            };

            let _ = this.update(cx, |app, cx| {
                let changed =
                    crate::browser_history::finish_root_browser_history_refresh(refresh, result);
                if !changed {
                    return;
                }
                app.invalidate_root_passive_and_grouped_cache();
                if app.current_query_includes_root_source(&app.computed_filter_text, source) {
                    app.reconcile_script_list_after_filter_change(
                        "browser_history_refresh_complete",
                        cx,
                    );
                }
                app.rebuild_main_window_preflight_if_needed();
                cx.notify();
            });
        })
        .detach();
    }

    fn root_passive_frame_for_current_query(
        &mut self,
        search_text: &str,
        advanced_query_active: bool,
        source_filters: crate::menu_syntax::RootUnifiedSourceFilterSet,
        todo_options: crate::menu_syntax::RootTodoSectionOptions,
        notes_options: crate::notes::RootNotesSectionOptions,
        clipboard_history_options: crate::clipboard_history::RootClipboardHistorySectionOptions,
        dictation_history_options: crate::dictation::RootDictationHistorySectionOptions,
        acp_history_options: crate::ai::acp::history::RootAcpHistorySectionOptions,
        ai_vault_options: crate::ai_vault::RootAiVaultSectionOptions,
        browser_tabs_options: crate::browser_tabs::RootBrowserTabsSectionOptions,
        browser_history_options: crate::browser_history::RootBrowserHistorySectionOptions,
    ) -> crate::RootPassiveFrame {
        let ai_vault_status = crate::ai_vault::root_ai_vault_snapshot_status();
        let browser_tabs_status = crate::browser_tabs::root_browser_tabs_snapshot_status();
        let browser_history_status = crate::browser_history::root_browser_history_snapshot_status();
        let key = crate::RootPassiveFrameKey {
            query: search_text.to_string(),
            advanced_query: advanced_query_active,
            source_filters: source_filters.clone(),
            todo_options,
            notes_options,
            clipboard_history_options,
            dictation_history_options,
            acp_history_options,
            ai_vault_options: ai_vault_options.clone(),
            ai_vault_snapshot_generation: ai_vault_status.generation,
            browser_tabs_options: browser_tabs_options.clone(),
            browser_tabs_snapshot_generation: browser_tabs_status.generation,
            browser_history_options: browser_history_options.clone(),
            browser_history_snapshot_generation: browser_history_status.generation,
        };

        if let Some(frame) = self.root_passive_frame.as_ref() {
            if frame.key == key {
                return frame.clone();
            }
        }

        let explicit_notes =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Notes);
        let explicit_todos =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Todo);
        let explicit_clipboard =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory);
        let explicit_dictation =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Dictation);
        let explicit_conversations =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Conversations);
        let explicit_ai_vault =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::AiVault);
        let explicit_browser_tabs =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs);
        let explicit_browser_history =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory);

        let allow_notes = source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Notes);
        let allow_todos = source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Todo);
        let allow_clipboard =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory);
        let allow_dictation =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Dictation);
        let allow_conversations =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Conversations);
        let allow_ai_vault =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::AiVault);
        let allow_browser_tabs =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs);
        let allow_browser_history =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory);

        let note_hits = timed_root_passive_source("notes", search_text, explicit_notes, || {
            if !advanced_query_active
                && allow_notes
                && crate::notes::root_notes_query_is_eligible(search_text, notes_options)
            {
                if explicit_notes {
                    crate::notes::search_root_notes_meta_direct(search_text, notes_options)
                } else {
                    crate::notes::search_root_notes_meta_cached(search_text, notes_options)
                }
            } else {
                Vec::new()
            }
        });

        let todo_hits = timed_root_passive_source("todo", search_text, explicit_todos, || {
            if (!advanced_query_active || explicit_todos)
                && allow_todos
                && crate::menu_syntax::root_todo_query_is_eligible(search_text, todo_options)
            {
                if explicit_todos {
                    crate::menu_syntax::search_root_todos_direct(search_text, todo_options)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        });

        let clipboard_history_hits =
            timed_root_passive_source("clipboard_history", search_text, explicit_clipboard, || {
                if !advanced_query_active
                    && allow_clipboard
                    && crate::clipboard_history::root_clipboard_history_query_is_eligible(
                        search_text,
                        clipboard_history_options,
                    )
                {
                    if explicit_clipboard {
                        crate::clipboard_history::search_root_clipboard_history_meta_direct(
                            search_text,
                            clipboard_history_options,
                        )
                    } else {
                        crate::clipboard_history::search_root_clipboard_history_meta_cached(
                            search_text,
                            clipboard_history_options,
                        )
                    }
                } else {
                    Vec::new()
                }
            });

        let dictation_history_hits =
            timed_root_passive_source("dictation_history", search_text, explicit_dictation, || {
                if !advanced_query_active
                    && allow_dictation
                    && crate::dictation::root_dictation_history_query_is_eligible(
                        search_text,
                        dictation_history_options,
                    )
                {
                    if explicit_dictation {
                        crate::dictation::search_root_dictation_history_direct(
                            search_text,
                            dictation_history_options,
                        )
                    } else {
                        crate::dictation::search_root_dictation_history_cached(
                            search_text,
                            dictation_history_options,
                        )
                    }
                } else {
                    Vec::new()
                }
            });

        let acp_history_hits =
            timed_root_passive_source("acp_history", search_text, explicit_conversations, || {
                if !advanced_query_active
                    && allow_conversations
                    && crate::ai::acp::history::root_acp_history_query_is_eligible(
                        search_text,
                        acp_history_options,
                    )
                {
                    if explicit_conversations {
                        crate::ai::acp::history::search_history_direct(
                            search_text,
                            acp_history_options.max_results,
                        )
                    } else {
                        crate::ai::acp::history::search_history_cached(
                            search_text,
                            acp_history_options.max_results,
                        )
                    }
                } else {
                    Vec::new()
                }
            });

        let ai_vault_hits =
            timed_root_passive_source("ai_vault", search_text, explicit_ai_vault, || {
                if explicit_ai_vault
                    && !advanced_query_active
                    && allow_ai_vault
                    && crate::ai_vault::root_ai_vault_query_is_eligible(
                        search_text,
                        &ai_vault_options,
                    )
                {
                    crate::ai_vault::search_root_ai_vault_direct(search_text, ai_vault_options)
                } else {
                    Vec::new()
                }
            });

        let browser_tab_hits =
            timed_root_passive_source("browser_tabs", search_text, explicit_browser_tabs, || {
                if !advanced_query_active
                    && allow_browser_tabs
                    && crate::browser_tabs::root_browser_tabs_query_is_eligible(
                        search_text,
                        browser_tabs_options.clone(),
                    )
                {
                    if explicit_browser_tabs {
                        crate::browser_tabs::search_root_browser_tabs_meta_direct(
                            search_text,
                            browser_tabs_options.clone(),
                        )
                    } else {
                        crate::browser_tabs::search_root_browser_tabs_meta_cached(
                            search_text,
                            browser_tabs_options.clone(),
                        )
                    }
                } else {
                    Vec::new()
                }
            });

        let browser_history_hits = timed_root_passive_source(
            "browser_history",
            search_text,
            explicit_browser_history,
            || {
                if explicit_browser_history
                    && !advanced_query_active
                    && allow_browser_history
                    && crate::browser_history::root_browser_history_query_is_eligible(
                        search_text,
                        browser_history_options.clone(),
                    )
                {
                    crate::browser_history::search_root_browser_history_meta_direct(
                        search_text,
                        browser_history_options.clone(),
                    )
                } else {
                    Vec::new()
                }
            },
        );

        let frame = crate::RootPassiveFrame {
            key,
            note_hits,
            todo_hits,
            clipboard_history_hits,
            dictation_history_hits,
            acp_history_hits,
            ai_vault_hits,
            browser_tab_hits,
            browser_history_hits,
            ai_vault_snapshot_generation: ai_vault_status.generation,
            browser_tabs_snapshot_generation: browser_tabs_status.generation,
            browser_history_snapshot_generation: browser_history_status.generation,
        };
        self.root_passive_frame = Some(frame.clone());
        frame
    }

    fn root_file_frame_for_current_query(
        &mut self,
        search_text: &str,
        advanced_query_active: bool,
        source_filters: crate::menu_syntax::RootUnifiedSourceFilterSet,
        root_file_options: crate::file_search::RootFileSectionOptions,
    ) -> crate::RootFileFrame {
        let key = crate::RootFileFrameKey {
            query: search_text.to_string(),
            advanced_query: advanced_query_active,
            source_filters,
            mode: self.root_file_search_mode,
            options: root_file_options,
        };

        if let Some(frame) = self.root_file_frame.as_ref() {
            if frame.key == key {
                return frame.clone();
            }
        }

        let frame = crate::RootFileFrame {
            key,
            mode: self.root_file_search_mode,
            visible_loading: self.root_file_search_loading,
            file_results: self.root_file_results.clone(),
            recent_file_results: self.root_recent_file_results.clone(),
        };
        self.root_file_frame = Some(frame.clone());
        frame
    }

    /// Shared recompute helper: every filtered search path routes through here
    /// so plugin skills are always included in main-menu results.
    fn recompute_filtered_results(&self, filter_text: &str) -> Vec<scripts::SearchResult> {
        let search_text =
            crate::menu_syntax::free_text_for_search(&self.menu_syntax_mode, filter_text);
        if self
            .menu_syntax_mode
            .advanced_query_for(filter_text)
            .is_some_and(|query| query.has_source_filters())
        {
            return Vec::new();
        }
        let search_start = std::time::Instant::now();
        let results = scripts::fuzzy_search_unified_all_with_skills(
            &self.scripts,
            &self.scriptlets,
            &self.builtin_entries,
            &self.apps,
            &self.skills,
            search_text,
        );
        let results = match self.menu_syntax_mode.advanced_query_for(filter_text) {
            Some(query) => crate::menu_syntax::apply_advanced_query(results, query),
            None => results,
        };
        let search_elapsed = search_start.elapsed();

        if !filter_text.is_empty() {
            logging::log(
                "PERF",
                &format!(
                    "Search '{}' (computed '{}') took {:.2}ms ({} results from {} total, including {} skills)",
                    filter_text,
                    search_text,
                    search_elapsed.as_secs_f64() * 1000.0,
                    results.len(),
                    self.scripts.len()
                        + self.scriptlets.len()
                        + self.builtin_entries.len()
                        + self.apps.len()
                        + self.skills.len(),
                    self.skills.len(),
                ),
            );
        }

        tracing::info!(
            filter_text = %filter_text,
            search_text = %search_text,
            result_count = results.len(),
            script_count = self.scripts.len(),
            scriptlet_count = self.scriptlets.len(),
            builtin_count = self.builtin_entries.len(),
            app_count = self.apps.len(),
            skill_count = self.skills.len(),
            "main_menu_filtered_results_recomputed"
        );

        results
    }

    /// P1: Now uses caching - invalidates only when filter_text changes
    pub(crate) fn filtered_results(&self) -> Vec<scripts::SearchResult> {
        let filter_text = self.filter_text();
        // When a composer-style menu-syntax popup owns the input (e.g. `;t`
        // or `!dep`), the main launcher should not report or render fuzzy
        // matches — the popup is the sole surface for the typed characters.
        // Without this gate, `getState.visibleChoiceCount`, automation
        // `getElements`, and selection-coercion code would keep iterating
        // over stale fuzzy results (e.g. 8 semicolon-ish script matches) behind
        // the popup.
        if self.menu_syntax_object_selector_state.owns_main_list()
            || self.menu_syntax_trigger_popup_state.owns_main_list()
            || self
                .menu_syntax_mode
                .capture_composer_owns_input_for(filter_text)
            || self.menu_syntax_mode.command_owns_input_for(filter_text)
        {
            return Vec::new();
        }

        // P1: Return cached results if filter hasn't changed
        if self
            .main_menu_result_caches
            .has_filtered_results_for(filter_text)
        {
            logging::log_debug("CACHE", &format!("Filter cache HIT for '{}'", filter_text));
            return self.main_menu_result_caches.clone_filtered_results();
        }

        // P1: Cache miss - need to recompute (will be done by get_filtered_results_mut)
        logging::log_debug(
            "CACHE",
            &format!(
                "Filter cache MISS - need recompute for '{}' (cached key: '{}')",
                filter_text,
                self.main_menu_result_caches.filtered_cache_key()
            ),
        );

        self.recompute_filtered_results(filter_text)
    }

    /// P1: Get filtered results with cache update (mutable version)
    /// Call this when you need to ensure cache is updated
    pub(crate) fn get_filtered_results_cached(&mut self) -> &Vec<scripts::SearchResult> {
        if self.menu_syntax_object_selector_state.owns_main_list()
            || self.menu_syntax_trigger_popup_state.owns_main_list()
            || self
                .menu_syntax_mode
                .capture_composer_owns_input_for(&self.filter_text)
            || self
                .menu_syntax_mode
                .command_owns_input_for(&self.filter_text)
        {
            self.main_menu_result_caches
                .store_filtered_results(self.filter_text.clone(), Vec::new());
            return self.main_menu_result_caches.filtered_results();
        }

        if !self
            .main_menu_result_caches
            .has_filtered_results_for(&self.filter_text)
        {
            let filter_text = self.filter_text.clone();
            if logging::filter_perf_trace_enabled() {
                logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[4a/5] SEARCH_START for '{}' (scripts={}, scriptlets={}, builtins={}, apps={}, skills={})",
                        filter_text,
                        self.scripts.len(),
                        self.scriptlets.len(),
                        self.builtin_entries.len(),
                        self.apps.len(),
                        self.skills.len(),
                    ),
                );
            }
            let search_start = std::time::Instant::now();
            let filtered_results = self.recompute_filtered_results(&filter_text);
            let filtered_result_count = filtered_results.len();
            self.main_menu_result_caches
                .store_filtered_results(filter_text.clone(), filtered_results);
            let search_elapsed = search_start.elapsed();

            if logging::filter_perf_trace_enabled()
                || search_elapsed >= std::time::Duration::from_millis(8)
            {
                logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[4a/5] SEARCH_DONE '{}' in {:.2}ms -> {} results (skills={})",
                        filter_text,
                        search_elapsed.as_secs_f64() * 1000.0,
                        filtered_result_count,
                        self.skills.len(),
                    ),
                );
            }
        }
        // NOTE: Removed cache HIT log - fires every render, only log MISS for diagnostics
        self.main_menu_result_caches.filtered_results()
    }

    /// P1: Invalidate filter cache (call when scripts/scriptlets change)
    #[allow(dead_code)]
    pub(crate) fn invalidate_filter_cache(&mut self) {
        logging::log_debug("CACHE", "Filter cache INVALIDATED");
        self.main_menu_result_caches.invalidate_filtered_results();
        self.main_menu_render_diagnostics.last_input_highlight_text.clear();
        self.main_menu_render_diagnostics.last_input_highlight_ranges.clear();
    }

    fn active_script_list_attachment_portal_kind(
        &self,
    ) -> Option<crate::ai::window::context_picker::types::PortalKind> {
        use crate::ai::window::context_picker::types::PortalKind;

        if !matches!(self.current_view, AppView::ScriptList) {
            return None;
        }

        match self.active_attachment_portal_kind {
            Some(
                kind @ (PortalKind::ScriptSearch
                | PortalKind::ScriptletSearch
                | PortalKind::SkillSearch),
            ) => Some(kind),
            _ => None,
        }
    }

    fn script_list_result_matches_attachment_portal(
        kind: crate::ai::window::context_picker::types::PortalKind,
        result: &scripts::SearchResult,
    ) -> bool {
        use crate::ai::window::context_picker::types::PortalKind;

        matches!(
            (kind, result),
            (PortalKind::ScriptSearch, scripts::SearchResult::Script(_))
                | (
                    PortalKind::ScriptletSearch,
                    scripts::SearchResult::Scriptlet(_)
                )
                | (PortalKind::SkillSearch, scripts::SearchResult::Skill(_))
        )
    }

    fn apply_script_list_attachment_portal_filter(
        &self,
        kind: crate::ai::window::context_picker::types::PortalKind,
        flat_results: Vec<scripts::SearchResult>,
    ) -> (Vec<GroupedListItem>, Vec<scripts::SearchResult>) {
        let filtered_results: Vec<scripts::SearchResult> = flat_results
            .into_iter()
            .filter(|result| Self::script_list_result_matches_attachment_portal(kind, result))
            .collect();
        let grouped_items: Vec<GroupedListItem> = filtered_results
            .iter()
            .enumerate()
            .map(|(index, _)| GroupedListItem::Item(index))
            .collect();

        (grouped_items, filtered_results)
    }

    /// P1: Get grouped results with caching - avoids recomputing 9+ times per keystroke
    ///
    /// This is the ONLY place that should call scripts::get_grouped_results().
    /// P3: Cache is keyed off computed_filter_text (not filter_text) for two-stage filtering.
    ///
    /// P1-Arc: Returns Arc clones for cheap sharing with render closures.
    pub(crate) fn get_grouped_results_cached(
        &mut self,
    ) -> (Arc<[GroupedListItem]>, Arc<[scripts::SearchResult]>) {
        // The grouped cache is keyed by `computed_filter_text`. Menu syntax is
        // an ownership boundary, so never return stale grouped rows while the
        // live input is owned by the trigger popup or capture composer.
        let live_filter_text = self.filter_text.as_str();
        let computed_filter_text = self.computed_filter_text.as_str();
        let live_menu_syntax_owns_main_list =
            self.menu_syntax_object_selector_state.owns_main_list()
                || self.menu_syntax_trigger_popup_state.owns_main_list()
                || self
                    .menu_syntax_mode
                    .capture_composer_owns_input_for(live_filter_text)
                || self
                    .menu_syntax_mode
                    .command_owns_input_for(live_filter_text);
        if live_menu_syntax_owns_main_list && live_filter_text != computed_filter_text {
            return (
                Arc::<[GroupedListItem]>::from(Vec::new()),
                Arc::<[scripts::SearchResult]>::from(Vec::new()),
            );
        }

        // ── Spine projection path ──────────────────────────────────────
        // When a sigil segment owns the list, build rows from the Spine
        // model instead of running normal fuzzy/root grouping.
        if self.spine_projection_owns_main_list()
            && self.spine_parse.input == live_filter_text
        {
            if let Some(projection) = self.spine_projection.as_ref() {
                let preview_needs = match &projection.active_segment_kind {
                    crate::spine::SpineSegmentKind::Style { .. } => {
                        Some(crate::spine::live_preview::SpinePreviewNeeds::STYLE)
                    }
                    crate::spine::SpineSegmentKind::ContextMention { sub_query, .. }
                        if sub_query.is_none() =>
                    {
                        Some(crate::spine::live_preview::SpinePreviewNeeds::CONTEXT_ROOT)
                    }
                    _ => None,
                };
                if let Some(needs) = preview_needs {
                    if needs.cheap_context {
                        self.spine_live_preview_cache
                            .set_script_count(self.scripts.len());
                    }
                    self.spine_live_preview_cache
                        .refresh_preview_nonblocking(needs);
                }

                let preview_generation = preview_needs
                    .map(|_| self.spine_live_preview_cache.generation)
                    .unwrap_or(0);
                let spine_cache_key = format!(
                    "{}\x1Fpreview-gen={preview_generation}",
                    crate::spine::spine_projection_cache_key(
                        live_filter_text,
                        computed_filter_text,
                        &self.spine_parse,
                        projection,
                    ),
                );
                if self
                    .main_menu_result_caches
                    .has_grouped_results_for(&spine_cache_key)
                {
                    return self.main_menu_result_caches.clone_grouped_results();
                }

                let subsearch_ctx =
                    crate::spine::catalog_subsearch::SpineSubsearchContext {
                        scripts: &self.scripts,
                        scriptlets: &self.scriptlets,
                        skills: &self.skills,
                    };

                let live_preview = preview_needs
                    .map(|_| &self.spine_live_preview_cache.current);

                let sections = crate::spine::list::build_spine_list_sections_full(
                    &self.spine_parse,
                    projection,
                    Some(&subsearch_ctx),
                    live_preview,
                );
                let mut grouped_items = Vec::new();
                let mut flat_results: Vec<scripts::SearchResult> = Vec::new();
                for section in sections {
                    grouped_items.push(GroupedListItem::SectionHeader(
                        section.title.to_string(),
                        section.icon.as_ref().map(|icon| icon.as_ref().to_string()),
                    ));
                    for row in section.rows {
                        if !row.is_selectable {
                            grouped_items.push(GroupedListItem::SectionHeader(
                                row.title.to_string(),
                                row.icon.as_ref().map(|icon| icon.as_ref().to_string()),
                            ));
                            continue;
                        }
                        let flat_index = flat_results.len();
                        flat_results
                            .push(scripts::SearchResult::SpineProjection(row));
                        grouped_items.push(GroupedListItem::Item(flat_index));
                    }
                }

                self.main_menu_result_caches.store_grouped_results(
                    spine_cache_key,
                    grouped_items,
                    flat_results,
                    None,
                    None,
                );
                return self.main_menu_result_caches.clone_grouped_results();
            }
        }

        #[cfg(target_os = "macos")]
        let tracked_frontmost_app = frontmost_app_tracker::get_last_real_app();
        #[cfg(target_os = "macos")]
        let current_app_commands_app_name = tracked_frontmost_app
            .as_ref()
            .map(|app| app.name.clone())
            .filter(|name| !name.trim().is_empty());
        #[cfg(not(target_os = "macos"))]
        let current_app_commands_app_name: Option<String> = None;

        let grouped_advanced_query = self
            .menu_syntax_mode
            .advanced_query_for(&self.computed_filter_text)
            .cloned();
        let grouped_source_filters = grouped_advanced_query
            .as_ref()
            .map(|query| query.source_filters.clone())
            .unwrap_or_default();
        let ai_vault_generation = crate::ai_vault::root_ai_vault_snapshot_status().generation;
        let browser_tabs_generation =
            crate::browser_tabs::root_browser_tabs_snapshot_status().generation;
        let browser_history_generation =
            crate::browser_history::root_browser_history_snapshot_status().generation;
        let root_windows_generation = self.root_windows_refresh_generation;
        let grouped_source_filter_key = format!("{grouped_source_filters:?}");
        let grouped_cache_key = match current_app_commands_app_name.as_deref() {
            Some(app_name) => format!(
                "{}\x1Fsource-filters={grouped_source_filter_key}\x1Fcurrent-app={app_name}\x1Fai-vault-gen={ai_vault_generation}\x1Fwindows-gen={root_windows_generation}\x1Fbrowser-tabs-gen={browser_tabs_generation}\x1Fbrowser-history-gen={browser_history_generation}",
                self.computed_filter_text
            ),
            None => format!(
                "{}\x1Fsource-filters={grouped_source_filter_key}\x1Fai-vault-gen={ai_vault_generation}\x1Fwindows-gen={root_windows_generation}\x1Fbrowser-tabs-gen={browser_tabs_generation}\x1Fbrowser-history-gen={browser_history_generation}",
                self.computed_filter_text
            ),
        };

        // P3: Key off computed_filter_text for two-stage filtering
        if self
            .main_menu_result_caches
            .has_grouped_results_for(&grouped_cache_key)
        {
            // NOTE: Removed cache HIT log - fires every render frame, causing log spam.
            // Cache hits are normal operation. Only log cache MISS (below) for diagnostics.
            return self.main_menu_result_caches.clone_grouped_results();
        }

        let should_refresh_root_recent_files = self.computed_filter_text.is_empty()
            || matches!(
                self.root_file_search_mode,
                Some(crate::file_search::RootFileSectionMode::GlobalQuery)
            )
            || self
                .menu_syntax_mode
                .advanced_query_for(&self.computed_filter_text)
                .is_some_and(|query| {
                    query.free_text.trim().is_empty()
                        && query
                            .source_filters
                            .includes(crate::menu_syntax::RootUnifiedSourceFilter::Files)
                });
        if should_refresh_root_recent_files {
            self.refresh_root_recent_file_results();
        }

        // Cache miss - need to recompute
        if logging::filter_perf_trace_enabled() {
            logging::log(
                "FILTER_PERF",
                &format!("[4b/5] GROUP_START for '{}'", self.computed_filter_text),
            );
        }

        let start = std::time::Instant::now();
        let suggested_config = self.config.get_suggested();

        // Get menu bar items from the background tracker (pre-fetched when apps activate)
        #[cfg(target_os = "macos")]
        let (menu_bar_items, menu_bar_bundle_id): (
            Vec<menu_bar::MenuBarItem>,
            Option<String>,
        ) = {
            let cached = frontmost_app_tracker::get_cached_menu_items();
            let bundle_id = tracked_frontmost_app
                .as_ref()
                .map(|app| app.bundle_id.clone());
            // No conversion needed - tracker is compiled as part of binary crate
            // so it already returns binary crate types
            (cached, bundle_id)
        };
        #[cfg(not(target_os = "macos"))]
        let (menu_bar_items, menu_bar_bundle_id): (
            Vec<menu_bar::MenuBarItem>,
            Option<String>,
        ) = (Vec::new(), None);

        if logging::filter_perf_trace_enabled() {
            logging::log(
                "APP",
                &format!(
                    "get_grouped_results: filter='{}', menu_bar_items={}, bundle_id={:?}",
                    self.computed_filter_text,
                    menu_bar_items.len(),
                    menu_bar_bundle_id
                ),
            );
        }
        let raw_filter_text = self.computed_filter_text.clone();
        let menu_syntax_owns_main_list = self.menu_syntax_object_selector_state.owns_main_list()
            || self.menu_syntax_trigger_popup_state.owns_main_list()
            || self
                .menu_syntax_mode
                .capture_composer_owns_input_for(&raw_filter_text)
            || self
                .menu_syntax_mode
                .command_owns_input_for(&raw_filter_text);

        let (grouped_items, flat_results) = if menu_syntax_owns_main_list {
            // A composer-style menu-syntax surface owns the input. Suppress
            // the main launcher list so fuzzy search, capture-handler rows,
            // and the "Use X with…" fallback section do not leak through the
            // transparent popup (e.g. typing `;todo` should not also render
            // capture-mode rows behind the picker). Refine (`:`) remains
            // structured launcher search and is handled below.
            (Vec::new(), Vec::new())
        } else if let Some(invocation) = self.menu_syntax_mode.capture_for(&raw_filter_text) {
            // Capture mode replaces the normal launcher grouping entirely.
            // Do not mix with Suggested/Favorites/Recent/menu-bar/fallback.
            crate::scripts::build_capture_mode_results(&self.scripts, invocation)
        } else if let Some(hint) = self.menu_syntax_mode.incomplete_hint_for(&raw_filter_text) {
            // Menu-syntax trigger picker rows are now owned by the detached
            // popup window at
            // `crate::menu_syntax_trigger_popup_window::MenuSyntaxTriggerPopupWindow`
            // (Oracle iter 015 D2b). The main launcher list shows only the
            // terse hint line so the two surfaces do not collide — the
            // rich qualifier / capture target rows live in the popup.
            crate::scripts::build_menu_syntax_hint_results(hint)
        } else {
            let search_text_owned =
                crate::menu_syntax::free_text_for_search(&self.menu_syntax_mode, &raw_filter_text)
                    .to_string();
            let search_text = search_text_owned.as_str();
            let advanced_query_owned = self
                .menu_syntax_mode
                .advanced_query_for(&raw_filter_text)
                .cloned();
            let source_filters = advanced_query_owned
                .as_ref()
                .map(|query| query.source_filters.clone())
                .unwrap_or_default();
            let advanced_query = advanced_query_owned.as_ref();
            let advanced_predicate_query = advanced_query.filter(|query| query.has_predicates());
            let advanced_predicate_active = advanced_predicate_query.is_some();
            let unified_search = self.config.get_unified_search();
            let mut root_file_options = unified_search.root_file_section_options();
            let mut todo_options = unified_search.todo_section_options();
            let mut notes_options = unified_search.notes_section_options();
            let mut acp_history_options = unified_search.acp_history_section_options();
            let mut ai_vault_options = unified_search.ai_vault_section_options();
            let mut clipboard_history_options =
                self.config.root_clipboard_history_section_options();
            let mut dictation_history_options = unified_search.dictation_history_section_options();
            let mut browser_tabs_options = unified_search.browser_tabs_section_options();
            let mut browser_history_options = unified_search.browser_history_section_options();
            let root_passive_source_order = unified_search.passive_source_order();
            let root_passive_result_limits = unified_search.passive_result_limits();
            let explicit_source_result_target = root_passive_result_limits.max_total_results;
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Files) {
                root_file_options.files_enabled = true;
                root_file_options.global_search_enabled = true;
                root_file_options.directory_browse_enabled = true;
                root_file_options.recent_files_enabled = true;
                root_file_options.query_intent =
                    crate::file_search::RootFileQueryIntent::ExplicitFilesSourceFilter;
                let visible_limit = self.root_file_source_chip_visible_limit_for(
                    &raw_filter_text,
                    search_text,
                    advanced_predicate_active,
                    self.root_file_search_mode,
                );
                root_file_options.source_chip_visible_limit = Some(visible_limit);
                if search_text.trim().is_empty() && !advanced_predicate_active {
                    root_file_options.source_filter_browse_target_visible_rows =
                        Some(visible_limit);
                }
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Notes) {
                notes_options.enabled = true;
                notes_options.min_query_chars = 0;
                notes_options.max_results =
                    notes_options.max_results.max(explicit_source_result_target);
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Todo) {
                todo_options.enabled = true;
                todo_options.min_query_chars = 0;
                todo_options.max_results =
                    todo_options.max_results.max(explicit_source_result_target);
            }
            if source_filters
                .includes(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory)
            {
                clipboard_history_options.enabled = true;
                clipboard_history_options.min_query_chars = 0;
                clipboard_history_options.max_results = clipboard_history_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Dictation) {
                dictation_history_options.enabled = true;
                dictation_history_options.min_query_chars = 0;
                dictation_history_options.max_results = dictation_history_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Conversations) {
                acp_history_options.enabled = true;
                acp_history_options.min_query_chars = 0;
                acp_history_options.max_results = acp_history_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::AiVault) {
                ai_vault_options.enabled = true;
                ai_vault_options.min_query_chars = 0;
                ai_vault_options.max_results = ai_vault_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs) {
                browser_tabs_options.enabled = true;
                browser_tabs_options.min_query_chars = 0;
                browser_tabs_options.max_results = browser_tabs_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory)
            {
                browser_history_options.enabled = true;
                browser_history_options.min_query_chars = 0;
                browser_history_options.max_age_days = 365;
                browser_history_options.max_results = browser_history_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            let root_passive_frame = self.root_passive_frame_for_current_query(
                search_text,
                advanced_predicate_active,
                source_filters.clone(),
                todo_options,
                notes_options,
                clipboard_history_options,
                dictation_history_options,
                acp_history_options,
                ai_vault_options.clone(),
                browser_tabs_options.clone(),
                browser_history_options.clone(),
            );
            let root_file_frame = (matches!(
                self.root_file_search_mode,
                Some(crate::file_search::RootFileSectionMode::GlobalQuery)
            ) && source_filters
                .allows(crate::menu_syntax::RootUnifiedSourceFilter::Files))
            .then(|| {
                self.root_file_frame_for_current_query(
                    search_text,
                    advanced_predicate_active,
                    source_filters.clone(),
                    root_file_options,
                )
            });
            let root_file_search_mode_for_grouping = root_file_frame
                .as_ref()
                .map(|frame| frame.mode)
                .unwrap_or(self.root_file_search_mode);
            let root_file_search_loading_for_grouping = root_file_frame
                .as_ref()
                .map(|frame| frame.visible_loading)
                .unwrap_or(self.root_file_search_loading);
            let root_file_results_for_grouping = root_file_frame
                .as_ref()
                .map(|frame| frame.file_results.as_slice())
                .unwrap_or(self.root_file_results.as_slice());
            let root_recent_file_results_for_grouping = root_file_frame
                .as_ref()
                .map(|frame| frame.recent_file_results.as_slice())
                .unwrap_or(self.root_recent_file_results.as_slice());
            let dynamic_builtin_entries =
                current_app_commands_app_name.as_deref().map(|app_name| {
                    let mut entries = self.builtin_entries.clone();
                    let commands_label =
                        crate::menu_bar::current_app_commands::current_app_commands_launcher_label(
                            Some(app_name),
                        );
                    if let Some(entry) = entries
                        .iter_mut()
                        .find(|entry| entry.id == "builtin/do-in-current-app")
                    {
                        entry.name = commands_label;
                    }
                    if let Some(entry) = entries
                        .iter_mut()
                        .find(|entry| entry.id == "builtin/dictation")
                    {
                        entry.name = format!("Dictate to {app_name}");
                        entry.description = format!("Voice dictation for {app_name}");
                    }
                    entries
                });
            let builtins_for_grouping = dynamic_builtin_entries
                .as_deref()
                .unwrap_or(&self.builtin_entries);
            crate::scripts::get_grouped_results_with_validation_query_and_root_files_with_options(
                &self.scripts,
                &self.scriptlets,
                builtins_for_grouping,
                &self.apps,
                &self.cached_root_windows,
                self.root_windows_provider_status.clone(),
                &self.skills,
                &self.frecency_store,
                search_text,
                &suggested_config,
                &menu_bar_items,
                menu_bar_bundle_id.as_deref(),
                Some(&self.input_history),
                self.script_validation_report.as_deref(),
                advanced_predicate_query,
                &source_filters,
                root_file_search_mode_for_grouping,
                root_file_search_loading_for_grouping,
                root_file_results_for_grouping,
                root_recent_file_results_for_grouping,
                root_file_options,
                &root_passive_frame.todo_hits,
                todo_options,
                &root_passive_frame.note_hits,
                notes_options,
                &root_passive_frame.clipboard_history_hits,
                clipboard_history_options,
                &root_passive_frame.dictation_history_hits,
                dictation_history_options,
                &root_passive_frame.acp_history_hits,
                acp_history_options,
                &root_passive_frame.ai_vault_hits,
                ai_vault_options,
                &root_passive_frame.browser_tab_hits,
                browser_tabs_options,
                &root_passive_frame.browser_history_hits,
                browser_history_options,
                &root_passive_source_order,
                root_passive_result_limits,
            )
        };
        let (grouped_items, flat_results) = if menu_syntax_owns_main_list {
            (grouped_items, flat_results)
        } else {
            prepend_inline_calculator_group(
                grouped_items,
                flat_results,
                self.inline_calculator.as_ref(),
            )
        };
        let (grouped_items, flat_results) =
            if let Some(kind) = self.active_script_list_attachment_portal_kind() {
                self.apply_script_list_attachment_portal_filter(kind, flat_results)
            } else {
                (grouped_items, flat_results)
            };
        let elapsed = start.elapsed();

        let mut first_selectable_index = None;
        let mut last_selectable_index = None;
        for (index, grouped_item) in grouped_items.iter().enumerate() {
            if matches!(grouped_item, GroupedListItem::Item(_)) {
                if first_selectable_index.is_none() {
                    first_selectable_index = Some(index);
                }
                last_selectable_index = Some(index);
            }
        }

        self.main_menu_result_caches.store_grouped_results(
            grouped_cache_key,
            grouped_items,
            flat_results,
            first_selectable_index,
            last_selectable_index,
        );

        self.refresh_ghost_from_cached_results();

        if logging::filter_perf_trace_enabled() || elapsed >= std::time::Duration::from_millis(8) {
            logging::log(
                "FILTER_PERF",
                &format!(
                    "[4b/5] GROUP_DONE '{}' in {:.2}ms -> {} items (from {} results)",
                    self.computed_filter_text,
                    elapsed.as_secs_f64() * 1000.0,
                    self.main_menu_result_caches.grouped_items().len(),
                    self.main_menu_result_caches.grouped_flat_result_count()
                ),
            );
        }

        // Log total time from input to grouped results if we have the start time
        if let Some(perf_start) = self.main_menu_render_diagnostics.filter_perf_start {
            let total_elapsed = perf_start.elapsed();
            if logging::filter_perf_trace_enabled()
                || total_elapsed >= std::time::Duration::from_millis(16)
            {
                logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[5/5] TOTAL_TIME '{}': {:.2}ms (input->grouped)",
                        self.computed_filter_text,
                        total_elapsed.as_secs_f64() * 1000.0
                    ),
                );
            }
        }

        self.main_menu_result_caches.clone_grouped_results()
    }

    pub(crate) fn cached_grouped_results_snapshot(
        &self,
    ) -> (Arc<[GroupedListItem]>, Arc<[scripts::SearchResult]>) {
        self.main_menu_result_caches.clone_grouped_results()
    }

    pub(crate) fn cached_source_statuses_snapshot(
        &self,
    ) -> Arc<[crate::list_item::SourceChipStatusRow]> {
        self.main_menu_result_caches
            .grouped_source_statuses()
            .to_vec()
            .into()
    }

    /// P1: Invalidate grouped results cache (call when scripts/scriptlets/apps change)
    pub(crate) fn invalidate_grouped_cache(&mut self) {
        logging::log_debug("CACHE", "Grouped cache INVALIDATED");
        // Set grouped_cache_key to a sentinel that won't match computed_filter_text.
        // This ensures the cache check (computed_filter_text == grouped_cache_key) fails,
        // forcing a recompute on the next get_grouped_results_cached() call.
        // DO NOT set computed_filter_text here - that would cause both to match (false cache HIT).
        self.main_menu_result_caches.invalidate_grouped_results();
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
    pub(crate) fn get_selected_result(&mut self) -> Option<scripts::SearchResult> {
        let selected_index = self.selected_index;
        self.get_grouped_results_cached();

        let result_idx = self
            .main_menu_result_caches
            .flat_result_index_for_grouped_item(selected_index)?;
        if self
            .inline_calculator_for_result_index(result_idx)
            .is_some()
        {
            None
        } else {
            self.main_menu_result_caches
                .cloned_search_result_for_flat_index(result_idx)
        }
    }

    pub(crate) fn inline_calculator_for_result_index(
        &self,
        result_idx: usize,
    ) -> Option<&crate::calculator::CalculatorInlineResult> {
        if result_idx == INLINE_CALCULATOR_RESULT_INDEX {
            self.inline_calculator.as_ref()
        } else {
            None
        }
    }

    /// Get or update the preview cache for syntax-highlighted code lines.
    /// Only re-reads and re-highlights when the script path actually changes.
    /// Returns cached lines if path matches, otherwise updates cache and returns new lines.
    pub(crate) fn get_or_update_preview_cache(
        &mut self,
        script_path: &str,
        lang: &str,
        is_dark: bool,
    ) -> &[syntax::HighlightedLine] {
        self.get_or_update_preview_cache_with_match(script_path, lang, is_dark, None)
    }

    /// Get or update the preview cache with optional content-match centering.
    /// When `content_match` is provided, the 15-line window is centered on the matched line
    /// and the matched span is emphasized with gold accent at ghost opacity.
    pub(crate) fn get_or_update_preview_cache_with_match(
        &mut self,
        script_path: &str,
        lang: &str,
        is_dark: bool,
        content_match: Option<&scripts::ScriptContentMatch>,
    ) -> &[syntax::HighlightedLine] {
        let match_signature = scripts::preview_match_signature(content_match);
        let matched_line = content_match.map(|cm| cm.line_number);

        let cached_path_matches = self.preview_cache_path.as_deref() == Some(script_path);
        let cached_signature_matches = self.preview_cache_match_signature == match_signature;
        let cache_has_lines = !self.preview_cache_lines.is_empty();

        // Check if cache is valid for this path and match signature
        if scripts::preview_cache_is_valid(
            self.preview_cache_path.as_deref(),
            self.preview_cache_match_signature,
            self.preview_cache_lines.is_empty(),
            script_path,
            content_match,
        ) {
            return &self.preview_cache_lines;
        }

        let miss_reason = if !cached_path_matches {
            "path_changed"
        } else if !cached_signature_matches {
            "match_signature_changed"
        } else if !cache_has_lines {
            "empty_cache"
        } else {
            "unknown"
        };

        // Cache miss - need to re-read and re-highlight
        let cache_miss_start = std::time::Instant::now();
        logging::log(
            "FILTER_PERF",
            &format!(
                "[PREVIEW_CACHE_MISS_REASON] path='{}' reason={} cached_path={:?} cached_match_signature={:?} requested_match_signature={:?}",
                script_path,
                miss_reason,
                self.preview_cache_path,
                self.preview_cache_match_signature,
                match_signature
            ),
        );
        logging::log(
            "FILTER_PERF",
            &format!(
                "[PREVIEW_CACHE_KEY] path='{}' match_signature={:?}",
                script_path, match_signature
            ),
        );
        logging::log(
            "FILTER_PERF",
            &format!(
                "[PREVIEW_CACHE_MISS] Loading '{}' matched_line={:?}",
                script_path, matched_line
            ),
        );

        self.preview_cache_path = Some(script_path.to_string());
        self.preview_cache_match_signature = match_signature;

        let read_start = std::time::Instant::now();
        self.preview_cache_lines = match std::fs::read_to_string(script_path) {
            Ok(content) => {
                let read_elapsed = read_start.elapsed();
                let all_lines: Vec<&str> = content.lines().collect();
                let total_lines = all_lines.len();

                // Compute the 15-line window: centered on match or starting from line 1
                let (window_start, window_lines) = if let Some(match_ln) = matched_line {
                    // match_ln is 1-based; center it in a 15-line window
                    let zero_idx = match_ln.saturating_sub(1);
                    let start = zero_idx.saturating_sub(7);
                    let end = (start + 15).min(total_lines);
                    let start = if end < 15 { 0 } else { end - 15 };
                    (start, &all_lines[start..end])
                } else {
                    let end = total_lines.min(15);
                    (0, &all_lines[..end])
                };

                let highlight_start = std::time::Instant::now();
                let preview: String = window_lines.join("\n");
                let mut lines = syntax::highlight_code_lines(&preview, lang, is_dark);
                let highlight_elapsed = highlight_start.elapsed();

                // Apply match emphasis to the matched line's spans
                if let Some(cm) = content_match {
                    let match_line_zero = cm.line_number.saturating_sub(1);
                    if match_line_zero >= window_start {
                        let line_idx_in_window = match_line_zero - window_start;
                        if line_idx_in_window < lines.len() {
                            let raw_line = window_lines[line_idx_in_window];
                            let leading_ws_chars =
                                raw_line.chars().take_while(|ch| ch.is_whitespace()).count();
                            // `line_match_indices` are relative to the trimmed snippet shown in
                            // the list row. Convert them back into offsets within the full preview
                            // line so indented matches highlight the correct span.
                            if let (Some(&first), Some(&last)) =
                                (cm.line_match_indices.first(), cm.line_match_indices.last())
                            {
                                Self::apply_match_emphasis_to_line(
                                    &mut lines[line_idx_in_window],
                                    leading_ws_chars + first,
                                    leading_ws_chars + last + 1,
                                );
                            }
                        }
                    }
                }

                logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[PREVIEW_CACHE_MISS] read={:.2}ms highlight={:.2}ms ({} bytes, {} lines, window_start={})",
                        read_elapsed.as_secs_f64() * 1000.0,
                        highlight_elapsed.as_secs_f64() * 1000.0,
                        content.len(),
                        lines.len(),
                        window_start
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

    /// Apply match emphasis to a specific character range within a highlighted line.
    /// Splits spans as needed so that only the matched range gets `is_match_emphasis = true`.
    fn apply_match_emphasis_to_line(
        line: &mut syntax::HighlightedLine,
        match_start: usize,
        match_end: usize,
    ) {
        if match_start >= match_end {
            return;
        }
        let mut new_spans = Vec::new();
        let mut char_offset: usize = 0;

        for span in line.spans.drain(..) {
            let span_len = span.text.chars().count();
            let span_end = char_offset + span_len;

            if span_end <= match_start || char_offset >= match_end {
                // Entirely outside the match range — keep as-is
                new_spans.push(span);
            } else {
                // This span overlaps with the match range — split it
                let overlap_start = match_start.saturating_sub(char_offset);
                let overlap_end = (match_end - char_offset).min(span_len);

                let chars: Vec<char> = span.text.chars().collect();

                // Before-match portion
                if overlap_start > 0 {
                    let before: String = chars[..overlap_start].iter().collect();
                    new_spans.push(syntax::HighlightedSpan::new(before, span.color));
                }

                // Matched portion — with emphasis
                let matched: String = chars[overlap_start..overlap_end].iter().collect();
                new_spans.push(syntax::HighlightedSpan::with_match_emphasis(
                    matched, span.color,
                ));

                // After-match portion
                if overlap_end < span_len {
                    let after: String = chars[overlap_end..].iter().collect();
                    new_spans.push(syntax::HighlightedSpan::new(after, span.color));
                }
            }

            char_offset = span_end;
        }

        line.spans = new_spans;
    }

    /// Invalidate the preview cache (call when scripts are reloaded or selection changes)
    pub(crate) fn invalidate_preview_cache(&mut self) {
        self.preview_cache_path = None;
        self.preview_cache_match_signature = None;
        self.preview_cache_lines.clear();
    }

    fn refresh_ghost_from_cached_results(&mut self) {
        if !matches!(self.current_view, AppView::ScriptList) {
            self.ghost_prediction = None;
            return;
        }
        if self.show_actions_popup {
            self.ghost_prediction = None;
            return;
        }
        if self.menu_syntax_owns_list() || self.menu_syntax_capture_form_owns_input() {
            self.ghost_prediction = None;
            return;
        }
        if self.inline_calculator.is_some() {
            self.ghost_prediction = None;
            return;
        }

        let query = &self.computed_filter_text;
        let (_, flat_results) = self.main_menu_result_caches.clone_grouped_results();
        self.ghost_prediction =
            crate::scripts::search::ghost::compute_ghost_prediction(query, &flat_results);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calculator_result() -> crate::calculator::CalculatorInlineResult {
        crate::calculator::CalculatorInlineResult {
            raw_input: "12 / 3".to_string(),
            normalized_expr: "12 / 3".to_string(),
            operation_name: "Divide".to_string(),
            value: 4.0,
            formatted: "4".to_string(),
            words: "Four".to_string(),
        }
    }

    #[test]
    fn test_prepend_inline_calculator_group_prepends_header_and_item() {
        let grouped_items = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
        ];
        let flat_results = Vec::new();

        let (grouped, flat) = prepend_inline_calculator_group(
            grouped_items,
            flat_results,
            Some(&calculator_result()),
        );

        assert!(matches!(
            grouped.first(),
            Some(GroupedListItem::SectionHeader(label, None))
            if label == INLINE_CALCULATOR_SECTION_LABEL
        ));
        assert!(matches!(
            grouped.get(1),
            Some(GroupedListItem::Item(INLINE_CALCULATOR_RESULT_INDEX))
        ));
        assert!(matches!(
            grouped.get(2),
            Some(GroupedListItem::SectionHeader(_, _))
        ));
        assert!(matches!(grouped.get(3), Some(GroupedListItem::Item(0))));
        assert!(matches!(grouped.get(4), Some(GroupedListItem::Item(1))));
        assert!(flat.is_empty());
    }

    #[test]
    fn test_prepend_inline_calculator_group_is_noop_without_calculator() {
        let grouped_items = vec![GroupedListItem::Item(0)];
        let flat_results = Vec::new();

        let (grouped, flat) = prepend_inline_calculator_group(grouped_items, flat_results, None);

        assert_eq!(grouped.len(), 1);
        assert!(matches!(grouped.first(), Some(GroupedListItem::Item(0))));
        assert!(flat.is_empty());
    }

    #[test]
    fn test_apply_match_emphasis_handles_indented_match_offsets() {
        let mut line = syntax::HighlightedLine {
            spans: vec![syntax::HighlightedSpan::new(
                "    const superUniqueToken = value;",
                0xcccccc,
            )],
        };

        let leading_ws_chars = 4;
        let snippet_match_start = 6;
        let snippet_match_end = 22;
        ScriptListApp::apply_match_emphasis_to_line(
            &mut line,
            leading_ws_chars + snippet_match_start,
            leading_ws_chars + snippet_match_end,
        );

        let emphasized: String = line
            .spans
            .iter()
            .filter(|span| span.is_match_emphasis)
            .map(|span| span.text.as_str())
            .collect();
        assert_eq!(emphasized, "superUniqueToken");
    }

    #[test]
    fn preview_match_signature_changes_when_byte_range_changes() {
        let alpha = scripts::ScriptContentMatch {
            line_number: 4,
            line_text: "const alpha = beta;".to_string(),
            line_match_indices: vec![6, 7, 8, 9, 10],
            byte_range: 20..25,
        };
        let beta = scripts::ScriptContentMatch {
            line_number: 4,
            line_text: "const alpha = beta;".to_string(),
            line_match_indices: vec![14, 15, 16, 17],
            byte_range: 28..32,
        };
        assert_ne!(
            scripts::preview_match_signature(Some(&alpha)),
            scripts::preview_match_signature(Some(&beta))
        );
    }

    #[test]
    fn preview_match_signature_is_none_without_content_match() {
        assert_eq!(scripts::preview_match_signature(None), None);
    }

    #[test]
    fn preview_cache_is_valid_for_identical_match_signature() {
        let alpha = scripts::ScriptContentMatch {
            line_number: 1,
            line_text: "const alpha = beta;".to_string(),
            line_match_indices: vec![6, 7, 8, 9, 10],
            byte_range: 6..11,
        };
        assert!(scripts::preview_cache_is_valid(
            Some("/tmp/demo.ts"),
            scripts::preview_match_signature(Some(&alpha)),
            false, // cached_lines_empty
            "/tmp/demo.ts",
            Some(&alpha),
        ));
    }

    #[test]
    fn preview_cache_is_invalid_when_same_line_match_moves_to_new_span() {
        let alpha = scripts::ScriptContentMatch {
            line_number: 1,
            line_text: "const alpha = beta;".to_string(),
            line_match_indices: vec![6, 7, 8, 9, 10],
            byte_range: 6..11,
        };
        let beta = scripts::ScriptContentMatch {
            line_number: 1,
            line_text: "const alpha = beta;".to_string(),
            line_match_indices: vec![14, 15, 16, 17],
            byte_range: 14..18,
        };
        assert!(!scripts::preview_cache_is_valid(
            Some("/tmp/demo.ts"),
            scripts::preview_match_signature(Some(&alpha)),
            false, // cached_lines_empty
            "/tmp/demo.ts",
            Some(&beta),
        ));
    }

    #[test]
    fn preview_cache_is_invalid_when_cached_lines_are_empty() {
        let alpha = scripts::ScriptContentMatch {
            line_number: 1,
            line_text: "const alpha = beta;".to_string(),
            line_match_indices: vec![6, 7, 8, 9, 10],
            byte_range: 6..11,
        };
        assert!(!scripts::preview_cache_is_valid(
            Some("/tmp/demo.ts"),
            scripts::preview_match_signature(Some(&alpha)),
            true, // cached_lines_empty
            "/tmp/demo.ts",
            Some(&alpha),
        ));
    }
}

