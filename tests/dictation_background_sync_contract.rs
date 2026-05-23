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
    let footer_popup = read_source("src/footer_popup.rs");
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
        dictation.contains("crate::platform::configure_dictation_overlay_window(")
            && dictation.contains("theme.should_use_dark_vibrancy()"),
        "Dictation native window must use the explicit refreshable dictation material helper"
    );
    assert!(
        dictation.contains("crate::footer_popup::sync_window_footer_popup(")
            && dictation.contains("dictation_native_footer_config(")
            && footer_popup.contains("pub(crate) fn sync_window_footer_popup")
            && footer_popup.contains("ensure_main_footer_host(ns_window)")
            && footer_popup.contains("refresh_main_footer_host(ns_window, config)"),
        "Dictation footer must use the same native footer NSVisualEffectView and button refresh path as the main footer"
    );
    assert!(
        footer_popup.contains("static DICTATION_FOOTER_ACTION_CHANNEL")
            && footer_popup.contains("pub(crate) fn dictation_footer_action_channel")
            && footer_popup.contains("send_footer_action_from_sender(sender")
            && footer_popup.contains("footer_sender_is_dictation_window(sender)"),
        "Shared native footer buttons must route dictation-window clicks through the dictation channel, not the main-window listener"
    );
    assert!(
        platform.contains("fn current_window_material()")
            && platform.contains("get_cached_theme().get_vibrancy().material"),
        "shared native window configuration must source material from the cached theme"
    );
    assert!(
        platform.contains("pub unsafe fn configure_dictation_overlay_window")
            && platform.contains(
                "configure_window_vibrancy_common(window, \"DICTATION\", \"Dictation overlay\", is_dark)"
            )
            && platform.contains("Script Kit Dictation")
            && platform.contains("title_string.contains(\"Script Kit Dictation\")"),
        "Dictation overlay must be title-addressable by secondary-window appearance refresh"
    );
    assert!(
        !dictation.contains("PromptFooterColors::from_theme")
            && !dictation.contains("crate::components::prompt_footer::footer_surface_rgba")
            && !dictation.contains("_surface_bg: gpui::Rgba"),
        "Dictation must not keep dead PromptFooter surface background plumbing for its live footer"
    );
}
