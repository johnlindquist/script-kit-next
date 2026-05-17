#[test]
fn root_unified_browser_tabs_config_is_opt_in_and_bounded() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let defaults = include_str!("../../src/config/defaults.rs");

    assert!(config_types.contains("pub struct UnifiedSearchBrowserTabsConfig"));
    assert!(config_types.contains("pub enum BrowserTabProvider"));
    assert!(config_types.contains("fn browser_tabs_section_options("));
    assert!(config_schema.contains("browserTabs?: UnifiedSearchBrowserTabsConfig"));
    assert!(config_schema.contains("export type BrowserTabProvider"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_ENABLED: bool = false"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_MIN_QUERY_CHARS: usize = 3"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_CACHE_TTL_MS: u64 = 10_000"));
}

#[test]
fn root_unified_browser_tabs_search_uses_cached_metadata_only_snapshot() {
    let browser_tabs = include_str!("../../src/browser_tabs.rs");
    let root_search_fn = browser_tabs
        .split("pub(crate) fn search_root_browser_tabs_meta(")
        .nth(1)
        .and_then(|rest| rest.split("pub(crate) fn focus_root_browser_tab(").next())
        .expect("search_root_browser_tabs_meta should exist before focus helper");

    assert!(browser_tabs.contains("pub(crate) struct RootBrowserTabsSectionOptions"));
    assert!(browser_tabs.contains("pub(crate) struct RootBrowserTabSearchHit"));
    assert!(browser_tabs.contains("root_browser_tabs_query_is_eligible("));
    assert!(browser_tabs.contains("static ROOT_BROWSER_TAB_SNAPSHOT"));
    assert!(root_search_fn.contains("ensure_root_browser_tabs_refresh("));
    assert!(root_search_fn.contains("cached_root_browser_tabs_snapshot(options.cache_ttl_ms)"));
    assert!(root_search_fn.contains(".take(options.scan_limit)"));
    assert!(root_search_fn.contains(".take(options.max_results)"));
    assert!(root_search_fn.contains("root_tab_provider_is_enabled("));
    assert!(!root_search_fn.contains("list_open_tabs("));
    assert!(!root_search_fn.contains("fetch_favicons"));
    assert!(!root_search_fn.contains("open::that"));
}

#[test]
fn root_unified_browser_tabs_uses_passive_grouping_order_and_score_cap() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("pub const ROOT_PASSIVE_RESULT_SCORE_BASE: i32 = 100_000"));
    assert!(grouping.contains("pub(crate) fn root_passive_result_score(rank: usize) -> i32"));
    assert!(grouping.contains("fn append_root_browser_tabs_section("));
    assert!(grouping
        .contains("append_root_passive_section(grouped, flat_results, \"Browser Tabs\", rows"));
    assert!(
        grouping.find("append_recent_root_file_section(")
            < grouping.find("append_root_browser_tabs_section("),
        "Browser Tabs rows should be appended after root/recent files"
    );
    assert!(
        grouping.find("append_root_browser_tabs_section(")
            < grouping.find("append_root_notes_section("),
        "Browser Tabs rows should be appended before Notes"
    );
    assert!(
        grouping.contains("score: root_passive_result_score(rank)"),
        "passive rows should use the capped passive score helper"
    );
}

#[test]
fn root_unified_browser_tabs_result_is_stable_non_bindable_and_tab_typed() {
    let types = include_str!("../../src/scripts/types.rs");
    let unified = include_str!("../../src/scripts/search/unified.rs");

    assert!(types.contains("pub struct BrowserTabMatch"));
    assert!(types.contains("BrowserTab(BrowserTabMatch)"));
    assert!(types.contains("SearchResult::BrowserTab(_) => None"));
    assert!(types.contains("SearchResult::BrowserTab(bm) => Some(bm.hit.stable_key.clone())"));
    assert!(types.contains("SearchResult::BrowserTab(_) => \"Switch to Tab\""));
    assert!(types.contains("SearchResult::BrowserTab(_) => (\"Tab\", 0x06B6D4)"));
    assert!(types.contains("SearchResult::BrowserTab(_) => Some(\"Browser Tabs\")"));
    assert!(unified.contains("SearchResult::BrowserTab(_) => 7"));
    assert!(unified.contains("SearchResult::Note(_) => 8"));
}

#[test]
fn root_unified_browser_tabs_enter_switches_existing_tab_without_opening_url() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");
    let browser_tabs = include_str!("../../src/browser_tabs.rs");

    assert!(selection.contains("SearchResult::BrowserTab(browser_tab_match)"));
    assert!(selection.contains("execute_root_browser_tab_switch("));
    assert!(selection.contains("crate::browser_tabs::focus_root_browser_tab(hit)"));
    assert!(selection.contains("self.hide_main_and_reset(cx);"));
    assert!(browser_tabs.contains("pub(crate) fn focus_root_browser_tab("));
    assert!(browser_tabs.contains("activate_tab(&hit.tab)"));
    assert!(!selection.contains("open::that(&browser_tab_match.hit.url)"));
}
