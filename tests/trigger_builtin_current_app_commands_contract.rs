//! Source-level contract for the `current-app-commands` triggerBuiltin route.
//!
//! The stdin files delegate to the shared triggerBuiltin dispatcher. This
//! contract pins the canonical registry aliases, pure route planner, and
//! imperative menu-capture branch that opens `CurrentAppCommandsView`.

const REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const ROUTES: &str = include_str!("../src/app_impl/routes.rs");
const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");
const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");

fn compact(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

// @lat: [[lat.md/builtins#Built-ins]]
#[test]
fn registry_accepts_current_app_commands_aliases() {
    for alias in [
        "\"current-app-commands\"",
        "\"currentappcommands\"",
        "\"current-app\"",
        "\"app-commands\"",
        "\"menu-commands\"",
    ] {
        assert!(
            REGISTRY.contains(alias),
            "trigger registry must keep current-app-commands alias {alias}"
        );
    }
}

// @lat: [[lat.md/builtins#Built-ins]]
#[test]
fn current_app_commands_routes_through_named_planner_branch() {
    assert!(
        ROUTES.contains("TriggerBuiltin::CurrentAppCommands => AppRoute::OpenCurrentAppCommands"),
        "route planner must map TriggerBuiltin::CurrentAppCommands to AppRoute::OpenCurrentAppCommands"
    );
    assert!(
        compact(DISPATCH).contains("AppRoute::OpenCurrentAppCommands=>"),
        "dispatcher must handle AppRoute::OpenCurrentAppCommands"
    );
}

// @lat: [[lat.md/builtins#Built-ins]]
#[test]
fn current_app_commands_dispatch_uses_tray_capture_helper() {
    assert!(
        DISPATCH.contains("self.open_current_app_commands_from_tray(cx)"),
        "current-app-commands dispatch must delegate to the tray-capture helper"
    );
    assert!(
        DISPATCH.contains("\"triggerBuiltin current-app-commands failed:"),
        "current-app-commands dispatch must log tray-capture errors without panicking"
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn semantic_surface_map_has_current_app_commands() {
    assert!(
        APP_VIEW_STATE.contains("AppView::CurrentAppCommandsView { .. }")
            && APP_VIEW_STATE.contains("\"currentAppCommands\""),
        "AppView::surface_contract must map CurrentAppCommandsView to currentAppCommands"
    );
}
