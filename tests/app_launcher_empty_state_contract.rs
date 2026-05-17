const APP_LAUNCHER: &str = include_str!("../src/render_builtins/app_launcher.rs");

#[test]
fn app_launcher_empty_state_copy_is_modeled() {
    assert!(
        APP_LAUNCHER.contains("enum AppLauncherEmptyState")
            && APP_LAUNCHER.contains("NoApplicationsFound")
            && APP_LAUNCHER.contains("NoFilteredMatches"),
        "App Launcher empty-state copy should use named states"
    );
    assert!(
        APP_LAUNCHER.contains("fn from_filter(filter: &str) -> Self")
            && APP_LAUNCHER.contains("fn message(self) -> &'static str"),
        "App Launcher empty states should own filter classification and visible copy"
    );
    assert!(
        APP_LAUNCHER.contains("AppLauncherEmptyState::from_filter(&filter).message()"),
        "App Launcher renderer should derive empty-state copy from the model"
    );
    assert!(
        !APP_LAUNCHER.contains("child(if filter.is_empty()"),
        "App Launcher empty-state copy must not regress to inline filter-empty branching"
    );
}
