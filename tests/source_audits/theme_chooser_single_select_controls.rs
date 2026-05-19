use super::read_source;

#[test]
fn theme_chooser_customize_opacity_controls_are_single_select() {
    let source = read_source("src/render_builtins/theme_chooser.rs");

    assert!(!source.contains("OPACITY_MATCH_TOLERANCE"));
    assert!(source.contains("fn closest_float_preset_index("));
    assert!(source.contains("let current_text_opacity_index = Self::closest_float_preset_index("));
    assert!(source.contains("let is_current = i == current_text_opacity_index;"));
    assert!(source.contains(
        "let current_focused_background_opacity_index = Self::closest_float_preset_index("
    ));
    assert!(source.contains("let is_current = i == current_focused_background_opacity_index;"));
    assert!(source.contains("let current_opacity_index = Self::find_opacity_preset_index("));
    assert!(source.contains("let is_current = i == current_opacity_index;"));
}

#[test]
fn theme_chooser_customize_opacity_controls_cover_full_percent_range() {
    let source = read_source("src/render_builtins/theme_chooser.rs");

    assert!(source.contains(
        "const OPACITY_PRESETS: &'static [(f32, &'static str)] = &[\n        (0.00, \"0%\"),"
    ));
    assert!(source.contains(
        "const TEXT_OPACITY_PRESETS: &'static [(f32, &'static str)] = &[\n        (0.00, \"0%\"),"
    ));
    assert!(source.contains(
        "const FOCUSED_BACKGROUND_OPACITY_PRESETS: &'static [(f32, &'static str)] = &[\n        (0.00, \"0%\"),"
    ));
    assert_eq!(
        source.matches("(1.00, \"100%\"),").count(),
        3,
        "each theme designer opacity control should expose a 100% endpoint"
    );
}
