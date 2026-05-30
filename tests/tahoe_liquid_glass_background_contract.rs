use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn tahoe_liquid_glass_is_gated_and_uses_shared_theme_tint() {
    let platform = read_source("src/platform/secondary_window_config.rs");
    let main_vibrancy = read_source("src/platform/vibrancy_config.rs");
    let ui_foundation = read_source("src/ui_foundation/mod.rs");

    assert!(
        platform.contains("NSClassFromString")
            && platform.contains("c\"NSGlassEffectView\".as_ptr()")
            && platform.contains("class availability is the capability gate"),
        "Liquid Glass must only be enabled when the macOS 26 NSGlassEffectView API is present"
    );
    assert!(
        platform.contains("setStyle: 0isize") && platform.contains("NSGlassEffectViewStyleRegular"),
        "Tahoe backgrounds must use the public regular Liquid Glass style"
    );
    assert!(
        ui_foundation.contains("pub fn main_window_matched_background_rgba(theme: &Theme) -> u32")
            && ui_foundation
                .contains("pub fn main_window_matched_background(theme: &Theme) -> Rgba")
            && ui_foundation.contains("main_window_matched_background_rgba(theme)"),
        "GPUI and native liquid-glass backgrounds must share one theme-derived tint helper"
    );
    assert!(
        platform.contains("crate::ui_foundation::main_window_matched_background_rgba(&theme)")
            && platform.contains("setTintColor: tint_color"),
        "Native Liquid Glass tint must be resolved from the shared main-window matched theme value"
    );
    assert!(
        platform.contains("configure_tahoe_liquid_glass_background(window, log_target, window_name)")
            && main_vibrancy
                .contains("configure_tahoe_liquid_glass_background(window, \"PANEL\", \"Main window\")"),
        "Both shared secondary-window vibrancy and main-window vibrancy paths must install Tahoe Liquid Glass"
    );
    assert!(
        platform.contains("configure_tahoe_liquid_glass_background(\n                        window,\n                        \"APPEARANCE\",")
            && platform.contains("title_string.contains(\"Script Kit Dictation\")"),
        "Theme/appearance refresh must retint existing secondary Liquid Glass backgrounds, including Dictation"
    );
    assert!(
        main_vibrancy
            .contains("crate::ui_foundation::main_window_matched_background_rgba(&theme)")
            && main_vibrancy.contains("material, background_tint"),
        "Main-window Liquid Glass refresh de-dupe must include the shared theme tint, not just dark/material state"
    );
}
