use super::*;

#[derive(Clone)]
enum RootFileSearchRequest {
    GlobalQuery {
        query: String,
    },
    DirectoryBrowse {
        query: String,
        directory: String,
        show_hidden: bool,
    },
}

impl RootFileSearchRequest {
    fn query(&self) -> &str {
        match self {
            Self::GlobalQuery { query } | Self::DirectoryBrowse { query, .. } => query,
        }
    }

    fn mode(&self) -> crate::file_search::RootFileSectionMode {
        match self {
            Self::GlobalQuery { .. } => crate::file_search::RootFileSectionMode::GlobalQuery,
            Self::DirectoryBrowse { .. } => {
                crate::file_search::RootFileSectionMode::DirectoryBrowse
            }
        }
    }
}

impl ScriptListApp {
    pub(crate) fn refresh_root_recent_file_results(&mut self) {
        let revision = self.frecency_store.revision();
        if self.root_recent_file_revision == revision {
            return;
        }

        let mut seen = std::collections::HashSet::new();
        let mut hydrated: Vec<_> = self
            .frecency_store
            .top_file_paths(crate::file_search::ROOT_FILE_RECENT_LIMIT * 3)
            .into_iter()
            .filter_map(|(path, score)| {
                if !seen.insert(path.clone()) {
                    return None;
                }
                crate::file_search::file_result_from_existing_path(&path).map(|file| (file, score))
            })
            .collect();

        hydrated.sort_by(|(a, a_score), (b, b_score)| {
            b_score
                .partial_cmp(a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.modified.cmp(&a.modified))
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.path.cmp(&b.path))
        });

        self.root_recent_file_results = hydrated
            .into_iter()
            .take(crate::file_search::ROOT_FILE_RECENT_LIMIT)
            .map(|(file, _)| file)
            .collect();
        self.root_recent_file_revision = revision;
        self.invalidate_grouped_cache();
    }

    fn cancel_root_file_search(&mut self) {
        if let Some(cancel) = self.root_file_search_cancel.take() {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn active_root_directory_browse_source_matches(
        &self,
        directory: &str,
        show_hidden: bool,
    ) -> bool {
        if self.root_file_search_mode
            != Some(crate::file_search::RootFileSectionMode::DirectoryBrowse)
        {
            return false;
        }

        crate::file_search::root_directory_browse_source_key(&self.root_file_search_query)
            .map(|(active_directory, active_show_hidden)| {
                active_directory == directory && active_show_hidden == show_hidden
            })
            .unwrap_or(false)
    }

    fn refresh_root_file_grouping_after_query_only_change(&mut self, cx: &mut Context<Self>) {
        self.invalidate_grouped_cache();
        if matches!(self.current_view, AppView::ScriptList) {
            self.sync_list_state_for_filter_replacement();
            self.validate_selection_bounds(cx);
            self.rebuild_main_window_preflight_if_needed();
        }
        cx.notify();
    }

    pub(crate) fn maybe_start_root_file_search(&mut self, query: &str, cx: &mut Context<Self>) {
        let trimmed = query.trim();
        let can_collect = matches!(self.current_view, AppView::ScriptList)
            && self
                .menu_syntax_mode
                .advanced_query_for(&self.filter_text)
                .is_none()
            && !self.menu_syntax_trigger_popup_state.owns_main_list()
            && !self
                .menu_syntax_mode
                .capture_composer_owns_input_for(trimmed)
            && !self.menu_syntax_mode.command_owns_input_for(trimmed);

        let request = if !can_collect {
            None
        } else if crate::file_search::should_search_root_files(trimmed) {
            Some(RootFileSearchRequest::GlobalQuery {
                query: trimmed.to_string(),
            })
        } else if crate::file_search::looks_like_root_directory_browse_query(trimmed) {
            crate::file_search::parse_directory_path(trimmed).map(|parsed| {
                RootFileSearchRequest::DirectoryBrowse {
                    query: trimmed.to_string(),
                    directory: parsed.directory,
                    show_hidden: parsed.show_hidden,
                }
            })
        } else {
            None
        };

        let Some(request) = request else {
            self.cancel_root_file_search();
            let had_results = !self.root_file_results.is_empty()
                || !self.root_file_search_query.is_empty()
                || self.root_file_search_loading
                || self.root_file_search_mode.is_some();
            self.root_file_results.clear();
            self.root_file_search_query.clear();
            self.root_file_search_mode = None;
            self.root_file_search_loading = false;
            if had_results {
                self.root_file_search_generation = self.root_file_search_generation.wrapping_add(1);
                self.invalidate_grouped_cache();
                cx.notify();
            }
            return;
        };

        let mode = request.mode();
        match &request {
            RootFileSearchRequest::GlobalQuery { .. }
                if self.root_file_search_query == request.query()
                    && self.root_file_search_mode == Some(mode) =>
            {
                return;
            }
            RootFileSearchRequest::DirectoryBrowse {
                query,
                directory,
                show_hidden,
            } if self.active_root_directory_browse_source_matches(directory, *show_hidden) => {
                if self.root_file_search_query != *query {
                    self.root_file_search_query = query.clone();
                    self.refresh_root_file_grouping_after_query_only_change(cx);
                }
                return;
            }
            _ => {}
        }

        self.cancel_root_file_search();
        self.root_file_search_generation = self.root_file_search_generation.wrapping_add(1);
        let generation = self.root_file_search_generation;
        self.root_file_search_query = request.query().to_string();
        self.root_file_search_mode = Some(mode);
        self.root_file_results.clear();
        self.root_file_search_loading = true;
        self.invalidate_grouped_cache();

        let cancel = crate::file_search::new_cancel_token();
        self.root_file_search_cancel = Some(cancel.clone());

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(120))
                .await;

            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn({
                let cancel = cancel.clone();
                let request = request.clone();
                move || match request {
                    RootFileSearchRequest::GlobalQuery { query } => {
                        crate::file_search::search_files_streaming_with_options(
                            &query,
                            None,
                            crate::file_search::ROOT_FILE_SOURCE_LIMIT,
                            cancel,
                            crate::file_search::SearchFilesStreamingOptions::root_search(),
                            |event| {
                                let _ = tx.send(event);
                            },
                        );
                    }
                    RootFileSearchRequest::DirectoryBrowse {
                        directory,
                        show_hidden,
                        ..
                    } => {
                        for result in crate::file_search::list_directory_with_options(
                            &directory,
                            crate::file_search::ROOT_FILE_BROWSE_SOURCE_LIMIT,
                            show_hidden,
                        ) {
                            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                                return;
                            }
                            let _ = tx.send(crate::file_search::SearchEvent::Result(result));
                        }
                        let _ = tx.send(crate::file_search::SearchEvent::Done);
                    }
                }
            });

            let mut batch = Vec::new();
            loop {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                match rx.try_recv() {
                    Ok(crate::file_search::SearchEvent::Result(result)) => batch.push(result),
                    Ok(crate::file_search::SearchEvent::Done) => break,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(16))
                            .await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }
            }

            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    if app.root_file_search_generation != generation {
                        return;
                    }

                    app.root_file_results = batch;
                    app.root_file_search_loading = false;
                    app.root_file_search_cancel = None;
                    app.invalidate_grouped_cache();
                    if matches!(app.current_view, AppView::ScriptList) {
                        app.sync_list_state_for_filter_replacement();
                        app.validate_selection_bounds(cx);
                        app.rebuild_main_window_preflight_if_needed();
                    }
                    cx.notify();
                })
            });
        })
        .detach();
    }
}
