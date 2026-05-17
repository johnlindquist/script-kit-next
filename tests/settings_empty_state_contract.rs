const SETTINGS: &str = include_str!("../src/render_builtins/settings.rs");

#[test]
fn settings_empty_state_copy_is_modeled() {
    assert!(
        SETTINGS.contains("enum SettingsEmptyState")
            && SETTINGS.contains("NoSettingsAvailable")
            && SETTINGS.contains("NoFilteredMatches"),
        "Settings empty-state copy should use named states"
    );
    assert!(
        SETTINGS.contains("fn from_filter(filter: &str) -> Self")
            && SETTINGS.contains("fn message(self) -> &'static str"),
        "Settings empty states should own filter classification and visible copy"
    );
    assert!(
        SETTINGS.contains("SettingsEmptyState::from_filter(&filter).message()"),
        "Settings renderer should derive empty-state copy from the model"
    );
    assert!(
        !SETTINGS.contains("child(if filter.is_empty()"),
        "Settings empty-state copy must not regress to inline filter-empty branching"
    );
}
