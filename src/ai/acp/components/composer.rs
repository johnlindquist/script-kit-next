use gpui::{div, prelude::*, px, rgba, Context, FocusHandle, Render, Window};

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
        let chrome = theme::AppChromeColors::from_theme(&theme);

        div().w_full().px(px(12.0)).py(px(10.0)).child(
            div()
                .min_h(px(
                    crate::ui::chrome::TAHOE_CHROME_METRICS.acp_composer_min_height
                ))
                .w_full()
                .flex()
                .items_center()
                .px(px(12.0))
                .rounded(px(crate::ui::chrome::TAHOE_CHROME_METRICS.control_md_radius))
                .bg(rgba(chrome.input_surface_rgba))
                .border_1()
                .border_color(rgba(chrome.border_rgba))
                .text_sm()
                .text_color(if self.input_text.is_empty() {
                    rgba(chrome.placeholder_text_rgba)
                } else {
                    rgba((chrome.text_primary_hex << 8) | 0xff)
                })
                .child(if self.input_text.is_empty() {
                    "Type something...".to_string()
                } else {
                    self.input_text.clone()
                }),
        )
    }
}
