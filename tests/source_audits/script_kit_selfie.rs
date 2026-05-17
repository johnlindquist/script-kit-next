use super::read_source;

#[test]
fn script_kit_selfie_is_registered_with_default_shortcut() {
    let builtins = read_source("src/builtins/mod.rs");
    assert!(builtins.contains("\"builtin/script-kit-selfie\""));
    assert!(builtins.contains("\"Script Kit Selfie\""));
    assert!(builtins.contains("UtilityCommandType::ScriptKitSelfie"));

    let registry = read_source("src/builtins/trigger_registry.rs");
    assert!(registry.contains("TriggerBuiltin::ScriptKitSelfie"));
    assert!(registry.contains("\"builtin/script-kit-selfie\""));

    let hotkeys = read_source("src/hotkeys/mod.rs");
    assert!(hotkeys.contains("SCRIPT_KIT_SELFIE_SHORTCUT: &str = \"cmd+alt+1\""));
    assert!(hotkeys.contains("SCRIPT_KIT_SELFIE_COMMAND_ID: &str = \"builtin/script-kit-selfie\""));
}

#[test]
fn script_kit_selfie_captures_composited_desktop_region_and_receipt() {
    let platform = read_source("src/platform/selfie_capture.rs");
    assert!(platform.contains("capture_region"));
    assert!(platform.contains("composited_desktop"));
    assert!(platform.contains("receipt_path"));
    assert!(platform.contains(".scriptkit"));
    assert!(platform.contains("screenshots"));
    assert!(platform.contains("selfies"));
    assert!(platform.contains("ScriptKitSelfieWindowKind::Dictation"));
    assert!(platform.contains("ScriptKitSelfieWindowKind::Notes"));
    assert!(platform.contains("select_script_kit_selfie_candidate_index"));
    assert!(platform.contains("crate::dictation::is_dictation_overlay_open()"));
    assert!(platform.contains("selfie_prefers_dictation_then_notes_before_main_window"));
    assert!(platform.contains("selfie_prefers_notes_over_focused_main_when_dictation_is_absent"));
    assert!(
        platform.contains("selfie_recognizes_titleless_dictation_overlay_when_dictation_is_open")
    );

    let executor = read_source("src/app_execute/builtin_execution.rs");
    assert!(executor.contains("capture_script_kit_selfie(&state)"));
    assert!(executor.contains("let state = self.app_view_name();"));
}
