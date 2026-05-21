#[test]
fn root_passive_frame_times_every_passive_source() {
    let filtering = include_str!("../../src/app_impl/filtering_cache.rs");
    let section = filtering
        .split("fn root_passive_frame_for_current_query(")
        .nth(1)
        .and_then(|rest| rest.split("fn root_file_frame_for_current_query(").next())
        .expect("root_passive_frame_for_current_query should exist");

    assert!(
        filtering.contains("fn timed_root_passive_source<T>("),
        "passive-source timing helper should exist"
    );
    assert!(
        filtering.contains("[PASSIVE_SOURCE_DONE]"),
        "passive-source timing logs should be parseable by benchmarks"
    );

    for source in [
        "notes",
        "todo",
        "clipboard_history",
        "dictation_history",
        "acp_history",
        "ai_vault",
        "browser_tabs",
        "browser_history",
    ] {
        assert!(
            section.contains(&format!("\"{source}\"")),
            "{source} should be timed in the root passive frame"
        );
    }
}

#[test]
fn implicit_browser_history_uses_cached_lookup_on_typing_path() {
    let filtering = include_str!("../../src/app_impl/filtering_cache.rs");
    let section = filtering
        .split("let browser_history_hits = timed_root_passive_source(")
        .nth(1)
        .and_then(|rest| rest.split("let frame = crate::RootPassiveFrame").next())
        .expect("browser history passive branch should exist");

    assert!(
        section.contains("explicit_browser_history"),
        "browser history branch must distinguish explicit from implicit source filters"
    );
    assert!(
        section.contains("search_root_browser_history_meta_cached"),
        "implicit browser history must use the cached/snapshot lookup"
    );
    assert!(
        section.find("search_root_browser_history_meta_direct")
            < section.find("search_root_browser_history_meta_cached"),
        "direct browser history lookup should be confined to the explicit branch"
    );
}

#[test]
fn implicit_browser_tabs_uses_cached_lookup_on_typing_path() {
    let filtering = include_str!("../../src/app_impl/filtering_cache.rs");
    let section = filtering
        .split("let browser_tab_hits =")
        .nth(1)
        .and_then(|rest| {
            rest.split("let browser_history_hits = timed_root_passive_source(")
                .next()
        })
        .expect("browser tabs passive branch should exist");

    assert!(
        section.contains("explicit_browser_tabs"),
        "browser tabs branch must distinguish explicit from implicit source filters"
    );
    assert!(
        section.contains("search_root_browser_tabs_meta_cached"),
        "implicit browser tabs must use the cached/snapshot lookup"
    );
    assert!(
        section.find("search_root_browser_tabs_meta_direct")
            < section.find("search_root_browser_tabs_meta_cached"),
        "direct browser tabs lookup should be confined to the explicit branch"
    );
}

#[test]
fn cached_browser_tabs_lookup_is_nonblocking() {
    let tabs = include_str!("../../src/browser_tabs.rs");
    let cached_section = tabs
        .split("pub(crate) fn search_root_browser_tabs_meta_cached")
        .nth(1)
        .and_then(|rest| {
            rest.split("pub(crate) fn search_root_browser_tabs_meta_direct")
                .next()
        })
        .expect("cached browser tabs helper should exist");

    for forbidden in [
        "ensure_root_browser_tabs_refresh",
        "std::thread::spawn",
        ".join(",
        ".lock().unwrap",
        ".lock().expect",
        "par_chunks",
        "par_iter",
    ] {
        assert!(
            !cached_section.contains(forbidden),
            "cached browser-tabs lookup must not contain {forbidden}"
        );
    }

    let internal_section = tabs
        .split("fn search_root_browser_tabs_internal(")
        .nth(1)
        .and_then(|rest| {
            rest.split("#[allow(dead_code)]\nfn cached_root_browser_tabs_snapshot")
                .next()
        })
        .expect("browser tabs internal lookup should exist");
    assert!(
        tabs.contains("RootBrowserTabsLookupMode::CachedOnly"),
        "browser tabs lookup should make cached-only mode explicit"
    );
    assert!(
        internal_section.contains("RootBrowserTabsLookupMode::RefreshThenCached"),
        "browser tabs internal lookup should isolate refresh-capable mode"
    );
}

#[test]
fn root_browser_tabs_fuzzy_search_is_sequential_on_ui_path() {
    let tabs = include_str!("../../src/browser_tabs.rs");
    let section = tabs
        .split("fn root_fuzzy_search_browser_tabs(")
        .nth(1)
        .and_then(|rest| {
            rest.split("#[allow(dead_code)]\nfn root_tab_provider_is_enabled")
                .next()
        })
        .expect("root browser tabs fuzzy search should exist");

    assert!(
        !section.contains("par_chunks"),
        "root browser tab fuzzy search must not use Rayon on the UI path"
    );
    assert!(
        !section.contains("par_iter"),
        "root browser tab fuzzy search must not use Rayon on the UI path"
    );
    assert!(
        section.contains(".iter()"),
        "root browser tab fuzzy search should remain a simple sequential scan"
    );
}

#[test]
fn root_grouped_cache_tracks_browser_passive_generations() {
    let filtering = include_str!("../../src/app_impl/filtering_cache.rs");
    let section = filtering
        .split("pub(crate) fn get_grouped_results_cached(")
        .nth(1)
        .and_then(|rest| {
            rest.split("pub(crate) fn cached_grouped_results_snapshot(")
                .next()
        })
        .expect("get_grouped_results_cached should exist");

    assert!(
        section.contains("browser-tabs-gen={browser_tabs_generation}"),
        "grouped cache key should include browser tabs passive snapshot generation"
    );
    assert!(
        section.contains("browser-history-gen={browser_history_generation}"),
        "grouped cache key should include browser history passive snapshot generation"
    );
}
