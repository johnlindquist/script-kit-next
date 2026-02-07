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

    // Dark mode: background token at ~12% opacity.
    assert_eq!(footer_surface_rgba(dark), 0x1234561f);
    // Light mode: always neutral warm gray.
    assert_eq!(footer_surface_rgba(light), 0xf0eeefff);
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
fn test_prompt_footer_from_design_delegates_to_theme() {
    let design_colors = DesignColors {
        background_selected: 0xabcdef,
        background_secondary: 0x123456,
        ..DesignColors::default()
    };

    // from_design() delegates to from_theme() for consistency with the app shell.
    let colors = PromptFooterColors::from_design(&design_colors);
    let expected = PromptFooterColors::from_theme(&Theme::default());
    assert_eq!(colors.background, expected.background);
}

#[test]
fn test_prompt_footer_default_background_uses_theme_token() {
    let expected = PromptFooterColors::from_theme(&Theme::default());
    assert_eq!(
        PromptFooterColors::default().background,
        expected.background
    );
}

#[test]
fn test_prompt_footer_text_max_width_contract() {
    let helper_max_width = std::hint::black_box(PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX);
    let info_max_width = std::hint::black_box(PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX);

    assert_eq!(helper_max_width, 420.0);
    assert_eq!(info_max_width, 220.0);
    assert!(
        helper_max_width > info_max_width,
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
