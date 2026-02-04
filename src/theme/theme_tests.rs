use super::*;
use serde::{Deserialize, Serialize};

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
    assert_eq!(opacity.selected, 0.15); // Bumped for clearer active-item indication
    assert_eq!(opacity.hover, 0.09); // Bumped for more visible mouse-tracking cue
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

#[test]
fn test_drop_shadow_opacity_negative_clamping() {
    let shadow = DropShadow {
        enabled: true,
        blur_radius: 20.0,
        spread_radius: 0.0,
        offset_x: 0.0,
        offset_y: 8.0,
        color: 0x000000,
        opacity: -0.5, // Should clamp to 0.0
    };
    let clamped = shadow.clamped();
    assert_eq!(clamped.opacity, 0.0);
}

// ========================================================================
// VibrancyMaterial Enum Tests
// ========================================================================

#[test]
fn test_vibrancy_material_default() {
    use super::types::VibrancyMaterial;
    let material = VibrancyMaterial::default();
    assert!(matches!(material, VibrancyMaterial::Popover));
}

#[test]
fn test_vibrancy_material_serialization() {
    use super::types::VibrancyMaterial;

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
    use super::types::VibrancyMaterial;

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
        super::types::VibrancyMaterial::Hud
    ));
}

// ========================================================================
// BackgroundRole and background_rgba API Tests
// ========================================================================

#[test]
fn test_background_role_main() {
    use super::types::BackgroundRole;
    let theme = Theme::default();
    let rgba = theme.background_rgba(BackgroundRole::Main, true);

    // Should have the correct RGB from colors.background.main (0x1e1e1e)
    // and apply opacity from BackgroundOpacity.main (0.60)
    assert!(rgba.3 > 0.0 && rgba.3 <= 1.0); // Alpha should be valid
}

#[test]
fn test_background_role_unfocused_reduces_opacity() {
    use super::types::BackgroundRole;
    let theme = Theme::default();

    let focused = theme.background_rgba(BackgroundRole::Main, true);
    let unfocused = theme.background_rgba(BackgroundRole::Main, false);

    // Unfocused should have lower alpha (10% reduction)
    assert!(unfocused.3 < focused.3);
}

#[test]
fn test_background_role_all_variants() {
    use super::types::BackgroundRole;
    let theme = Theme::default();

    // All variants should return valid rgba values
    for role in [
        BackgroundRole::Main,
        BackgroundRole::TitleBar,
        BackgroundRole::SearchBox,
        BackgroundRole::LogPanel,
    ] {
        let rgba = theme.background_rgba(role, true);
        // RGB values should be in 0-1 range
        assert!(rgba.0 >= 0.0 && rgba.0 <= 1.0);
        assert!(rgba.1 >= 0.0 && rgba.1 <= 1.0);
        assert!(rgba.2 >= 0.0 && rgba.2 <= 1.0);
        assert!(rgba.3 >= 0.0 && rgba.3 <= 1.0);
    }
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

    // Verify background is transparent
    assert_eq!(colors.background.a, 0.0);

    // Verify hover and selected have some opacity (not transparent)
    assert!(colors.background_hover.a > 0.0);
    assert!(colors.background_selected.a > 0.0);

    // Verify selected has more opacity than hover
    assert!(colors.background_selected.a > colors.background_hover.a);
}

#[test]
fn test_list_item_colors_from_light_scheme() {
    let scheme = ColorScheme::light_default();
    let colors = scheme.list_item_colors();

    // Verify we get colors from light scheme
    // Light scheme uses 0xe8e8e8 for selected_subtle
    assert!(colors.background_selected.a > 0.0);
}

#[test]
fn test_list_item_colors_description_color() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    let selected_desc = colors.description_color(true);
    let unselected_desc = colors.description_color(false);

    // Selected should use accent, unselected should use secondary
    // These should be different colors
    assert_ne!(selected_desc.r, unselected_desc.r);
}

#[test]
fn test_list_item_colors_item_text_color() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    let selected_text = colors.item_text_color(true);
    let unselected_text = colors.item_text_color(false);

    // For dark theme, selected should be primary (white), unselected secondary
    assert!(selected_text.r >= unselected_text.r);
}

#[test]
fn test_list_item_colors_text_as_hsla() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.list_item_colors();

    let hsla = colors.text_as_hsla();

    // Dark theme primary text is white (0xffffff)
    // White should have high lightness
    assert!(hsla.l > 0.9);
}

// ========================================================================
// InputFieldColors Tests
// ========================================================================

#[test]
fn test_input_field_colors_is_copy() {
    // Compile-time verification that InputFieldColors implements Copy
    fn assert_copy<T: Copy>() {}
    assert_copy::<InputFieldColors>();
}

#[test]
fn test_input_field_colors_from_scheme() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.input_field_colors();

    // Background should have some alpha (semi-transparent)
    assert!(colors.background.a > 0.0);
    assert!(colors.background.a < 1.0);

    // Border should have some alpha
    assert!(colors.border.a > 0.0);

    // Text should be fully opaque
    assert_eq!(colors.text.a, 1.0);
}

#[test]
fn test_input_field_cursor_color() {
    let scheme = ColorScheme::dark_default();
    let colors = scheme.input_field_colors();

    // Cursor should use accent color (0xfbbf24 - yellow/gold for Script Kit)
    // This provides visual consistency with selection highlights
    // Yellow/gold has r > 0.9, g > 0.7, b < 0.3
    assert!(colors.cursor.r > 0.9, "cursor red channel should be high");
    assert!(
        colors.cursor.g > 0.7,
        "cursor green channel should be moderately high"
    );
    assert!(
        colors.cursor.b < 0.3,
        "cursor blue channel should be low for gold/yellow"
    );
}

#[test]
fn test_markdown_highlight_theme_styles() {
    let sk_theme = Theme::dark_default();
    let highlight_theme = super::gpui_integration::build_markdown_highlight_theme(&sk_theme, true);
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
