use gpui::{div, prelude::*, px, rgb, Context, FocusHandle, Render, Window};

use crate::theme;

pub struct AcpComposer {
    focus_handle: FocusHandle,
    input_text: String,
    cursor_visible: bool,
}

impl AcpComposer {
    pub fn new(focus_handle: FocusHandle, _cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle,
            input_text: String::new(),
            cursor_visible: true,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpComposer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();

        div().w_full().px(px(12.0)).py(px(10.0)).child(
            div()
                .text_sm()
                .text_color(rgb(theme.colors.text.primary))
                .child(if self.input_text.is_empty() {
                    "Type something...".to_string()
                } else {
                    self.input_text.clone()
                }),
        )
    }
}
