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
    let valid = model.copy_available();
    if model.kind == Kind::Beat {
        let active = model.beat.map(|b| b.number);
        let marks = div().flex().flex_row().gap_3().children((1..=4u8).map(|i| {
            let selected = active == Some(i);
            div()
                .w(px(units * 2.5))
                .h(px(units * 2.5))
                .rounded_lg()
                .border_1()
                .border_color(gpui::rgba(if selected {
                    colors.text_strong_rgba
                } else {
                    colors.text_hint_rgba
                }))
                .flex()
                .items_center()
                .justify_center()
                .text_color(gpui::rgba(if selected {
                    colors.text_strong_rgba
                } else {
                    colors.text_hint_rgba
                }))
                .child(i.to_string())
        }));
        return panel
            .child(marks)
            .child(words)
            .child("Visual only · no sound")
            .into_any_element();
    }
    if let Some(grid) = model.qr().cloned() {
        // QR modules are encoded data: opaque white/black with a four-module quiet zone,
        // independent of theme, vibrancy, and display scale. Whole-point cells avoid seams.
        let count = (grid.size + 2 * sk_one_window::display_code::QUIET) as f32;
        let cell = ((units * 14.).min(220.) / count).floor().max(1.);
        let side = px(cell * count);
        let code = gpui::canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                window.paint_quad(gpui::fill(bounds, gpui::rgb(0xffffff)));
                grid.runs(|x, y, n| {
                    window.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::point(
                                bounds.origin.x + px(x as f32 * cell),
                                bounds.origin.y + px(y as f32 * cell),
                            ),
                            gpui::size(px(n as f32 * cell), px(cell)),
                        ),
                        gpui::rgb(0x000000),
                    ))
                });
            },
        )
        .w(side)
        .h(side);
        return panel
            .child(div().id("local-display-qr").child(code))
            .child(words)
            .into_any_element();
    }
    if let Some(tones) = model.palette() {
        let strip =
            div()
                .w_full()
                .flex()
                .flex_row()
                .gap_2()
                .children(tones.into_iter().enumerate().map(|(i, color)| {
                    let label = format!("#{color:06X}");
                    div()
                        .id(("palette-tone", i))
                        .flex_1()
                        .min_w(px(0.))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .w_full()
                                .h(px(units * 6.))
                                .rounded_lg()
                                .border_1()
                                .border_color(gpui::rgba(colors.text_hint_rgba))
                                .bg(gpui::rgb(color)),
                        )
                        .child(div().text_size(px(units)).child(label))
                }));
        return panel.child(strip).child(words).into_any_element();
    }
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
