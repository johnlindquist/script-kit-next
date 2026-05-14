//! Source-level contract for Settings visible-row ownership and automation.

const SETTINGS: &str = include_str!("../src/render_builtins/settings.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const MATRIX: &str = include_str!("../scripts/agentic/filterable-surface-matrix.ts");

#[test]
fn settings_defines_visible_row_helper_family() {
    for helper in [
        "settings_filtered_rows",
        "settings_visible_row_labels",
        "settings_dataset_and_visible_counts",
        "settings_selected_visible_row",
    ] {
        assert!(
            SETTINGS.contains(helper),
            "missing Settings helper {helper}"
        );
    }
}

#[test]
fn collect_elements_has_settings_arm_without_fallback() {
    assert!(COLLECT_ELEMENTS.contains("AppView::SettingsView"));
    assert!(COLLECT_ELEMENTS.contains("settings_visible_row_names(filter)"));
    let settings_arm = COLLECT_ELEMENTS
        .find("AppView::SettingsView")
        .expect("Settings arm must exist");
    let fallback = COLLECT_ELEMENTS
        .find("collector_used_current_view_fallback")
        .expect("fallback warning must exist");
    assert!(
        settings_arm < fallback,
        "Settings must be handled before the catch-all fallback"
    );
}

#[test]
fn filterable_matrix_has_settings_case() {
    assert!(MATRIX.contains("id: \"settings-visible-rows\""));
    assert!(MATRIX.contains("surface: \"settings\""));
    assert!(MATRIX
        .contains("entryCommand: { type: \"triggerBuiltin\", builtinId: \"builtin/settings\" }"));
}
