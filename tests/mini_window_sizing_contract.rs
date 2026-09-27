const PROMPT_AI: &str = include_str!("../src/app_impl/prompt_ai.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let end_index = source[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &source[start_index..end_index]
}

#[test]
fn inline_mini_ai_uses_mode_aware_resize_helper() {
    let body = source_between(
        PROMPT_AI,
        "pub fn show_inline_ai_chat(",
        "    /// Compatibility shim",
    );
    assert!(
        !body.contains("resize_to_view_sync(ViewType::DivPrompt, 0)"),
        "show_inline_ai_chat must not resize Mini AI through raw DivPrompt"
    );
    assert!(
        body.contains("compact_ai_view_type_for_mode(self.main_window_mode)"),
        "show_inline_ai_chat must size ChatPrompt through main-window mode"
    );
}

#[test]
fn file_search_sizing_branches_on_presentation() {
    let calculate = source_between(
        UI_WINDOW,
        "pub(crate) fn calculate_window_size_params",
        "pub(crate) fn calculate_window_size_params_if_current_view",
    );
    let file_search_arm = source_between(
        calculate,
        "AppView::FileSearchView",
        "AppView::ThemeChooserView",
    );
    assert!(
        file_search_arm.contains("presentation"),
        "FileSearchView sizing must inspect FileSearchPresentation"
    );
    assert!(
        file_search_arm.contains("FileSearchPresentation::Mini => ViewType::MainWindow")
            && file_search_arm.contains("FileSearchPresentation::Full => ViewType::MainWindow"),
        "both file search presentations must size through the unified MainWindow view type"
    );
}
