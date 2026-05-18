use script_kit_gpui::test_utils::read_source;

#[test]
fn global_actions_source_keeps_public_builtin_copy() {
    let source = read_source("src/actions/builders/script_context.rs");

    for expected in [
        "pub fn get_global_actions()",
        "\"reload_scripts\"",
        "\"Reload Scripts\"",
        "\"Re-scan ~/.scriptkit and rebuild the script index\"",
        "\"settings\"",
        "\"Open Settings\"",
        "\"Open ~/.scriptkit/config.ts in your editor\"",
        "\"view_logs\"",
        "\"Show Logs\"",
        "\"Toggle the in-launcher log panel\"",
    ] {
        assert!(
            source.contains(expected),
            "global actions source must keep public copy/token: {expected}"
        );
    }
}

#[test]
fn builtin_primary_action_copy_contract_stays_in_source_tests() {
    let source = read_source("src/actions/builders/script_context.rs");

    for expected in [
        "ScriptContextKind::BuiltIn",
        "PrimaryActionPlan::PreserveCatalogActionText",
        "test_get_script_context_actions_preserves_builtin_action_text",
        "test_get_script_context_actions_skips_toggle_favorite_for_builtin_items",
    ] {
        assert!(
            source.contains(expected),
            "built-in primary action copy contract must stay covered: {expected}"
        );
    }
}

#[test]
fn main_actions_path_appends_global_actions() {
    let source = read_source("src/app_actions/handle_action/mod.rs");

    for expected in [
        "actions.extend(crate::actions::get_global_actions())",
        "on_script_list",
        "has_actions.check",
    ] {
        assert!(
            source.contains(expected),
            "main actions path must keep global actions discoverable: {expected}"
        );
    }
}
