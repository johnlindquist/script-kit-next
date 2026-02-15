impl ScriptListApp {
    fn resolve_file_search_results_with(
        query: &str,
        is_directory_path: impl Fn(&str) -> bool,
        expand_path: impl Fn(&str) -> Option<String>,
        list_directory: impl Fn(&str, usize) -> Vec<crate::file_search::FileResult>,
        search_files: impl Fn(&str, Option<&str>, usize) -> Vec<crate::file_search::FileResult>,
    ) -> Vec<crate::file_search::FileResult> {
        if is_directory_path(query) {
            logging::log(
                "EXEC",
                &format!("Detected directory path, listing: {}", query),
            );

            let expanded = expand_path(query);
            let is_real_dir = expanded
                .as_deref()
                .map(|path| std::path::Path::new(path).is_dir())
                .unwrap_or(false);

            let directory_results = list_directory(query, crate::file_search::DEFAULT_CACHE_LIMIT);
            if directory_results.is_empty() && !is_real_dir {
                logging::log(
                    "EXEC",
                    "Path mode not a real directory; falling back to Spotlight search",
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
            crate::file_search::is_directory_path,
            crate::file_search::expand_path,
            crate::file_search::list_directory,
            crate::file_search::search_files,
        )
    }

    pub(crate) fn update_file_search_results(
        &mut self,
        results: Vec<crate::file_search::FileResult>,
    ) {
        let previous_cached_count = self.cached_file_results.len();
        self.cached_file_results = results;
        self.file_search_display_indices.clear();
        self.recompute_file_search_display_indices();
        logging::log(
            "SEARCH",
            &format!(
                "update_file_search_results: cached {} -> {} display={}",
                previous_cached_count,
                self.cached_file_results.len(),
                self.file_search_display_indices.len()
            ),
        );
    }

    /// Open a terminal with a specific command (for fallback "Run in Terminal")
    pub fn open_terminal_with_command(&mut self, command: String, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Opening terminal with command: {}", command),
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
                logging::log("ERROR", &format!("Failed to create terminal: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to open terminal: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
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
        logging::log(
            "EXEC",
            &format!("Opening File Search with query: {}", query),
        );

        let results = Self::resolve_file_search_results(&query);
        logging::log(
            "EXEC",
            &format!("File search found {} results", results.len()),
        );

        // Set up the view state
        self.filter_text = query.clone();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search files...".to_string());

        // Switch to file search view
        self.current_view = AppView::FileSearchView {
            query,
            selected_index: 0,
        };
        self.hovered_index = None;

        // Use standard height for file search view (same as window switcher)
        resize_to_view_sync(ViewType::ScriptList, 0);

        // Focus the main filter input so cursor blinks and typing works
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;

        // Initialize file search state for streaming
        self.file_search_gen = 0;
        self.file_search_cancel = None;
        self.update_file_search_results(results);

        cx.notify();
    }

    /// Sort directory listing results: directories first, then alphabetically
    pub fn sort_directory_results(&mut self) {
        // Sort the cached results in place
        self.cached_file_results.sort_by(|a, b| {
            let a_is_dir = matches!(a.file_type, crate::file_search::FileType::Directory);
            let b_is_dir = matches!(b.file_type, crate::file_search::FileType::Directory);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
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
    pub fn recompute_file_search_display_indices(&mut self) {
        // Get current filter pattern from the query
        let filter_pattern = if let AppView::FileSearchView { ref query, .. } = self.current_view {
            // Use frozen filter if set (during directory transitions)
            if let Some(ref frozen) = self.file_search_frozen_filter {
                frozen.clone()
            } else if let Some(parsed) = crate::file_search::parse_directory_path(query) {
                parsed.filter
            } else if !query.is_empty() {
                Some(query.clone())
            } else {
                None
            }
        } else {
            None
        };

        let cached_count = self.cached_file_results.len();

        // Compute display indices
        self.file_search_display_indices = if let Some(ref pattern) = filter_pattern {
            // Use Nucleo fuzzy matching and return only the indices, sorted by score
            let indices: Vec<usize> = crate::file_search::filter_results_nucleo_simple(
                &self.cached_file_results,
                pattern,
            )
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
            logging::log(
                "SEARCH",
                &format!(
                    "recompute_display_indices: pattern='{}' cached={} -> display={}",
                    pattern,
                    cached_count,
                    indices.len()
                ),
            );
            indices
        } else {
            // No filter - show all results in order
            logging::log(
                "SEARCH",
                &format!(
                    "recompute_display_indices: no_filter cached={} -> display={}",
                    cached_count, cached_count
                ),
            );
            (0..self.cached_file_results.len()).collect()
        };
    }

    /// Open the quick terminal
    fn open_quick_terminal(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "Opening Quick Terminal");

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
                logging::log("ERROR", &format!("Failed to create quick terminal: {}", e));
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to open terminal: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    /// Open the webcam prompt
    #[cfg(target_os = "macos")]
    fn open_webcam(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "Opening Webcam prompt");

        let focus_handle = self.focus_handle.clone();
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(|_id: String, value: Option<String>| {
                if let Some(data) = value {
                    logging::log(
                        "EXEC",
                        &format!("Webcam capture data: {} bytes", data.len()),
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
                logging::log("ERROR", &format!("Failed to start webcam: {}", err));
                // Still show the prompt with an error
                let entity_weak2 = entity.downgrade();
                let err_msg = err.to_string();
                cx.spawn(async move |_this, cx| {
                    let _ = cx.update(|cx| {
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
                gpui::Timer::after(std::time::Duration::from_millis(16)).await;

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

                match result {
                    Ok(true) => continue,
                    Ok(false) | Err(_) => break,
                }
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
        logging::log("EXEC", "Opening Webcam prompt (unsupported platform)");

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

        self.toast_manager.push(
            components::toast::Toast::error(
                "Webcam capture is only supported on macOS",
                &self.theme,
            )
            .duration_ms(Some(4000)),
        );

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
            |_| true,
            |_| Some("/definitely/not/a/real/dir".to_string()),
            |_, _| Vec::new(),
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
            |_| true,
            |_| Some("/definitely/not/a/real/dir".to_string()),
            |query, limit| {
                assert_eq!(query, "~/dir");
                assert_eq!(limit, crate::file_search::DEFAULT_CACHE_LIMIT);
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
            |_| false,
            |_| None,
            |_, _| panic!("list_directory should not be called for non-directory query"),
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
}
