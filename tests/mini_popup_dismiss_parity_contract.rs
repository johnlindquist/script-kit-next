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

/// Footer buttons stay live while the actions popup is open: a click on a
/// non-Actions footer button must close the popup AND dispatch the clicked
/// action in the same event (standard macOS menu dismissal). The old contract
/// (close-only, swallow the click) made visible footer buttons dead until a
/// second click.
#[test]
fn non_actions_footer_click_closes_actions_then_dispatches() {
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
        "main_window_footer_action_closed_actions_then_dispatched",
        "main_window_mode",
    ] {
        assert!(
            body.contains(required),
            "footer dismiss path missing {required}"
        );
    }
    assert!(
        body.contains("(shared_actions_open || detached_actions_open) && !action.is_actions()"),
        "non-Actions footer clicks must still close the actions popup before dispatching"
    );
    assert!(
        !body.contains("return;"),
        "footer dismiss path must fall through to `match action` so the clicked action dispatches; \
         an early return reintroduces the dead-click bug"
    );
}
