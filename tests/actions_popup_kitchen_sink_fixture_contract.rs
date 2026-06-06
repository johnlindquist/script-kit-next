const ACTIONS_FIXTURE_SOURCE: &str = include_str!("../src/actions/kitchen_sink_fixture.rs");
const ACTIONS_MOD_SOURCE: &str = include_str!("../src/actions/mod.rs");
const MAIN_FIXTURE_SOURCE: &str = include_str!("../src/main_sections/kitchen_sink_fixture.rs");
const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../src/app_impl/actions_toggle.rs");

#[test]
fn actions_popup_kitchen_sink_fixture_has_stable_identity_and_manifest() {
    assert!(ACTIONS_MOD_SOURCE.contains("pub(crate) mod kitchen_sink_fixture;"));
    assert!(ACTIONS_FIXTURE_SOURCE.contains(
        "pub(crate) const ACTIONS_POPUP_KITCHEN_SINK_FIXTURE_ID: &str = \"actions-popup-kitchen-sink\";"
    ));
    assert!(ACTIONS_FIXTURE_SOURCE.contains("actions_popup_kitchen_sink_feature_manifest"));

    for feature in [
        "shell:popup-window",
        "search:visible",
        "list:scroll-overflow",
        "row:selected",
        "row:hover-worthy",
        "row:long-title",
        "row:long-description",
        "row:destructive",
        "row:unsectioned",
        "section:headers",
        "context-header:long-title",
        "shortcut:none",
        "shortcut:single-keycap",
        "shortcut:multi-token",
        "shortcut:long-string",
        "icon:present",
        "icon:absent",
        "empty:no-match",
    ] {
        assert!(
            ACTIONS_FIXTURE_SOURCE.contains(feature),
            "actions popup fixture manifest must include {feature}"
        );
    }
}

#[test]
fn actions_popup_kitchen_sink_builds_real_actions_dialog_rows() {
    assert!(ACTIONS_FIXTURE_SOURCE.contains("Action::new"));
    assert!(ACTIONS_FIXTURE_SOURCE.contains(".with_shortcut"));
    assert!(ACTIONS_FIXTURE_SOURCE.contains(".with_icon"));
    assert!(ACTIONS_FIXTURE_SOURCE.contains(".with_section"));
    assert!(ACTIONS_FIXTURE_SOURCE.contains("delete_kitchen_sink_fixture"));
    assert!(!ACTIONS_FIXTURE_SOURCE.contains("panel:actions-popup-kitchen-sink"));
}

#[test]
fn actions_popup_kitchen_sink_opens_real_popup_window() {
    assert!(MAIN_FIXTURE_SOURCE.contains("ActionsDialog::from_actions_with_context"));
    assert!(MAIN_FIXTURE_SOURCE.contains("begin_actions_popup_window_open"));
    assert!(MAIN_FIXTURE_SOURCE.contains("spawn_open_actions_window_with_parent_id"));
    assert!(MAIN_FIXTURE_SOURCE.contains("ACTIONS_POPUP_KITCHEN_SINK_NO_MATCH_QUERY"));
    assert!(ACTIONS_TOGGLE_SOURCE.contains("spawn_open_actions_window_with_parent_id"));
    let production_source = ACTIONS_TOGGLE_SOURCE
        .split("#[cfg(test)]")
        .next()
        .expect("actions_toggle source should have production section");
    assert_eq!(
        production_source
            .matches("match open_actions_window(")
            .count(),
        1,
        "production actions toggle should keep one direct open_actions_window call path"
    );
}
