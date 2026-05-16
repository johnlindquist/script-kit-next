//! Source-level contract for the `process-manager` triggerBuiltin route.
//!
//! The route is now centralized: registry aliases resolve to
//! `TriggerBuiltin::ProcessManager`, the pure planner produces
//! `FilterableView::ProcessManager`, and the imperative dispatcher seeds the
//! process cache before opening `ProcessManagerView`.

const REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const ROUTES: &str = include_str!("../src/app_impl/routes.rs");
const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");

// doc-anchor-removed: [[removed-docs]]
#[test]
fn registry_accepts_process_manager_aliases() {
    for alias in ["\"process-manager\"", "\"processmanager\"", "\"processes\""] {
        assert!(
            REGISTRY.contains(alias),
            "trigger registry must keep process-manager alias {alias}"
        );
    }
}

// doc-anchor-removed: [[removed-docs]]
#[test]
fn planner_routes_process_manager_to_filterable_view() {
    assert!(
        ROUTES.contains("TriggerBuiltin::ProcessManager => {\n            AppRoute::ShowFilterableView(FilterableView::ProcessManager)\n        }"),
        "route planner must map TriggerBuiltin::ProcessManager to FilterableView::ProcessManager"
    );
}

// doc-anchor-removed: [[removed-docs]]
#[test]
fn dispatcher_seeds_process_manager_view_state() {
    let start = DISPATCH
        .find("FilterableView::ProcessManager =>")
        .expect("dispatcher must handle FilterableView::ProcessManager");
    let body = &DISPATCH[start..];
    let end = body
        .find("\n        }\n    }\n\n    /// Rate-limited")
        .unwrap_or(body.len());
    let body = &body[..end];

    for required in [
        "crate::process_manager::PROCESS_MANAGER.get_active_processes_sorted()",
        "Search running scripts...",
        "AppView::ProcessManagerView",
        "pending_focus: Some(FocusTarget::MainFilter),",
        "self.update_window_size_deferred(window, cx);",
    ] {
        assert!(
            body.contains(required),
            "process-manager dispatcher body must contain {required}"
        );
    }
}
