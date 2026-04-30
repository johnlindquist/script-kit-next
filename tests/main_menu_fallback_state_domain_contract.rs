//! Source-level contract for AURP-09 fallback-state ownership.
//!
//! Fallback commands are a main-menu search concern. Their active flag,
//! selected index, and cached items should stay behind one named owner.

const APP_STATE: &str = include_str!("../src/main_sections/app_state.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");
const SELECTION_FALLBACK: &str = include_str!("../src/app_impl/selection_fallback.rs");
const RENDER_SCRIPT_LIST: &str = include_str!("../src/render_script_list/mod.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_SIMULATE_KEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

// @lat: [[lat.md/architecture#Architecture#App State Domains]]
#[test]
fn fallback_state_has_a_named_domain_owner() {
    let fallback_body = source_between(
        APP_STATE,
        "struct MainMenuFallbackState {",
        "\n}\n\nimpl MainMenuFallbackState",
    );
    for field in [
        "active: bool,",
        "selected_index: usize,",
        "cached_items: Vec<crate::fallbacks::FallbackItem>,",
    ] {
        assert!(
            fallback_body.contains(field),
            "MainMenuFallbackState must own {field}"
        );
    }

    let app_body = source_between(
        APP_STATE,
        "struct ScriptListApp {",
        "\n}\n\n#[derive(Clone, Debug)]\nstruct AttachmentPortalHostSnapshot",
    );
    assert!(
        app_body.contains("main_menu_fallback_state: MainMenuFallbackState,"),
        "ScriptListApp must expose one named fallback owner field."
    );
    for loose_field in [
        "fallback_mode: bool,",
        "fallback_selected_index: usize,",
        "cached_fallbacks: Vec<crate::fallbacks::FallbackItem>,",
    ] {
        assert!(
            !app_body.contains(loose_field),
            "ScriptListApp must not keep {loose_field} as loose state."
        );
    }
}

#[test]
fn fallback_owner_exposes_small_behavior_methods() {
    for method in [
        "fn is_active(&self) -> bool",
        "fn clear(&mut self)",
        "fn replace_items(&mut self, items: Vec<crate::fallbacks::FallbackItem>)",
        "fn selected_item(&self) -> Option<&crate::fallbacks::FallbackItem>",
        "fn move_up(&mut self) -> bool",
        "fn move_down(&mut self) -> bool",
    ] {
        assert!(
            APP_STATE.contains(method),
            "MainMenuFallbackState must expose {method}"
        );
    }
}

#[test]
fn startup_and_filtering_route_through_fallback_owner() {
    assert!(
        STARTUP.contains("main_menu_fallback_state: MainMenuFallbackState::default(),"),
        "Live startup must initialize fallback state through its named default."
    );
    assert!(
        FILTER_INPUT_UPDATES.contains("self.main_menu_fallback_state.replace_items(fallbacks);"),
        "Filtering must activate fallback state through the owner."
    );
    assert!(
        SELECTION_FALLBACK.contains("self.main_menu_fallback_state.selected_item().cloned()"),
        "Fallback execution must resolve the selected fallback through the owner."
    );
}

#[test]
fn keyboard_and_stdin_navigation_route_through_fallback_owner() {
    for (name, source) in [
        ("render_script_list", RENDER_SCRIPT_LIST),
        ("runtime_stdin", RUNTIME_STDIN),
        (
            "runtime_stdin_match_simulate_key",
            RUNTIME_STDIN_MATCH_SIMULATE_KEY,
        ),
    ] {
        assert!(
            source.contains("main_menu_fallback_state.is_active()"),
            "{name} must branch on the fallback owner."
        );
        assert!(
            source.contains("main_menu_fallback_state.move_up()")
                && source.contains("main_menu_fallback_state.move_down()"),
            "{name} fallback navigation must move selection through the owner."
        );
    }
}
