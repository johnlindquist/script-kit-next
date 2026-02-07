use super::*;

impl ScriptListApp {
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

        if self.current_view_uses_shared_filter_input() {
            // Keep shared input state synchronized with view-scoped query/filter fields.
            self.filter_text = new_text.clone();
            self.pending_filter_sync = false;
        }

        // Sync filter to builtin views that use the shared input
        match &mut self.current_view {
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => {
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
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
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    self.list_scroll_handle
                        .scroll_to_item(0, ScrollStrategy::Top);
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => {
                if Self::sync_builtin_query_state(filter, selected_index, &new_text) {
                    self.window_list_scroll_handle
                        .scroll_to_item(0, ScrollStrategy::Top);
                    cx.notify();
                }
                return; // Don't run main menu filter logic
            }
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => {
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

}
