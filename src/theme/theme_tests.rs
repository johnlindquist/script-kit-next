use super::*;
use serde::{Deserialize, Serialize};

#[test]
fn test_default_theme() {
    let theme = Theme::default();
    // Theme::default() now uses system appearance detection
    // So we verify the colors match the expected mode
    if theme.is_dark_mode {
        assert_eq!(theme.colors.background.main, 0x1e1e1e);
        assert_eq!(theme.colors.text.primary, 0xffffff);
        assert_eq!(theme.colors.accent.selected, 0xfbbf24);
        assert_eq!(theme.colors.ui.border, 0x464647);
    } else {
        assert_eq!(theme.colors.background.main, 0xffffff);
        assert_eq!(theme.colors.text.primary, 0x000000);
        assert_eq!(theme.colors.accent.selected, 0x0078d4);
        assert_eq!(theme.colors.ui.border, 0xd0d0d0);
    }
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
        is_dark_mode: false,
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
    assert_eq!(opacity.main, 0.30);
    assert_eq!(opacity.title_bar, 0.30);
    assert_eq!(opacity.search_box, 0.40);
    assert_eq!(opacity.log_panel, 0.40);
    assert_eq!(opacity.selected, 0.12); // Subtle selection with vibrancy like Raycast
    assert_eq!(opacity.hover, 0.07); // Very subtle hover with vibrancy
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
    };
    let clamped = opacity.clamped();
    assert_eq!(clamped.main, 1.0);
    assert_eq!(clamped.title_bar, 1.0);
    assert_eq!(clamped.search_box, 0.0);
    assert_eq!(clamped.log_panel, 1.0);
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

// ============================================================================
// Light Mode Tests
// ============================================================================

#[test]
fn test_theme_is_dark_mode_field_exists() {
    // Verify the is_dark_mode field is accessible
    let theme = Theme::default();
    let _is_dark = theme.is_dark_mode;
    // The value depends on system appearance, so just verify the field exists
}

#[test]
fn test_theme_is_dark_mode_from_json() {
    // Test is_dark_mode serialization/deserialization via round-trip
    // Create a dark theme
    let dark_theme = Theme {
        colors: ColorScheme::dark_default(),
        is_dark_mode: true,
        focus_aware: None,
        opacity: None,
        drop_shadow: None,
        vibrancy: None,
        fonts: None,
    };

    let dark_json = serde_json::to_string(&dark_theme).unwrap();
    let parsed_dark: Theme = serde_json::from_str(&dark_json).unwrap();
    assert!(parsed_dark.is_dark_mode, "Expected is_dark_mode to be true after round-trip");

    // Create a light theme
    let light_theme = Theme {
        colors: ColorScheme::light_default(),
        is_dark_mode: false,
        focus_aware: None,
        opacity: None,
        drop_shadow: None,
        vibrancy: None,
        fonts: None,
    };

    let light_json = serde_json::to_string(&light_theme).unwrap();
    let parsed_light: Theme = serde_json::from_str(&light_json).unwrap();
    assert!(!parsed_light.is_dark_mode, "Expected is_dark_mode to be false after round-trip");
}

#[test]
fn test_background_opacity_for_mode_dark() {
    let opacity = BackgroundOpacity::for_mode(true);
    // Dark mode uses default values
    assert!((opacity.main - 0.30).abs() < 0.01, "Dark mode main should be ~0.30");
    assert!((opacity.selected - 0.12).abs() < 0.01, "Dark mode selected should be ~0.12");
    assert!((opacity.hover - 0.07).abs() < 0.01, "Dark mode hover should be ~0.07");
}

#[test]
fn test_background_opacity_for_mode_light() {
    let opacity = BackgroundOpacity::for_mode(false);
    // Light mode needs higher opacity values
    assert!(opacity.main > 0.5, "Light mode main should be higher (got {})", opacity.main);
    assert!(opacity.selected > 0.1, "Light mode selected should be higher (got {})", opacity.selected);
    assert!(opacity.hover > 0.07, "Light mode hover should be higher or equal (got {})", opacity.hover);
}

#[test]
fn test_light_mode_color_scheme_contrast() {
    let scheme = ColorScheme::light_default();

    // Light mode should have light backgrounds
    assert!(scheme.background.main > 0xF00000, "Light mode background should be light");

    // Light mode should have dark text
    assert!(scheme.text.primary < 0x111111, "Light mode text should be dark");

    // Light mode should have lighter borders than dark mode
    let dark_scheme = ColorScheme::dark_default();
    assert!(scheme.ui.border > dark_scheme.ui.border, "Light mode border should be lighter");
}

#[test]
fn test_theme_default_uses_system_appearance() {
    // The default theme should use system appearance detection
    let theme = Theme::default();

    // Verify theme has proper colors based on mode
    if theme.is_dark_mode {
        assert_eq!(theme.colors.background.main, 0x1e1e1e, "Dark mode should use dark background");
        assert_eq!(theme.colors.text.primary, 0xffffff, "Dark mode should use light text");
    } else {
        assert_eq!(theme.colors.background.main, 0xffffff, "Light mode should use light background");
        assert_eq!(theme.colors.text.primary, 0x000000, "Light mode should use dark text");
    }
}

#[test]
fn test_theme_serialization_includes_is_dark_mode() {
    let theme = Theme {
        is_dark_mode: false, // Force light mode
        ..Theme::default()
    };

    let json = serde_json::to_string(&theme).unwrap();
    assert!(json.contains("is_dark_mode"), "Serialized theme should include is_dark_mode");
    assert!(json.contains("false"), "Serialized theme should have is_dark_mode: false");
}
