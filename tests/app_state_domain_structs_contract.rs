//! Source-level contract for AURP-07 app-state domain grouping.
//!
//! `ScriptListApp` is still the launcher state root, but related state should
//! move into named owner structs when that makes intent clearer for agents.

const APP_STATE: &str = include_str!("../src/main_sections/app_state.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const RENDER_SCRIPT_LIST: &str = include_str!("../src/render_script_list/mod.rs");
const PREVIEW_PANEL: &str = include_str!("../src/app_render/preview_panel.rs");
const FILTERING_CACHE: &str = include_str!("../src/app_impl/filtering_cache.rs");
const RENDER_IMPL: &str = include_str!("../src/main_sections/render_impl.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

// doc-anchor-removed: [[removed-docs State Domains]]
#[test]
fn main_menu_render_diagnostics_have_a_named_domain_owner() {
    let diagnostics_body = source_between(
        APP_STATE,
        "struct MainMenuRenderDiagnosticsState {",
        "\n}\n\nimpl Default for MainMenuRenderDiagnosticsState",
    );
    for field in [
        "last_render_log_filter: String,",
        "last_render_log_selection: usize,",
        "last_render_log_item_count: usize,",
        "log_this_render: bool,",
        "filter_perf_start: Option<std::time::Instant>,",
    ] {
        assert!(
            diagnostics_body.contains(field),
            "MainMenuRenderDiagnosticsState must own {field}"
        );
    }

    let app_body = source_between(
        APP_STATE,
        "struct ScriptListApp {",
        "\n}\n\n#[derive(Clone, Debug)]\nstruct AttachmentPortalHostSnapshot",
    );
    assert!(
        app_body.contains("main_menu_render_diagnostics: MainMenuRenderDiagnosticsState,"),
        "ScriptListApp must expose one named diagnostics owner field."
    );
    for loose_field in [
        "last_render_log_filter: String,",
        "last_render_log_selection: usize,",
        "last_render_log_item_count: usize,",
        "log_this_render: bool,",
        "filter_perf_start: Option<std::time::Instant>,",
    ] {
        assert!(
            !app_body.contains(loose_field),
            "ScriptListApp must not keep {loose_field} as loose state."
        );
    }
}

#[test]
fn live_startup_initializes_the_diagnostics_owner_from_its_default_contract() {
    assert!(
        STARTUP
            .contains("main_menu_render_diagnostics: MainMenuRenderDiagnosticsState::default(),"),
        "Live startup must initialize the diagnostics owner through its named default."
    );
    assert!(
        APP_STATE.contains("last_render_log_selection: usize::MAX,")
            && APP_STATE.contains("log_this_render: true,")
            && APP_STATE.contains("filter_perf_start: None,"),
        "The default contract must preserve first-render logging and empty filter timing."
    );
}

#[test]
fn render_and_filter_paths_route_through_the_diagnostics_owner() {
    assert!(
        RENDER_SCRIPT_LIST.contains("self.main_menu_render_diagnostics.last_render_log_filter")
            && RENDER_SCRIPT_LIST
                .contains("self.main_menu_render_diagnostics.last_render_log_selection")
            && RENDER_SCRIPT_LIST
                .contains("self.main_menu_render_diagnostics.last_render_log_item_count")
            && RENDER_SCRIPT_LIST
                .contains("self.main_menu_render_diagnostics.log_this_render = state_changed"),
        "Script-list render dedupe must route through the diagnostics owner."
    );
    assert!(
        PREVIEW_PANEL.contains("self.main_menu_render_diagnostics.log_this_render"),
        "Preview diagnostics must read the render flag from the diagnostics owner."
    );
    assert!(
        FILTERING_CACHE.contains("self.main_menu_render_diagnostics.filter_perf_start"),
        "Grouped-results timing must read from the diagnostics owner."
    );
    assert!(
        RENDER_IMPL.contains("self.main_menu_render_diagnostics.filter_perf_start = None"),
        "A complete render must clear filter timing through the diagnostics owner."
    );
}
