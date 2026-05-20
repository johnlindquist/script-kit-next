use super::read_source;

#[test]
fn theme_chooser_customize_opacity_controls_are_single_select() {
    let source = read_source("src/render_builtins/theme_chooser.rs");
    let customize_controls = read_source("src/render_builtins/theme_chooser_customize_controls.rs");

    assert!(!source.contains("OPACITY_MATCH_TOLERANCE"));
    assert!(source.contains("fn closest_float_preset_index("));
    assert!(source.contains("ThemeChooserSliderBinding::SecondaryTextOpacity"));
    assert!(source.contains("Self::apply_text_opacity_preset("));
    assert!(source.contains("ThemeChooserSliderBinding::FocusedBackgroundOpacity"));
    assert!(source.contains("Self::apply_focused_background_opacity_preset("));
    assert!(
        customize_controls.contains("let current_opacity_index = Self::find_opacity_preset_index(")
    );
    assert!(customize_controls.contains("let is_current = i == current_opacity_index;"));
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

#[test]
fn theme_chooser_exposes_user_theme_management_and_gradient_actions() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let actions = read_source("src/render_builtins/actions.rs");
    let user_themes = read_source("src/theme/user_themes.rs");
    let theme_types = read_source("src/theme/types.rs");
    let render_impl = read_source("src/main_sections/render_impl.rs");

    assert!(chooser.contains("fn theme_chooser_catalog()"));
    assert!(chooser.contains("theme::user_themes::list_user_themes()"));
    assert!(chooser.contains("theme::user_themes::load_user_theme"));
    assert!(chooser.contains("save_current_theme_as_user_theme"));
    assert!(chooser.contains("delete_selected_user_theme"));
    assert!(chooser.contains("cycle_theme_chooser_gradient"));
    assert!(chooser.contains("self.apply_theme_chooser_theme(next_theme, reason, cx);"));
    assert!(
        !chooser.contains("this.apply_and_persist_theme("),
        "Theme Designer customization clicks should preview only; Done/Enter owns persistence"
    );
    assert!(
        !chooser
            .contains("\"theme_chooser_mouse_click\",\n                                    true"),
        "Theme Designer row clicks should not persist active theme.json"
    );
    assert!(
        !chooser.contains("persist_theme_and_sync_all_windows(\n                    cx,\n                    self.theme.as_ref(),\n                    reason"),
        "Save as user theme should write the library preset without applying active theme.json"
    );

    for action_id in [
        "theme_chooser_save_as_user_theme",
        "theme_chooser_delete_user_theme",
        "theme_chooser_gradient_cycle",
    ] {
        assert!(
            actions.contains(action_id) && chooser.contains(action_id),
            "Theme Designer action `{action_id}` must be exposed in actions and executed"
        );
    }

    assert!(user_themes.contains("pub fn save_theme_as_user_theme("));
    assert!(user_themes.contains("pub fn save_user_theme_unique("));
    assert!(user_themes.contains("pub fn load_user_theme("));
    assert!(user_themes.contains(".get(\"hover\")"));
    assert!(user_themes.contains(".get(\"selected\")"));
    assert!(theme_types.contains("pub struct BackgroundGradient"));
    assert!(theme_types.contains("pub fn active_background_gradient(&self)"));
    assert!(render_impl.contains("get_theme_background_gradients(&self.theme)"));
}

#[test]
fn theme_chooser_controls_are_devtools_visible_and_drivable() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let collector = read_source("src/app_layout/collect_elements.rs");
    let prompt_handler = read_source("src/prompt_handler/mod.rs");
    let protocol = read_source("src/protocol/types/batch_wait.rs");

    for control in [
        "accent-color",
        "surface-opacity",
        "secondary-text-opacity",
        "focused-background-opacity",
        "vibrancy-enabled",
        "gradient-enabled",
        "gradient-base-from",
        "gradient-base-to",
        "gradient-base-angle",
        "gradient-base-opacity",
        "ui-font-size",
        "gradient-layer-",
    ] {
        assert!(
            collector.contains(control),
            "getElements must expose Theme Designer control `{control}`"
        );
        assert!(
            chooser.contains(control),
            "Theme Designer devtools setter must handle control `{control}`"
        );
    }

    for element_type in [
        "ElementType::Slider",
        "ElementType::ColorPicker",
        "ElementType::Toggle",
    ] {
        assert!(
            collector.contains(element_type),
            "Theme Designer controls must expose semantic {element_type} elements"
        );
    }

    assert!(protocol.contains("SetThemeControl"));
    assert!(chooser.contains("strip_prefix(\"control:theme-chooser:\")"));
    assert!(prompt_handler.contains("set_theme_chooser_control_from_devtools"));
    assert!(prompt_handler.contains("\"setThemeControl\".to_string()"));
    assert!(prompt_handler.contains("setThemeControl requires ThemeChooserView"));
    assert!(!prompt_handler.contains(".set_theme_chooser_control_from_devtools(&control, &value, cx)\n                                                .ok()"));
}
