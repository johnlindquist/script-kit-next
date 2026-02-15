use super::super::*;
use super::super::types::relative_luminance_srgb;
use serde::{Deserialize, Serialize};

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
    relative_luminance_srgb(rgb) as f64
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
// VibrancyMaterial Enum Tests
// ========================================================================

#[test]
fn test_vibrancy_material_default() {
    use super::super::types::VibrancyMaterial;
    let material = VibrancyMaterial::default();
    assert!(matches!(material, VibrancyMaterial::Popover));
}

#[test]
fn test_vibrancy_material_serialization() {
    use super::super::types::VibrancyMaterial;

    // Test each variant serializes correctly
    assert_eq!(
        serde_json::to_string(&VibrancyMaterial::Hud).unwrap(),
        "\"hud\""
    );
    assert_eq!(
        serde_json::to_string(&VibrancyMaterial::Popover).unwrap(),
        "\"popover\""
    );
    assert_eq!(
        serde_json::to_string(&VibrancyMaterial::Menu).unwrap(),
        "\"menu\""
    );
    assert_eq!(
        serde_json::to_string(&VibrancyMaterial::Sidebar).unwrap(),
        "\"sidebar\""
    );
    assert_eq!(
        serde_json::to_string(&VibrancyMaterial::Content).unwrap(),
        "\"content\""
    );
}

#[test]
fn test_vibrancy_material_deserialization() {
    use super::super::types::VibrancyMaterial;

    // Test each variant deserializes correctly
    assert!(matches!(
        serde_json::from_str::<VibrancyMaterial>("\"hud\"").unwrap(),
        VibrancyMaterial::Hud
    ));
    assert!(matches!(
        serde_json::from_str::<VibrancyMaterial>("\"popover\"").unwrap(),
        VibrancyMaterial::Popover
    ));
    assert!(matches!(
        serde_json::from_str::<VibrancyMaterial>("\"menu\"").unwrap(),
        VibrancyMaterial::Menu
    ));
    assert!(matches!(
        serde_json::from_str::<VibrancyMaterial>("\"sidebar\"").unwrap(),
        VibrancyMaterial::Sidebar
    ));
    assert!(matches!(
        serde_json::from_str::<VibrancyMaterial>("\"content\"").unwrap(),
        VibrancyMaterial::Content
    ));
}

#[test]
fn test_vibrancy_settings_with_material_enum() {
    let json = r#"{"enabled": true, "material": "hud"}"#;
    let settings: VibrancySettings = serde_json::from_str(json).unwrap();
    assert!(settings.enabled);
    assert!(matches!(
        settings.material,
        super::super::types::VibrancyMaterial::Hud
    ));
}

// ========================================================================
// ListItemColors Tests
// ========================================================================

#[test]
fn test_list_item_colors_is_copy() {
    // Compile-time verification that ListItemColors implements Copy
    fn assert_copy<T: Copy>() {}
    assert_copy::<ListItemColors>();
}

#[test]
fn test_list_item_colors_from_dark_scheme() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    // Canonical struct should preserve direct theme color mappings
    assert_eq!(colors.background, scheme.background.main);
    assert_eq!(colors.background_selected, scheme.accent.selected_subtle);
    assert_eq!(colors.text_primary, scheme.text.primary);
    assert_eq!(colors.text_secondary, scheme.text.secondary);

    // Selection state should remain stronger than hover
    assert!(colors.hover_opacity > 0.0);
    assert!(colors.selected_opacity > colors.hover_opacity);
}

#[test]
fn test_list_item_colors_from_light_scheme() {
    let scheme = ColorScheme::light_default();
    let colors = scheme.list_item_colors();

    // Light scheme values should still map directly
    assert_eq!(colors.background_selected, scheme.accent.selected_subtle);
    assert_eq!(colors.warning_bg, scheme.ui.warning);
    assert_eq!(colors.text_on_accent, scheme.text.on_accent);
}

#[test]
fn test_list_item_colors_description_color() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    // Canonical struct exposes selected and secondary description colors directly
    assert_eq!(colors.accent_selected, scheme.accent.selected);
    assert_eq!(colors.text_secondary, scheme.text.secondary);
    assert_ne!(colors.accent_selected, colors.text_secondary);
}

#[test]
fn test_list_item_colors_item_text_color() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    // Selected item text uses primary text; unselected uses secondary
    assert_eq!(colors.text_primary, scheme.text.primary);
    assert_eq!(colors.text_secondary, scheme.text.secondary);
}

#[test]
fn test_list_item_colors_text_as_hsla() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    // Keep explicit coverage for text-on-accent mapping in canonical struct
    assert_eq!(colors.text_on_accent, scheme.text.on_accent);
}

#[test]
fn test_markdown_highlight_theme_styles() {
    let sk_theme = Theme::dark_default();
    let highlight_theme = super::super::gpui_integration::build_markdown_highlight_theme(&sk_theme, true);
    let syntax = &highlight_theme.style.syntax;

    let title = syntax
        .style("title")
        .expect("title highlight should be configured");
    assert_eq!(title.font_weight, Some(gpui::FontWeight::BOLD));
    assert!(title.color.is_some());

    let emphasis = syntax
        .style("emphasis")
        .expect("emphasis highlight should be configured");
    assert_eq!(emphasis.font_style, Some(gpui::FontStyle::Italic));

    let strong = syntax
        .style("emphasis.strong")
        .expect("strong emphasis highlight should be configured");
    assert_eq!(strong.font_weight, Some(gpui::FontWeight::BOLD));

    let literal = syntax
        .style("text.literal")
        .expect("text literal highlight should be configured");
    assert!(literal.color.is_some());

    let link = syntax
        .style("link_text")
        .expect("link highlight should be configured");
    assert!(link.color.is_some());

    let list_marker = syntax
        .style("punctuation.list_marker")
        .expect("list marker highlight should be configured");
    assert!(list_marker.color.is_some());
}

// ========================================================================
// Hex Color Parsing Tests
// ========================================================================

// --- merged from part_03.rs ---
#[test]
fn test_hex_color_parse_hash_prefix() {
    let result = hex_color_serde::parse_color_string("#FBBF24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_lowercase() {
    let result = hex_color_serde::parse_color_string("#fbbf24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_0x_prefix() {
    let result = hex_color_serde::parse_color_string("0xFBBF24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_bare_hex() {
    let result = hex_color_serde::parse_color_string("FBBF24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_rgb() {
    let result = hex_color_serde::parse_color_string("rgb(251, 191, 36)");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_rgba() {
    let result = hex_color_serde::parse_color_string("rgba(251, 191, 36, 1.0)");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_black() {
    assert_eq!(
        hex_color_serde::parse_color_string("#000000").unwrap(),
        0x000000
    );
    assert_eq!(
        hex_color_serde::parse_color_string("rgb(0, 0, 0)").unwrap(),
        0x000000
    );
}

#[test]
fn test_hex_color_parse_white() {
    assert_eq!(
        hex_color_serde::parse_color_string("#FFFFFF").unwrap(),
        0xFFFFFF
    );
    assert_eq!(
        hex_color_serde::parse_color_string("rgb(255, 255, 255)").unwrap(),
        0xFFFFFF
    );
}

#[test]
fn test_hex_color_parse_invalid() {
    assert!(hex_color_serde::parse_color_string("invalid").is_err());
    assert!(hex_color_serde::parse_color_string("#GGG").is_err());
    assert!(hex_color_serde::parse_color_string("rgb(300, 0, 0)").is_err());
    // 300 > 255
}

#[test]
fn test_hex_color_json_deserialize_string() {
    let json = r##"{"main": "#1E1E1E"}"##;
    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(with = "hex_color_serde")]
        main: HexColor,
    }
    let parsed: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.main, 0x1E1E1E);
}

#[test]
fn test_hex_color_json_deserialize_number() {
    let json = r##"{"main": 1973790}"##; // 0x1E1E1E = 1973790
    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(with = "hex_color_serde")]
        main: HexColor,
    }
    let parsed: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.main, 0x1E1E1E);
}

#[test]
fn test_hex_color_json_serialize() {
    #[derive(Serialize)]
    struct TestStruct {
        #[serde(with = "hex_color_serde")]
        main: HexColor,
    }
    let data = TestStruct { main: 0xFBBF24 };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r##"{"main":"#FBBF24"}"##);
}

#[test]
fn test_theme_deserialize_hex_strings() {
    let json = r##"{
        "colors": {
            "background": {
                "main": "#1E1E1E",
                "title_bar": "#2D2D30",
                "search_box": "#3C3C3C",
                "log_panel": "#0D0D0D"
            },
            "text": {
                "primary": "#FFFFFF",
                "secondary": "#CCCCCC",
                "tertiary": "#999999",
                "muted": "#808080",
                "dimmed": "#666666"
            },
            "accent": {
                "selected": "#FBBF24"
            },
            "ui": {
                "border": "#464647",
                "success": "#00FF00"
            }
        }
    }"##;

    let theme: Theme = serde_json::from_str(json).unwrap();
    assert_eq!(theme.colors.background.main, 0x1E1E1E);
    assert_eq!(theme.colors.accent.selected, 0xFBBF24);
    assert_eq!(theme.colors.text.secondary, 0xCCCCCC);
}

#[test]
fn test_theme_deserialize_mixed_formats() {
    // Mix of hex strings and numbers should work
    let json = r##"{
        "colors": {
            "background": {
                "main": "#1E1E1E",
                "title_bar": 2960688,
                "search_box": "rgb(60, 60, 60)",
                "log_panel": "0x0D0D0D"
            },
            "text": {
                "primary": "#FFFFFF",
                "secondary": "#CCCCCC",
                "tertiary": "#999999",
                "muted": "#808080",
                "dimmed": "#666666"
            },
            "accent": {
                "selected": "rgba(251, 191, 36, 1.0)"
            },
            "ui": {
                "border": "#464647",
                "success": "#00FF00"
            }
        }
    }"##;

    let theme: Theme = serde_json::from_str(json).unwrap();
    assert_eq!(theme.colors.background.main, 0x1E1E1E);
    assert_eq!(theme.colors.background.title_bar, 2960688);
    assert_eq!(theme.colors.background.search_box, 0x3C3C3C);
    assert_eq!(theme.colors.accent.selected, 0xFBBF24);
}

#[test]
fn test_theme_prelude_exports_core_theme_types() {
    let theme = crate::theme::prelude::Theme::default();
    let colors = crate::theme::prelude::ColorScheme::default();

    assert_eq!(
        theme.colors.background.main,
        Theme::default().colors.background.main
    );
    assert_eq!(colors.ui.border, ColorScheme::default().ui.border);
}
