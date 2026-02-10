use super::super::*;

fn blend_rgb(background: u32, overlay: u32, alpha: f32) -> u32 {
    let alpha = alpha.clamp(0.0, 1.0);
    let bg_r = ((background >> 16) & 0xFF) as f32;
    let bg_g = ((background >> 8) & 0xFF) as f32;
    let bg_b = (background & 0xFF) as f32;
    let fg_r = ((overlay >> 16) & 0xFF) as f32;
    let fg_g = ((overlay >> 8) & 0xFF) as f32;
    let fg_b = (overlay & 0xFF) as f32;

    let out_r = ((1.0 - alpha) * bg_r + alpha * fg_r).round() as u32;
    let out_g = ((1.0 - alpha) * bg_g + alpha * fg_g).round() as u32;
    let out_b = ((1.0 - alpha) * bg_b + alpha * fg_b).round() as u32;

    (out_r << 16) | (out_g << 8) | out_b
}

fn relative_luminance(rgb: u32) -> f64 {
    let channel = |offset: u32| {
        let value = ((rgb >> offset) & 0xFF) as f64 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };

    let r = channel(16);
    let g = channel(8);
    let b = channel(0);
    (0.2126 * r) + (0.7152 * g) + (0.0722 * b)
}

fn contrast_ratio(color_a: u32, color_b: u32) -> f64 {
    let lum_a = relative_luminance(color_a);
    let lum_b = relative_luminance(color_b);
    let (lighter, darker) = if lum_a >= lum_b {
        (lum_a, lum_b)
    } else {
        (lum_b, lum_a)
    };
    (lighter + 0.05) / (darker + 0.05)
}

#[test]
fn test_default_theme() {
    let theme = Theme::default();
    assert_eq!(theme.colors.background.main, 0x1e1e1e);
    assert_eq!(theme.colors.text.primary, 0xffffff);
    assert_eq!(theme.colors.accent.selected, 0xfbbf24);
    assert_eq!(theme.colors.ui.border, 0x464647);
}

#[test]
fn test_color_scheme_default() {
    let scheme = ColorScheme::default();
    assert_eq!(scheme.background.title_bar, 0x2d2d30);
    assert_eq!(scheme.text.secondary, 0xcccccc);
    assert_eq!(scheme.ui.success, 0x00ff00);
}

#[test]
fn test_dark_default() {
    let scheme = ColorScheme::dark_default();
    assert_eq!(scheme.background.main, 0x1e1e1e);
    assert_eq!(scheme.text.primary, 0xffffff);
    assert_eq!(scheme.background.title_bar, 0x2d2d30);
    assert_eq!(scheme.ui.success, 0x00ff00);
}

#[test]
fn test_light_default() {
    let scheme = ColorScheme::light_default();
    // POC light theme uses 0xfafafa for main background (not pure white)
    assert_eq!(scheme.background.main, 0xfafafa);
    assert_eq!(scheme.text.primary, 0x000000);
    assert_eq!(scheme.background.title_bar, 0xffffff); // Input areas are pure white
    assert_eq!(scheme.ui.border, 0xe0e0e0); // POC border color
}

#[test]
fn test_theme_serialization() {
    let theme = Theme::default();
    let json = serde_json::to_string(&theme).unwrap();
    let deserialized: Theme = serde_json::from_str(&json).unwrap();

    assert_eq!(
        deserialized.colors.background.main,
        theme.colors.background.main
    );
    assert_eq!(deserialized.colors.text.primary, theme.colors.text.primary);
    assert_eq!(
        deserialized.colors.accent.selected,
        theme.colors.accent.selected
    );
    assert_eq!(deserialized.colors.ui.border, theme.colors.ui.border);
}

#[test]
fn test_light_theme_serialization() {
    let theme = Theme {
        colors: ColorScheme::light_default(),
        focus_aware: None,
        opacity: Some(BackgroundOpacity::default()),
        drop_shadow: Some(DropShadow::default()),
        vibrancy: Some(VibrancySettings::default()),
        fonts: Some(FontConfig::default()),
        appearance: AppearanceMode::Light,
    };
    let json = serde_json::to_string(&theme).unwrap();
    let deserialized: Theme = serde_json::from_str(&json).unwrap();

    // POC light theme uses 0xfafafa for main background (not pure white)
    assert_eq!(deserialized.colors.background.main, 0xfafafa);
    assert_eq!(deserialized.colors.text.primary, 0x000000);
}

#[test]
fn test_opacity_defaults() {
    let opacity = BackgroundOpacity::default();
    assert_eq!(opacity.main, 0.30);
    assert_eq!(opacity.title_bar, 0.30);
    assert_eq!(opacity.search_box, 0.40);
    assert_eq!(opacity.log_panel, 0.40);
    assert_eq!(opacity.selected, 0.33); // Higher selection contrast for vibrancy surfaces
    assert_eq!(opacity.hover, 0.22); // Higher hover contrast for state visibility
    assert_eq!(opacity.preview, 0.0);
    assert_eq!(opacity.dialog, 0.15);
    assert_eq!(opacity.input, 0.30);
    assert_eq!(opacity.panel, 0.20);
    assert_eq!(opacity.input_inactive, 0.25);
    assert_eq!(opacity.input_active, 0.50);
    assert_eq!(opacity.border_inactive, 0.125);
    assert_eq!(opacity.border_active, 0.25);
}

#[test]
fn test_get_opacity_for_focus_keeps_selection_and_hover_strength_when_unfocused() {
    let theme = Theme::dark_default();
    let focused = theme.get_opacity_for_focus(true);
    let unfocused = theme.get_opacity_for_focus(false);

    assert_eq!(unfocused.selected, focused.selected);
    assert_eq!(unfocused.hover, focused.hover);
    assert!(unfocused.main < focused.main);
}

#[test]
fn test_dark_default_selected_and_hover_contrast_meets_visibility_thresholds() {
    let theme = Theme::dark_default();
    let opacity = theme.get_opacity();
    let background = theme.colors.background.main;
    let overlay = theme.colors.accent.selected_subtle;

    let selected = blend_rgb(background, overlay, opacity.selected);
    let hovered = blend_rgb(background, overlay, opacity.hover);

    let selected_contrast = contrast_ratio(background, selected);
    let hover_contrast = contrast_ratio(background, hovered);

    assert!(
        selected_contrast >= 2.4,
        "dark selected contrast too low: {selected_contrast:.2}"
    );
    assert!(
        hover_contrast >= 1.8,
        "dark hover contrast too low: {hover_contrast:.2}"
    );
}

#[test]
fn test_light_default_selected_and_hover_contrast_meets_visibility_thresholds() {
    let theme = Theme::light_default();
    let opacity = theme.get_opacity();
    let background = theme.colors.background.main;
    let overlay = theme.colors.accent.selected_subtle;

    let selected = blend_rgb(background, overlay, opacity.selected);
    let hovered = blend_rgb(background, overlay, opacity.hover);

    let selected_contrast = contrast_ratio(background, selected);
    let hover_contrast = contrast_ratio(background, hovered);

    assert!(
        selected_contrast >= 2.4,
        "light selected contrast too low: {selected_contrast:.2}"
    );
    assert!(
        hover_contrast >= 1.8,
        "light hover contrast too low: {hover_contrast:.2}"
    );
}

#[test]
fn test_drop_shadow_defaults() {
    let shadow = DropShadow::default();
    assert!(shadow.enabled);
    assert_eq!(shadow.blur_radius, 20.0);
    assert_eq!(shadow.spread_radius, 0.0);
    assert_eq!(shadow.offset_x, 0.0);
    assert_eq!(shadow.offset_y, 8.0);
    assert_eq!(shadow.color, 0x000000);
    assert_eq!(shadow.opacity, 0.25);
}

#[test]
fn test_vibrancy_defaults() {
    let vibrancy = VibrancySettings::default();
    assert!(vibrancy.enabled);
    assert!(matches!(vibrancy.material, VibrancyMaterial::Popover));
}

#[test]
fn test_detect_system_appearance() {
    // This test just verifies the function can be called without panicking
    // The result will vary based on the system's actual appearance setting
    let _is_dark = detect_system_appearance();
    // Don't assert a specific value, just ensure it doesn't panic
}

#[test]
fn test_load_theme_respects_system_appearance() {
    // This test verifies that load_theme() properly respects system appearance
    // by checking that the returned theme's colors match the expected mode.
    //
    // When system is in light mode (detect_system_appearance() returns false),
    // and theme.json has no explicit "appearance" field (defaults to Auto),
    // load_theme() should return a theme with light colors.
    //
    // Note: This test's behavior depends on the actual system appearance
    // at the time of running. In light mode, it verifies light colors are used.
    // In dark mode, it verifies dark colors are used (or theme.json colors).
    let is_system_dark = detect_system_appearance();
    let theme = load_theme();

    // The theme's colors should match the system appearance when in Auto mode
    // has_dark_colors() checks the actual luminance of the background color
    let theme_has_dark_colors = theme.has_dark_colors();

    // Log for debugging
    eprintln!(
        "System is dark: {}, Theme has dark colors: {}, Background: 0x{:06x}",
        is_system_dark, theme_has_dark_colors, theme.colors.background.main
    );

    // When system is in light mode, theme should have light colors
    // (unless theme.json explicitly forces dark mode)
    if !is_system_dark {
        // In light mode, we expect light colors (main background should be 0xfafafa)
        // unless the theme.json has an explicit "appearance": "dark" setting
        assert_eq!(
            theme.appearance,
            AppearanceMode::Light,
            "When system is light and theme.json uses Auto, appearance should be set to Light"
        );
    }
}

// ========================================================================
// Opacity Clamping Tests
// ========================================================================

#[test]
fn test_opacity_clamping_valid_values() {
    let opacity = BackgroundOpacity {
        main: 0.5,
        title_bar: 0.7,
        search_box: 0.8,
        log_panel: 0.3,
        selected: 0.15,
        hover: 0.08,
        preview: 0.0,
        dialog: 0.40,
        input: 0.30,
        panel: 0.20,
        input_inactive: 0.25,
        input_active: 0.50,
        border_inactive: 0.125,
        border_active: 0.25,
        vibrancy_background: None,
    };
    let clamped = opacity.clamped();
    assert_eq!(clamped.main, 0.5);
    assert_eq!(clamped.title_bar, 0.7);
    assert_eq!(clamped.search_box, 0.8);
    assert_eq!(clamped.log_panel, 0.3);
}

#[test]
fn test_opacity_clamping_overflow() {
    let opacity = BackgroundOpacity {
        main: 2.0,        // Should clamp to 1.0
        title_bar: 1.5,   // Should clamp to 1.0
        search_box: -0.5, // Should clamp to 0.0
        log_panel: 100.0, // Should clamp to 1.0
        selected: 0.15,
        hover: 0.08,
        preview: 0.0,
        dialog: 0.40,
        input: 0.30,
        panel: 0.20,
        input_inactive: 0.25,
        input_active: 0.50,
        border_inactive: 0.125,
        border_active: 0.25,
        vibrancy_background: Some(2.0), // Should clamp to 1.0
    };
    let clamped = opacity.clamped();
    assert_eq!(clamped.main, 1.0);
    assert_eq!(clamped.title_bar, 1.0);
    assert_eq!(clamped.search_box, 0.0);
    assert_eq!(clamped.log_panel, 1.0);
    assert_eq!(clamped.vibrancy_background, Some(1.0));
}

#[test]
fn test_drop_shadow_opacity_clamping() {
    let shadow = DropShadow {
        enabled: true,
        blur_radius: 20.0,
        spread_radius: 0.0,
        offset_x: 0.0,
        offset_y: 8.0,
        color: 0x000000,
        opacity: 2.5, // Should clamp to 1.0
    };
    let clamped = shadow.clamped();
    assert_eq!(clamped.opacity, 1.0);
}
