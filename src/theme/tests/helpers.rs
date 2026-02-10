use super::super::*;

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
// BackgroundRole and background_rgba API Tests
// ========================================================================

#[test]
fn test_background_role_main() {
    use super::super::types::BackgroundRole;
    let theme = Theme::default();
    let rgba = theme.background_rgba(BackgroundRole::Main, true);

    // Should have the correct RGB from colors.background.main (0x1e1e1e)
    // and apply opacity from BackgroundOpacity.main (0.60)
    assert!(rgba.3 > 0.0 && rgba.3 <= 1.0); // Alpha should be valid
}

#[test]
fn test_background_role_unfocused_reduces_opacity() {
    use super::super::types::BackgroundRole;
    let theme = Theme::default();

    let focused = theme.background_rgba(BackgroundRole::Main, true);
    let unfocused = theme.background_rgba(BackgroundRole::Main, false);

    // Unfocused should have lower alpha (10% reduction)
    assert!(unfocused.3 < focused.3);
}

#[test]
fn test_background_role_all_variants() {
    use super::super::types::BackgroundRole;
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
    let highlight_theme =
        super::super::gpui_integration::build_markdown_highlight_theme(&sk_theme, true);
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
