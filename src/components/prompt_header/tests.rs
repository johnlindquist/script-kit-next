use super::PromptHeaderColors;
use crate::theme::Theme;
use std::fs;

#[test]
fn test_prompt_header_colors_from_theme_uses_on_accent_text_token_for_logo() {
    let mut theme = Theme::default();
    theme.colors.text.on_accent = 0x223344;

    let colors = PromptHeaderColors::from_theme(&theme);
    assert_eq!(colors.logo_icon, 0x223344);
}

#[test]
fn test_render_ask_ai_hint_uses_transparent_backgrounds() {
    let content = fs::read_to_string("src/components/prompt_header/component.rs")
        .expect("Failed to read src/components/prompt_header/component.rs");

    let start = content
        .find("fn render_ask_ai_hint")
        .expect("render_ask_ai_hint not found in prompt_header/component.rs");
    let section = &content[start..content.len().min(start + 1600)];

    let transparent_bg_count = section.matches(".bg(rgba(transparent_bg))").count();
    assert_eq!(
        transparent_bg_count, 2,
        "Expected transparent backgrounds for Ask AI and Tab hint buttons. Section:\n{}",
        section
    );
}

#[test]
fn test_render_ask_ai_hint_uses_pointer_cursor_for_hint_buttons() {
    let content = fs::read_to_string("src/components/prompt_header/component.rs")
        .expect("Failed to read src/components/prompt_header/component.rs");

    let start = content
        .find("fn render_ask_ai_hint")
        .expect("render_ask_ai_hint not found in prompt_header/component.rs");
    let section = &content[start..content.len().min(start + 1600)];

    let pointer_count = section.matches(".cursor_pointer()").count();
    assert_eq!(
        pointer_count, 2,
        "Expected cursor_pointer on Ask AI and Tab hint buttons. Section:\n{}",
        section
    );
}
