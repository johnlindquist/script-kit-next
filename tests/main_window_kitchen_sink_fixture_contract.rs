const MAIN_FIXTURE_SOURCE: &str = include_str!("../src/main_sections/kitchen_sink_fixture.rs");
const MAIN_RS_SOURCE: &str = include_str!("../src/main.rs");
const FILTERING_CACHE_SOURCE: &str = include_str!("../src/app_impl/filtering_cache.rs");

#[test]
fn main_window_kitchen_sink_fixture_has_stable_identity_and_manifest() {
    assert!(MAIN_RS_SOURCE.contains("include!(\"main_sections/kitchen_sink_fixture.rs\");"));
    assert!(MAIN_FIXTURE_SOURCE.contains(
        "pub(crate) const MAIN_WINDOW_KITCHEN_SINK_FIXTURE_ID: &str = \"main-window-kitchen-sink\";"
    ));
    assert!(MAIN_FIXTURE_SOURCE.contains("main_window_kitchen_sink_feature_manifest"));

    for feature in [
        "shell:content-insets",
        "search:input-text",
        "list:section-header",
        "list:source-status-row",
        "list:scroll-overflow",
        "row:selected",
        "row:hover-worthy",
        "row:long-title",
        "row:long-description",
        "row:empty-description",
        "icon:svg",
        "icon:app",
        "icon:missing-fallback",
        "metadata:source",
        "metadata:keycap",
        "footer:run-actions-ai",
        "header-info:pills",
        "empty:no-match",
    ] {
        assert!(
            MAIN_FIXTURE_SOURCE.contains(feature),
            "main fixture manifest must include {feature}"
        );
    }
}

#[test]
fn main_window_kitchen_sink_uses_real_grouped_list_cache() {
    assert!(MAIN_FIXTURE_SOURCE.contains("main_window_kitchen_sink_grouped_results"));
    assert!(MAIN_FIXTURE_SOURCE.contains("GroupedListItem::SectionHeader"));
    assert!(MAIN_FIXTURE_SOURCE.contains("GroupedListItem::Status"));
    assert!(MAIN_FIXTURE_SOURCE.contains("GroupedListItem::Item"));
    assert!(MAIN_FIXTURE_SOURCE.contains("store_grouped_results"));
    assert!(MAIN_FIXTURE_SOURCE.contains("store_filtered_results"));
    assert!(FILTERING_CACHE_SOURCE.contains("MAIN_WINDOW_KITCHEN_SINK_QUERY"));
    assert!(FILTERING_CACHE_SOURCE.contains("main_window_kitchen_sink_grouped_results"));
    assert!(!MAIN_FIXTURE_SOURCE.contains("panel:main-window-kitchen-sink"));
}

#[test]
fn main_window_kitchen_sink_exercises_many_rows_and_sections() {
    assert!(MAIN_FIXTURE_SOURCE.contains("let sections = ["));
    assert!(MAIN_FIXTURE_SOURCE.contains("\"Suggested\""));
    assert!(MAIN_FIXTURE_SOURCE.contains("\"Built-ins\""));
    assert!(MAIN_FIXTURE_SOURCE.contains("\"Apps\""));
    assert!(MAIN_FIXTURE_SOURCE.contains("\"Skills\""));
    assert!(MAIN_FIXTURE_SOURCE.contains("\"Files\""));
    assert!(MAIN_FIXTURE_SOURCE.contains("\"Diagnostics\""));
    assert!(MAIN_FIXTURE_SOURCE.contains("for item_index in 0..5"));
    assert!(MAIN_FIXTURE_SOURCE.contains("MAIN_WINDOW_KITCHEN_SINK_NO_MATCH_QUERY"));
}

#[test]
fn main_window_kitchen_sink_openers_are_available_to_dev_style_tool() {
    assert!(MAIN_FIXTURE_SOURCE.contains("open_main_window_kitchen_sink_fixture"));
    assert!(MAIN_FIXTURE_SOURCE.contains("open_main_window_no_match_kitchen_sink_fixture"));
    assert!(MAIN_FIXTURE_SOURCE.contains("AppView::ScriptList"));
    assert!(MAIN_FIXTURE_SOURCE.contains("set_main_window_visible(true)"));
}
