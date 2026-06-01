use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn root_launcher_placeholder_uses_selected_guidance_copy() {
    let app_state = read_source("src/main_sections/app_state.rs");

    assert!(
        app_state.contains("pub(crate) const ROOT_LAUNCHER_PLACEHOLDER"),
        "root launcher placeholder should live behind one named constant"
    );
    assert!(
        app_state.contains("Search • @ context • / skills • . profile"),
        "root launcher placeholder should match the selected WebChoices copy"
    );
}

#[test]
fn root_launcher_placeholder_replaces_theme_identifier_resets() {
    let startup = read_source("src/app_impl/startup.rs");
    let startup_new = read_source("src/app_impl/startup_new_prelude.rs");
    let lifecycle_reset = read_source("src/app_impl/lifecycle_reset.rs");
    let registries_state = read_source("src/app_impl/registries_state.rs");
    let theme_focus = read_source("src/app_impl/theme_focus.rs");

    for (path, source) in [
        ("src/app_impl/startup.rs", startup.as_str()),
        ("src/app_impl/startup_new_prelude.rs", startup_new.as_str()),
        ("src/app_impl/lifecycle_reset.rs", lifecycle_reset.as_str()),
        (
            "src/app_impl/registries_state.rs",
            registries_state.as_str(),
        ),
        ("src/app_impl/theme_focus.rs", theme_focus.as_str()),
    ] {
        assert!(
            source.contains("ROOT_LAUNCHER_PLACEHOLDER"),
            "{path} should reset the root launcher placeholder through the shared constant"
        );
    }

    assert!(
        !startup.contains("MainMenuThemeVariant::default().placeholder()"),
        "startup should not expose the theme identifier as the root placeholder"
    );
    assert!(
        !startup_new.contains("AccentVariation::default().placeholder()"),
        "new startup path should not expose the accent identifier as the root placeholder"
    );
    assert!(
        !lifecycle_reset.contains("current_main_menu_theme.placeholder()"),
        "script-exit reset should restore the selected launcher guidance copy"
    );
    assert!(
        !registries_state.contains("current_main_menu_theme.placeholder()"),
        "focus reset should restore the selected launcher guidance copy"
    );
    assert!(
        !theme_focus.contains("new_theme.placeholder()"),
        "theme cycling should not replace the root launcher guidance copy"
    );
}
