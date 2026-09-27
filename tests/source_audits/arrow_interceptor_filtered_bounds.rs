use std::fs;

fn read_live_startup_source() -> String {
    fs::read_to_string("src/app_impl/startup.rs").expect("Failed to read src/app_impl/startup.rs")
}

#[test]
fn test_script_list_source_filters_block_input_history_recall() {
    let live_source = read_live_startup_source();
    let helper_source = fs::read_to_string("src/app_impl/filter_input_core.rs")
        .expect("Failed to read src/app_impl/filter_input_core.rs");

    assert!(
        helper_source.contains("fn source_filter_mode_blocks_input_history_recall")
            && helper_source.contains(".advanced_query_for(&self.filter_text)")
            && helper_source.contains("query.has_source_filters()"),
        "ScriptList history suppression should be a shared source-filter predicate, not duplicated arrow-handler parsing"
    );

    assert!(
        live_source.contains("let source_filter_mode =")
            && live_source.contains("this.source_filter_mode_blocks_input_history_recall()")
            && live_source.contains("if !source_filter_mode && (in_history || at_top_of_list)")
            && live_source.contains("if !source_filter_mode && in_history"),
        "live arrow path must route source-filter Up/Down to list movement instead of launcher input history"
    );
}

#[test]
fn test_search_ai_presets_arrow_navigation_clamps_to_filtered_count() {
    let source = read_live_startup_source();

    let arm_start = source
        .find("AppView::SearchAiPresetsView")
        .expect("search ai presets arrow arm should exist");
    let arm_len = source[arm_start + 1..]
        .find("AppView::")
        .map_or(source.len() - arm_start, |next| next + 1);
    let arm = &source[arm_start..arm_start + arm_len];

    assert!(
        arm.contains("*selected_index + 1 < filtered_len"),
        "search ai presets down-arrow must clamp to filtered_len"
    );
    assert!(
        arm.contains("*selected_index >= filtered_len"),
        "search ai presets must clamp index when filter shrinks result set"
    );
}

#[test]
fn test_app_launcher_arrow_navigation_scrolls_selected_item() {
    let source = read_live_startup_source();

    let arm_start = source
        .find("AppView::AppLauncherView")
        .expect("AppLauncherView arrow handler not found");
    let arm = &source[arm_start..(arm_start + 2000).min(source.len())];

    assert!(
        arm.contains("list_scroll_handle"),
        "AppLauncherView should use list_scroll_handle for keyboard navigation"
    );
    assert!(
        arm.contains("scroll_to_item"),
        "AppLauncherView keyboard navigation should keep the selected row visible"
    );
}

#[test]
fn test_favorites_browse_arrow_navigation_clamps_to_filtered_count() {
    let source = read_live_startup_source();

    let arm_start = source
        .find("AppView::FavoritesBrowseView")
        .expect("favorites browse arrow arm should exist");
    let arm = &source[arm_start..(arm_start + 4500).min(source.len())];

    assert!(
        arm.contains("*selected_index + 1 < filtered_len"),
        "favorites browse down-arrow must clamp to filtered_len"
    );
    assert!(
        arm.contains("*selected_index >= filtered_len"),
        "favorites browse must clamp index when filter shrinks result set"
    );
}
