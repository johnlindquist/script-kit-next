const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");
const REGISTRIES_STATE: &str = include_str!("../src/app_impl/registries_state.rs");
const LIFECYCLE_RESET: &str = include_str!("../src/app_impl/lifecycle_reset.rs");
const RENDER_IMPL: &str = include_str!("../src/main_sections/render_impl.rs");

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
fn main_window_mode_helper_updates_prompt_footer_popup_and_size() {
    let helper = source_between(
        UI_WINDOW,
        "pub(crate) fn set_main_window_mode(",
        "pub(crate) fn set_main_window_mode_state_only(",
    );
    for required in [
        "if old == mode",
        "chat.set_mini_mode",
        "close_actions_popup",
        "close_actions_window",
        "update_window_size_deferred",
        "sync_main_footer_popup",
        "main_window_mode_changed",
    ] {
        assert!(
            helper.contains(required),
            "set_main_window_mode missing {required}"
        );
    }
}

#[test]
fn mode_callers_route_through_helpers_not_direct_assignment() {
    for (name, source) in [
        ("builtin_execution", BUILTIN_EXECUTION),
        ("registries_state", REGISTRIES_STATE),
        ("lifecycle_reset", LIFECYCLE_RESET),
        ("render_impl", RENDER_IMPL),
    ] {
        assert!(
            !source.contains("self.main_window_mode = MainWindowMode::"),
            "{name} must route mode changes through set_main_window_mode helpers"
        );
    }
    assert!(
        BUILTIN_EXECUTION.contains("set_main_window_mode_state_only")
            && RENDER_IMPL.contains("set_main_window_mode(MainWindowMode::Mini"),
        "mini/full launcher and inline handoff paths must use mode helpers"
    );
}

#[test]
fn resize_current_view_to_width_clamps_mini_width() {
    let body = source_between(
        UI_WINDOW,
        "pub(crate) fn resize_current_view_to_width",
        "pub(crate) fn can_accept_dictation_into_main_filter",
    );
    assert!(
        body.contains("self.main_window_mode == MainWindowMode::Mini")
            && body.contains("width_for_view(ViewType::MainWindow)"),
        "resize_current_view_to_width must clamp width to MainWindow in mini mode"
    );
}
