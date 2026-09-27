//! Source-level contract for generic BuiltinList actions routing.
//!
//! BuiltinList surfaces such as Settings, Browser History, and Process Manager
//! do not yet build selection-specific action catalogs. They
//! must not advertise a live Cmd+K host or explicit host toggle that falls back
//! to stale main-list actions.

const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");

#[test]
fn builtin_list_views_are_filtered_out_of_live_actions_host() {
    let helper_start = ACTIONS_DIALOG
        .find("fn is_builtin_list_actions_view")
        .expect("BuiltinList view helper must exist");
    let helper_body = &ACTIONS_DIALOG[helper_start..];

    for variant in [
        "AppView::BrowserHistoryView",
        "AppView::BrowserTabsView",
        "AppView::WindowSwitcherView",
        "AppView::CurrentAppCommandsView",
        "AppView::ProcessManagerView",
        "AppView::SettingsView",
        "AppView::FavoritesBrowseView",
        "AppView::BrowseKitsView",
        "AppView::InstalledKitsView",
    ] {
        assert!(
            helper_body.contains(variant),
            "generic BuiltinList live-host filter must include {variant}"
        );
    }

    let live_host_start = ACTIONS_DIALOG
        .find("pub(crate) fn live_actions_host_for_view")
        .expect("live_actions_host_for_view must exist");
    let live_host_body = &ACTIONS_DIALOG[live_host_start..];
    assert!(
        live_host_body.contains("if Self::is_builtin_list_actions_view(view)") &&
            live_host_body.contains("None") &&
            live_host_body.contains("Self::actions_host_for_view(view)"),
        "live_actions_host_for_view must return None for generic BuiltinList views before falling back to actions_host_for_view"
    );
}

#[test]
fn toggle_actions_uses_live_host_not_static_support_map() {
    let helper_start = ACTIONS_TOGGLE
        .find("fn actions_dialog_host_for_current_view")
        .expect("actions_dialog_host_for_current_view must exist");
    let helper_body = &ACTIONS_TOGGLE[helper_start..];

    assert!(
        helper_body.contains("self.current_actions_host()"),
        "toggle_actions must use the live host resolver so generic BuiltinList views cannot reach stale actions"
    );
    assert!(
        !helper_body
            .split("fn actions_dialog_host_for_current_view")
            .nth(1)
            .unwrap_or_default()
            .split("}")
            .next()
            .unwrap_or_default()
            .contains("actions_support_for_view"),
        "toggle_actions host helper must not use the broader static support map"
    );
}
