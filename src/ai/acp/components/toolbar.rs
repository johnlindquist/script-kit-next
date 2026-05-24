use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Entity, FocusHandle, IntoElement, ParentElement,
    Render, Window,
};

use super::super::thread::AcpThreadStatus;
use crate::theme;

pub enum AcpToolbarEvent {
    ToggleModelSelector,
    ExportThread,
    ClearThread,
    OpenHistory,
    CloseChat,
}

impl gpui::EventEmitter<AcpToolbarEvent> for AcpToolbar {}

pub struct AcpToolbar {
    status: AcpThreadStatus,
    model_name: String,
    focus_handle: FocusHandle,
}

impl AcpToolbar {
    pub fn new(status: AcpThreadStatus, model_name: String, cx: &mut Context<Self>) -> Self {
        Self {
            status,
            model_name,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn set_status(&mut self, status: AcpThreadStatus, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    pub fn set_model_name(&mut self, model_name: String, cx: &mut Context<Self>) {
        self.model_name = model_name;
        cx.notify();
    }
}

impl Render for AcpToolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();

        div()
            .id("acp-toolbar")
            .w_full()
            .h(px(32.0))
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .bg(if theme.is_vibrancy_enabled() {
                rgba(0x00000000)
            } else {
                rgb(theme.colors.background.main)
            })
            .border_b_1()
            .border_color(rgb(theme.colors.ui.border))
            .child(
                div()
                    .id("acp-toolbar-model-selector")
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .cursor_pointer()
                    .on_click(cx.listener(|_view, _event, _window, cx| {
                        cx.emit(AcpToolbarEvent::ToggleModelSelector);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.colors.text.primary))
                            .child(self.model_name.clone()),
                    ),
            )
            .child(
                div().flex().items_center().gap(px(12.0)).child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child(format!("{:?}", self.status)),
                ),
            )
    }
}
