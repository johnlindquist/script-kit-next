use crate::theme::Theme;

pub(crate) const FOOTER_HINT_FONT_SIZE_PX: f32 = 12.5;
pub(crate) const FOOTER_HINT_FONT_WEIGHT_APPKIT: f64 = 0.18;
pub(crate) const FOOTER_KEYCAP_HEIGHT_PX: f32 = 20.0;
pub(crate) const FOOTER_KEYCAP_PADDING_X_PX: f32 = 4.0;
pub(crate) const FOOTER_KEYCAP_RADIUS_PX: f32 = 4.0;
pub(crate) const FOOTER_KEY_GLYPH_NUDGE_Y_PX: f32 = 1.0;
pub(crate) const FOOTER_RETURN_GLYPH_NUDGE_Y_PX: f32 = 1.0;
pub(crate) const FOOTER_BUTTON_VERTICAL_INSET_PX: f32 = 2.0;

fn normalize_footer_key_token(token: &str) -> String {
    match token.trim().to_lowercase().as_str() {
        "esc" | "escape" => "⎋".to_string(),
        _ => token.to_string(),
    }
}

pub(crate) fn footer_keycap_border_alpha(theme: &Theme, selected: bool) -> f32 {
    let opacity = theme.get_opacity();
    if selected {
        opacity.selected
    } else {
        opacity.hover
    }
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

pub(crate) fn footer_button_height(footer_height: f32) -> f32 {
    (footer_height - (FOOTER_BUTTON_VERTICAL_INSET_PX * 2.0)).max(0.0)
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
        assert_eq!(footer_appkit_glyph_y("⌘", 20.0, 10.0), 4.0);
        assert_eq!(footer_appkit_glyph_y("↵", 20.0, 10.0), 3.0);
        assert_eq!(footer_button_height(32.0), 28.0);
    }

    #[test]
    fn footer_keycap_border_alpha_tracks_list_row_state_opacity() {
        let mut theme = Theme::dark_default();
        let mut opacity = theme.get_opacity();
        opacity.hover = 0.21;
        opacity.selected = 0.47;
        theme.opacity = Some(opacity);
        let list_colors = crate::list_item::ListItemColors::from_theme(&theme);

        assert_eq!(
            footer_keycap_border_alpha(&theme, false),
            list_colors.hover_opacity
        );
        assert_eq!(
            footer_keycap_border_alpha(&theme, true),
            list_colors.selected_opacity
        );
    }
}
