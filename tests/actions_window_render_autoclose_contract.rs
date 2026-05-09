//! Source-audit contract for actions popup passive blur fallback.
//!
//! AppKit activation observation can miss the path where the popup renders
//! inactive after desktop click-away, so render must close through the same
//! guarded lifecycle path when both main and actions windows are inactive.

const ACTIONS_WINDOW: &str = include_str!("../src/actions/window.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &after_start[..end_index]
}

#[test]
fn actions_window_request_close_is_idempotent() {
    let request_close = source_between(
        ACTIONS_WINDOW,
        "fn request_close(",
        "fn ensure_activation_subscription",
    );

    assert!(request_close.contains("if self.close_requested"));
    assert!(request_close.contains("self.close_requested = true;"));
}

#[test]
fn actions_window_render_falls_back_to_autoclose_when_inactive() {
    let render = source_between(
        ACTIONS_WINDOW,
        "impl Render for ActionsWindow",
        "let handle_key =",
    );

    assert!(render.contains("let main_window_focused = platform::is_main_window_focused();"));
    assert!(
        render.contains("should_auto_close_actions_window(main_window_focused, window_is_active)")
    );
    assert!(render.contains("ACTIONS_WINDOW_LIFECYCLE render_auto_close"));
    assert!(render.contains("self.request_close(window, cx, \"render_focus_lost\", false);"));
}
