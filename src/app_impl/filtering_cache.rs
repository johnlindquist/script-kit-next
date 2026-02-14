use super::*;

impl ScriptListApp {
    pub(crate) fn filter_text(&self) -> &str {
        self.filter_text.as_str()
    }

    /// P1: Now uses caching - invalidates only when filter_text changes
    pub(crate) fn filtered_results(&self) -> Vec<scripts::SearchResult> {
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
    pub(crate) fn get_filtered_results_cached(&mut self) -> &Vec<scripts::SearchResult> {
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
    pub(crate) fn invalidate_filter_cache(&mut self) {
        logging::log_debug("CACHE", "Filter cache INVALIDATED");
        self.filter_cache_key = String::from("\0_INVALIDATED_\0");
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

        // P1-Arc: Convert to Arc<[T]> for cheap clone
        self.cached_grouped_first_selectable_index = first_selectable_index;
        self.cached_grouped_last_selectable_index = last_selectable_index;
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
    pub(crate) fn invalidate_grouped_cache(&mut self) {
        logging::log_debug("CACHE", "Grouped cache INVALIDATED");
        // Set grouped_cache_key to a sentinel that won't match computed_filter_text.
        // This ensures the cache check (computed_filter_text == grouped_cache_key) fails,
        // forcing a recompute on the next get_grouped_results_cached() call.
        // DO NOT set computed_filter_text here - that would cause both to match (false cache HIT).
        self.cached_grouped_first_selectable_index = None;
        self.cached_grouped_last_selectable_index = None;
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
    pub(crate) fn get_selected_result(&mut self) -> Option<scripts::SearchResult> {
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
    pub(crate) fn get_or_update_preview_cache(
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
    pub(crate) fn invalidate_preview_cache(&mut self) {
        self.preview_cache_path = None;
        self.preview_cache_lines.clear();
    }

}
