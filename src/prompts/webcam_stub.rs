//! Webcam prompt fallback UI for non-macOS platforms.
//!
//! The webcam command remains available so scripts can keep a stable surface,
//! but we show an explicit unsupported state instead of attempting capture.

use gpui::{div, prelude::*, rgb, Context, FocusHandle, Focusable, Render, Styled, Window};

use super::base::DesignContext;
use super::base::PromptBase;
use super::SubmitCallback;
use crate::theme;

#[derive(Debug, Clone)]
pub enum WebcamState {
    Unsupported(String),
    Error(String),
}

pub struct WebcamPrompt {
    pub base: PromptBase,
    pub state: WebcamState,
}

impl WebcamPrompt {
    pub fn new(
        id: String,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: std::sync::Arc<theme::Theme>,
    ) -> Self {
        Self {
            base: PromptBase::new(id, focus_handle, on_submit, theme),
            state: WebcamState::Unsupported(
                "Webcam capture is not supported on this platform".to_string(),
            ),
        }
    }

    pub fn set_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.state = WebcamState::Error(message);
        cx.notify();
    }

    fn state_label(&self) -> &str {
        match &self.state {
            WebcamState::Unsupported(msg) | WebcamState::Error(msg) => msg.as_str(),
        }
    }
}

impl Focusable for WebcamPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.base.focus_handle.clone()
    }
}

impl Render for WebcamPrompt {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let dc = DesignContext::new(&self.base.theme, self.base.design_variant);
        let colors = self.base.theme.colors.prompt_colors();

        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .bg(dc.bg_secondary())
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_secondary))
                    .child(self.state_label()),
            )
    }
}
