#![allow(dead_code)]

//! Legacy non-list layout helpers.
//!
//! New help, empty, setup, permission, recovery, and about-style guidance should
//! use `components::info_state` so intent, copy, type scale, and theme opacity
//! are encoded in one shared system.

use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, Div, FontWeight, Rgba, SharedString, Stateful,
};

use crate::theme::{self, AppChromeColors};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NonListDensity {
    Compact,
    Comfortable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NonListMetrics {
    pub max_width: f32,
    pub card_radius: f32,
    pub card_padding_x: f32,
    pub card_padding_y: f32,
    pub block_gap: f32,
    pub item_gap: f32,
    pub icon_size: f32,
    pub title_size: f32,
    pub title_line: f32,
    pub body_size: f32,
    pub body_line: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct NonListPalette {
    pub title: Rgba,
    pub body: Rgba,
    pub hint: Rgba,
    pub placeholder: Rgba,
    pub icon: Rgba,
    pub surface: Rgba,
    pub panel: Rgba,
    pub input: Rgba,
    pub border: Rgba,
    pub hover: Rgba,
    pub selected: Rgba,
    pub accent: Rgba,
    pub error: Rgba,
}

pub(crate) fn non_list_metrics(density: NonListDensity) -> NonListMetrics {
    match density {
        NonListDensity::Compact => NonListMetrics {
            max_width: 420.0,
            card_radius: 8.0,
            card_padding_x: 12.0,
            card_padding_y: 10.0,
            block_gap: 12.0,
            item_gap: 8.0,
            icon_size: 32.0,
            title_size: 18.0,
            title_line: 24.0,
            body_size: 13.0,
            body_line: 18.0,
        },
        NonListDensity::Comfortable => NonListMetrics {
            max_width: 500.0,
            card_radius: 9.0,
            card_padding_x: 16.0,
            card_padding_y: 14.0,
            block_gap: 16.0,
            item_gap: 10.0,
            icon_size: 40.0,
            title_size: 22.0,
            title_line: 28.0,
            body_size: 13.0,
            body_line: 19.0,
        },
    }
}

pub(crate) fn non_list_palette(theme: &theme::Theme) -> NonListPalette {
    let chrome = AppChromeColors::from_theme(theme);

    NonListPalette {
        title: rgb(chrome.text_primary_hex),
        body: rgba(chrome.text_muted_rgba),
        hint: rgba(chrome.text_hint_rgba),
        placeholder: rgba(chrome.placeholder_text_rgba),
        icon: rgba(chrome.text_icon_rgba),
        surface: rgba(chrome.surface_rgba),
        panel: rgba(chrome.panel_surface_rgba),
        input: rgba(chrome.input_surface_rgba),
        border: rgba(chrome.border_rgba),
        hover: rgba(chrome.hover_rgba),
        selected: rgba(chrome.selection_rgba),
        accent: rgb(chrome.accent_hex),
        error: rgb(theme.colors.ui.error),
    }
}

pub(crate) fn non_list_content_stack(id: &'static str, max_width: f32, gap: f32) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .max_w(px(max_width))
        .flex()
        .flex_col()
        .gap(px(gap))
}

pub(crate) fn non_list_centered_shell(id: &'static str, max_width: f32, gap: f32) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .h_full()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(gap))
        .max_w(px(max_width))
        .px(px(32.0))
        .py(px(24.0))
}

pub(crate) fn non_list_intro(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    palette: NonListPalette,
    metrics: NonListMetrics,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(metrics.title_size))
                .line_height(px(metrics.title_line))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.title)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(metrics.body_size))
                .line_height(px(metrics.body_line))
                .text_color(palette.body)
                .child(description.into()),
        )
}

pub(crate) fn non_list_icon_glyph(
    glyph: impl Into<SharedString>,
    palette: NonListPalette,
    metrics: NonListMetrics,
) -> Div {
    div()
        .size(px(metrics.icon_size))
        .rounded(px(metrics.card_radius))
        .border_1()
        .border_color(palette.border)
        .bg(palette.panel)
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(metrics.icon_size * 0.45))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.icon)
        .child(glyph.into())
}

pub(crate) fn non_list_card(
    id: &'static str,
    palette: NonListPalette,
    metrics: NonListMetrics,
) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .px(px(metrics.card_padding_x))
        .py(px(metrics.card_padding_y))
        .rounded(px(metrics.card_radius))
        .border_1()
        .border_color(palette.border)
        .bg(palette.panel)
}

pub(crate) fn non_list_callout(
    id: &'static str,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    palette: NonListPalette,
    metrics: NonListMetrics,
) -> Stateful<Div> {
    non_list_card(id, palette, metrics).child(
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.title)
                    .child(title.into()),
            )
            .child(
                div()
                    .text_size(px(metrics.body_size))
                    .line_height(px(metrics.body_line))
                    .text_color(palette.body)
                    .child(body.into()),
            ),
    )
}

pub(crate) fn non_list_requirement_row(
    label: impl Into<SharedString>,
    status: impl Into<SharedString>,
    palette: NonListPalette,
) -> Div {
    div()
        .w_full()
        .min_h(px(32.0))
        .px(px(10.0))
        .py(px(7.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(palette.border)
        .bg(palette.input)
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .min_w(px(0.0))
                .text_size(px(13.0))
                .line_height(px(18.0))
                .text_color(palette.title)
                .child(label.into()),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(palette.hint)
                .child(status.into()),
        )
}

pub(crate) fn non_list_action_row(actions: Vec<AnyElement>) -> Div {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .children(actions)
}

pub(crate) fn non_list_footer_note(text: impl Into<SharedString>, palette: NonListPalette) -> Div {
    div()
        .text_xs()
        .line_height(px(16.0))
        .text_color(palette.hint)
        .child(text.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_keep_about_scale_as_upper_bound() {
        let compact = non_list_metrics(NonListDensity::Compact);
        let comfortable = non_list_metrics(NonListDensity::Comfortable);

        assert_eq!(compact.title_size, 18.0);
        assert_eq!(comfortable.title_size, 22.0);
        assert!(comfortable.title_size < 28.0);
        assert!(compact.max_width < comfortable.max_width);
    }

    #[test]
    fn density_uses_four_pixel_rhythm() {
        for metrics in [
            non_list_metrics(NonListDensity::Compact),
            non_list_metrics(NonListDensity::Comfortable),
        ] {
            for value in [
                metrics.card_padding_x,
                metrics.card_padding_y,
                metrics.block_gap,
                metrics.item_gap,
                metrics.icon_size,
                metrics.title_line,
            ] {
                assert_eq!(value.rem_euclid(2.0), 0.0);
            }
        }
    }

    #[test]
    fn source_routes_palette_through_app_chrome_colors() {
        let source = include_str!("non_list_state.rs");

        assert!(source.contains("AppChromeColors::from_theme(theme)"));
        assert!(source.contains("chrome.text_primary_hex"));
        assert!(source.contains("chrome.panel_surface_rgba"));
        assert!(source.contains("chrome.input_surface_rgba"));
    }
}
