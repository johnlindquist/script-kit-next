use gpui::{div, px, AnyElement, FontWeight, IntoElement, ParentElement, SharedString, Styled};

use crate::list_item::FONT_SYSTEM_UI;
use crate::theme::opacity::OPACITY_TEXT_MUTED;
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

pub(crate) const FOOTER_HINT_FONT_SIZE_PX: f32 = 12.5;
pub(crate) const FOOTER_HINT_FONT_WEIGHT_APPKIT: f64 = 0.18;
pub(crate) const FOOTER_HINT_FONT_WEIGHT_GPUI: FontWeight = FontWeight::SEMIBOLD;
pub(crate) const FOOTER_KEYCAP_HEIGHT_PX: f32 = 16.0;
pub(crate) const FOOTER_KEYCAP_PADDING_X_PX: f32 = 4.0;
pub(crate) const FOOTER_KEYCAP_RADIUS_PX: f32 = 4.0;
pub(crate) const FOOTER_KEY_GLYPH_NUDGE_Y_PX: f32 = 1.0;
pub(crate) const FOOTER_RETURN_GLYPH_NUDGE_Y_PX: f32 = 1.0;
pub(crate) const FOOTER_KEYCAP_BORDER_ALPHA: f32 = 0x50 as f32 / 255.0;
pub(crate) const FOOTER_KEYCAP_BG_ALPHA: f32 = 0x15 as f32 / 255.0;

pub(crate) enum FooterHintKeyMode {
    Shortcut,
    TextValue { max_width_px: f32 },
}

fn normalize_footer_key_token(token: &str) -> String {
    match token.trim().to_lowercase().as_str() {
        "esc" | "escape" => "⎋".to_string(),
        _ => token.to_string(),
    }
}

pub(crate) fn footer_hint_text_color(theme: &Theme) -> gpui::Rgba {
    theme
        .colors
        .text
        .primary
        .with_opacity(OPACITY_TEXT_MUTED)
        .to_rgb()
}

pub(crate) fn footer_keycap_border_color(theme: &Theme) -> gpui::Hsla {
    theme
        .colors
        .ui
        .border
        .with_opacity(FOOTER_KEYCAP_BORDER_ALPHA)
}

pub(crate) fn footer_keycap_bg_color(theme: &Theme) -> gpui::Hsla {
    theme.colors.ui.border.with_opacity(FOOTER_KEYCAP_BG_ALPHA)
}

pub(crate) fn split_footer_shortcut(shortcut: &str) -> Vec<String> {
    let s = shortcut.trim();
    if s.is_empty() {
        return Vec::new();
    }
    if s.contains('+') {
        return s.split('+').map(normalize_footer_key_token).collect();
    }
    if s.chars().any(char::is_whitespace) {
        return s
            .split_whitespace()
            .map(normalize_footer_key_token)
            .collect();
    }

    let lower = s.to_lowercase();
    let known_words = [
        "enter",
        "return",
        "space",
        "tab",
        "esc",
        "escape",
        "backspace",
        "del",
        "delete",
        "up",
        "down",
        "left",
        "right",
        "home",
        "end",
        "cmd",
        "ctrl",
        "alt",
        "shift",
        "win",
        "click",
    ];
    if known_words.contains(&lower.as_str()) {
        return vec![normalize_footer_key_token(s)];
    }

    let mut tokens = Vec::new();
    let mut text_run = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            text_run.push(ch);
        } else {
            if !text_run.is_empty() {
                tokens.push(normalize_footer_key_token(&text_run));
                text_run.clear();
            }
            tokens.push(ch.to_string());
        }
    }
    if !text_run.is_empty() {
        tokens.push(normalize_footer_key_token(&text_run));
    }
    tokens
}

pub(crate) fn is_footer_return_key_glyph(key: &str) -> bool {
    matches!(key, "↵")
}

pub(crate) fn footer_key_glyph_nudge_y(key: &str) -> f32 {
    if is_footer_return_key_glyph(key) {
        FOOTER_KEY_GLYPH_NUDGE_Y_PX + FOOTER_RETURN_GLYPH_NUDGE_Y_PX
    } else {
        FOOTER_KEY_GLYPH_NUDGE_Y_PX
    }
}

pub(crate) fn footer_appkit_glyph_y(key: &str, chip_height: f64, glyph_height: f64) -> f64 {
    let centered_y = ((chip_height - glyph_height) / 2.0).round();
    centered_y - footer_key_glyph_nudge_y(key) as f64
}

pub(crate) fn render_footer_hint_content(
    label: SharedString,
    key: SharedString,
    mode: FooterHintKeyMode,
    theme: &Theme,
) -> AnyElement {
    let footer_text = footer_hint_text_color(theme);

    div()
        .px(px(4.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(3.0))
        .child(
            div()
                .font_family(FONT_SYSTEM_UI)
                .font_weight(FOOTER_HINT_FONT_WEIGHT_GPUI)
                .text_size(px(FOOTER_HINT_FONT_SIZE_PX))
                .text_color(footer_text)
                .child(label),
        )
        .child(match mode {
            FooterHintKeyMode::Shortcut => render_footer_shortcut_keycaps(key.to_string(), theme),
            FooterHintKeyMode::TextValue { max_width_px } => {
                render_footer_text_keycap(key.to_string(), max_width_px, theme)
            }
        })
        .into_any_element()
}

fn render_footer_shortcut_keycaps(shortcut: String, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(3.0))
        .children(
            split_footer_shortcut(&shortcut)
                .into_iter()
                .map(|token| render_footer_keycap(token, None, theme)),
        )
        .into_any_element()
}

fn render_footer_text_keycap(text: String, max_width_px: f32, theme: &Theme) -> AnyElement {
    render_footer_keycap(text, Some(max_width_px), theme)
}

fn render_footer_keycap(token: String, max_width_px: Option<f32>, theme: &Theme) -> AnyElement {
    let footer_text = footer_hint_text_color(theme);
    let token_child: AnyElement = div()
        .h(px(FOOTER_KEYCAP_HEIGHT_PX))
        .line_height(px(FOOTER_KEYCAP_HEIGHT_PX))
        .mt(px(footer_key_glyph_nudge_y(&token)))
        .child(token)
        .into_any_element();

    let mut keycap = div()
        .flex_none()
        .min_w(px(FOOTER_KEYCAP_HEIGHT_PX))
        .min_h(px(FOOTER_KEYCAP_HEIGHT_PX))
        .h(px(FOOTER_KEYCAP_HEIGHT_PX))
        .line_height(px(FOOTER_KEYCAP_HEIGHT_PX))
        .px(px(FOOTER_KEYCAP_PADDING_X_PX))
        .rounded(px(FOOTER_KEYCAP_RADIUS_PX))
        .border_1()
        .border_color(footer_keycap_border_color(theme))
        .bg(footer_keycap_bg_color(theme))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_SYSTEM_UI)
        .font_weight(FOOTER_HINT_FONT_WEIGHT_GPUI)
        .text_size(px(FOOTER_HINT_FONT_SIZE_PX))
        .text_color(footer_text)
        .child(token_child);

    if let Some(max_width_px) = max_width_px {
        keycap = keycap
            .max_w(px(max_width_px))
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap();
    }

    keycap.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_footer_shortcut_parses_simple_and_complex_keys() {
        assert_eq!(split_footer_shortcut(""), Vec::<String>::new());
        assert_eq!(split_footer_shortcut("↵"), vec!["↵"]);
        assert_eq!(split_footer_shortcut("⌘K"), vec!["⌘", "K"]);
        assert_eq!(split_footer_shortcut("⌥↵"), vec!["⌥", "↵"]);
        assert_eq!(split_footer_shortcut("Enter"), vec!["Enter"]);
        assert_eq!(split_footer_shortcut("esc"), vec!["⎋"]);
        assert_eq!(split_footer_shortcut("Escape"), vec!["⎋"]);
        assert_eq!(split_footer_shortcut("Cmd+K"), vec!["Cmd", "K"]);
        assert_eq!(split_footer_shortcut("⌘F1"), vec!["⌘", "F1"]);
        assert_eq!(split_footer_shortcut("⌥⌘I"), vec!["⌥", "⌘", "I"]);
        assert_eq!(split_footer_shortcut("click"), vec!["click"]);
    }

    #[test]
    fn footer_key_glyph_nudges_match_footer_contract() {
        assert!(is_footer_return_key_glyph("↵"));
        assert!(!is_footer_return_key_glyph("Enter"));
        assert_eq!(footer_key_glyph_nudge_y("⌘"), 1.0);
        assert_eq!(footer_key_glyph_nudge_y("↵"), 2.0);
        assert_eq!(footer_appkit_glyph_y("⌘", 16.0, 10.0), 2.0);
        assert_eq!(footer_appkit_glyph_y("↵", 16.0, 10.0), 1.0);
    }
}
