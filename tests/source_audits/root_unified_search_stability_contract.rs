use std::fs;

#[test]
fn global_root_file_search_does_not_stream_into_active_frame() {
    let source = fs::read_to_string("src/app_impl/root_file_search.rs")
        .expect("read src/app_impl/root_file_search.rs");
    let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        normalized.contains("fn cache_root_file_search_results_for_generation("),
        "global provider completion should have a cache-only path"
    );
    assert!(
        normalized.contains("let publish_active_results =")
            && normalized
                .contains("matches!(&request, RootFileSearchRequest::DirectoryBrowse { .. })"),
        "only explicit directory browse should be allowed to publish into the active frame"
    );
    assert!(
        normalized.contains("app.cache_root_file_search_results_for_generation( generation, request_cache_key, batch, true, );"),
        "global provider completion should warm cache instead of applying visible rows"
    );
    assert!(
        !normalized.contains("publish_partial_results"),
        "root global file search must not publish partial result batches"
    );
}

#[test]
fn selection_snapshots_use_stable_selection_keys_not_history_memory() {
    let app_state =
        fs::read_to_string("src/main_sections/app_state.rs").expect("read app_state.rs");
    let types = fs::read_to_string("src/scripts/types.rs").expect("read src/scripts/types.rs");

    assert!(types.contains("pub fn stable_selection_key(&self) -> Option<String>"));
    assert!(
        app_state.contains("grouped_index_for_stable_selection_key")
            && app_state.contains("result.stable_selection_key()")
            && !app_state.contains("grouped_index_for_history_result_key"),
        "selection restoration should use selection identity, not input-history identity"
    );
    assert!(
        types.contains(
            "SearchResult::Fallback(fm) => Some(format!(\"fallback/{}\", fm.fallback.name()))"
        ) && types.contains("SearchResult::Fallback(_) | SearchResult::Agent(_) => None"),
        "fallback rows need selection keys without becoming input-history promotion keys"
    );
}

#[test]
fn grouped_cache_read_is_pure_before_recent_file_refresh() {
    let filtering = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("read src/app_impl/filtering_cache.rs");

    let cache_check = filtering
        .find(".has_grouped_results_for(&self.computed_filter_text)")
        .expect("grouped cache check should exist");
    let recent_refresh = filtering
        .find("self.refresh_root_recent_file_results();")
        .expect("recent file refresh should exist");

    assert!(
        cache_check < recent_refresh,
        "grouped-result cache hits should return before refreshing recent files"
    );
}

#[test]
fn main_window_preflight_exposes_selection_key_and_frame_fingerprint() {
    let types =
        fs::read_to_string("src/main_window_preflight/types.rs").expect("read preflight types");
    let build =
        fs::read_to_string("src/main_window_preflight/build.rs").expect("read preflight builder");

    assert!(types.contains("pub selected_result_key: Option<String>"));
    assert!(types.contains("pub visible_result_key_fingerprint: String"));
    assert!(types.contains("pub visible_result_count: usize"));
    assert!(build.contains("result.stable_selection_key()"));
    assert!(build.contains("visible_result_keys(app).join(\"|\")"));
    assert!(build.contains("selected_result_key = ?receipt.selected_result_key"));
}
