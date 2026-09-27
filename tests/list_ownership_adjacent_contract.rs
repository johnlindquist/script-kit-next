//! Source-level contract for adjacent built-in visible-row ownership.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");

#[test]
fn adjacent_get_elements_paths_use_visible_row_label_helpers() {
    for helper in [
        "clipboard_history_visible_row_labels",
        "browser_tabs_visible_row_labels",
        "dictation_history_visible_row_labels",
        "notes_browse_visible_row_labels",
        "agent_chat_history_visible_row_labels",
    ] {
        assert!(
            COLLECT_ELEMENTS.contains(helper),
            "getElements must use visible-row label helper {helper}"
        );
    }
}

#[test]
fn about_surface_is_explicitly_exempted_as_static_content() {
    assert!(
        APP_VIEW_STATE.contains("ABOUT_SURFACE_EXEMPTION")
            && APP_VIEW_STATE.contains("static content surface with no list selection owner"),
        "About must carry an explicit visible-row ownership exemption"
    );
    assert!(
        APP_VIEW_STATE.contains("AppView::About { .. } => SurfaceKind::About"),
        "About must remain an explicit surface kind, not a list-like surface"
    );
}
