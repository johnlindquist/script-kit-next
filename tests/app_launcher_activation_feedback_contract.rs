const APP_LAUNCHER: &str = include_str!("../src/render_builtins/app_launcher.rs");

#[test]
fn app_launcher_activation_uses_named_action_state() {
    assert!(
        APP_LAUNCHER.contains("enum AppLauncherActivationAction")
            && APP_LAUNCHER.contains("LaunchSelectedApp"),
        "App Launcher activation feedback should be driven by a named action state"
    );
    assert!(
        APP_LAUNCHER.contains("AppLauncherActivationAction::LaunchSelectedApp")
            && APP_LAUNCHER.contains("activation_action.launch_log(&app.name)")
            && APP_LAUNCHER.contains("activation_action.success_log(&app.name)")
            && APP_LAUNCHER.contains("activation_action.failure_log(e)")
            && APP_LAUNCHER.contains("activation_action")
            && APP_LAUNCHER.contains(".double_click_launch_log("),
        "App Launcher keyboard and double-click launch feedback should derive from the named action state"
    );
    assert!(
        APP_LAUNCHER.contains("format!(\"Launching app: {app_name}\")")
            && APP_LAUNCHER.contains("format!(\"Double-click launching app: {app_name}\")")
            && APP_LAUNCHER.contains("format!(\"Launched: {app_name}\")")
            && APP_LAUNCHER.contains("format!(\"Failed to launch app: {error}\")"),
        "App Launcher activation feedback should preserve existing log copy"
    );
}
