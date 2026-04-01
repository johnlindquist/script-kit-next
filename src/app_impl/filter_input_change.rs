use super::*;

impl ScriptListApp {
    pub(crate) fn handle_filter_input_change(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let handler_start = std::time::Instant::now();

        if self.suppress_filter_events {
            return;
        }

        let new_text = self.gpui_input_state.read(cx).value().to_string();
        let shared_filter_view = self.current_view_uses_shared_filter_input();

        // Skip filter updates when actions popup is open
        // (text input should go to actions dialog search, not main filter)
        if self.show_actions_popup {
            if shared_filter_view && new_text != self.filter_text {
                self.pending_filter_sync = true;
            }
            return;
        }

        if !shared_filter_view {
            return;
        }
        self.pending_filter_sync = false;

        // Sync filter to builtin views that use the shared input
        match &mut self.current_view {
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    let input_mode = if self.input_mode == InputMode::Keyboard {
                        "keyboard"
                    } else {
                        "mouse"
                    };
                    Self::scroll_builtin_to_top_with_log(
                        &self.clipboard_list_scroll_handle,
                        "clipboard_history",
                        self.cached_clipboard_entries.len(),
                        &new_text,
                        input_mode,
                    );
                }
                let filtered_entries: Vec<_> = if filter.is_empty() {
                    self.cached_clipboard_entries.iter().enumerate().collect()
                } else {
                    let filter_lower = filter.to_lowercase();
                    self.cached_clipboard_entries
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            e.text_preview.to_lowercase().contains(&filter_lower)
                                || e.ocr_text
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_lowercase()
                                    .contains(&filter_lower)
                        })
                        .collect()
                };
                self.focused_clipboard_entry_id = filtered_entries
                    .get(*selected_index)
                    .map(|(_, entry)| entry.id.clone());
                cx.notify();
                return; // Don't run main menu filter logic
            }
            AppView::EmojiPickerView {
                filter,
                selected_index,
                selected_category,
            } => {
                // Skip entirely when the text hasn't changed — avoids a
                // spurious cx.notify() that can reset the scroll
                // position during trackpad/mouse scrolling.
                if new_text == *filter {
                    return;
                }

                self.filter_text = new_text.clone();
                *selected_index = 0;
                *filter = new_text.clone();

                let input_mode = if self.input_mode == InputMode::Keyboard {
                    "keyboard"
                } else {
                    "mouse"
                };
                let emoji_count =
                    crate::emoji::filtered_ordered_emojis(&new_text, *selected_category).len();

                Self::scroll_builtin_to_top_with_log(
                    &self.emoji_scroll_handle,
                    "emoji_picker",
                    emoji_count,
                    &new_text,
                    input_mode,
                );

                cx.notify();
                return; // Don't run main menu filter logic
            }
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    let input_mode = if self.input_mode == InputMode::Keyboard {
                        "keyboard"
                    } else {
                        "mouse"
                    };
                    Self::scroll_builtin_to_top_with_log(
                        &self.list_scroll_handle,
                        "app_launcher",
                        self.apps.len(),
                        &new_text,
                        input_mode,
                    );
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    let input_mode = if self.input_mode == InputMode::Keyboard {
                        "keyboard"
                    } else {
                        "mouse"
                    };
                    Self::scroll_builtin_to_top_with_log(
                        &self.window_list_scroll_handle,
                        "window_switcher",
                        self.cached_windows.len(),
                        &new_text,
                        input_mode,
                    );
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::ProcessManagerView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    let input_mode = if self.input_mode == InputMode::Keyboard {
                        "keyboard"
                    } else {
                        "mouse"
                    };
                    Self::scroll_builtin_to_top_with_log(
                        &self.process_list_scroll_handle,
                        "process_manager",
                        self.cached_processes.len(),
                        &new_text,
                        input_mode,
                    );
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::CurrentAppCommandsView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    let input_mode = if self.input_mode == InputMode::Keyboard {
                        "keyboard"
                    } else {
                        "mouse"
                    };

                    let (_filtered, receipt) =
                        crate::builtins::filter_menu_bar_entries(&self.cached_current_app_entries, &new_text);

                    tracing::debug!(
                        query = %receipt.query,
                        normalized_query = %receipt.normalized_query,
                        total_entries = receipt.total_entries,
                        matched_entries = receipt.matched_entries,
                        input_mode = %input_mode,
                        "current_app_commands.filter_updated"
                    );

                    Self::scroll_builtin_to_top_with_log(
                        &self.current_app_commands_scroll_handle,
                        "current_app_commands",
                        self.cached_current_app_entries.len(),
                        &new_text,
                        input_mode,
                    );
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::SearchAiPresetsView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::FavoritesBrowseView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    self.design_gallery_scroll_handle
                        .scroll_to_item(0, ScrollStrategy::Top);
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::ThemeChooserView {
                filter,
                selected_index,
            } => {
                self.filter_text = new_text.clone();
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    self.theme_chooser_scroll_handle
                        .scroll_to_item(0, ScrollStrategy::Top);
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::FileSearchView {
                query,
                selected_index,
                presentation,
            } => {
                // Copy presentation early so we can use it after releasing
                // the mutable borrow on self.current_view.
                let presentation = *presentation;

                // ── Mini presentation: return to ScriptList when query
                //    no longer matches the ~ trigger ──────────────────
                if presentation == FileSearchPresentation::Mini
                    && !Self::should_enter_file_search_from_script_list(&new_text)
                {
                    let _ = self.begin_file_search_session();
                    self.reset_file_search_transient_state();
                    self.current_view = AppView::ScriptList;
                    self.filter_text = new_text.clone();
                    self.pending_placeholder = None;
                    self.pending_focus = Some(FocusTarget::MainFilter);
                    self.focused_input = FocusedInput::MainFilter;
                    self.queue_filter_compute(new_text.clone(), cx);
                    cx.notify();
                    return;
                }

                self.filter_text = new_text.clone();
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

                    // Check if this is a directory path with potential filter
                    if let Some(parsed) = crate::file_search::parse_directory_path(&new_text) {
                        let dir_changed =
                            self.file_search_current_dir.as_ref() != Some(&parsed.directory);

                        if dir_changed {
                            self.restart_file_search_stream_for_query(
                                new_text.clone(),
                                presentation,
                                Some(old_filter),
                                true,
                                cx,
                            );
                        } else {
                            // Same directory - just filter existing results (instant!)
                            self.file_search_frozen_filter = None;
                            self.file_search_loading = false;
                            self.recompute_file_search_display_indices();
                            Self::resize_file_search_window_after_results_change(
                                presentation,
                                self.file_search_display_indices.len(),
                                true,
                                true,
                            );
                            cx.notify();
                        }
                        return;
                    }

                    // Not a directory path - do regular file search with streaming
                    logging::log(
                        "SEARCH",
                        &format!("Starting mdfind search for query: '{}'", new_text),
                    );
                    self.restart_file_search_stream_for_query(
                        new_text.clone(),
                        presentation,
                        None,
                        false,
                        cx,
                    );
                }
                return; // Don't run main menu filter logic
            }
            _ => {} // Continue with main menu logic
        }

        // ── ~ trigger: hand off to mini file search ──────────────────
        if Self::should_enter_file_search_from_script_list(&new_text) {
            let query = Self::normalize_mini_file_search_query(&new_text);
            self.open_file_search_view(query, FileSearchPresentation::Mini, cx);
            return;
        }

        if new_text == self.filter_text {
            return;
        }

        let previous_text = std::mem::replace(&mut self.filter_text, new_text.clone());

        // Reset input history navigation when user types (they're no longer navigating history)
        self.input_history.reset_navigation();

        let new_calc = crate::calculator::try_build(&new_text);
        if self.inline_calculator != new_calc {
            self.inline_calculator = new_calc;
            self.invalidate_grouped_cache();
            self.list_scroll_handle
                .scroll_to_item(0, ScrollStrategy::Top);
        }

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
}

/// Describes the source of a file search stream.
#[derive(Clone, Debug)]
enum FileSearchStreamSource {
    Directory { dir: String },
    Spotlight { query: String },
}

impl ScriptListApp {
    /// Return the path of the currently selected file-search row, if any.
    pub(crate) fn current_file_search_selected_path(&self) -> Option<String> {
        let AppView::FileSearchView { selected_index, .. } = &self.current_view else {
            return None;
        };
        let display_len = self.file_search_display_indices.len();
        if display_len == 0 {
            return None;
        }
        let clamped = (*selected_index).min(display_len.saturating_sub(1));
        let result_index = *self.file_search_display_indices.get(clamped)?;
        self.cached_file_results
            .get(result_index)
            .map(|entry| entry.path.clone())
    }

    /// After results change, restore selection to `preferred_path` if still
    /// visible, otherwise clamp to the nearest valid row.
    pub(crate) fn restore_file_search_selection_after_results_change(
        &mut self,
        preferred_path: Option<&str>,
    ) {
        let len = self.file_search_display_indices.len();
        let fallback_index = match &self.current_view {
            AppView::FileSearchView { selected_index, .. } if len > 0 => {
                (*selected_index).min(len.saturating_sub(1))
            }
            _ => 0,
        };

        let next_index = preferred_path
            .and_then(|path| self.file_search_display_index_for_path(path))
            .unwrap_or(fallback_index);

        if let AppView::FileSearchView { selected_index, .. } = &mut self.current_view {
            *selected_index = if len == 0 { 0 } else { next_index.min(len.saturating_sub(1)) };
        }
    }

    /// Reset all transient file-search state so exit/reopen cycles start
    /// from a clean slate.
    fn reset_file_search_transient_state(&mut self) {
        self.file_search_current_dir = None;
        self.file_search_loading = false;
        self.file_search_frozen_filter = None;
        self.file_search_actions_path = None;
        self.file_search_cancel = None;
        self.file_search_debounce_task = None;
        self.cached_file_results.clear();
        self.file_search_display_indices.clear();
        self.file_search_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);
    }

    /// Spawn a streaming file-search task that feeds batched results back
    /// into `apply_file_search_stream_batch`.  Used by both the directory
    /// and Spotlight paths so the batch/cancel/resize logic lives in one
    /// place.
    fn spawn_file_search_stream_task(
        &mut self,
        gen: u64,
        source: FileSearchStreamSource,
        presentation: FileSearchPresentation,
        debounce_ms: u64,
        query_guard: Option<String>,
        clear_on_first_batch: bool,
        sort_on_done: bool,
        cx: &mut Context<Self>,
    ) {
        let cancel = crate::file_search::new_cancel_token();
        self.file_search_cancel = Some(cancel.clone());

        let task = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(debounce_ms))
                .await;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn({
                let cancel = cancel.clone();
                let source = source.clone();
                move || match source {
                    FileSearchStreamSource::Directory { dir } => {
                        crate::file_search::list_directory_streaming(
                            &dir,
                            cancel,
                            false,
                            |event| {
                                let _ = tx.send(event);
                            },
                        );
                    }
                    FileSearchStreamSource::Spotlight { query } => {
                        crate::file_search::search_files_streaming(
                            &query,
                            None,
                            crate::file_search::DEFAULT_SEARCH_LIMIT,
                            cancel,
                            false,
                            |event| {
                                let _ = tx.send(event);
                            },
                        );
                    }
                }
            });

            let mut pending: Vec<crate::file_search::FileResult> = Vec::new();
            let mut done = false;
            let mut first_batch = true;

            while !done {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(16))
                    .await;

                while let Ok(event) = rx.try_recv() {
                    match event {
                        crate::file_search::SearchEvent::Result(result) => {
                            pending.push(result);
                        }
                        crate::file_search::SearchEvent::Done => {
                            done = true;
                            break;
                        }
                    }
                }

                if !pending.is_empty() || done {
                    let batch = std::mem::take(&mut pending);
                    let is_done = done;
                    let is_first_batch = first_batch;
                    let guard = query_guard.clone();
                    first_batch = false;

                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| {
                            app.apply_file_search_stream_batch(
                                gen,
                                presentation,
                                guard.as_deref(),
                                batch,
                                clear_on_first_batch,
                                sort_on_done,
                                is_first_batch,
                                is_done,
                                cx,
                            );
                        })
                    });
                }
            }
        });

        self.file_search_debounce_task = Some(task);
    }

    /// Apply one batch of streaming results.  Stale generations and
    /// query-guard mismatches are silently dropped.
    fn apply_file_search_stream_batch(
        &mut self,
        gen: u64,
        presentation: FileSearchPresentation,
        query_guard: Option<&str>,
        batch: Vec<crate::file_search::FileResult>,
        clear_on_first_batch: bool,
        sort_on_done: bool,
        is_first_batch: bool,
        is_done: bool,
        cx: &mut Context<Self>,
    ) {
        if self.file_search_gen != gen {
            return;
        }

        if let Some(expected_query) = query_guard {
            let AppView::FileSearchView { query, .. } = &self.current_view else {
                return;
            };
            if query != expected_query {
                return;
            }
        }

        // Capture selected path before mutating results so we can restore it.
        let preferred_selected_path = self.current_file_search_selected_path();

        let mut needs_recompute = false;

        if clear_on_first_batch && is_first_batch {
            self.cached_file_results.clear();
            needs_recompute = true;
        }

        if !batch.is_empty() {
            self.cached_file_results.extend(batch);
            needs_recompute = true;
        }

        if needs_recompute {
            self.recompute_file_search_display_indices();
            self.restore_file_search_selection_after_results_change(
                preferred_selected_path.as_deref(),
            );
        }

        if is_done {
            self.file_search_loading = false;
            self.file_search_frozen_filter = None;

            if sort_on_done {
                self.apply_file_search_sort_mode();
                self.recompute_file_search_display_indices();
                self.restore_file_search_selection_after_results_change(
                    preferred_selected_path.as_deref(),
                );
            }

            if !self.file_search_display_indices.is_empty() {
                if let AppView::FileSearchView { selected_index, .. } = &self.current_view {
                    self.file_search_scroll_handle
                        .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                }
            }
        }

        Self::resize_file_search_window_after_results_change(
            presentation,
            self.file_search_display_indices.len(),
            is_first_batch,
            is_done,
        );
        cx.notify();
    }

    /// Kick off a new streaming file-search session.  Cancels any
    /// in-flight work, advances the generation counter, and delegates
    /// to `spawn_file_search_stream_task`.
    pub(crate) fn restart_file_search_stream_for_query(
        &mut self,
        query: String,
        presentation: FileSearchPresentation,
        old_filter: Option<Option<String>>,
        preserve_old_results_until_first_batch: bool,
        cx: &mut Context<Self>,
    ) {
        let gen = self.begin_file_search_session();
        self.file_search_loading = true;

        if let Some(parsed) = crate::file_search::parse_directory_path(&query) {
            self.file_search_current_dir = Some(parsed.directory.clone());
            self.file_search_frozen_filter = if preserve_old_results_until_first_batch {
                old_filter
            } else {
                None
            };

            if !preserve_old_results_until_first_batch {
                self.cached_file_results.clear();
                self.file_search_display_indices.clear();
                Self::resize_file_search_window_after_results_change(
                    presentation,
                    0,
                    true,
                    false,
                );
            }

            self.spawn_file_search_stream_task(
                gen,
                FileSearchStreamSource::Directory {
                    dir: parsed.directory,
                },
                presentation,
                30,
                None,
                preserve_old_results_until_first_batch,
                true,
                cx,
            );
            cx.notify();
            return;
        }

        // Spotlight search path
        self.file_search_current_dir = None;
        self.file_search_frozen_filter = None;
        self.cached_file_results.clear();
        self.file_search_display_indices.clear();
        Self::resize_file_search_window_after_results_change(
            presentation,
            0,
            true,
            false,
        );

        self.spawn_file_search_stream_task(
            gen,
            FileSearchStreamSource::Spotlight {
                query: query.clone(),
            },
            presentation,
            75,
            Some(query),
            false,
            false,
            cx,
        );
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn read_filter_input_change_source() -> String {
        fs::read_to_string("src/app_impl/filter_input_change.rs")
            .expect("Failed to read src/app_impl/filter_input_change.rs")
    }

    #[test]
    fn test_handle_filter_input_change_marks_pending_sync_when_actions_popup_suppresses_updates() {
        let source = read_filter_input_change_source();
        let popup_guard_pos = source
            .find("if self.show_actions_popup")
            .expect("show_actions_popup guard not found");
        let popup_guard_end = (popup_guard_pos + 260).min(source.len());
        let popup_guard_section = &source[popup_guard_pos..popup_guard_end];

        assert!(
            popup_guard_section.contains("new_text != self.filter_text"),
            "show_actions_popup guard must detect divergence between input text and canonical filter_text"
        );
        assert!(
            popup_guard_section.contains("self.pending_filter_sync = true"),
            "show_actions_popup guard must queue pending_filter_sync to prevent persistent UI/filter desync"
        );
    }

    #[test]
    fn test_handle_filter_input_change_updates_canonical_filter_text_in_shared_builtin_views() {
        let source = read_filter_input_change_source();
        let shared_builtin_views = [
            "AppView::ClipboardHistoryView",
            "AppView::EmojiPickerView",
            "AppView::AppLauncherView",
            "AppView::WindowSwitcherView",
            "AppView::DesignGalleryView",
            "AppView::ThemeChooserView",
            "AppView::FileSearchView",
        ];

        for view in shared_builtin_views {
            let view_pos = source
                .find(view)
                .unwrap_or_else(|| panic!("{} match arm not found", view));
            let view_end = (view_pos + 500).min(source.len());
            let view_section = &source[view_pos..view_end];
            assert!(
                view_section.contains("self.filter_text = new_text.clone();"),
                "{} must keep ScriptListApp.filter_text synchronized as canonical query state",
                view
            );
        }
    }

    #[test]
    fn test_emoji_picker_filter_change_guards_scroll_behind_real_query_change() {
        let source = read_filter_input_change_source();
        let emoji_pos = source
            .find("AppView::EmojiPickerView")
            .expect("EmojiPickerView match arm not found");
        let emoji_section = &source[emoji_pos..(emoji_pos + 1200).min(source.len())];

        // Must have an early-return guard that skips cx.notify() when text is unchanged
        assert!(
            emoji_section.contains("new_text == *filter"),
            "emoji picker must early-return when the filter text has not changed"
        );

        // Must still scroll to top on real changes (via shared helper)
        assert!(
            emoji_section.contains("scroll_builtin_to_top_with_log"),
            "emoji picker should reset to top on real filter changes via shared helper"
        );
    }

    #[test]
    fn test_scrollable_builtin_views_use_shared_scroll_logging_helper() {
        let source = read_filter_input_change_source();
        for view in [
            "AppView::ClipboardHistoryView",
            "AppView::EmojiPickerView",
            "AppView::AppLauncherView",
            "AppView::WindowSwitcherView",
            "AppView::ProcessManagerView",
        ] {
            let view_pos = source
                .find(view)
                .unwrap_or_else(|| panic!("{} match arm not found", view));
            let view_section = &source[view_pos..(view_pos + 1200).min(source.len())];
            assert!(
                view_section.contains("scroll_builtin_to_top_with_log"),
                "{} should use the shared structured scroll helper",
                view,
            );
        }
    }
}
