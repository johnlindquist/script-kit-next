const BUILTINS: &str = include_str!("../src/builtins/mod.rs");
const TRIGGER_REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const TRIGGER_RESOLVE: &str = include_str!("../src/builtins/trigger_resolve.rs");
const ROUTES: &str = include_str!("../src/app_impl/routes.rs");
const EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");
const HOTKEYS: &str = include_str!("../src/hotkeys/mod.rs");
const SELFIE_CAPTURE: &str = include_str!("../src/platform/selfie_capture.rs");

#[test]
fn script_kit_selfie_builtin_is_pruned_from_registry_and_routes() {
    for required in [
        "ScriptKitSelfie",
        "\"builtin/script-kit-selfie\"",
        "\"script-kit-selfie\"",
        "\"scriptkitselfie\"",
        "\"selfie\"",
        "\"screenshot-selfie\"",
    ] {
        assert!(
            !TRIGGER_REGISTRY.contains(required),
            "trigger registry should not include pruned Script Kit Selfie field: {required}"
        );
    }
    assert!(
        !TRIGGER_RESOLVE.contains("TriggerBuiltin::ScriptKitSelfie => \"ScriptKitSelfie\""),
        "trigger resolver should not render pruned ScriptKitSelfie golden outcomes"
    );
    assert!(
        !ROUTES.contains(
            "TriggerBuiltin::ScriptKitSelfie => AppRoute::ExecuteBuiltin(\"builtin/script-kit-selfie\")"
        ),
        "trigger route planner should not execute the pruned Script Kit Selfie builtin"
    );
}

#[test]
fn script_kit_selfie_builtin_text_matches_capture_receipt_behavior() {
    for required in [
        "UtilityCommandType::ScriptKitSelfie",
        "\"Script Kit Selfie\"",
        "\"Capture Script Kit with the current desktop background and save a receipt\"",
        "UtilityCommandType::ScriptKitSelfie => \"Capture Selfie\"",
        "UtilityCommandType::ScriptKitSelfie => \"Selfie\"",
    ] {
        assert!(
            !BUILTINS.contains(required),
            "builtin command text should not expose pruned Script Kit Selfie behavior: {required}"
        );
    }
    assert!(
        SELFIE_CAPTURE.contains("pub fn capture_script_kit_selfie(state: &str)")
            && SELFIE_CAPTURE.contains("capture_method: \"xcap.monitor.capture_region.composited_desktop\"")
            && SELFIE_CAPTURE.contains("std::fs::write(&receipt_path, receipt_json)"),
        "Script Kit Selfie platform implementation should capture the composited desktop region and write a receipt"
    );
}

#[test]
fn script_kit_selfie_execution_path_is_pruned() {
    for required in [
        "enum UtilitySelfieBuiltinAction",
        "ScriptKitSelfie => Self::Selfie",
        "fn starting_hud(self, state: &str) -> String",
        "fn saved_hud(self, receipt: &crate::platform::ScriptKitSelfieReceipt) -> String",
        "fn failure_message(self, error: &dyn std::fmt::Display) -> String",
        "execute_utility_selfie_builtin",
        "crate::platform::capture_script_kit_selfie(&state)",
    ] {
        assert!(
            !EXECUTION.contains(required),
            "Script Kit Selfie execution should not expose pruned action behavior: {required}"
        );
    }
}

#[test]
fn script_kit_selfie_default_hotkey_is_pruned() {
    assert!(
        !HOTKEYS.contains("SCRIPT_KIT_SELFIE_COMMAND_ID")
            && !HOTKEYS.contains("\"builtin/script-kit-selfie\"")
            && !HOTKEYS.contains("\"Script Kit Selfie\""),
        "Script Kit Selfie should not register a default launcher shortcut"
    );
}
