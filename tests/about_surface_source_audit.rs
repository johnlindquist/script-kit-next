const ABOUT_RENDER_SOURCE: &str = include_str!("../src/about/render.rs");
const ABOUT_ROUTE_SOURCE: &str = include_str!("../src/app_impl/about_route.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const ABOUT_STORY_SOURCE: &str = include_str!("../src/stories/about_surface.rs");

#[test]
fn about_route_owns_focus_without_launcher_filter_on_open() {
    // doc-anchor-removed: [[removed-docs behavior]]
    assert!(
        ABOUT_ROUTE_SOURCE.contains("self.focused_input = FocusedInput::None"),
        "About open should clear launcher input focus"
    );
    assert!(
        ABOUT_ROUTE_SOURCE.contains("self.pending_focus = Some(FocusTarget::AppRoot)"),
        "About open should request app-root/About focus"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains(".track_focus(focus)"),
        "About surface should track the passed focus handle"
    );
}

#[test]
fn about_surface_contract_declares_no_editable_input_and_explicit_dismissal() {
    // doc-anchor-removed: [[removed-docs contract]]
    let start = APP_VIEW_STATE_SOURCE
        .find("SurfaceKind::About => LauncherSurfaceContract::new")
        .expect("About surface contract arm exists");
    let end = APP_VIEW_STATE_SOURCE[start..]
        .find("SurfaceKind::ActionsDialog")
        .map(|ix| start + ix)
        .expect("ActionsDialog arm follows About arm");
    let about_arm = &APP_VIEW_STATE_SOURCE[start..end];

    assert!(about_arm.contains("NoEditableInput"));
    assert!(about_arm.contains("ContentPane"));
    assert!(about_arm.contains("explicit"));
    assert!(about_arm.contains("\"about\""));
}

#[test]
fn about_surface_controls_are_keyboard_reachable_in_contract_order() {
    // doc-anchor-removed: [[removed-docs behavior]]
    let ids = [
        "about-close-button",
        "about-open-github",
        "about-open-discord",
        "about-follow-x",
        "about-update-button",
        "about-acknowledgements-toggle",
    ];
    let mut last = 0;
    for id in ids {
        let literal_id = format!("\"{id}\"");
        let literal_pos = ABOUT_RENDER_SOURCE
            .find(&literal_id)
            .unwrap_or_else(|| panic!("{id} should be present as a stable control id"));
        assert!(
            literal_pos >= last,
            "{id} should stay in the expected source/tab order"
        );
        last = literal_pos;
    }

    assert!(
        ABOUT_RENDER_SOURCE.contains(".id(id)"),
        "About action_button should apply each stable control id to its tab stop"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains(".tab_index(0)"),
        "About controls should expose tab stops"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains("is_about_activation_key"),
        "About controls should centralize Enter/Space activation"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains(".focus_visible("),
        "About tab stops should expose a keyboard focus affordance"
    );
}

#[test]
fn about_escape_is_surface_captured() {
    // doc-anchor-removed: [[removed-docs behavior]]
    assert!(
        ABOUT_RENDER_SOURCE.contains(".capture_key_down("),
        "About should capture Escape at the surface before child controls can swallow it"
    );
}

#[test]
fn about_dismiss_restores_filter_owned_previous_routes() {
    // doc-anchor-removed: [[removed-docs behavior]]
    assert!(
        ABOUT_ROUTE_SOURCE.contains("focus_for_about_restore"),
        "About dismiss should centralize previous-route focus restoration"
    );
    assert!(
        ABOUT_ROUTE_SOURCE.contains("LauncherSurfaceInputOwnership::LauncherFilter"),
        "About dismiss should restore main-filter focus for any filter-owned previous route"
    );
    assert!(
        !ABOUT_ROUTE_SOURCE.contains("matches!(self.current_view, AppView::ScriptList)"),
        "About dismiss should not restore filter focus only for ScriptList"
    );
}

#[test]
fn about_story_covers_update_states_and_acknowledgements_open() {
    // doc-anchor-removed: [[removed-docs coverage]]
    for id in ["idle", "checking", "up-to-date", "available", "error"] {
        assert!(
            ABOUT_STORY_SOURCE.contains(id),
            "About story should cover update state {id}"
        );
    }
    assert!(
        ABOUT_STORY_SOURCE.contains("acknowledgements-open")
            && ABOUT_STORY_SOURCE.contains("acks_open"),
        "About story should cover expanded acknowledgements"
    );
}
