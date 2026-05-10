#[test]
fn root_unified_browser_history_config_is_opt_in_and_scoped() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let defaults = include_str!("../../src/config/defaults.rs");

    assert!(config_types.contains("pub struct UnifiedSearchBrowserHistoryConfig"));
    assert!(config_types.contains("pub enum BrowserHistoryProvider"));
    assert!(config_types.contains("fn browser_history_section_options("));
    assert!(config_schema.contains("browserHistory?: UnifiedSearchBrowserHistoryConfig"));
    assert!(config_schema.contains("export type BrowserHistoryProvider"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_ENABLED: bool = false"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MIN_QUERY_CHARS: usize = 4"));
}

#[test]
fn root_unified_browser_history_search_is_chromium_metadata_only_and_bounded() {
    let browser_history = include_str!("../../src/browser_history.rs");
    let root_search_fn = browser_history
        .split("pub(crate) fn search_root_browser_history_meta(")
        .nth(1)
        .and_then(|rest| rest.split("pub fn list_recent_history(").next())
        .expect("search_root_browser_history_meta should exist");
    let root_query_fn = browser_history
        .split("fn query_root_chromium_history_conn(")
        .nth(1)
        .and_then(|rest| rest.split("fn query_safari_history(").next())
        .expect("query_root_chromium_history_conn should exist before safari query");
    let root_db_fn = browser_history
        .split("fn query_root_chromium_history_db(")
        .nth(1)
        .and_then(|rest| rest.split("fn query_root_chromium_history_conn(").next())
        .expect("query_root_chromium_history_db should exist before root connection query");

    assert!(browser_history.contains("pub(crate) struct RootBrowserHistorySectionOptions"));
    assert!(browser_history.contains("pub(crate) struct RootBrowserHistorySearchHit"));
    assert!(browser_history.contains("root_browser_history_query_is_eligible("));
    assert!(root_search_fn.contains("ROOT_BROWSER_HISTORY_PROVIDERS"));
    assert!(root_db_fn.contains("copy_sqlite_db_snapshot("));
    assert!(root_query_fn.contains("FROM urls"));
    assert!(root_query_fn.contains("WHERE last_visit_time >= ?1"));
    assert!(root_query_fn.contains("title LIKE ?2 ESCAPE '\\'"));
    assert!(root_query_fn.contains("url LIKE ?2 ESCAPE '\\'"));
    assert!(root_query_fn.contains("LIMIT ?4"));
    assert!(!root_query_fn.contains("history_visits"));
    assert!(!root_query_fn.contains("moz_places"));
    assert!(!root_query_fn.contains("favicon"));
    assert!(!root_query_fn.contains("cookies"));
    assert!(!root_query_fn.contains("downloads"));
    assert!(!root_query_fn.contains("content"));
}

#[test]
fn root_unified_browser_history_safe_open_rejects_non_web_schemes() {
    let browser_history = include_str!("../../src/browser_history.rs");

    assert!(browser_history.contains("pub(crate) fn open_browser_history_url("));
    assert!(browser_history.contains("ensure_browser_history_url_is_http_or_https(url)?"));
    assert!(browser_history.contains("open::that(url)"));
    assert!(browser_history.contains("scheme.eq_ignore_ascii_case(\"http\")"));
    assert!(browser_history.contains("scheme.eq_ignore_ascii_case(\"https\")"));
}

#[test]
fn root_unified_browser_history_uses_passive_grouping_contract() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("fn append_root_browser_history_section("));
    assert!(grouping
        .contains("append_root_passive_section(grouped, flat_results, \"Browser History\", rows)"));
    assert!(grouping.contains("root_browser_history_query_is_eligible("));
    assert!(
        grouping.find("append_root_acp_history_section(")
            < grouping.find("append_root_browser_history_section("),
        "Browser History rows should be appended after AI Conversations"
    );
    assert!(
        grouping.contains("label.starts_with(\"Use \\\"\") && label.ends_with(\"\\\" with...\")"),
        "passive insertion should target the fallback section header, not the first fallback row"
    );
}

#[test]
fn root_unified_browser_history_result_is_stable_non_bindable_and_web_typed() {
    let types = include_str!("../../src/scripts/types.rs");
    let unified = include_str!("../../src/scripts/search/unified.rs");

    assert!(types.contains("pub struct BrowserHistoryMatch"));
    assert!(types.contains("BrowserHistory(BrowserHistoryMatch)"));
    assert!(types.contains("SearchResult::BrowserHistory(_) => None"));
    assert!(types.contains("SearchResult::BrowserHistory(_) => \"Open Page\""));
    assert!(types.contains("SearchResult::BrowserHistory(_) => (\"Web\", 0x38BDF8)"));
    assert!(types.contains("SearchResult::BrowserHistory(_) => Some(\"Browser History\")"));
    assert!(unified.contains("SearchResult::BrowserHistory(_) => 10"));
}

#[test]
fn root_unified_browser_history_enter_uses_safe_open_helper() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");

    assert!(selection.contains("SearchResult::BrowserHistory(browser_match)"));
    assert!(selection.contains("execute_root_browser_history_open("));
    assert!(selection.contains("crate::browser_history::open_browser_history_url(url)"));
    assert!(selection.contains("self.hide_main_and_reset(cx);"));
}
