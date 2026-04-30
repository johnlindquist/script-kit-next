//! Source-level contract for triggerBuiltin route and semantic-surface symmetry.
//!
//! The string arms no longer live in three stdin files. The symmetry now runs
//! through one registry, one pure route planner, one imperative dispatcher, and
//! the `AppView::surface_contract()` semantic-surface registry.

const REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const ROUTES: &str = include_str!("../src/app_impl/routes.rs");
const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");
const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
const AUTOMATION_DOC: &str = include_str!("../lat.md/automation.md");

/// `(TriggerBuiltin variant, route snippet, AppView variant, semanticSurface tag)`.
const EXPECTED: &[(&str, &str, &str, &str)] = &[
    (
        "DesignGallery",
        "TriggerBuiltin::DesignGallery => {\n            AppRoute::ShowFilterableView(FilterableView::DesignGallery)\n        }",
        "DesignGalleryView",
        "designGallery",
    ),
    (
        "ClipboardHistory",
        "TriggerBuiltin::ClipboardHistory => {\n            AppRoute::ShowFilterableView(FilterableView::ClipboardHistory)\n        }",
        "ClipboardHistoryView",
        "clipboardHistory",
    ),
    (
        "AppLauncher",
        "TriggerBuiltin::AppLauncher => AppRoute::ShowFilterableView(FilterableView::AppLauncher)",
        "AppLauncherView",
        "appLauncher",
    ),
    (
        "FileSearch",
        "TriggerBuiltin::FileSearch => AppRoute::OpenFileSearch",
        "FileSearchView",
        "fileSearch",
    ),
    (
        "BrowserTabs",
        "TriggerBuiltin::BrowserTabs => AppRoute::ShowFilterableView(FilterableView::BrowserTabs)",
        "BrowserTabsView",
        "browserTabs",
    ),
    (
        "EmojiPicker",
        "TriggerBuiltin::EmojiPicker => AppRoute::ShowFilterableView(FilterableView::EmojiPicker)",
        "EmojiPickerView",
        "emojiPicker",
    ),
    (
        "WindowSwitcher",
        "TriggerBuiltin::WindowSwitcher => {\n            AppRoute::ShowFilterableView(FilterableView::WindowSwitcher)\n        }",
        "WindowSwitcherView",
        "windowSwitcher",
    ),
    (
        "ProcessManager",
        "TriggerBuiltin::ProcessManager => {\n            AppRoute::ShowFilterableView(FilterableView::ProcessManager)\n        }",
        "ProcessManagerView",
        "processManager",
    ),
    (
        "CurrentAppCommands",
        "TriggerBuiltin::CurrentAppCommands => AppRoute::OpenCurrentAppCommands",
        "CurrentAppCommandsView",
        "currentAppCommands",
    ),
    (
        "TabAi",
        "TriggerBuiltin::TabAi => AppRoute::OpenTabAi",
        "AcpChatView",
        "acpChat",
    ),
];

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start.find(end).unwrap_or(after_start.len());
    &after_start[..end_index]
}

fn compact(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn trigger_registry_declares_every_surface_rekey_target() {
    for (variant, _, _, _) in EXPECTED {
        assert!(
            REGISTRY.contains(&format!("TriggerBuiltin::{variant}")),
            "trigger registry must declare TriggerBuiltin::{variant}"
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn route_planner_covers_every_semantic_surface_target() {
    let routes_compact = compact(ROUTES);
    for (variant, route_snippet, _, _) in EXPECTED {
        assert!(
            routes_compact.contains(&compact(route_snippet)),
            "plan_trigger_builtin_route must map TriggerBuiltin::{variant} through {route_snippet}"
        );
    }
    assert!(
        DISPATCH.contains("match plan_trigger_builtin_route(id)"),
        "imperative triggerBuiltin dispatch must route through the pure planner"
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn semantic_surface_map_covers_every_route_flip_target() {
    let registry_body = source_between(
        APP_VIEW_STATE,
        "pub(crate) fn surface_contract(&self) -> LauncherSurfaceContract",
        "/// Dismiss policy for the active top-level launcher view.",
    );
    for (_, _, variant, surface) in EXPECTED {
        let expected_variant = format!("AppView::{variant}");
        let expected_surface = format!("\"{surface}\"");
        assert!(
            registry_body.contains(&expected_variant) && registry_body.contains(&expected_surface),
            "AppView::surface_contract is missing {expected_variant} mapped to {expected_surface}"
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn lat_automation_doc_mentions_every_surface_tag() {
    for (_, _, _, surface) in EXPECTED {
        assert!(
            AUTOMATION_DOC.contains(&format!("`{surface}`")),
            "lat.md/automation.md must list surface tag `{surface}` among triggerBuiltin subview tags"
        );
    }
}
