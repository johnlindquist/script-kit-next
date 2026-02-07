use super::PromptHeaderColors;
use crate::theme::Theme;

#[test]
fn test_prompt_header_colors_from_theme_uses_on_accent_text_token_for_logo() {
    let mut theme = Theme::default();
    theme.colors.text.on_accent = 0x223344;

    let colors = PromptHeaderColors::from_theme(&theme);
    assert_eq!(colors.logo_icon, 0x223344);
}
