//! Source contract for Phase 1 Design Picker sizing.
//!
//! The MVP picker is a single-column searchable list, so
//! `calculate_window_size_params` must classify it like ThemeChooser:
//! `ViewType::ScriptList` with a filtered catalog count.

const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start.find(end).unwrap_or(after_start.len());
    &after_start[..end_index]
}

#[test]
fn design_picker_uses_script_list_window_sizing() {
    let arm = source_between(
        UI_WINDOW_SOURCE,
        "AppView::DesignPickerView { ref filter, .. } =>",
        "AppView::CreationFeedback",
    );
    assert!(
        arm.contains("crate::designs::registry::catalog()"),
        "DesignPicker sizing must count the design registry catalog"
    );
    assert!(
        arm.contains("Some((ViewType::ScriptList, filtered_count))"),
        "calculate_window_size_params must return ViewType::ScriptList for AppView::DesignPickerView"
    );
    assert!(
        !arm.contains("ViewType::MiniMainWindow"),
        "DesignPicker must not be classified as MiniMainWindow in Phase 1"
    );
}
