use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path))
}

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source
        .find(start)
        .unwrap_or_else(|| panic!("'{}' not found", start));
    let tail = &source[start_ix..];
    let end_rel = tail.find(end).unwrap_or(tail.len());
    &tail[..end_rel]
}

#[test]
fn mini_main_list_actions_stay_top_center() {
    let source = read("src/app_impl/actions_toggle.rs");
    let body = section_between(
        &source,
        "fn main_list_actions_window_position(&self)",
        "fn begin_actions_popup_window_open",
    );

    assert!(
        body.contains("MainWindowMode::Mini => crate::actions::WindowPosition::TopCenter"),
        "mini main-list actions must stay anchored at TopCenter"
    );

    assert!(
        body.contains("MainWindowMode::Full => crate::actions::WindowPosition::BottomRight"),
        "full main-list actions must stay anchored at BottomRight"
    );
}

#[test]
fn toggle_actions_uses_position_from_main_list_actions_window_position() {
    let source = read("src/app_impl/actions_toggle.rs");
    let body = section_between(
        &source,
        "pub(crate) fn toggle_actions(",
        "pub(crate) fn toggle_arg_actions(",
    );

    assert!(
        body.contains("let position = self.main_list_actions_window_position();"),
        "toggle_actions must resolve position via main_list_actions_window_position"
    );

    assert!(
        body.contains("position,"),
        "toggle_actions must pass the resolved position to spawn_open_actions_window"
    );
}
