//! A live cue or color sample below the normal launcher header. Unlike a list row the cue
//! needs display-scale type. Both variants use ordinary chrome, shared spacing and the same
//! input; the color chip is user data, not a new theme or a second material.
use crate::theme::{AppChromeColors, Theme};
use gpui::{div, prelude::*, px, AnyElement};
use sk_one_window::display::{self, Display, Kind};

pub fn render(model: &Display, theme: &Theme) -> AnyElement {
    let colors = AppChromeColors::from_theme(theme);
    let words = model.shown();
    let units = theme.get_fonts().ui_size;
    let mut panel = div()
        .id("local-display-preview")
        .w_full()
        .flex_1()
        .min_h(px(0.))
        .px_6()
        .py_4()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .text_center()
        .text_color(gpui::rgba(colors.text_strong_rgba));
    let valid = model.value().is_some();
    if model.kind == Kind::Color {
        if let Some(value) = display::hex(model.text.trim()) {
            panel = panel.child(
                div()
                    .id("local-display-swatch")
                    .w_full()
                    .h(px(units * 8.))
                    .rounded_lg()
                    .border_1()
                    .border_color(gpui::rgba(colors.text_hint_rgba))
                    .bg(gpui::rgb(value)),
            );
        }
    }
    let scale = if !valid {
        1.25
    } else if model.kind == Kind::Color {
        2.25
    } else {
        match words.chars().count() {
            0..=28 => 4.5,
            29..=80 => 3.25,
            _ => 2.25,
        }
    };
    panel
        .child(
            div()
                .w_full()
                .text_size(px((units * scale).clamp(14., 90.)))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(words),
        )
        .into_any_element()
}
