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
