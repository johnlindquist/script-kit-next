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
    assert_eq!(scheme.background.main, 0xffffff);
    assert_eq!(scheme.text.primary, 0x000000);
    assert_eq!(scheme.background.title_bar, 0xf3f3f3);
    assert_eq!(scheme.ui.border, 0xd0d0d0);
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
    };
    let json = serde_json::to_string(&theme).unwrap();
    let deserialized: Theme = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.colors.background.main, 0xffffff);
    assert_eq!(deserialized.colors.text.primary, 0x000000);
}

#[test]
fn test_opacity_defaults() {
    let opacity = BackgroundOpacity::default();
    assert_eq!(opacity.main, 0.60);
    assert_eq!(opacity.title_bar, 0.65);
    assert_eq!(opacity.search_box, 0.70);
    assert_eq!(opacity.log_panel, 0.55);
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
    assert_eq!(vibrancy.material, "popover");
}

#[test]
fn test_detect_system_appearance() {
    // This test just verifies the function can be called without panicking
    // The result will vary based on the system's actual appearance setting
    let _is_dark = detect_system_appearance();
    // Don't assert a specific value, just ensure it doesn't panic
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

    // Cursor should be cyan (0x00ffff)
    // In rgba, cyan has g=1.0, b=1.0, r=0.0
    assert!(colors.cursor.g > 0.9);
    assert!(colors.cursor.b > 0.9);
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
