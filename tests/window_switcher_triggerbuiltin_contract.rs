//! Source-level contract for the `window-switcher` triggerBuiltin route.
//!
//! The stdin entry path delegates to the shared dispatcher. This test pins the
//! registry aliases, route-planner branch, and dispatcher side effects needed
//! to open a populated `WindowSwitcherView`.

const REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const ROUTES: &str = include_str!("../src/app_impl/routes.rs");
const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");

fn window_switcher_body() -> &'static str {
    let start = DISPATCH
        .find("FilterableView::WindowSwitcher =>")
        .expect("dispatcher must handle FilterableView::WindowSwitcher");
    let body = &DISPATCH[start..];
    let end = body
        .find("FilterableView::ProcessManager =>")
        .expect("window-switcher arm must be followed by process-manager arm");
    &body[..end]
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn registry_accepts_window_switcher_aliases() {
    for alias in ["\"window-switcher\"", "\"windowswitcher\"", "\"windows\""] {
        assert!(
            REGISTRY.contains(alias),
            "trigger registry must keep window-switcher alias {alias}"
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn planner_routes_window_switcher_to_filterable_view() {
    assert!(
        ROUTES.contains("TriggerBuiltin::WindowSwitcher => {\n            AppRoute::ShowFilterableView(FilterableView::WindowSwitcher)\n        }"),
        "route planner must map TriggerBuiltin::WindowSwitcher to FilterableView::WindowSwitcher"
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn dispatcher_loads_windows_before_opening_view() {
    let body = window_switcher_body();
    for required in [
        "crate::window_control::list_windows()",
        "self.cached_windows = windows;",
        "AppView::WindowSwitcherView",
        "resize: true",
    ] {
        assert!(
            body.contains(required),
            "window-switcher dispatcher body must contain {required}"
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn dispatcher_handles_window_listing_errors_without_panic() {
    let body = window_switcher_body();
    assert!(
        body.contains("Err(error) =>") && body.contains("Failed to list windows"),
        "window-switcher dispatcher must log list_windows errors instead of panicking"
    );
}
