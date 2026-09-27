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

#[test]
fn hide_path_snapshots_mini_mode_before_deferred_reset() {
    let body = source_between(
        WINDOW_VISIBILITY,
        "fn hide_main_window_helper",
        "pub fn show_main_window()",
    );
    let was_mini = body
        .find("let was_mini = view.main_window_mode == MainWindowMode::Mini")
        .expect("hide helper must snapshot mini mode before deferred reset");
    let hide = body
        .find("view.defer_calibrated_main_window_hide(")
        .expect("hide helper must enqueue native hide");
    let deferred_reset = body
        .find("MainWindowPostHide::ResetScriptList {")
        .expect("hide helper must schedule the hidden ScriptList reset");
    assert!(
        was_mini < hide && hide < deferred_reset,
        "mini mode must be captured before hide, and ScriptList reset must be deferred until after native hide"
    );
    assert!(
        body.contains("reset_mini_bounds_after_hidden_reset,")
            && body.contains("cancel_script_execution_without_view_reset")
            && body.contains("reason: \"hide_main_window_helper\""),
        "hide helper must reset hidden mini bounds when pre- or post-reset mode is Mini"
    );
}
