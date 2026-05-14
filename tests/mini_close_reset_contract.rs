const WINDOW_VISIBILITY: &str = include_str!("../src/main_sections/window_visibility.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let end_index = source[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or(source.len());
    &source[start_index..end_index]
}

// @lat: [[lat.md/tests/mini-window-contract#Mini close reset]]
#[test]
fn hide_path_snapshots_mini_mode_before_reset() {
    let body = source_between(
        WINDOW_VISIBILITY,
        "fn hide_main_window_helper",
        "pub fn show_main_window()",
    );
    let was_mini = body
        .find("let was_mini = view.main_window_mode == MainWindowMode::Mini")
        .expect("hide helper must snapshot mini mode before reset");
    let reset = body
        .find("view.reset_to_script_list(ctx)")
        .expect("hide helper must reset to script list");
    assert!(
        was_mini < reset,
        "mini mode must be captured before reset_to_script_list"
    );
    assert!(
        body.contains("let post_reset_is_mini = view.main_window_mode == MainWindowMode::Mini")
            && body.contains("was_mini || post_reset_is_mini")
            && body.contains("if should_reset_to_mini_bounds"),
        "hide helper must reset hidden mini bounds when pre- or post-reset mode is Mini"
    );
}
