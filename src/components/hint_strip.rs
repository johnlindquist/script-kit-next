#![allow(dead_code)]

use gpui::{
    div, prelude::*, px, rgba, AnyElement, App, FontWeight, IntoElement, RenderOnce, SharedString,
    Window,
};

use crate::ui::chrome::{
    alpha_from_opacity, HINT_STRIP_HEIGHT, HINT_STRIP_PADDING_X, HINT_STRIP_PADDING_Y,
    HINT_TEXT_OPACITY,
};

const HINT_STRIP_CONTENT_GAP: f32 = 8.0;

#[derive(IntoElement)]
pub struct HintStrip {
    hints: Vec<SharedString>,
    leading: Option<AnyElement>,
}

impl HintStrip {
    pub fn new(hints: impl IntoHints) -> Self {
        Self {
            hints: hints.into_hints(),
            leading: None,
        }
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }
}

pub trait IntoHints {
    fn into_hints(self) -> Vec<SharedString>;
}

impl IntoHints for Vec<SharedString> {
    fn into_hints(self) -> Vec<SharedString> {
        self
    }
}

impl IntoHints for SharedString {
    fn into_hints(self) -> Vec<SharedString> {
        vec![self]
    }
}

impl IntoHints for &str {
    fn into_hints(self) -> Vec<SharedString> {
        vec![self.to_string().into()]
    }
}

impl IntoHints for String {
    fn into_hints(self) -> Vec<SharedString> {
        vec![self.into()]
    }
}

fn text_color_with_opacity(primary: u32, opacity: f32) -> u32 {
    // Theme text colors are stored as 0xAARRGGBB; strip the original alpha, shift RGB into
    // RRGGBB00, then inject the requested alpha byte for gpui::rgba.
    ((primary & 0x00FF_FFFF) << 8) | alpha_from_opacity(opacity)
}

impl RenderOnce for HintStrip {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let text_rgba = text_color_with_opacity(theme.colors.text.primary, HINT_TEXT_OPACITY);
        let joined_hints: SharedString = if self.hints.is_empty() {
            "".into()
        } else {
            self.hints
                .into_iter()
                .map(|hint| hint.to_string())
                .collect::<Vec<_>>()
                .join("  ")
                .into()
        };

        let mut row = div()
            .w_full()
            .h(px(HINT_STRIP_HEIGHT))
            .px(px(HINT_STRIP_PADDING_X))
            .py(px(HINT_STRIP_PADDING_Y))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(HINT_STRIP_CONTENT_GAP));

        if let Some(leading) = self.leading {
            row = row.child(leading);
        }

        row.child(div().flex_1()).child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(text_rgba))
                .child(joined_hints),
        )
    }
}
