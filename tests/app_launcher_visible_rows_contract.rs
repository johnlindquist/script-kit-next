//! Source-level contract for AURP-18 App Launcher row projection.
//!
//! App Launcher filtering should have one visible-row owner shared by render,
//! keyboard navigation, getState counts, and getElements rows.

const APP_LAUNCHER: &str = include_str!("../src/render_builtins/app_launcher.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

// doc-anchor-removed: [[removed-docs Surface Matrix]]
#[test]
fn app_launcher_declares_visible_row_helper_family() {
    for required in [
        "fn app_launcher_filtered_entries",
        "fn app_launcher_visible_row_names(&self, filter: &str) -> Vec<String>",
        "fn app_launcher_dataset_and_visible_counts(&self, filter: &str) -> (usize, usize)",
        "fn app_launcher_selected_visible_entry(",
        "fn app_launcher_visible_target_rows(",
        "app.name.to_lowercase().contains(&filter_lower)",
    ] {
        assert!(
            APP_LAUNCHER.contains(required),
            "app launcher visible-row helper family must contain: {required}"
        );
    }
}

#[test]
fn app_launcher_render_and_keyboard_use_visible_entry_helper() {
    let render_body = source_between(
        APP_LAUNCHER,
        "fn render_app_launcher(",
        "        let content = div()",
    );

    assert!(
        render_body
            .matches("app_launcher_filtered_entries(")
            .count()
            >= 2,
        "render and keyboard paths should both use app_launcher_filtered_entries"
    );
    assert!(
        APP_LAUNCHER.contains("app_launcher_dataset_and_visible_counts(&current_filter)"),
        "wheel reanchor should use the named count helper"
    );
}

#[test]
fn app_launcher_state_and_elements_use_visible_row_helpers() {
    let elements_body = source_between(
        COLLECT_ELEMENTS,
        "AppView::AppLauncherView {\n                filter,",
        "\n            AppView::WindowSwitcherView",
    );
    assert!(
        elements_body.contains("let rows = self.app_launcher_visible_row_names(filter);"),
        "getElements must read App Launcher rows from the shared helper"
    );

    let state_body = source_between(
        PROMPT_HANDLER,
        "AppView::AppLauncherView {\n                        filter,",
        "\n                    // P0 FIX: View state only - data comes from self.cached_windows",
    );
    assert!(
        state_body.contains("self.app_launcher_dataset_and_visible_counts(filter)"),
        "getState must read App Launcher counts from the shared helper"
    );
}

#[test]
fn app_launcher_tab_ai_targets_use_visible_row_helpers() {
    let arm_body = source_between(
        TAB_AI_MODE,
        "AppView::AppLauncherView {\n                filter,",
        "\n            AppView::ProcessManagerView",
    );

    assert!(
        TAB_AI_MODE.contains("fn tab_ai_target_from_app_launcher_row("),
        "Tab AI target shaping should have a named App Launcher adapter"
    );
    assert!(
        arm_body.contains(".app_launcher_selected_visible_entry(filter, *selected_index)"),
        "focused Tab AI target must resolve selected_index against filtered visible rows"
    );
    assert!(
        arm_body.contains(".app_launcher_visible_target_rows(filter, TAB_AI_VISIBLE_TARGET_LIMIT)"),
        "visible Tab AI targets must come from the shared App Launcher row projection"
    );
    assert!(
        !arm_body.contains("self.apps.get(*selected_index)"),
        "App Launcher Tab AI must not read selected_index from the raw app dataset"
    );
    assert!(
        !arm_body.contains("self.apps\n                    .iter()"),
        "App Launcher Tab AI must not build visible targets from the raw app dataset"
    );
}
