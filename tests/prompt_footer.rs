use script_kit_gpui::components::prompt_footer::{
    footer_surface_rgba, PromptFooterColors, PromptFooterConfig,
    PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX, PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX,
};
use script_kit_gpui::components::prompt_header::{
    HeaderActionsDensity, HEADER_ACTIONS_MIN_WIDTH_COMPACT_PX, HEADER_PATH_PREFIX_MAX_WIDTH_PX,
};
use script_kit_gpui::designs::DesignColors;
use script_kit_gpui::theme::Theme;

#[test]
fn test_prompt_footer_button_states_default_to_enabled() {
    let config = PromptFooterConfig::default();

    assert!(!config.primary_disabled);
    assert!(!config.secondary_disabled);
}

#[test]
fn test_prompt_footer_builder_sets_button_disabled_state() {
    let config = PromptFooterConfig::new()
        .primary_disabled(true)
        .secondary_disabled(true);

    assert!(config.primary_disabled);
    assert!(config.secondary_disabled);
}

#[test]
fn test_prompt_footer_surface_rgba_preserves_legacy_light_and_dark_behavior() {
    let dark = PromptFooterColors {
        background: 0x123456,
        is_light_mode: false,
        ..PromptFooterColors::default()
    };
    let light = PromptFooterColors {
        background: 0x123456,
        is_light_mode: true,
        ..PromptFooterColors::default()
    };

    assert_eq!(footer_surface_rgba(dark), 0x12345633);
    assert_eq!(footer_surface_rgba(light), 0xf2f1f1ff);
}

#[test]
fn test_prompt_footer_from_theme_uses_selected_subtle_surface_token() {
    let mut theme = Theme::default();
    theme.colors.accent.selected_subtle = 0xa1b2c3;
    theme.colors.background.search_box = 0x010203;

    let colors = PromptFooterColors::from_theme(&theme);
    assert_eq!(colors.background, 0xa1b2c3);
}

#[test]
fn test_prompt_footer_from_design_uses_selected_background_token() {
    let mut design_colors = DesignColors::default();
    design_colors.background_selected = 0xabcdef;
    design_colors.background_secondary = 0x123456;

    let colors = PromptFooterColors::from_design(&design_colors);
    assert_eq!(colors.background, 0xabcdef);
}

#[test]
fn test_prompt_footer_default_background_is_legacy_white() {
    assert_eq!(PromptFooterColors::default().background, 0xffffff);
}

#[test]
fn test_prompt_footer_text_max_width_contract() {
    assert_eq!(PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX, 420.0);
    assert_eq!(PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX, 220.0);
    assert!(
        PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX > PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX,
        "Helper text should get more space than the info label"
    );
}

#[test]
fn test_prompt_header_actions_density_compact_reduces_reserved_width() {
    assert_eq!(HeaderActionsDensity::Compact.reserved_min_width_px(), 168.0);
    assert!(
        HeaderActionsDensity::Compact.reserved_min_width_px()
            < HeaderActionsDensity::Normal.reserved_min_width_px()
    );
    assert!(
        HeaderActionsDensity::Normal.reserved_min_width_px()
            < HeaderActionsDensity::Expanded.reserved_min_width_px()
    );
    assert_eq!(
        HeaderActionsDensity::Compact.reserved_min_width_px(),
        HEADER_ACTIONS_MIN_WIDTH_COMPACT_PX
    );
}

#[test]
fn test_prompt_header_path_prefix_has_explicit_max_width_contract() {
    assert_eq!(HEADER_PATH_PREFIX_MAX_WIDTH_PX, 320.0);
}
