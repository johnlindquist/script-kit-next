use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn dictation_window_uses_shared_theme_background_and_material() {
    let ui_foundation = read_source("src/ui_foundation/mod.rs");
    let dictation = read_source("src/dictation/window.rs");
    let platform = read_source("src/platform/secondary_window_config.rs");

    assert!(
        ui_foundation.contains("pub fn main_window_matched_background(theme: &Theme) -> Rgba"),
        "ui_foundation must expose one shared helper for main-window-matched background tint"
    );
    assert!(
        ui_foundation.contains("vibrancy_background")
            && ui_foundation.contains("unwrap_or(opacity.main)")
            && ui_foundation.contains("hex_to_rgba_with_opacity(")
            && ui_foundation.contains("theme.colors.background.main"),
        "shared helper must use the HUD/main-window opacity path: vibrancy_background or main opacity over theme background"
    );
    assert_eq!(
        dictation
            .matches("crate::ui_foundation::main_window_matched_background(&theme)")
            .count(),
        2,
        "Dictation runtime render and static preview must both use the shared background helper"
    );
    assert!(
        !dictation.contains("crate::ui_foundation::get_vibrancy_background(&theme)"),
        "Dictation must not use get_vibrancy_background because it returns None when vibrancy is enabled"
    );
    assert!(
        dictation.contains("theme_background_gradient_layers(\"dictation-bg-layer\", &theme)")
            && dictation.contains("\"dictation-preview-bg-layer\""),
        "Dictation must preserve runtime and preview theme gradient layers"
    );
    assert!(
        dictation.contains("crate::platform::configure_secondary_window_vibrancy(")
            && dictation.contains("theme.should_use_dark_vibrancy()"),
        "Dictation native window must stay on the shared secondary-window vibrancy path"
    );
    assert!(
        platform.contains("fn current_window_material()")
            && platform.contains("get_cached_theme().get_vibrancy().material"),
        "shared native window configuration must source material from the cached theme"
    );
}
