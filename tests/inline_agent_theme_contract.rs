use script_kit_gpui::inline_agent::{
    InlineAgentColors, INLINE_AGENT_DISABLED_TEXT_MIN_CONTRAST,
    INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST, INLINE_AGENT_SECONDARY_TEXT_MIN_CONTRAST,
};
use script_kit_gpui::theme::Theme;

fn assert_contrast_contract(colors: InlineAgentColors) {
    let summary = colors.contrast_summary();
    assert!(summary.primary_text >= INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST);
    assert!(summary.secondary_text >= INLINE_AGENT_SECONDARY_TEXT_MIN_CONTRAST);
    assert!(summary.disabled_text >= INLINE_AGENT_DISABLED_TEXT_MIN_CONTRAST);
    assert!(summary.thinking >= INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST);
    assert!(summary.accent_text >= INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST);
}

#[test]
fn inline_agent_colors_pass_for_default_dark_and_light_themes() {
    assert_contrast_contract(InlineAgentColors::from_theme(&Theme::dark_default()));
    assert_contrast_contract(InlineAgentColors::from_theme(&Theme::light_default()));
}

#[test]
fn inline_agent_colors_repair_low_contrast_theme_text() {
    let mut theme = Theme::dark_default();
    theme.colors.background.search_box = 0x202020;
    theme.colors.text.primary = 0x222222;
    theme.colors.text.secondary = 0x252525;
    theme.colors.text.muted = 0x282828;
    theme.colors.accent.selected = 0x303030;
    theme.colors.text.on_accent = 0x323232;

    let colors = InlineAgentColors::from_theme(&theme);
    assert_contrast_contract(colors);
    assert_ne!(colors.text_primary, theme.colors.text.primary);
    assert_ne!(colors.text_secondary, theme.colors.text.secondary);
    assert_ne!(colors.text_disabled, theme.colors.text.muted);
}
