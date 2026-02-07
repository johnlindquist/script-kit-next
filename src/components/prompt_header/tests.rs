use super::PromptHeaderColors;
use crate::theme::Theme;
use std::fs;

fn render_ask_ai_hint_section() -> String {
    let content = fs::read_to_string("src/components/prompt_header/component.rs")
        .expect("Failed to read src/components/prompt_header/component.rs");

    let start = content
        .find("fn render_ask_ai_hint")
        .expect("render_ask_ai_hint not found in prompt_header/component.rs");
    content[start..content.len().min(start + 2600)].to_string()
}

#[test]
fn test_prompt_header_colors_from_theme_uses_on_accent_text_token_for_logo() {
    let mut theme = Theme::default();
    theme.colors.text.on_accent = 0x223344;

    let colors = PromptHeaderColors::from_theme(&theme);
    assert_eq!(colors.logo_icon, 0x223344);
}

#[test]
fn test_render_ask_ai_hint_uses_transparent_backgrounds() {
    let section = render_ask_ai_hint_section();

    let transparent_bg_count = section
        .as_str()
        .matches(".bg(rgba(transparent_bg))")
        .count();
    assert_eq!(
        transparent_bg_count, 2,
        "Expected transparent backgrounds for Ask AI and Tab hint buttons. Section:\n{}",
        section
    );
}

#[test]
fn test_render_ask_ai_hint_uses_pointer_cursor_for_hint_buttons() {
    let section = render_ask_ai_hint_section();

    let pointer_count = section.as_str().matches(".cursor_pointer()").count();
    assert_eq!(
        pointer_count, 2,
        "Expected cursor_pointer on Ask AI and Tab hint buttons. Section:\n{}",
        section
    );
}

#[test]
fn test_render_ask_ai_hint_adds_hover_feedback_for_both_hint_buttons() {
    let section = render_ask_ai_hint_section();

    let hover_count = section.as_str().matches(".hover(move |style|").count();
    assert_eq!(
        hover_count, 2,
        "Expected hover handlers on Ask AI and Tab hint buttons. Section:\n{}",
        section
    );

    let hover_bg_count = section
        .as_str()
        .matches(".bg(rgba(colors.hover_overlay))")
        .count();
    assert_eq!(
        hover_bg_count, 2,
        "Expected hover background token usage on both hint buttons. Section:\n{}",
        section
    );

    let hover_text_count = section
        .as_str()
        .matches(".text_color(colors.text_primary.to_rgb())")
        .count();
    assert_eq!(
        hover_text_count, 2,
        "Expected hover text color token usage on both hint buttons. Section:\n{}",
        section
    );
}

#[test]
fn test_render_ask_ai_hint_uses_ghost_button_spacing_tokens_for_hint_buttons() {
    let section = render_ask_ai_hint_section();

    let ghost_height_count = section
        .as_str()
        .matches(".min_h(px(BUTTON_GHOST_HEIGHT))")
        .count();
    assert_eq!(
        ghost_height_count, 2,
        "Expected ghost button height token on Ask AI and Tab hint buttons. Section:\n{}",
        section
    );

    let ghost_padding_x_count = section
        .as_str()
        .matches(".px(px(BUTTON_GHOST_PADDING_X))")
        .count();
    assert_eq!(
        ghost_padding_x_count, 2,
        "Expected ghost button horizontal padding token on both hint buttons. Section:\n{}",
        section
    );

    let ghost_padding_y_count = section
        .as_str()
        .matches(".py(px(BUTTON_GHOST_PADDING_Y))")
        .count();
    assert_eq!(
        ghost_padding_y_count, 2,
        "Expected ghost button vertical padding token on both hint buttons. Section:\n{}",
        section
    );

    let ghost_radius_count = section.as_str().matches(".rounded(px(6.))").count();
    assert_eq!(
        ghost_radius_count, 2,
        "Expected ghost button radius token on both hint buttons. Section:\n{}",
        section
    );
}
