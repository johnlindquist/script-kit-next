//! Source-level contract for the `current-app-commands` triggerBuiltin route.
//!
//! The stdin files delegate to the shared triggerBuiltin dispatcher. This
//! contract pins the canonical registry aliases, pure route planner, and
//! imperative menu-capture branch that opens `CurrentAppCommandsView`.

const REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const ROUTES: &str = include_str!("../src/app_impl/routes.rs");
const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");
const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");

fn compact(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

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

#[test]
fn current_app_commands_trigger_builtin_deferred_resize_stays_mini() {
    let dispatch_compact = compact(DISPATCH);
    let branch_start = dispatch_compact
        .find("AppRoute::OpenCurrentAppCommands=>")
        .expect("dispatcher must have an OpenCurrentAppCommands branch");
    let branch_tail = &dispatch_compact[branch_start..];
    let branch_end = branch_tail["AppRoute::OpenCurrentAppCommands=>".len()..]
        .find("AppRoute::")
        .map(|offset| "AppRoute::OpenCurrentAppCommands=>".len() + offset)
        .unwrap_or(branch_tail.len());
    let current_app_branch = &branch_tail[..branch_end];
    assert!(
        current_app_branch.contains(
            "ifletErr(e)=self.open_current_app_commands_from_tray(cx)"
        ) && current_app_branch.contains("self.update_window_size_deferred(window,cx);"),
        "triggerBuiltin current-app-commands must keep the deferred resize path visible to this regression contract"
    );

    let sizing_start = UI_WINDOW
        .find("pub(crate) fn calculate_window_size_params")
        .expect("calculate_window_size_params must exist");
    let sizing = compact(&UI_WINDOW[sizing_start..]);
    let current_view_start = sizing
        .find("AppView::CurrentAppCommandsView{filter,..}")
        .expect("calculate_window_size_params must handle CurrentAppCommandsView");
    let current_view_tail = &sizing[current_view_start..];
    let current_view_arrow = current_view_tail
        .find("=>")
        .expect("CurrentAppCommandsView sizing arm must have a match expression");
    let current_view_expr_tail = &current_view_tail[current_view_arrow..];
    let current_view_end = current_view_expr_tail
        .find(")),")
        .map(|offset| current_view_arrow + offset + 3)
        .expect("CurrentAppCommandsView sizing arm must return a Some tuple");
    let current_view_arm = &current_view_tail[..current_view_end];
    assert!(
        current_view_arm.contains("Some((ViewType::MainWindow,filtered_count))")
            && !current_view_arm.contains("ViewType::ScriptList"),
        "the deferred resize used by triggerBuiltin current-app-commands must resolve CurrentAppCommandsView through the consolidated MainWindow sizing, not ScriptList"
    );

    let opener_start = BUILTIN_EXECUTION
        .find("fn open_builtin_filterable_view_with_filter(")
        .expect("open_builtin_filterable_view_with_filter must exist");
    let opener = compact(&BUILTIN_EXECUTION[opener_start..]);
    assert!(
        opener.contains("ifexpanded{self.set_main_window_mode_state_only(MainWindowMode::Full")
            && opener.contains("}else{self.set_main_window_mode_state_only(MainWindowMode::Mini")
            && opener.contains("resize_to_view_sync(ViewType::MainWindow,0);"),
        "the shared filterable opener must keep mini/full mode ownership separate from the consolidated MainWindow view type"
    );
}

#[test]
fn semantic_surface_map_has_current_app_commands() {
    assert!(
        APP_VIEW_STATE.contains("AppView::CurrentAppCommandsView { .. }")
            && APP_VIEW_STATE.contains("\"currentAppCommands\""),
        "AppView::surface_contract must map CurrentAppCommandsView to currentAppCommands"
    );
}
