use super::*;

const INLINE_CALCULATOR_SECTION_LABEL: &str = "Calculator";
const INLINE_CALCULATOR_RESULT_INDEX: usize = usize::MAX;

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

impl ScriptListApp {
    pub(crate) fn filter_text(&self) -> &str {
        self.filter_text.as_str()
    }

    fn root_passive_frame_for_current_query(
        &mut self,
        search_text: &str,
        advanced_query_active: bool,
        source_filters: crate::menu_syntax::RootUnifiedSourceFilterSet,
        notes_options: crate::notes::RootNotesSectionOptions,
        clipboard_history_options: crate::clipboard_history::RootClipboardHistorySectionOptions,
        dictation_history_options: crate::dictation::RootDictationHistorySectionOptions,
        acp_history_options: crate::ai::acp::history::RootAcpHistorySectionOptions,
        browser_tabs_options: crate::browser_tabs::RootBrowserTabsSectionOptions,
        browser_history_options: crate::browser_history::RootBrowserHistorySectionOptions,
    ) -> crate::RootPassiveFrame {
        let key = crate::RootPassiveFrameKey {
            query: search_text.to_string(),
            advanced_query: advanced_query_active,
            source_filters: source_filters.clone(),
            notes_options,
            clipboard_history_options,
            dictation_history_options,
            acp_history_options,
            browser_tabs_options: browser_tabs_options.clone(),
            browser_history_options: browser_history_options.clone(),
        };

        if let Some(frame) = self.root_passive_frame.as_ref() {
            if frame.key == key {
                return frame.clone();
            }
        }

        let explicit_notes =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Notes);
        let explicit_clipboard =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory);
        let explicit_dictation =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Dictation);
        let explicit_conversations =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Conversations);
        let explicit_browser_tabs =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs);
        let explicit_browser_history =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory);

        let allow_notes = source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Notes);
        let allow_clipboard =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory);
        let allow_dictation =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Dictation);
        let allow_conversations =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Conversations);
        let allow_browser_tabs =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs);
        let allow_browser_history =
            source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory);

        let note_hits = if !advanced_query_active
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
        };

        let clipboard_history_hits = if !advanced_query_active
            && allow_clipboard
            && crate::clipboard_history::root_clipboard_history_query_is_eligible(
                search_text,
                clipboard_history_options,
            ) {
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
        };

        let dictation_history_hits = if !advanced_query_active
            && allow_dictation
            && crate::dictation::root_dictation_history_query_is_eligible(
                search_text,
                dictation_history_options,
            ) {
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
        };

        let acp_history_hits = if !advanced_query_active
            && allow_conversations
            && crate::ai::acp::history::root_acp_history_query_is_eligible(
                search_text,
                acp_history_options,
            ) {
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
        };

        let browser_tab_hits = if !advanced_query_active
            && allow_browser_tabs
            && crate::browser_tabs::root_browser_tabs_query_is_eligible(
                search_text,
                browser_tabs_options.clone(),
            ) {
            if explicit_browser_tabs {
                crate::browser_tabs::search_root_browser_tabs_meta_direct(
                    search_text,
                    browser_tabs_options.clone(),
                )
            } else {
                crate::browser_tabs::search_root_browser_tabs_meta(
                    search_text,
                    browser_tabs_options.clone(),
                )
            }
        } else {
            Vec::new()
        };
        let browser_tabs_status = crate::browser_tabs::root_browser_tabs_snapshot_status();

        let browser_history_hits = if !advanced_query_active
            && allow_browser_history
            && crate::browser_history::root_browser_history_query_is_eligible(
                search_text,
                browser_history_options.clone(),
            ) {
            if explicit_browser_history {
                crate::browser_history::search_root_browser_history_meta_direct(
                    search_text,
                    browser_history_options.clone(),
                )
            } else {
                crate::browser_history::search_root_browser_history_meta(
                    search_text,
                    browser_history_options.clone(),
                )
            }
        } else {
            Vec::new()
        };
        let browser_history_status = crate::browser_history::root_browser_history_snapshot_status();

        let frame = crate::RootPassiveFrame {
            key,
            note_hits,
            clipboard_history_hits,
            dictation_history_hits,
            acp_history_hits,
            browser_tab_hits,
            browser_history_hits,
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
        if self.menu_syntax_trigger_popup_state.owns_main_list()
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
        if self.menu_syntax_trigger_popup_state.owns_main_list()
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
            let search_start = std::time::Instant::now();
            let filtered_results = self.recompute_filtered_results(&filter_text);
            let filtered_result_count = filtered_results.len();
            self.main_menu_result_caches
                .store_filtered_results(filter_text.clone(), filtered_results);
            let search_elapsed = search_start.elapsed();

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
        // NOTE: Removed cache HIT log - fires every render, only log MISS for diagnostics
        self.main_menu_result_caches.filtered_results()
    }

    /// P1: Invalidate filter cache (call when scripts/scriptlets change)
    #[allow(dead_code)]
    pub(crate) fn invalidate_filter_cache(&mut self) {
        logging::log_debug("CACHE", "Filter cache INVALIDATED");
        self.main_menu_result_caches.invalidate_filtered_results();
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
        let live_menu_syntax_owns_main_list = self.menu_syntax_trigger_popup_state.owns_main_list()
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

        #[cfg(target_os = "macos")]
        let tracked_frontmost_app = frontmost_app_tracker::get_last_real_app();
        #[cfg(target_os = "macos")]
        let current_app_commands_app_name = tracked_frontmost_app
            .as_ref()
            .map(|app| app.name.clone())
            .filter(|name| !name.trim().is_empty());
        #[cfg(not(target_os = "macos"))]
        let current_app_commands_app_name: Option<String> = None;

        let grouped_cache_key = match current_app_commands_app_name.as_deref() {
            Some(app_name) => format!("{}\x1Fcurrent-app={app_name}", self.computed_filter_text),
            None => self.computed_filter_text.clone(),
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

        logging::log(
            "APP",
            &format!(
                "get_grouped_results: filter='{}', menu_bar_items={}, bundle_id={:?}",
                self.computed_filter_text,
                menu_bar_items.len(),
                menu_bar_bundle_id
            ),
        );
        let raw_filter_text = self.computed_filter_text.clone();
        let menu_syntax_owns_main_list = self.menu_syntax_trigger_popup_state.owns_main_list()
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
            let mut notes_options = unified_search.notes_section_options();
            let mut acp_history_options = unified_search.acp_history_section_options();
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
                browser_history_options.max_results = browser_history_options
                    .max_results
                    .max(explicit_source_result_target);
            }
            let root_passive_frame = self.root_passive_frame_for_current_query(
                search_text,
                advanced_predicate_active,
                source_filters.clone(),
                notes_options,
                clipboard_history_options,
                dictation_history_options,
                acp_history_options,
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
                    let label =
                        crate::menu_bar::current_app_commands::current_app_commands_launcher_label(
                            Some(app_name),
                        );
                    if let Some(entry) = entries
                        .iter_mut()
                        .find(|entry| entry.id == "builtin/do-in-current-app")
                    {
                        entry.name = label;
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
                &self.cached_windows,
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
                &root_passive_frame.note_hits,
                notes_options,
                &root_passive_frame.clipboard_history_hits,
                clipboard_history_options,
                &root_passive_frame.dictation_history_hits,
                dictation_history_options,
                &root_passive_frame.acp_history_hits,
                acp_history_options,
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

        // Log total time from input to grouped results if we have the start time
        if let Some(perf_start) = self.main_menu_render_diagnostics.filter_perf_start {
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
