use super::*;

const ROOT_FILE_RESULT_CACHE_LIMIT: usize = 24;
const ROOT_FILE_SEARCH_DEBOUNCE_MS: u64 = 60;
const SPINE_FILE_SEARCH_DEBOUNCE_MS: u64 = 80;

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

    fn cache_key(&self) -> String {
        match self {
            Self::GlobalQuery { query } => format!("global:{query}"),
            Self::DirectoryBrowse {
                query,
                directory,
                show_hidden,
            } => format!("dir:{directory}:{show_hidden}:{query}"),
        }
    }
}

impl ScriptListApp {
    pub(crate) fn refresh_root_recent_file_results(&mut self) {
        let mut options = self.config.get_unified_search().root_file_section_options();
        if self
            .menu_syntax_mode
            .advanced_query_for(&self.computed_filter_text)
            .is_some_and(|query| {
                query
                    .source_filters
                    .includes(crate::menu_syntax::RootUnifiedSourceFilter::Files)
            })
        {
            options.files_enabled = true;
            options.recent_files_enabled = true;
        }
        if !options.files_enabled || !options.recent_files_enabled {
            if !self.root_recent_file_results.is_empty() {
                self.root_recent_file_results.clear();
                self.invalidate_grouped_cache();
            }
            self.root_recent_file_revision = u64::MAX;
            return;
        }

        let revision = self.frecency_store.revision();
        if self.root_recent_file_revision == revision {
            return;
        }

        let next_results =
            self.recent_file_results_from_frecency(crate::file_search::ROOT_FILE_RECENT_SEED_LIMIT);
        let changed = root_file_result_fingerprint(&self.root_recent_file_results)
            != root_file_result_fingerprint(&next_results);
        self.root_recent_file_results = next_results;
        self.root_recent_file_revision = revision;
        if changed {
            self.invalidate_grouped_cache();
        }
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

    fn apply_root_file_search_results_for_generation(
        &mut self,
        generation: u64,
        results: Vec<crate::file_search::FileResult>,
        loading: bool,
        clear_cancel: bool,
        cx: &mut Context<Self>,
    ) {
        if self.root_file_search_generation != generation {
            return;
        }

        let selection_before = if matches!(self.current_view, AppView::ScriptList) {
            Some(self.main_menu_selection_snapshot())
        } else {
            None
        };

        self.root_file_results = results;
        self.root_file_search_loading = loading;
        self.root_file_provider_loading = loading;
        self.root_file_frame = None;
        if clear_cancel {
            self.root_file_search_cancel = None;
        }
        self.invalidate_grouped_cache();
        if matches!(self.current_view, AppView::ScriptList) {
            self.sync_list_state_for_filter_replacement();
            if let Some(snapshot) = selection_before {
                self.restore_main_menu_selection_from_snapshot(snapshot);
            }
            self.validate_selection_bounds(cx);
            self.reveal_main_list_selection_above_footer("root_file_active_publish");
            self.schedule_main_list_selection_reveal_above_footer(
                "root_file_active_publish_deferred",
                cx,
            );
            self.invalidate_main_window_preflight();
            self.rebuild_main_window_preflight_if_needed();
        }
        cx.notify();
    }

    fn cached_root_file_results_for_request(
        &self,
        request: &RootFileSearchRequest,
    ) -> Vec<crate::file_search::FileResult> {
        let cache_key = request.cache_key();
        self.root_file_result_cache
            .iter()
            .find_map(|(key, results)| {
                if key == &cache_key {
                    Some(results.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    fn cache_root_file_search_results_for_generation(
        &mut self,
        generation: u64,
        cache_key: String,
        results: Vec<crate::file_search::FileResult>,
        clear_cancel: bool,
    ) {
        if self.root_file_search_generation != generation {
            return;
        }

        if let Some(index) = self
            .root_file_result_cache
            .iter()
            .position(|(key, _)| key == &cache_key)
        {
            self.root_file_result_cache.remove(index);
        }
        self.root_file_result_cache
            .push_front((cache_key, dedupe_root_file_results(results)));
        while self.root_file_result_cache.len() > ROOT_FILE_RESULT_CACHE_LIMIT {
            self.root_file_result_cache.pop_back();
        }

        if clear_cancel {
            self.root_file_search_cancel = None;
        }
        self.root_file_provider_loading = false;
    }

    pub(crate) fn active_root_file_cache_result_count(&self) -> usize {
        let Some(mode) = self.root_file_search_mode else {
            return 0;
        };
        let request = match mode {
            crate::file_search::RootFileSectionMode::GlobalQuery => {
                RootFileSearchRequest::GlobalQuery {
                    query: self.root_file_search_query.clone(),
                }
            }
            crate::file_search::RootFileSectionMode::DirectoryBrowse => return 0,
        };
        let cache_key = request.cache_key();
        self.root_file_result_cache
            .iter()
            .find_map(|(key, results)| (key == &cache_key).then_some(results.len()))
            .unwrap_or(0)
    }

    pub(crate) fn maybe_start_root_file_search(&mut self, query: &str, cx: &mut Context<Self>) {
        let search_text =
            crate::menu_syntax::free_text_for_search(&self.menu_syntax_mode, query).to_string();
        let trimmed = search_text.trim();
        let advanced_query_owned = self.menu_syntax_mode.advanced_query_for(query).cloned();
        let source_filters = advanced_query_owned
            .as_ref()
            .map(|advanced_query| advanced_query.source_filters.clone())
            .unwrap_or_default();
        let advanced_predicate_active = advanced_query_owned
            .as_ref()
            .is_some_and(|advanced_query| advanced_query.has_predicates());
        let mut root_file_options = self.config.get_unified_search().root_file_section_options();
        if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Files) {
            root_file_options.files_enabled = true;
            root_file_options.global_search_enabled = true;
            root_file_options.directory_browse_enabled = true;
            root_file_options.recent_files_enabled = true;
            root_file_options.query_intent =
                crate::file_search::RootFileQueryIntent::ExplicitFilesSourceFilter;
            root_file_options.source_chip_visible_limit =
                Some(self.root_file_source_chip_visible_limit_for(
                    query,
                    trimmed,
                    advanced_predicate_active,
                    self.root_file_search_mode,
                ));
        }
        if !root_file_options.files_enabled
            || !source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Files)
        {
            self.cancel_root_file_search();
            let had_results = !self.root_file_results.is_empty()
                || !self.root_recent_file_results.is_empty()
                || !self.root_file_search_query.is_empty()
                || self.root_file_search_loading
                || self.root_file_provider_loading
                || self.root_file_search_mode.is_some();
            self.root_file_results.clear();
            self.root_recent_file_results.clear();
            self.root_file_search_query.clear();
            self.root_file_search_mode = None;
            self.root_file_search_loading = false;
            self.root_file_provider_loading = false;
            self.root_file_frame = None;
            if had_results {
                self.root_file_search_generation = self.root_file_search_generation.wrapping_add(1);
                self.invalidate_grouped_cache();
                cx.notify();
            }
            return;
        }

        let can_collect = matches!(self.current_view, AppView::ScriptList)
            && self
                .menu_syntax_mode
                .advanced_query_for(&self.filter_text)
                .is_none_or(|advanced_query| !advanced_query.has_predicates())
            && !self.menu_syntax_object_selector_state.owns_main_list()
            && !self.menu_syntax_trigger_popup_state.owns_main_list()
            && !self
                .menu_syntax_mode
                .capture_composer_owns_input_for(trimmed)
            && !self.menu_syntax_mode.command_owns_input_for(trimmed);

        let request = if !can_collect {
            None
        } else if root_file_options.global_search_enabled
            && crate::file_search::should_search_root_files_for_intent(
                trimmed,
                root_file_options.query_intent,
            )
        {
            Some(RootFileSearchRequest::GlobalQuery {
                query: trimmed.to_string(),
            })
        } else if root_file_options.directory_browse_enabled
            && crate::file_search::looks_like_root_directory_browse_query(trimmed)
        {
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
                || self.root_file_provider_loading
                || self.root_file_search_mode.is_some();
            self.root_file_results.clear();
            self.root_file_search_query.clear();
            self.root_file_search_mode = None;
            self.root_file_search_loading = false;
            self.root_file_provider_loading = false;
            self.root_file_frame = None;
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
                let cached_results = self.cached_root_file_results_for_request(&request);
                if root_file_result_fingerprint(&self.root_file_results)
                    != root_file_result_fingerprint(&cached_results)
                {
                    self.root_file_results = cached_results;
                    self.root_file_search_loading = self.root_file_results.is_empty();
                    self.root_file_frame = None;
                    self.invalidate_grouped_cache();
                }
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
        let cached_results = self.cached_root_file_results_for_request(&request);
        self.root_file_results = cached_results;
        self.root_file_search_loading = self.root_file_results.is_empty();
        self.root_file_provider_loading = true;
        self.invalidate_grouped_cache();

        let cancel = crate::file_search::new_cancel_token();
        self.root_file_search_cancel = Some(cancel.clone());
        let publish_active_results = source_filters
            .includes(crate::menu_syntax::RootUnifiedSourceFilter::Files)
            || matches!(&request, RootFileSearchRequest::DirectoryBrowse { .. });
        let request_cache_key = request.cache_key();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(ROOT_FILE_SEARCH_DEBOUNCE_MS))
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
                        if emit_root_file_search_test_fixture(&query, &cancel, &tx) {
                            return;
                        }

                        let provider_query =
                            crate::file_search::root_file_provider_query_for_user_query(&query);
                        crate::file_search::search_files_streaming_with_options(
                            &provider_query,
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
                    Ok(crate::file_search::SearchEvent::Result(result)) => {
                        batch.push(result);
                    }
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
                    if publish_active_results {
                        app.apply_root_file_search_results_for_generation(
                            generation, batch, false, true, cx,
                        );
                    } else {
                        app.cache_root_file_search_results_for_generation(
                            generation,
                            request_cache_key,
                            batch,
                            true,
                        );
                    }
                })
            });
        })
        .detach();
    }
}

fn root_file_result_fingerprint(files: &[crate::file_search::FileResult]) -> Vec<&str> {
    files.iter().map(|file| file.path.as_str()).collect()
}

fn dedupe_root_file_results(
    results: Vec<crate::file_search::FileResult>,
) -> Vec<crate::file_search::FileResult> {
    let mut seen = std::collections::HashSet::new();
    results
        .into_iter()
        .filter(|file| seen.insert(file.path.clone()))
        .collect()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RootFileSearchTestFixture {
    query: String,
    #[serde(default = "default_root_file_test_delay_ms")]
    delay_ms: u64,
    results: Vec<RootFileSearchTestFixtureResult>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum RootFileSearchTestProvider {
    Single(RootFileSearchTestFixture),
    Multi {
        fixtures: Vec<RootFileSearchTestFixture>,
        #[serde(default)]
        passthrough_unmatched: bool,
    },
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RootFileSearchTestFixtureResult {
    path: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    file_type: Option<String>,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    modified: u64,
}

fn default_root_file_test_delay_ms() -> u64 {
    250
}

fn emit_root_file_search_test_fixture(
    query: &str,
    cancel: &crate::file_search::CancelToken,
    tx: &std::sync::mpsc::Sender<crate::file_search::SearchEvent>,
) -> bool {
    let Ok(raw) = std::env::var("SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER") else {
        return false;
    };
    let Ok(provider) = serde_json::from_str::<RootFileSearchTestProvider>(&raw) else {
        return false;
    };
    let fixture = match provider {
        RootFileSearchTestProvider::Single(fixture) => {
            if fixture.query != query {
                return false;
            }
            Some(fixture)
        }
        RootFileSearchTestProvider::Multi {
            fixtures,
            passthrough_unmatched,
        } => {
            let found = fixtures
                .into_iter()
                .find(|fixture| fixture.query == query);
            if found.is_none() && !passthrough_unmatched {
                let _ = tx.send(crate::file_search::SearchEvent::Done);
                return true;
            }
            found
        }
    };
    let Some(fixture) = fixture else {
        return false;
    };

    std::thread::sleep(std::time::Duration::from_millis(fixture.delay_ms));
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }

    for result in fixture.results {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }
        let _ = tx.send(crate::file_search::SearchEvent::Result(
            result.into_file_result(),
        ));
    }
    let _ = tx.send(crate::file_search::SearchEvent::Done);
    true
}

impl RootFileSearchTestFixtureResult {
    fn into_file_result(self) -> crate::file_search::FileResult {
        let name = self.name.unwrap_or_else(|| {
            std::path::Path::new(&self.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&self.path)
                .to_string()
        });
        crate::file_search::FileResult {
            path: self.path,
            name,
            size: self.size,
            modified: self.modified,
            file_type: match self.file_type.as_deref() {
                Some("directory") => crate::file_search::FileType::Directory,
                Some("application") => crate::file_search::FileType::Application,
                Some("image") => crate::file_search::FileType::Image,
                Some("document") => crate::file_search::FileType::Document,
                Some("audio") => crate::file_search::FileType::Audio,
                Some("video") => crate::file_search::FileType::Video,
                Some("other") => crate::file_search::FileType::Other,
                _ => crate::file_search::FileType::File,
            },
        }
    }
}

impl ScriptListApp {
    // ── Spine @file: subsearch ───────────────────────────────────────

    fn cancel_spine_file_subsearch(&mut self) {
        if let Some(cancel) = self.spine_file_search_cancel.take() {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn clear_spine_file_subsearch_state(&mut self, cx: &mut Context<Self>) {
        self.cancel_spine_file_subsearch();
        let had_state = !self.spine_file_search_query.is_empty()
            || !self.spine_file_search_results.is_empty()
            || self.spine_file_search_loading;
        self.spine_file_search_query.clear();
        self.spine_file_search_results.clear();
        self.spine_file_search_loading = false;
        if had_state {
            self.spine_file_search_generation =
                self.spine_file_search_generation.wrapping_add(1);
            self.invalidate_grouped_cache();
            cx.notify();
        }
    }

    pub(crate) fn active_spine_context_subsearch(
        &self,
    ) -> Option<(
        crate::spine::catalog_subsearch::ContextSubsearchSource,
        String,
    )> {
        if !self.spine_projection_owns_main_list() {
            return None;
        }
        let projection = self.spine_projection.as_ref()?;
        match &projection.active_segment_kind {
            crate::spine::SpineSegmentKind::ContextMention {
                context_type,
                sub_query,
            } => {
                let (source, query) =
                    crate::spine::catalog_subsearch::parse_context_subsearch(
                        context_type,
                        sub_query.as_deref(),
                    )?;
                Some((source, query.to_string()))
            }
            _ => None,
        }
    }

    pub(crate) fn maybe_start_spine_file_subsearch_for_current_projection(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let Some((source, query)) = self.active_spine_context_subsearch() else {
            self.clear_spine_file_subsearch_state(cx);
            return;
        };
        if source != crate::spine::catalog_subsearch::ContextSubsearchSource::File {
            self.clear_spine_file_subsearch_state(cx);
            return;
        }
        self.maybe_start_spine_file_subsearch(&query, cx);
    }

    fn maybe_start_spine_file_subsearch(
        &mut self,
        query: &str,
        cx: &mut Context<Self>,
    ) {
        let query = query.trim();
        if query.is_empty() {
            self.clear_spine_file_subsearch_state(cx);
            return;
        }
        if self.spine_file_search_query == query {
            return;
        }

        self.cancel_spine_file_subsearch();
        self.spine_file_search_generation =
            self.spine_file_search_generation.wrapping_add(1);
        let generation = self.spine_file_search_generation;
        self.spine_file_search_query = query.to_string();
        self.spine_file_search_loading = true;
        self.spine_file_search_results.clear();
        self.invalidate_grouped_cache();
        cx.notify();

        let cancel = crate::file_search::new_cancel_token();
        self.spine_file_search_cancel = Some(cancel.clone());
        let query_owned = query.to_string();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(
                    SPINE_FILE_SEARCH_DEBOUNCE_MS,
                ))
                .await;
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn({
                let cancel = cancel.clone();
                let query_owned = query_owned.clone();
                move || {
                    if emit_root_file_search_test_fixture(
                        &query_owned,
                        &cancel,
                        &tx,
                    ) {
                        return;
                    }
                    let provider_query =
                        crate::file_search::root_file_provider_query_for_user_query(
                            &query_owned,
                        );
                    crate::file_search::search_files_streaming_with_options(
                        &provider_query,
                        None,
                        crate::file_search::ROOT_FILE_SOURCE_LIMIT,
                        cancel,
                        crate::file_search::SearchFilesStreamingOptions::root_search(),
                        |event| {
                            let _ = tx.send(event);
                        },
                    );
                }
            });

            let mut batch = Vec::new();
            loop {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                match rx.try_recv() {
                    Ok(crate::file_search::SearchEvent::Result(result)) => {
                        batch.push(result);
                    }
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
                    app.apply_spine_file_subsearch_results(
                        generation,
                        batch,
                        cx,
                    );
                })
            });
        })
        .detach();
    }

    fn apply_spine_file_subsearch_results(
        &mut self,
        generation: u64,
        results: Vec<crate::file_search::FileResult>,
        cx: &mut Context<Self>,
    ) {
        if self.spine_file_search_generation != generation {
            return;
        }
        let results = dedupe_root_file_results(results);
        self.spine_file_search_results = results;
        self.spine_file_search_loading = false;
        self.spine_file_search_cancel = None;
        self.invalidate_grouped_cache();
        if matches!(self.current_view, AppView::ScriptList) {
            self.sync_list_state_for_filter_replacement();
            self.validate_selection_bounds(cx);
        }
        cx.notify();
    }
}
