#[test]
fn source_filter_parser_is_inline_and_capture_safe() {
    let parse = include_str!("../../src/menu_syntax/parse.rs");
    let query = include_str!("../../src/menu_syntax/query.rs");
    let mode = include_str!("../../src/menu_syntax/mode.rs");

    assert!(query.contains("pub fn parse_source_filter_query("));
    for alias in [
        "\"f\" | \"file\" | \"files\"",
        "\"n\" | \"note\" | \"notes\"",
        "\"c\" | \"clip\" | \"clips\" | \"clipboard\"",
    ] {
        assert!(
            query.contains(alias),
            "source-filter alias table should contain {alias}"
        );
    }

    let source_filter_route = parse
        .rfind("if let Some(query) = parse_source_filter_query(input)")
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
    assert!(filtering
        .contains("source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Files)"));
    assert!(filtering.contains("let allow_other_passive = !source_filters.active();"));
    assert!(filtering.contains(".is_some_and(|query| query.has_source_filters())"));

    assert!(root_file.contains("free_text_for_search(&self.menu_syntax_mode, query)"));
    assert!(root_file
        .contains("!source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Files)"));
    assert!(root_file.contains(".is_none_or(|advanced_query| !advanced_query.has_predicates())"));
    assert!(root_file.contains("self.cached_root_file_results_for_request(&request)"));
    assert!(root_file.contains("root_file_result_fingerprint(&self.root_file_results)"));
}

#[test]
fn grouping_suppresses_primary_and_disallowed_sources_when_filter_active() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(
        grouping.contains("root_source_filters: &crate::menu_syntax::RootUnifiedSourceFilterSet")
    );
    assert!(grouping.contains("if root_source_filters.active()"));
    assert!(grouping.contains("(Vec::new(), Vec::new())"));
    assert!(grouping.contains(
        "root_source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Files)"
    ));
    assert!(grouping.contains("root_source_filters.active(),"));
    assert!(grouping.contains("let handoff = if suppress_handoff"));
    assert!(grouping.contains(
        "root_source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Notes)"
    ));
    assert!(grouping.contains("root_source_filters\n                    .allows(crate::menu_syntax::RootUnifiedSourceFilter::ClipboardHistory)"));
}

#[test]
fn preflight_receipt_exposes_source_filter_state_for_agentic_proof() {
    let types = include_str!("../../src/main_window_preflight/types.rs");
    let build = include_str!("../../src/main_window_preflight/build.rs");

    assert!(types.contains("pub computed_search_text: String"));
    assert!(types.contains("pub source_filters: Vec<String>"));
    assert!(build.contains("free_text_for_search(&app.menu_syntax_mode, &app.filter_text)"));
    assert!(build.contains("query.source_filters.labels()"));
    assert!(build.contains("source_filters: frame"));
}
