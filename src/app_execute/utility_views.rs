impl ScriptListApp {
    fn resolve_file_search_results_with(
        query: &str,
        parse_directory_path: impl Fn(&str) -> Option<crate::file_search::ParsedDirPath>,
        is_directory_path: impl Fn(&str) -> bool,
        expand_path: impl Fn(&str) -> Option<String>,
        list_directory: impl Fn(&str, usize, bool) -> Vec<crate::file_search::FileResult>,
        search_files: impl Fn(&str, Option<&str>, usize) -> Vec<crate::file_search::FileResult>,
    ) -> Vec<crate::file_search::FileResult> {
        // Try structured parse first — handles ~/dev/fin → list ~/dev/
        if let Some(parsed) = parse_directory_path(query) {
            tracing::info!(
                message = %format!(
                    "Detected browseable path, listing parent directory: {}",
                    parsed.directory
                ),
            );
            return list_directory(
                &parsed.directory,
                crate::file_search::DEFAULT_CACHE_LIMIT,
                parsed.show_hidden,
            );
        }

        if is_directory_path(query) {
            tracing::info!(message = %format!("Detected directory path, listing: {}", query),
            );

            let expanded = expand_path(query);
            let is_real_dir = expanded
                .as_deref()
                .map(|path| std::path::Path::new(path).is_dir())
                .unwrap_or(false);

            let directory_results =
                list_directory(query, crate::file_search::DEFAULT_CACHE_LIMIT, false);
            if directory_results.is_empty() && !is_real_dir {
                tracing::info!(message = %"Path mode not a real directory; falling back to Spotlight search",
                );
                return search_files(query, None, crate::file_search::DEFAULT_SEARCH_LIMIT);
            }

            return directory_results;
        }

        search_files(query, None, crate::file_search::DEFAULT_SEARCH_LIMIT)
    }

    pub(crate) fn resolve_file_search_results(query: &str) -> Vec<crate::file_search::FileResult> {
        Self::resolve_file_search_results_with(
            query,
            crate::file_search::parse_directory_path,
            crate::file_search::is_directory_path,
            crate::file_search::expand_path,
            crate::file_search::list_directory_with_options,
            crate::file_search::search_files,
        )
    }

    pub(crate) fn update_file_search_results(
        &mut self,
        results: Vec<crate::file_search::FileResult>,
    ) {
        let previous_cached_count = self.cached_file_results.len();
        self.cached_file_results = results;

        let directory_sort_applied = matches!(
            &self.current_view,
            AppView::FileSearchView { query, .. }
                if crate::file_search::parse_directory_path(query).is_some()
        );

        if directory_sort_applied {
            self.sort_directory_results();
        }

        self.file_search_display_indices.clear();
        self.recompute_file_search_display_indices();

        let first_display_rows: Vec<String> = self
            .file_search_display_indices
            .iter()
            .take(5)
            .filter_map(|&result_index| {
                self.cached_file_results
                    .get(result_index)
                    .map(|entry| entry.name.clone())
            })
            .collect();

        tracing::debug!(
            category = "FILE_SEARCH",
            event = "update_file_search_results",
            previous_cached_count,
            cached_count = self.cached_file_results.len(),
            display_count = self.file_search_display_indices.len(),
            directory_sort_applied,
            ?self.file_search_sort_mode,
            first_display_rows = ?first_display_rows,
            "Updated file-search results"
        );
    }

    /// Open a terminal with a specific command (for fallback "Run in Terminal")
    pub fn open_terminal_with_command(&mut self, command: String, cx: &mut Context<Self>) {
        tracing::info!(message = %&format!("Opening terminal with command: {}", command),
        );

        // Create submit callback that just closes on exit/escape
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {
                // Terminal exited - nothing special to do
            });

        // Get the target height for terminal view (subtract footer height)
        let term_height =
            window_resize::layout::MAX_HEIGHT - px(window_resize::layout::FOOTER_HEIGHT);

        // Create terminal with the specified command
        match term_prompt::TermPrompt::with_height(
            "fallback-terminal".to_string(),
            Some(command), // Run the specified command
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        ) {
            Ok(term_prompt) => {
                let entity = cx.new(|_| term_prompt);
                self.current_view = AppView::QuickTerminalView { entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::TermPrompt);
                // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                // to after the current GPUI update cycle completes. Synchronous Cocoa
                // setFrame: calls during render can trigger events that re-borrow GPUI state.
                cx.spawn(async move |_this, _cx| {
                    resize_to_view_sync(ViewType::TermPrompt, 0);
                })
                .detach();
                cx.notify();
            }
            Err(e) => {
                tracing::error!(message = %&format!("Failed to create terminal: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to open terminal: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_ERROR_MS)),
                );
                cx.notify();
            }
        }
    }

    // =========================================================================
    // File Search Implementation
    // =========================================================================
    //
    // BLOCKED: Requires the following changes to main.rs (not in worker reservations):
    //
    // 1. Add to AppView enum:
    //    ```rust
    //    /// Showing file search results (Spotlight/mdfind based)
    //    FileSearchView {
    //        query: String,
    //        selected_index: usize,
    //    },
    //    ```
    //
    // 2. Add to ScriptListApp struct:
    //    ```rust
    //    /// Cached file search results
    //    cached_file_results: Vec<file_search::FileResult>,
    //    /// Scroll handle for file search list
    //    file_search_scroll_handle: UniformListScrollHandle,
    //    ```
    //
    // 3. Add initialization in app_impl.rs ScriptListApp::new():
    //    ```rust
    //    cached_file_results: Vec::new(),
    //    file_search_scroll_handle: UniformListScrollHandle::new(),
    //    ```
    //
    // 4. Add render call in main.rs Render impl match arm:
    //    ```rust
    //    AppView::FileSearchView { query, selected_index } => {
    //        self.render_file_search(query.clone(), *selected_index, cx)
    //    }
    //    ```
    //
    // 5. Wire up in app_impl.rs execute_fallback():
    //    ```rust
    //    FallbackResult::SearchFiles { query } => {
    //        self.open_file_search(query, cx);
    //    }
    //    ```
    //
    // Once those are added, uncomment the method below.
    // =========================================================================

    /// Open file search with the given query
    ///
    /// This performs an mdfind-based file search and displays results in a Raycast-like UI.
    ///
    /// # Arguments
    /// * `query` - The search query (passed from the "Search Files" fallback action)
    ///
    /// # Usage
    /// Called when user selects "Search Files" fallback with a search term.
    /// Features:
    /// - Live search as user types (debounced)
    /// - File type icons (folder, document, image, audio, video, code, etc.)
    /// - File size and modified date display
    /// - Enter: Open file in default application
    /// - Cmd+Enter: Reveal in Finder
    pub fn open_file_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.open_file_search_view(query, FileSearchPresentation::Full, cx);
    }

    /// Return the rendered display index for the first visible row matching `path`.
    pub(crate) fn file_search_display_index_for_path(&self, path: &str) -> Option<usize> {
        self.file_search_display_indices
            .iter()
            .position(|&result_index| {
                self.cached_file_results
                    .get(result_index)
                    .map(|entry| entry.path == path)
                    .unwrap_or(false)
            })
    }

    /// Look up a file search result by its position in the rendered display list
    /// (after filtering, scoring, and directory-first sorting).
    pub(crate) fn file_search_result_at_display_index(
        &self,
        display_index: usize,
    ) -> Option<&crate::file_search::FileResult> {
        self.file_search_display_indices
            .get(display_index)
            .and_then(|&result_index| self.cached_file_results.get(result_index))
    }

    /// Clamp a display index to the currently rendered file search list.
    pub(crate) fn clamp_file_search_display_index(&self, selected_index: usize) -> Option<usize> {
        if self.file_search_display_indices.is_empty() {
            None
        } else {
            Some(selected_index.min(self.file_search_display_indices.len() - 1))
        }
    }

    /// Return the currently selected file-search entry in display order.
    pub(crate) fn selected_file_search_result(
        &self,
        selected_index: usize,
    ) -> Option<(usize, &crate::file_search::FileResult)> {
        let display_index = self.clamp_file_search_display_index(selected_index)?;
        let entry = self.file_search_result_at_display_index(display_index)?;
        Some((display_index, entry))
    }

    /// Number of rows in the precomputed file-search display list.
    pub(crate) fn file_search_display_len(&self) -> usize {
        self.file_search_display_indices.len()
    }

    /// Clamp the file-search selection to the visible row count, updating
    /// `selected_index` inside `AppView::FileSearchView` in place.
    /// Returns the clamped index, or `None` when the display list is empty.
    pub(crate) fn clamp_current_file_search_selection(&mut self) -> Option<usize> {
        let len = self.file_search_display_len();
        let AppView::FileSearchView { selected_index, .. } = &mut self.current_view else {
            return None;
        };
        if len == 0 {
            *selected_index = 0;
            None
        } else {
            let clamped = (*selected_index).min(len - 1);
            *selected_index = clamped;
            Some(clamped)
        }
    }

    /// Return the currently selected file-search entry (owned clone) after
    /// clamping the selection to the visible display list.
    pub(crate) fn selected_file_search_result_owned(
        &mut self,
    ) -> Option<(usize, crate::file_search::FileResult)> {
        let display_index = self.clamp_current_file_search_selection()?;
        let entry = self
            .file_search_result_at_display_index(display_index)?
            .clone();
        Some((display_index, entry))
    }

    /// Cancel any in-flight file-search work and advance the generation
    /// counter. Returns the new generation value.
    pub(crate) fn begin_file_search_session(&mut self) -> u64 {
        if let Some(cancel) = self.file_search_cancel.take() {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.file_search_debounce_task = None;
        self.file_search_loading = false;
        self.file_search_frozen_filter = None;
        self.file_search_gen = self.file_search_gen.wrapping_add(1);
        self.file_search_gen
    }

    /// Row labels for file search in the exact order shown to the user.
    pub(crate) fn file_search_display_row_names(&self) -> Vec<String> {
        self.file_search_display_indices
            .iter()
            .filter_map(|&result_index| self.cached_file_results.get(result_index))
            .map(|entry| entry.name.clone())
            .collect()
    }

    /// Sort cached directory results using the active file-search sort mode.
    ///
    /// This wrapper keeps directory-refresh callers aligned with
    /// `compare_file_search_results_for_mode`.
    pub fn sort_directory_results(&mut self) {
        tracing::info!(
            category = "FILE_SEARCH",
            event = "sort_directory_results_delegate",
            ?self.file_search_sort_mode,
            cached_count = self.cached_file_results.len(),
            "Delegating directory result sorting to apply_file_search_sort_mode"
        );
        self.apply_file_search_sort_mode();
    }

    /// Recompute file_search_display_indices based on current filter pattern
    ///
    /// This is called when:
    /// 1. Results change (new directory listing or search results)
    /// 2. Filter pattern changes (user types in existing directory)
    /// 3. Loading completes (final sort/rank)
    ///
    /// By computing this OUTSIDE of render, we ensure that animation tickers
    /// calling cx.notify() at 60fps don't re-run expensive Nucleo scoring.
    /// Re-sort display indices for a directory-browse view using the active
    /// `file_search_sort_mode`.  This ensures filtered directory views respect
    /// the user-selected sort instead of drifting to Nucleo match order.
    fn sort_file_search_display_indices_for_directory(&self, indices: &mut [usize]) {
        let mode = self.file_search_sort_mode;
        indices.sort_by(|a_idx, b_idx| {
            let a = self.cached_file_results.get(*a_idx);
            let b = self.cached_file_results.get(*b_idx);
            match (a, b) {
                (Some(a), Some(b)) => Self::compare_file_search_results_for_mode(mode, a, b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a_idx.cmp(b_idx),
            }
        });
        tracing::debug!(
            category = "FILE_SEARCH",
            event = "sort_directory_display_indices",
            ?mode,
            display_count = indices.len(),
            "Applied active sort mode to directory display indices"
        );
    }

    pub fn recompute_file_search_display_indices(&mut self) {
        let (filter_pattern, is_directory_query) =
            if let AppView::FileSearchView { ref query, .. } = self.current_view {
                let parsed = crate::file_search::parse_directory_path(query);
                let filter_pattern = if let Some(ref frozen) = self.file_search_frozen_filter {
                    frozen.clone()
                } else if let Some(ref parsed) = parsed {
                    parsed.filter.clone()
                } else if !query.is_empty() {
                    Some(query.clone())
                } else {
                    None
                };
                (filter_pattern, parsed.is_some())
            } else {
                (None, false)
            };

        let cached_count = self.cached_file_results.len();

        self.file_search_display_indices = if let Some(ref pattern) = filter_pattern {
            let mut indices: Vec<usize> = crate::file_search::filter_results_nucleo_simple(
                &self.cached_file_results,
                pattern,
            )
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
            if is_directory_query {
                self.sort_file_search_display_indices_for_directory(&mut indices);
            }
            tracing::debug!(
                event = "recompute_display_indices",
                pattern = %pattern,
                cached_count,
                display_count = indices.len(),
                is_directory_query,
                ?self.file_search_sort_mode
            );
            indices
        } else {
            let mut indices: Vec<usize> = (0..self.cached_file_results.len()).collect();
            if is_directory_query {
                self.sort_file_search_display_indices_for_directory(&mut indices);
            }
            tracing::debug!(
                event = "recompute_display_indices",
                mode = "no_filter",
                cached_count,
                display_count = indices.len(),
                is_directory_query,
                ?self.file_search_sort_mode
            );
            indices
        };
    }

    /// Open the quick terminal
    pub(crate) fn open_quick_terminal(&mut self, cx: &mut Context<Self>) {
        tracing::info!(message = %"Opening Quick Terminal");

        // Create submit callback that just closes on exit/escape
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {
                // Terminal exited - nothing special to do
            });

        // Get the target height for terminal view (subtract footer height)
        let term_height =
            window_resize::layout::MAX_HEIGHT - px(window_resize::layout::FOOTER_HEIGHT);

        // Create terminal without a specific command (opens default shell)
        match term_prompt::TermPrompt::with_height(
            "quick-terminal".to_string(),
            None, // No command - opens default shell
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        ) {
            Ok(term_prompt) => {
                let entity = cx.new(|_| term_prompt);
                self.current_view = AppView::QuickTerminalView { entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::TermPrompt);
                // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                // to after the current GPUI update cycle completes. Synchronous Cocoa
                // setFrame: calls during render can trigger events that re-borrow GPUI state.
                cx.spawn(async move |_this, _cx| {
                    resize_to_view_sync(ViewType::TermPrompt, 0);
                })
                .detach();
                cx.notify();
            }
            Err(e) => {
                tracing::error!(message = %&format!("Failed to create quick terminal: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to open terminal: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_ERROR_MS)),
                );
                cx.notify();
            }
        }
    }

    /// Open Claude Code in the quick terminal view
    fn open_claude_code_terminal(&mut self, cx: &mut Context<Self>) {
        tracing::info!(message = %"Opening Claude Code Terminal");

        let command = r#"zsh -lc 'mkdir -p "$HOME/.scriptkit/sessions" && cd "$HOME/.scriptkit/sessions" && exec claude'"#.to_string();
        self.open_terminal_with_command(command, cx);
    }

    /// Open the webcam prompt
    #[cfg(target_os = "macos")]
    fn open_webcam(&mut self, cx: &mut Context<Self>) {
        tracing::info!(message = %"Opening Webcam prompt");

        let focus_handle = self.focus_handle.clone();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(|_id: String, value: Option<String>| {
                if let Some(data) = value {
                    tracing::info!(message = %&format!("Webcam capture data: {} bytes", data.len()),
                    );
                }
            });

        let webcam_prompt = prompts::WebcamPrompt::new(
            "webcam".to_string(),
            focus_handle,
            submit_callback,
            std::sync::Arc::clone(&self.theme),
        );

        let entity = cx.new(|_| webcam_prompt);

        // Zero-copy camera capture via AVFoundation.
        // Camera frames arrive as CVPixelBuffer on a dispatch queue,
        // then we poll and pass them to gpui::surface() — no CPU conversion.
        let entity_weak = entity.downgrade();

        let (frame_rx, capture_handle) = match crate::camera::start_capture(640) {
            Ok(pair) => pair,
            Err(err) => {
                tracing::error!(message = %&format!("Failed to start webcam: {}", err));
                // Still show the prompt with an error
                let entity_weak2 = entity.downgrade();
                let err_msg = err.to_string();
                cx.spawn(async move |_this, cx| {
                    cx.update(|cx| {
                        if let Some(entity) = entity_weak2.upgrade() {
                            entity.update(cx, |prompt, cx| {
                                prompt.set_error(err_msg, cx);
                            });
                        }
                    });
                })
                .detach();

                self.current_view = AppView::WebcamView { entity };
                self.focused_input = FocusedInput::None;
                self.pending_focus = Some(FocusTarget::AppRoot);
                resize_to_view_sync(ViewType::DivPrompt, 0);
                cx.notify();
                return;
            }
        };

        // Store the capture handle in the prompt — when the prompt entity is
        // dropped, the handle drops too, stopping the camera and releasing resources.
        entity.update(cx, |prompt, _cx| {
            prompt.capture_handle = Some(capture_handle);
        });

        // Async poller: drain CVPixelBuffers from channel, push latest to prompt.
        // Exits when the channel disconnects (CaptureHandle dropped) or the entity is gone.
        cx.spawn(async move |_this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(16))
                    .await;

                // Drain to latest frame, detect channel disconnect
                let mut latest = None;
                loop {
                    match frame_rx.try_recv() {
                        Ok(buf) => latest = Some(buf),
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                    }
                }

                let Some(buf) = latest else {
                    continue;
                };

                let result = cx.update(|cx| {
                    if let Some(entity) = entity_weak.upgrade() {
                        entity.update(cx, |prompt, cx| {
                            prompt.set_pixel_buffer(buf, cx);
                        });
                        true
                    } else {
                        false
                    }
                });

                if result {
                    continue;
                }
                break;
            }
        })
        .detach();

        self.current_view = AppView::WebcamView { entity };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);

        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }

    /// Open the webcam prompt
    #[cfg(not(target_os = "macos"))]
    fn open_webcam(&mut self, cx: &mut Context<Self>) {
        tracing::info!(message = %"Opening Webcam prompt (unsupported platform)");

        let focus_handle = self.focus_handle.clone();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(|_id: String, _value: Option<String>| {});

        let webcam_prompt = prompts::WebcamPrompt::new(
            "webcam".to_string(),
            focus_handle,
            submit_callback,
            std::sync::Arc::clone(&self.theme),
        );

        let entity = cx.new(|_| webcam_prompt);
        entity.update(cx, |prompt, cx| {
            prompt.set_error("Webcam capture is only supported on macOS".to_string(), cx);
        });

        self.show_error_toast("Webcam capture is only supported on macOS", cx);

        self.current_view = AppView::WebcamView { entity };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);
        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }
}

#[cfg(test)]
mod utility_views_file_search_tests {
    use super::*;
    use crate::file_search::{FileResult, FileType};

    fn test_file_result(name: &str) -> FileResult {
        FileResult {
            path: format!("/tmp/{}", name),
            name: name.to_string(),
            size: 0,
            modified: 0,
            file_type: FileType::File,
        }
    }

    #[test]
    fn test_resolve_file_search_results_with_falls_back_when_directory_path_is_not_real_directory()
    {
        let results = ScriptListApp::resolve_file_search_results_with(
            "~/missing-dir",
            |_| None, // parse_directory_path returns None
            |_| true,
            |_| Some("/definitely/not/a/real/dir".to_string()),
            |_, _, _| Vec::new(),
            |query, onlyin, limit| {
                assert_eq!(query, "~/missing-dir");
                assert!(onlyin.is_none());
                assert_eq!(limit, crate::file_search::DEFAULT_SEARCH_LIMIT);
                vec![test_file_result("fallback-result")]
            },
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "fallback-result");
    }

    #[test]
    fn test_resolve_file_search_results_with_uses_directory_listing_when_results_exist() {
        let results = ScriptListApp::resolve_file_search_results_with(
            "~/dir",
            |_| None, // parse_directory_path returns None
            |_| true,
            |_| Some("/definitely/not/a/real/dir".to_string()),
            |query, limit, show_hidden| {
                assert_eq!(query, "~/dir");
                assert_eq!(limit, crate::file_search::DEFAULT_CACHE_LIMIT);
                assert!(!show_hidden);
                vec![test_file_result("directory-result")]
            },
            |_, _, _| {
                panic!("search_files should not be called when directory listing returns results")
            },
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "directory-result");
    }

    #[test]
    fn test_resolve_file_search_results_with_uses_search_for_non_directory_queries() {
        let results = ScriptListApp::resolve_file_search_results_with(
            "invoice",
            |_| None, // parse_directory_path returns None
            |_| false,
            |_| None,
            |_, _, _| panic!("list_directory should not be called for non-directory query"),
            |query, onlyin, limit| {
                assert_eq!(query, "invoice");
                assert!(onlyin.is_none());
                assert_eq!(limit, crate::file_search::DEFAULT_SEARCH_LIMIT);
                vec![test_file_result("search-result")]
            },
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "search-result");
    }

    #[test]
    fn test_resolve_file_search_results_with_uses_parsed_directory_when_available() {
        use crate::file_search::ParsedDirPath;
        let results = ScriptListApp::resolve_file_search_results_with(
            "~/dev/fin",
            |_| {
                Some(ParsedDirPath {
                    directory: "~/dev/".to_string(),
                    filter: Some("fin".to_string()),
                    show_hidden: false,
                })
            },
            |_| true,
            |_| None,
            |query, limit, show_hidden| {
                assert_eq!(query, "~/dev/");
                assert_eq!(limit, crate::file_search::DEFAULT_CACHE_LIMIT);
                assert!(!show_hidden);
                vec![test_file_result("parsed-dir-result")]
            },
            |_, _, _| {
                panic!("search_files should not be called when parse_directory_path succeeds")
            },
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "parsed-dir-result");
    }
}
