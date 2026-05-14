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

// @lat: [[lat.md/tests/mini-window-contract#Mini popup dismiss parity]]
#[test]
fn non_actions_footer_click_closes_shared_and_detached_actions_only() {
    let body = source_between(
        UI_WINDOW,
        "pub(crate) fn dispatch_main_window_footer_action",
        "match action",
    );
    for required in [
        "shared_actions_open",
        "detached_actions_open",
        "close_actions_popup",
        "close_actions_window",
        "main_window_footer_action_closed_actions_only",
        "main_window_mode",
        "return;",
    ] {
        assert!(
            body.contains(required),
            "footer dismiss path missing {required}"
        );
    }
    assert!(
        body.contains("(shared_actions_open || detached_actions_open) && !action.is_actions()"),
        "non-Actions footer clicks must close actions and not dispatch the clicked action"
    );
}
