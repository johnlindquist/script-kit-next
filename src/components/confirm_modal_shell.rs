use gpui::{div, prelude::*, px, AnyElement, Div, Rgba, SharedString, Stateful};

use crate::dev_style_tool::{
    runtime_overrides, StyleValue, CONFIRM_MODAL_GAP_KNOB_ID,
    CONFIRM_MODAL_HEADER_ACCENT_HEIGHT_KNOB_ID, CONFIRM_MODAL_HEADER_ACCENT_WIDTH_KNOB_ID,
    CONFIRM_MODAL_HEADER_GAP_KNOB_ID, CONFIRM_MODAL_PADDING_X_KNOB_ID,
    CONFIRM_MODAL_PADDING_Y_KNOB_ID, CONFIRM_MODAL_RADIUS_KNOB_ID,
};

pub(crate) const CONFIRM_MODAL_SHELL_ID: &str = "modal-shell:confirm";
pub(crate) const CONFIRM_MODAL_HEADER_ACCENT_WIDTH: f32 = 2.0;
pub(crate) const CONFIRM_MODAL_HEADER_ACCENT_HEIGHT: f32 = 14.0;
pub(crate) const CONFIRM_MODAL_HEADER_GAP: f32 = 8.0;
pub(crate) const CONFIRM_MODAL_RADIUS: f32 = 8.0;

#[derive(Clone)]
pub(crate) struct ConfirmModalShellConfig {
    pub(crate) content_id: &'static str,
    pub(crate) width: Option<f32>,
    pub(crate) padding_x: f32,
    pub(crate) padding_y: f32,
    pub(crate) gap: f32,
    pub(crate) background: Option<Rgba>,
    pub(crate) border: Rgba,
    pub(crate) radius: f32,
    pub(crate) offset_y: f32,
    pub(crate) opacity: f32,
}

pub(crate) fn confirm_modal_header(
    title: impl Into<SharedString>,
    accent: Rgba,
    title_color: Rgba,
) -> Div {
    let accent_width = confirm_modal_number_override(
        CONFIRM_MODAL_HEADER_ACCENT_WIDTH_KNOB_ID,
        CONFIRM_MODAL_HEADER_ACCENT_WIDTH,
    );
    let accent_height = confirm_modal_number_override(
        CONFIRM_MODAL_HEADER_ACCENT_HEIGHT_KNOB_ID,
        CONFIRM_MODAL_HEADER_ACCENT_HEIGHT,
    );
    let header_gap =
        confirm_modal_number_override(CONFIRM_MODAL_HEADER_GAP_KNOB_ID, CONFIRM_MODAL_HEADER_GAP);

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(header_gap))
        .child(
            div()
                .w(px(accent_width))
                .h(px(accent_height))
                .rounded(px(1.0))
                .bg(accent),
        )
        .child(
            div()
                .min_w(px(0.0))
                .truncate()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title.into()),
        )
}

pub(crate) fn confirm_modal_shell(
    config: ConfirmModalShellConfig,
    children: Vec<AnyElement>,
) -> Stateful<Div> {
    let padding_x =
        confirm_modal_number_override(CONFIRM_MODAL_PADDING_X_KNOB_ID, config.padding_x);
    let padding_y =
        confirm_modal_number_override(CONFIRM_MODAL_PADDING_Y_KNOB_ID, config.padding_y);
    let gap = confirm_modal_number_override(CONFIRM_MODAL_GAP_KNOB_ID, config.gap);
    let radius = confirm_modal_number_override(CONFIRM_MODAL_RADIUS_KNOB_ID, config.radius);

    let mut content = div()
        .id(config.content_id)
        .w_full()
        .p_0()
        .flex()
        .flex_col()
        .gap(px(gap));

    for child in children {
        content = content.child(child);
    }

    let mut shell = div()
        .id(CONFIRM_MODAL_SHELL_ID)
        .px(px(padding_x))
        .py(px(padding_y))
        .border_1()
        .border_color(config.border)
        .rounded(px(radius))
        .flex()
        .flex_col()
        .mt(px(config.offset_y))
        .opacity(config.opacity)
        .overflow_hidden()
        .child(content);

    if let Some(width) = config.width {
        shell = shell.w(px(width));
    } else {
        shell = shell.size_full();
    }

    if let Some(background) = config.background {
        shell = shell.bg(background);
    }

    shell
}

pub(crate) fn confirm_modal_number_override(
    id: crate::dev_style_tool::ConfirmModalKnobId,
    fallback: f32,
) -> f32 {
    match runtime_overrides::current_confirm_modal_value(id) {
        Some(StyleValue::Number(value)) => value,
        None => fallback,
    }
}
