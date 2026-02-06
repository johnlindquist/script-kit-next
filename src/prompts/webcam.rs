//! Webcam prompt UI component — renders CVPixelBuffer via gpui::surface()

use core_video::pixel_buffer::CVPixelBuffer;
use gpui::{
    div, prelude::*, px, rgb, Context, FocusHandle, Focusable, ObjectFit, Render, Styled, Window,
};

use super::base::DesignContext;
use super::base::PromptBase;
use super::SubmitCallback;
use crate::camera::CaptureHandle;
use crate::theme;

/// Webcam prompt state
#[derive(Debug, Clone)]
pub enum WebcamState {
    Initializing,
    Live,
    Error(String),
}

/// Webcam prompt component
pub struct WebcamPrompt {
    pub base: PromptBase,
    pub state: WebcamState,
    pub mirror: bool,
    /// Latest CVPixelBuffer from camera — rendered via gpui::surface()
    pub pixel_buffer: Option<CVPixelBuffer>,
    pub frame_width: u32,
    pub frame_height: u32,
    /// Owns the AVFoundation capture session — dropped when prompt closes,
    /// which stops the camera and releases all resources.
    pub capture_handle: Option<CaptureHandle>,
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
            state: WebcamState::Initializing,
            mirror: false,
            pixel_buffer: None,
            frame_width: 0,
            frame_height: 0,
            capture_handle: None,
        }
    }

    /// Set the latest CVPixelBuffer from camera (zero-copy)
    pub fn set_pixel_buffer(&mut self, buf: CVPixelBuffer, cx: &mut Context<Self>) {
        self.frame_width = buf.get_width() as u32;
        self.frame_height = buf.get_height() as u32;
        self.pixel_buffer = Some(buf);
        self.state = WebcamState::Live;
        cx.notify();
    }

    pub fn set_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.state = WebcamState::Error(message);
        cx.notify();
    }

    fn state_label(&self) -> String {
        match &self.state {
            WebcamState::Initializing => "Starting camera...".into(),
            WebcamState::Live => format!("{}x{}", self.frame_width, self.frame_height),
            WebcamState::Error(msg) => msg.clone(),
        }
    }
}

impl Focusable for WebcamPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.base.focus_handle.clone()
    }
}

impl Render for WebcamPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dc = DesignContext::new(&self.base.theme, self.base.design_variant);
        let colors = self.base.theme.colors.prompt_colors();

        let handle_key = cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.to_ascii_lowercase();
            match key.as_str() {
                "enter" | "space" => {
                    this.base.submit(None);
                    cx.notify();
                }
                "escape" | "esc" => {
                    this.base.cancel();
                    cx.notify();
                }
                "m" => {
                    this.mirror = !this.mirror;
                    cx.notify();
                }
                _ => {}
            }
        });

        let mirror_label = if self.mirror { "On" } else { "Off" };

        // Camera preview: use gpui::surface() for zero-copy GPU rendering
        let preview = if let Some(ref buf) = self.pixel_buffer {
            div()
                .w_full()
                .flex_1()
                .rounded(px(8.0))
                .overflow_hidden()
                .child(
                    gpui::surface(buf.clone())
                        .object_fit(ObjectFit::Contain)
                        .w_full()
                        .h_full(),
                )
        } else {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .w_full()
                .bg(dc.bg_secondary())
                .border_1()
                .border_color(dc.border())
                .rounded(px(8.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_secondary))
                        .child(self.state_label()),
                )
        };

        div()
            .id("webcam-prompt")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            // No bg — let vibrancy show through from Root (matches other prompts)
            .text_color(rgb(colors.text_primary))
            .p(px(12.0))
            .track_focus(&self.base.focus_handle)
            .on_key_down(handle_key)
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pb(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Webcam"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_secondary))
                            .child(self.state_label()),
                    ),
            )
            // Preview (fills remaining space)
            .child(preview)
            // Footer: shortcuts + status
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .pt(px(8.0))
                    .text_xs()
                    .text_color(rgb(colors.text_tertiary))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .child("Enter: Capture")
                            .child("Esc: Close")
                            .child("M: Mirror"),
                    )
                    .child(
                        div()
                            .text_color(rgb(colors.text_secondary))
                            .child(format!("Mirror: {mirror_label}")),
                    ),
            )
    }
}
