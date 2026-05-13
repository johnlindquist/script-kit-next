#[test]
fn source_filter_parser_is_inline_and_capture_safe() {
    let parse = include_str!("../../src/menu_syntax/parse.rs");
    let query = include_str!("../../src/menu_syntax/query.rs");
    let mode = include_str!("../../src/menu_syntax/mode.rs");
    let payload = include_str!("../../src/menu_syntax/payload.rs");

    assert!(query.contains("pub fn parse_filter_query("));
    assert!(query.contains("pub fn parse_source_filter_query("));
    assert!(query.contains("if input.starts_with(':')"));
    assert!(payload.contains("pub const SOURCE_HEAD_SPECS"));
    assert!(payload.contains("pub fn source_for_head(head_with_colon: &str)"));
    for alias in [
        "canonical: \"files:\"",
        "short: Some(\"f:\")",
        "canonical: \"notes:\"",
        "short: Some(\"n:\")",
        "canonical: \"clipboard:\"",
        "short: Some(\"c:\")",
        "canonical: \"conversations:\"",
        "short: Some(\"ai:\")",
        "canonical: \"commands:\"",
        "short: Some(\"cmd:\")",
        "canonical: \"tabs:\"",
        "short: Some(\"t:\")",
        "canonical: \"history:\"",
        "short: Some(\"h:\")",
        "canonical: \"dictation:\"",
        "short: Some(\"d:\")",
        "canonical: \"windows:\"",
        "short: Some(\"w:\")",
    ] {
        assert!(
            payload.contains(alias),
            "source-filter alias table should contain {alias}"
        );
    }
    assert!(!payload.contains("canonical: \"processes:\""));
    assert!(!payload.contains("short: Some(\"p:\")"));
    assert!(query.contains("source_filters.exclude(source)"));
    assert!(query.contains("source_filters.insert(source)"));

    let source_filter_route = parse
        .rfind("if let Some(query) = parse_filter_query(input)")
        .expect("parse should route inline source filters");
    let capture_keyword_route = parse
        .rfind("return finalize_capture(input, registered_capture_targets);")
        .expect("keyword capture route should exist");
    assert!(
        capture_keyword_route < source_filter_route,
        "keyword capture syntax must keep owning input before inline source filters are considered"
    );
    assert!(mode.contains("free_text_for_search"));
    assert!(mode.contains("query.free_text.as_str()"));
}

#[test]
fn source_filters_are_frame_keyed_and_gate_async_sources() {
    let app_state = include_str!("../../src/main_sections/app_state.rs");
    let filtering = include_str!("../../src/app_impl/filtering_cache.rs");
    let root_file = include_str!("../../src/app_impl/root_file_search.rs");

    assert!(app_state
        .contains("pub(crate) source_filters: crate::menu_syntax::RootUnifiedSourceFilterSet"));
    assert!(filtering.contains("source_filters: source_filters.clone()"));
    assert!(filtering
        .contains("source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Notes)"));
    assert!(filtering.contains(
        "source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory)"
    ));
    assert!(filtering.contains(
        "source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Conversations)"
    ));
    assert!(filtering.contains(
        "source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs)"
    ));
    assert!(filtering.contains(".allows(crate::menu_syntax::RootUnifiedSourceFilter::Files))"));
    assert!(filtering.contains("clipboard_history_options.enabled = true;"));
    assert!(filtering.contains("notes_options.enabled = true;"));
    assert!(filtering.contains("dictation_history_options.enabled = true;"));
    assert!(filtering.contains("browser_tabs_options.enabled = true;"));
    assert!(filtering.contains("browser_history_options.enabled = true;"));
    assert!(filtering.contains("acp_history_options.enabled = true;"));
    assert!(filtering
        .contains("crate::notes::search_root_notes_meta_direct(search_text, notes_options)"));
    assert!(
        filtering.contains("crate::clipboard_history::search_root_clipboard_history_meta_direct(")
    );
    assert!(filtering.contains("crate::dictation::search_root_dictation_history_direct("));
    assert!(filtering.contains("crate::ai::acp::history::search_history_direct("));
    assert!(filtering.contains("search_root_browser_tabs_meta_direct"));
    assert!(filtering.contains("search_root_browser_history_meta_direct"));
    assert!(filtering.contains(".is_some_and(|query| query.has_source_filters())"));

    assert!(root_file.contains("free_text_for_search(&self.menu_syntax_mode, query)"));
    assert!(root_file
        .contains("!source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Files)"));
    assert!(root_file.contains("root_file_options.files_enabled = true;"));
    assert!(root_file.contains("let publish_active_results = source_filters"));
    assert!(root_file.contains(".includes(crate::menu_syntax::RootUnifiedSourceFilter::Files)"));
    assert!(root_file.contains("|| matches!(&request, RootFileSearchRequest::DirectoryBrowse"));
    assert!(root_file.contains("self.root_file_frame = None;"));
    assert!(root_file.contains(".is_none_or(|advanced_query| !advanced_query.has_predicates())"));
    assert!(root_file.contains("self.cached_root_file_results_for_request(&request)"));
    assert!(root_file.contains("root_file_result_fingerprint(&self.root_file_results)"));
}

#[test]
fn explicit_source_filters_raise_passive_source_caps() {
    let filtering = include_str!("../../src/app_impl/filtering_cache.rs");

    assert!(filtering.contains(
        "let explicit_source_result_target = root_passive_result_limits.max_total_results"
    ));
    for source in [
        "notes_options.max_results",
        "clipboard_history_options.max_results",
        "dictation_history_options.max_results",
        "acp_history_options.max_results",
        "browser_tabs_options.max_results",
        "browser_history_options.max_results",
    ] {
        assert!(
            filtering.contains(source) && filtering.contains(".max(explicit_source_result_target)"),
            "explicit source filters must raise {source} above passive preview defaults"
        );
    }
}

#[test]
fn grouping_suppresses_primary_and_disallowed_sources_when_filter_active() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(
        grouping.contains("root_source_filters: &crate::menu_syntax::RootUnifiedSourceFilterSet")
    );
    assert!(grouping.contains("filter_grouped_results_by_root_sources"));
    assert!(grouping.contains("root_unified_source()"));
    assert!(grouping.contains(
        "root_source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Files)"
    ));
    assert!(grouping.contains("root_source_filters.active(),"));
    assert!(grouping.contains("let handoff = if suppress_handoff"));
    assert!(grouping.contains(
        "root_source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Notes)"
    ));
    assert!(grouping.contains("root_source_filters\n                    .allows(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory)"));
    assert!(grouping.contains(
        "root_source_filters\n                    .allows(crate::menu_syntax::RootUnifiedSourceFilter::Conversations)"
    ));
    assert!(grouping.contains(
        "root_source_filters\n                    .allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserHistory)"
    ));
    assert!(grouping.contains(
        "root_source_filters\n                    .allows(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs)"
    ));
    assert!(grouping.contains("append_base_source_status_rows"));
    assert!(grouping.contains("root_source_filters.positive_includes()"));
    assert!(grouping.contains("GroupedListItem::Status(source_chip_result_status("));
    assert!(grouping.contains("if limit == 0 && !explicit_source_filter"));
    assert!(grouping.contains(
        "root_source_filters\n                        .includes(crate::menu_syntax::RootUnifiedSourceFilter::BrowserTabs)"
    ));
    assert!(grouping.contains(
        "root_source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Windows)"
    ));
}

#[test]
fn preflight_receipt_exposes_source_filter_state_for_agentic_proof() {
    let types = include_str!("../../src/main_window_preflight/types.rs");
    let build = include_str!("../../src/main_window_preflight/build.rs");

    assert!(types.contains("pub computed_search_text: String"));
    assert!(types.contains("pub source_filters: Vec<String>"));
    assert!(types.contains("pub filter_indicators: Vec<crate::menu_syntax::FilterIndicator>"));
    assert!(build.contains("free_text_for_search(&app.menu_syntax_mode, &app.filter_text)"));
    assert!(build.contains("query.source_filters.labels()"));
    assert!(build.contains("query.filter_indicators()"));
    assert!(build.contains("source_filters: frame"));
}

#[test]
fn colon_opens_discoverability_picker_while_source_filters_do_not_open_hint() {
    let trigger = include_str!("../../src/menu_syntax/trigger_picker.rs");
    let popup = include_str!("../../src/app_impl/menu_syntax_trigger_popup.rs");
    let hint = include_str!("../../src/menu_syntax/main_hint.rs");
    let render = include_str!("../../src/render_script_list/mod.rs");
    let prompt_handler = include_str!("../../src/prompt_handler/mod.rs");

    assert!(trigger.contains("SOURCE_HEAD_SPECS"));
    assert!(popup.contains("fn source_filter_query_does_not_open_power_popup()"));
    assert!(hint.contains("if query.is_source_filter_only()"));
    assert!(hint.contains("return None;"));
    assert!(hint.contains("pub fn active_head_is_source_filter(raw: &str) -> bool"));
    assert!(render.contains("!crate::menu_syntax::main_hint::active_head_is_source_filter"));
    assert!(prompt_handler.contains("source_head_has_results"));
    assert!(prompt_handler.contains("detector_owns_head && !source_head_has_results"));
    assert!(render.contains("There are no search results with this filter applied."));
    assert!(render.contains("query.has_source_filters() || query.has_predicates()"));
}
