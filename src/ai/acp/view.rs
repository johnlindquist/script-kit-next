//! ACP chat view.
//!
//! Thin render wrapper around an `AcpThread` entity. Renders the message
//! list and composer input. Intentionally minimal - rich Markdown rendering
//! and permission dialogs will be added in a follow-up cycle.

use gpui::{
    div, prelude::*, App, Context, Entity, FocusHandle, Focusable, IntoElement, Render,
    SharedString, Window,
};

use super::thread::{AcpThread, AcpThreadMessageRole, AcpThreadStatus};

/// GPUI view entity wrapping an `AcpThread` for the Tab AI surface.
pub(crate) struct AcpChatView {
    pub(crate) thread: Entity<AcpThread>,
    focus_handle: FocusHandle,
}

impl AcpChatView {
    pub(crate) fn new(thread: Entity<AcpThread>, cx: &mut Context<Self>) -> Self {
        Self {
            thread,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Consume Tab / Shift+Tab so the global interceptors do not re-open a
    /// fresh ACP chat while one is already active.
    pub(crate) fn handle_tab_key(
        &mut self,
        _has_shift: bool,
        _cx: &mut Context<Self>,
    ) -> bool {
        true
    }

    pub(crate) fn set_input(&mut self, value: String, cx: &mut Context<Self>) {
        self.thread.update(cx, |thread, cx| thread.set_input(value, cx));
    }

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        if modifiers.platform || modifiers.control || modifiers.alt {
            cx.propagate();
            return;
        }

        if crate::ui_foundation::is_key_enter(key) && !modifiers.shift {
            let _ = self.thread.update(cx, |thread, cx| thread.submit_input(cx));
            cx.stop_propagation();
            return;
        }

        if crate::ui_foundation::is_key_backspace(key) {
            self.thread.update(cx, |thread, cx| {
                let mut text = thread.input.to_string();
                text.pop();
                thread.set_input(text, cx);
            });
            cx.stop_propagation();
            return;
        }

        if crate::ui_foundation::is_key_delete(key) {
            cx.stop_propagation();
            return;
        }

        if let Some(ch) = event.keystroke.key_char.as_deref() {
            if !ch.is_empty() {
                self.thread.update(cx, |thread, cx| {
                    let mut text = thread.input.to_string();
                    text.push_str(ch);
                    thread.set_input(text, cx);
                });
                cx.stop_propagation();
                return;
            }
        }

        cx.propagate();
    }
}

impl Focusable for AcpChatView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpChatView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let thread = self.thread.read(cx);
        let status = thread.status;

        let messages = thread
            .messages
            .iter()
            .map(|msg| {
                let role_label: SharedString = match msg.role {
                    AcpThreadMessageRole::User => "You".into(),
                    AcpThreadMessageRole::Assistant => "Assistant".into(),
                    AcpThreadMessageRole::Thought => "Thinking".into(),
                    AcpThreadMessageRole::Tool => "Tool".into(),
                    AcpThreadMessageRole::System => "System".into(),
                    AcpThreadMessageRole::Error => "Error".into(),
                };
                let body = msg.body.clone();
                (role_label, body)
            })
            .collect::<Vec<_>>();

        let input_text = thread.input.clone();
        let has_pending_permission = thread.pending_permission.is_some();

        div()
            .size_full()
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                this.handle_key_down(event, cx);
            }))
            .child(
                div()
                    .id("acp-message-list")
                    .flex_grow()
                    .overflow_y_scroll()
                    .min_h(gpui::px(0.))
                    .children(messages.into_iter().map(|(role_label, body)| {
                        div()
                            .w_full()
                            .p_2()
                            .child(
                                div()
                                    .text_xs()
                                    .opacity(0.5)
                                    .child(role_label),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .child(body),
                            )
                    })),
            )
            .child(
                div()
                    .w_full()
                    .p_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_grow()
                            .text_sm()
                            .child(input_text),
                    )
                    .child(
                        div()
                            .text_xs()
                            .opacity(0.4)
                            .child(SharedString::from(match status {
                                AcpThreadStatus::Idle => {
                                    if has_pending_permission {
                                        "Permission required"
                                    } else {
                                        "Ready"
                                    }
                                }
                                AcpThreadStatus::Streaming => "Streaming...",
                                AcpThreadStatus::WaitingForPermission => "Permission required",
                                AcpThreadStatus::Error => "Error",
                            })),
                    ),
            )
    }
}
