use super::read_source;

#[test]
fn system_action_hotkeys_execute_without_opening_main_menu() {
    let execution = read_source("src/app_impl/execution_scripts.rs");
    assert!(execution.contains("fn builtin_entry_needs_main_window("));
    assert!(execution.contains("builtins::BuiltInFeature::App(_) => false"));
    assert!(execution.contains("builtins::BuiltInFeature::MenuBarAction(_) => false"));
    assert!(execution.contains("builtins::BuiltInFeature::SystemAction(_) => false"));
    assert!(
        execution.contains("builtins::BuiltInFeature::SettingsCommand(command) => match command")
    );
    assert!(execution.contains("builtins::SettingsCommandType::ResetWindowPositions"));
    assert!(
        execution.contains("builtins::BuiltInFeature::UtilityCommand(command) => match command")
    );
    assert!(!execution.contains("builtins::UtilityCommandType::ScriptKitSelfie"));
    assert!(execution.contains("return builtin_entry_needs_main_window(&entry);"));
    assert!(execution.contains("builtin/volume-0"));
    assert!(execution.contains("builtin/reset-window-positions"));
    assert!(execution.contains("Volume shortcuts should execute headlessly"));

    let builtin_execution = read_source("src/app_execute/builtin_execution.rs");
    assert!(builtin_execution.contains("self.show_hud(message, Some(HUD_MEDIUM_MS), cx);"));
    assert!(builtin_execution.contains("Window positions reset"));
    assert!(builtin_execution.contains("self.hide_main_and_reset(cx);"));
}
