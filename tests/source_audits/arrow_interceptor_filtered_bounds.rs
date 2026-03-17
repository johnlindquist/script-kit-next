use std::fs;

fn read_arrow_interceptor_source() -> String {
    fs::read_to_string("src/app_impl/startup_new_arrow.rs")
        .expect("Failed to read src/app_impl/startup_new_arrow.rs")
}

#[test]
fn test_emoji_picker_arrows_delegate_to_shared_navigation_helper() {
    let source = read_arrow_interceptor_source();

    let arm_start = source
        .find("AppView::EmojiPickerView { .. } =>")
        .expect("emoji picker arrow arm should exist");
    let arm = &source[arm_start..(arm_start + 1200).min(source.len())];

    assert!(
        arm.contains("this.navigate_emoji_picker(direction, cx);"),
        "emoji picker arrow navigation should call the shared helper"
    );
    assert!(
        !arm.contains("old_idx.saturating_sub"),
        "emoji picker arrow navigation should not re-introduce inline index math"
    );
}

#[test]
fn test_window_switcher_arrow_navigation_uses_filtered_count() {
    let source = read_arrow_interceptor_source();

    let arm_start = source
        .find("AppView::WindowSwitcherView")
        .expect("window switcher arrow arm should exist");
    let arm = &source[arm_start..(arm_start + 1200).min(source.len())];

    assert!(
        arm.contains("let filter_lower = filter.to_lowercase();"),
        "window switcher arrow navigation should derive its count from the active filter"
    );
    assert!(
        !arm.contains("let filtered_len = this.cached_windows.len();"),
        "window switcher arrow navigation must not use raw cached_windows.len() when a filter is active"
    );
}

#[test]
fn test_process_manager_arrow_navigation_uses_filtered_count() {
    let source = read_arrow_interceptor_source();

    let arm_start = source
        .find("AppView::ProcessManagerView")
        .expect("process manager arrow arm should exist");
    let arm = &source[arm_start..(arm_start + 1200).min(source.len())];

    assert!(
        arm.contains("let filter_lower = filter.to_lowercase();"),
        "process manager arrow navigation should derive its count from the active filter"
    );
    assert!(
        !arm.contains("let filtered_len = this.cached_processes.len();"),
        "process manager arrow navigation must not use raw cached_processes.len() when a filter is active"
    );
}

#[test]
fn test_search_ai_presets_arrow_navigation_clamps_to_filtered_count() {
    let source = read_arrow_interceptor_source();

    let arm_start = source
        .find("AppView::SearchAiPresetsView")
        .expect("search ai presets arrow arm should exist");
    let arm = &source[arm_start..(arm_start + 4000).min(source.len())];

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
    let source = read_arrow_interceptor_source();

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
    let source = read_arrow_interceptor_source();

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
