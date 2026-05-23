use gpui::{
    div, px, svg, AnyElement, FontWeight, InteractiveElement, IntoElement, ParentElement,
    SharedString, Styled,
};

use crate::list_item::FONT_SYSTEM_UI;
use crate::theme::opacity::{OPACITY_HIDDEN, OPACITY_TEXT_MUTED};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

pub(crate) const FOOTER_HINT_FONT_SIZE_PX: f32 = 12.0;
pub(crate) const FOOTER_HINT_FONT_WEIGHT_APPKIT: f64 = 0.14;
pub(crate) const FOOTER_HINT_FONT_WEIGHT_GPUI: FontWeight = FontWeight(560.0);
pub(crate) const FOOTER_KEYCAP_HEIGHT_PX: f32 = 20.0;
pub(crate) const FOOTER_KEYCAP_PADDING_X_PX: f32 = 3.0;
pub(crate) const FOOTER_KEYCAP_RADIUS_PX: f32 = 4.0;
pub(crate) const FOOTER_KEY_GLYPH_NUDGE_Y_PX: f32 = 1.0;
pub(crate) const FOOTER_RETURN_GLYPH_NUDGE_Y_PX: f32 = 1.0;
pub(crate) const FOOTER_SEMICOLON_GLYPH_NUDGE_Y_PX: f32 = -1.0;
pub(crate) const FOOTER_BUTTON_VERTICAL_INSET_PX: f32 = 2.0;
pub(crate) const FOOTER_ACTION_ITEM_GAP_PX: f32 = 3.0;
pub(crate) const FOOTER_ACTION_CONTENT_GAP_PX: f32 = 2.0;
pub(crate) const FOOTER_ACTION_CONTENT_PADDING_X_PX: f32 = 2.0;
pub(crate) const FOOTER_ACTION_BUTTON_RADIUS_PX: f32 = 4.0;
pub(crate) const FOOTER_RUN_SLOT_MIN_WIDTH_PX: f32 = 92.0;
pub(crate) const FOOTER_RUN_SLOT_MAX_WIDTH_PX: f32 = 172.0;
pub(crate) const FOOTER_ACTIONS_SLOT_WIDTH_PX: f32 = 92.0;
pub(crate) const FOOTER_AI_SLOT_WIDTH_PX: f32 = 52.0;
pub(crate) const FOOTER_APPLY_SLOT_WIDTH_PX: f32 = 84.0;
pub(crate) const FOOTER_CLOSE_SLOT_WIDTH_PX: f32 = 84.0;
pub(crate) const FOOTER_STOP_SLOT_WIDTH_PX: f32 = 76.0;
pub(crate) const FOOTER_PASTE_RESPONSE_SLOT_WIDTH_PX: f32 = 140.0;

pub(crate) const FOOTER_LABELCAP_BORDER_ALPHA: f32 = 0.0;
pub(crate) const FOOTER_MIC_ICON_TOKEN: &str = "mic";
pub(crate) const FOOTER_MIC_ICON_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/gpui-component/crates/assets/assets/icons/mic.svg"
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum FooterActionSlot {
    Run,
    Actions,
    Ai,
    Apply,
    Close,
    Stop,
    PasteResponse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FooterRailChrome {
    pub height_px: f32,
    pub side_inset_px: f32,
    pub item_gap_px: f32,
    pub surface_rgba: u32,
    pub divider_rgba: u32,
    pub hover_rgba: u32,
    pub active_rgba: u32,
    pub button_radius_px: f32,
}

pub(crate) enum FooterHintKeyMode {
    Shortcut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FooterHintContentJustify {
    Start,
    Center,
    KeyAnchored,
}

pub(crate) fn footer_action_slot_width(slot: FooterActionSlot) -> f32 {
    match slot {
        FooterActionSlot::Run => FOOTER_RUN_SLOT_MIN_WIDTH_PX,
        FooterActionSlot::Actions => FOOTER_ACTIONS_SLOT_WIDTH_PX,
        FooterActionSlot::Ai => FOOTER_AI_SLOT_WIDTH_PX,
        FooterActionSlot::Apply => FOOTER_APPLY_SLOT_WIDTH_PX,
        FooterActionSlot::Close => FOOTER_CLOSE_SLOT_WIDTH_PX,
        FooterActionSlot::Stop => FOOTER_STOP_SLOT_WIDTH_PX,
        FooterActionSlot::PasteResponse => FOOTER_PASTE_RESPONSE_SLOT_WIDTH_PX,
    }
}

pub(crate) fn footer_rail_chrome(theme: &Theme) -> FooterRailChrome {
    let chrome = crate::theme::AppChromeColors::from_theme(theme);
    FooterRailChrome {
        height_px: crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT,
        side_inset_px: crate::window_resize::mini_layout::HINT_STRIP_PADDING_X,
        item_gap_px: FOOTER_ACTION_ITEM_GAP_PX,
        surface_rgba: chrome.inline_dropdown_surface_rgba,
        divider_rgba: chrome.divider_rgba,
        hover_rgba: chrome.hover_rgba,
        active_rgba: chrome.selection_rgba,
        button_radius_px: FOOTER_ACTION_BUTTON_RADIUS_PX,
    }
}

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

pub(crate) fn footer_hint_text_color(theme: &Theme) -> gpui::Rgba {
    theme
        .colors
        .text
        .primary
        .with_opacity(OPACITY_TEXT_MUTED)
        .to_rgb()
}

pub(crate) fn footer_keycap_border_color_for_state(theme: &Theme, selected: bool) -> gpui::Hsla {
    let alpha = footer_keycap_border_alpha(theme, selected);
    theme.colors.text.primary.with_opacity(alpha)
}

pub(crate) fn footer_keycap_border_color(theme: &Theme) -> gpui::Hsla {
    footer_keycap_border_color_for_state(theme, false)
}

pub(crate) fn footer_labelcap_border_color(theme: &Theme) -> gpui::Hsla {
    theme
        .colors
        .text
        .primary
        .with_opacity(FOOTER_LABELCAP_BORDER_ALPHA.max(OPACITY_HIDDEN))
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
    } else if key == ";" {
        FOOTER_SEMICOLON_GLYPH_NUDGE_Y_PX
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

pub(crate) fn render_footer_hint_content(
    label: SharedString,
    key: SharedString,
    mode: FooterHintKeyMode,
    theme: &Theme,
) -> AnyElement {
    render_footer_hint_content_impl(
        label,
        key,
        mode,
        theme,
        None,
        false,
        FooterHintContentJustify::Center,
    )
}

pub(crate) fn render_footer_hint_content_constrained(
    label: SharedString,
    key: SharedString,
    mode: FooterHintKeyMode,
    theme: &Theme,
    slot_width_px: f32,
    key_first: bool,
    justify: FooterHintContentJustify,
) -> AnyElement {
    render_footer_hint_content_impl(
        label,
        key,
        mode,
        theme,
        Some(slot_width_px),
        key_first,
        justify,
    )
}

fn render_footer_hint_content_impl(
    label: SharedString,
    key: SharedString,
    mode: FooterHintKeyMode,
    theme: &Theme,
    slot_width_px: Option<f32>,
    key_first: bool,
    justify: FooterHintContentJustify,
) -> AnyElement {
    let footer_text = footer_hint_text_color(theme);
    let full_text = theme.colors.text.primary.to_rgb();
    let key_width_px = match mode {
        FooterHintKeyMode::Shortcut => footer_shortcut_keycaps_width_px(key.as_ref()),
    };
    let label_max_width_px =
        slot_width_px.map(|slot| footer_labelcap_max_width_for_slot(slot, key_width_px));
    let labelcap = if let Some(max_width_px) = label_max_width_px {
        render_footer_labelcap_constrained(
            label,
            theme,
            footer_text,
            full_text,
            Some(max_width_px),
            matches!(justify, FooterHintContentJustify::KeyAnchored),
        )
    } else {
        render_footer_labelcap(label, theme, footer_text, full_text)
    };
    let keycaps = match mode {
        FooterHintKeyMode::Shortcut => render_footer_shortcut_keycaps(key.to_string(), theme),
    };

    let mut row = div()
        .px(px(FOOTER_ACTION_CONTENT_PADDING_X_PX))
        .py(px(2.0))
        .rounded(px(FOOTER_ACTION_BUTTON_RADIUS_PX))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(FOOTER_ACTION_CONTENT_GAP_PX))
        .group("footer-action-button")
        .min_w(px(0.0))
        .overflow_hidden();

    if let Some(slot_width_px) = slot_width_px {
        row = row.w_full().max_w(px(slot_width_px));
    }

    row = match justify {
        FooterHintContentJustify::Start => row.justify_start(),
        FooterHintContentJustify::Center => row.justify_center(),
        FooterHintContentJustify::KeyAnchored => row.justify_start(),
    };

    if key_first {
        row.child(keycaps).child(labelcap).into_any_element()
    } else {
        row.child(labelcap).child(keycaps).into_any_element()
    }
}

pub(crate) fn footer_shortcut_keycaps_width_px(shortcut: &str) -> f32 {
    let tokens = split_footer_shortcut(shortcut);
    if tokens.is_empty() {
        return 0.0;
    }

    let keys_width = tokens
        .iter()
        .map(|token| footer_keycap_estimated_width_px(token))
        .sum::<f32>();
    keys_width + tokens.len().saturating_sub(1) as f32 * FOOTER_ACTION_CONTENT_GAP_PX
}

pub(crate) fn footer_hint_content_estimated_width_px(
    label: &str,
    key: &str,
    mode: FooterHintKeyMode,
) -> f32 {
    let label_width_px = footer_labelcap_estimated_width_px(label);
    let key_width_px = match mode {
        FooterHintKeyMode::Shortcut => footer_shortcut_keycaps_width_px(key),
    };
    let content_gap = if !label.trim().is_empty() && key_width_px > 0.0 {
        FOOTER_ACTION_CONTENT_GAP_PX
    } else {
        0.0
    };

    FOOTER_ACTION_CONTENT_PADDING_X_PX * 2.0 + label_width_px + content_gap + key_width_px
}

fn footer_labelcap_estimated_width_px(label: &str) -> f32 {
    let estimated_text_width = label
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                FOOTER_HINT_FONT_SIZE_PX * 0.62
            } else if ch.is_whitespace() {
                FOOTER_HINT_FONT_SIZE_PX * 0.35
            } else {
                FOOTER_HINT_FONT_SIZE_PX * 0.82
            }
        })
        .sum::<f32>();

    (estimated_text_width + FOOTER_KEYCAP_PADDING_X_PX * 2.0)
        .max(FOOTER_KEYCAP_HEIGHT_PX)
        .ceil()
}

fn footer_keycap_estimated_width_px(token: &str) -> f32 {
    if token == FOOTER_MIC_ICON_TOKEN {
        return FOOTER_KEYCAP_HEIGHT_PX;
    }

    let estimated_text_width = token
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                FOOTER_HINT_FONT_SIZE_PX * 0.62
            } else {
                FOOTER_HINT_FONT_SIZE_PX * 0.82
            }
        })
        .sum::<f32>();
    (estimated_text_width + FOOTER_KEYCAP_PADDING_X_PX * 2.0)
        .max(FOOTER_KEYCAP_HEIGHT_PX)
        .ceil()
}

pub(crate) fn footer_labelcap_max_width_for_slot(slot_width_px: f32, key_width_px: f32) -> f32 {
    let key_gap = if key_width_px > 0.0 {
        FOOTER_ACTION_CONTENT_GAP_PX
    } else {
        0.0
    };
    (slot_width_px - (FOOTER_ACTION_CONTENT_PADDING_X_PX * 2.0) - key_gap - key_width_px)
        .max(FOOTER_KEYCAP_HEIGHT_PX)
}

fn render_footer_labelcap(
    label: SharedString,
    theme: &Theme,
    footer_text: gpui::Rgba,
    full_text: gpui::Hsla,
) -> AnyElement {
    render_footer_labelcap_constrained(label, theme, footer_text, full_text, None, false)
}

fn render_footer_labelcap_constrained(
    label: SharedString,
    theme: &Theme,
    footer_text: gpui::Rgba,
    full_text: gpui::Hsla,
    max_width_px: Option<f32>,
    force_width: bool,
) -> AnyElement {
    let mut cap = div()
        .flex_none()
        .min_w(px(FOOTER_KEYCAP_HEIGHT_PX))
        .min_h(px(FOOTER_KEYCAP_HEIGHT_PX))
        .h(px(FOOTER_KEYCAP_HEIGHT_PX))
        .line_height(px(FOOTER_KEYCAP_HEIGHT_PX))
        .px(px(FOOTER_KEYCAP_PADDING_X_PX))
        .rounded(px(FOOTER_KEYCAP_RADIUS_PX))
        .border_1()
        .border_color(footer_labelcap_border_color(theme))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_SYSTEM_UI)
        .font_weight(FOOTER_HINT_FONT_WEIGHT_GPUI)
        .text_size(px(FOOTER_HINT_FONT_SIZE_PX))
        .text_color(footer_text)
        .group_hover("footer-action-button", move |s| s.text_color(full_text));

    if let Some(max_width_px) = max_width_px {
        cap = cap.max_w(px(max_width_px)).overflow_hidden();
        if force_width {
            cap = cap.w(px(max_width_px));
        }
    }

    cap.child(
        div()
            .min_w(px(0.0))
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(label),
    )
    .into_any_element()
}

fn render_footer_shortcut_keycaps(shortcut: String, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(FOOTER_ACTION_CONTENT_GAP_PX))
        .children(
            split_footer_shortcut(&shortcut)
                .into_iter()
                .map(|token| render_footer_keycap(token, None, theme)),
        )
        .into_any_element()
}

fn render_footer_keycap(token: String, max_width_px: Option<f32>, theme: &Theme) -> AnyElement {
    let footer_text = footer_hint_text_color(theme);
    let full_text = theme.colors.text.primary.to_rgb();
    let token_child: AnyElement = if token == FOOTER_MIC_ICON_TOKEN {
        svg()
            .external_path(FOOTER_MIC_ICON_PATH)
            .size(px(13.0))
            .flex_shrink_0()
            .text_color(footer_text)
            .group_hover("footer-action-button", move |s| s.text_color(full_text))
            .into_any_element()
    } else {
        div()
            .h(px(FOOTER_KEYCAP_HEIGHT_PX))
            .line_height(px(FOOTER_KEYCAP_HEIGHT_PX))
            .mt(px(footer_key_glyph_nudge_y(&token)))
            .child(token)
            .into_any_element()
    };

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
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_SYSTEM_UI)
        .font_weight(FOOTER_HINT_FONT_WEIGHT_GPUI)
        .text_size(px(FOOTER_HINT_FONT_SIZE_PX))
        .text_color(footer_text)
        .group_hover("footer-action-button", move |s| s.text_color(full_text))
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
    fn footer_shortcut_width_reserves_split_keycaps() {
        assert_eq!(footer_shortcut_keycaps_width_px(""), 0.0);
        assert!(
            footer_shortcut_keycaps_width_px("⌘K")
                >= (FOOTER_KEYCAP_HEIGHT_PX * 2.0) + FOOTER_ACTION_CONTENT_GAP_PX
        );
        assert!(footer_shortcut_keycaps_width_px("↵") >= FOOTER_KEYCAP_HEIGHT_PX);
    }

    #[test]
    fn constrained_footer_content_leaves_room_for_keycaps() {
        let key_width = footer_shortcut_keycaps_width_px("↵");
        let label_max = footer_labelcap_max_width_for_slot(FOOTER_RUN_SLOT_MIN_WIDTH_PX, key_width);

        assert!(label_max >= FOOTER_KEYCAP_HEIGHT_PX);
        assert!(
            label_max
                + key_width
                + FOOTER_ACTION_CONTENT_GAP_PX
                + FOOTER_ACTION_CONTENT_PADDING_X_PX * 2.0
                <= FOOTER_RUN_SLOT_MIN_WIDTH_PX
        );
    }

    #[test]
    fn footer_key_glyph_nudges_match_footer_contract() {
        assert!(is_footer_return_key_glyph("↵"));
        assert!(!is_footer_return_key_glyph("Enter"));
        assert_eq!(footer_key_glyph_nudge_y("⌘"), 1.0);
        assert_eq!(footer_key_glyph_nudge_y("↵"), 2.0);
        assert_eq!(footer_key_glyph_nudge_y(";"), -1.0);
        assert_eq!(footer_appkit_glyph_y("⌘", 20.0, 10.0), 4.0);
        assert_eq!(footer_appkit_glyph_y("↵", 20.0, 10.0), 3.0);
        assert_eq!(footer_appkit_glyph_y(";", 20.0, 10.0), 6.0);
        assert_eq!(footer_button_height(32.0), 28.0);
    }

    #[test]
    fn footer_action_chrome_tokens_match_native_footer_contract() {
        assert_eq!(FOOTER_ACTION_ITEM_GAP_PX, 3.0);
        assert_eq!(FOOTER_ACTION_CONTENT_GAP_PX, 2.0);
        assert_eq!(FOOTER_ACTION_CONTENT_PADDING_X_PX, 2.0);
        assert_eq!(FOOTER_ACTION_BUTTON_RADIUS_PX, 4.0);
        assert_eq!(FOOTER_RUN_SLOT_MIN_WIDTH_PX, 92.0);
        assert_eq!(FOOTER_RUN_SLOT_MAX_WIDTH_PX, 172.0);
        assert_eq!(footer_action_slot_width(FooterActionSlot::Actions), 92.0);
        assert_eq!(footer_action_slot_width(FooterActionSlot::Ai), 52.0);
        assert_eq!(footer_action_slot_width(FooterActionSlot::Apply), 84.0);
        assert_eq!(footer_action_slot_width(FooterActionSlot::Close), 84.0);
        assert_eq!(footer_action_slot_width(FooterActionSlot::Stop), 76.0);
        assert_eq!(
            footer_action_slot_width(FooterActionSlot::PasteResponse),
            140.0
        );

        let mut theme = Theme::dark_default();
        let mut opacity = theme.get_opacity();
        opacity.hover = 0.21;
        opacity.selected = 0.47;
        theme.opacity = Some(opacity);

        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let rail = footer_rail_chrome(&theme);
        assert_eq!(
            rail.height_px,
            crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT
        );
        assert_eq!(
            rail.side_inset_px,
            crate::window_resize::mini_layout::HINT_STRIP_PADDING_X
        );
        assert_eq!(rail.surface_rgba, chrome.inline_dropdown_surface_rgba);
        assert_eq!(rail.divider_rgba, chrome.divider_rgba);
        assert_eq!(rail.hover_rgba, chrome.hover_rgba);
        assert_eq!(rail.active_rgba, chrome.selection_rgba);
        assert_eq!(rail.button_radius_px, FOOTER_ACTION_BUTTON_RADIUS_PX);
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
        assert_eq!(
            footer_keycap_border_color(&theme).a,
            ((list_colors.hover_opacity * 255.0) as u8) as f32 / 255.0
        );
        assert_eq!(
            footer_keycap_border_color_for_state(&theme, true).a,
            ((list_colors.selected_opacity * 255.0) as u8) as f32 / 255.0
        );
    }
}
