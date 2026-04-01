//! ACP chat view.
//!
//! Renders an ACP conversation thread with markdown-rendered messages,
//! role-aware cards, empty/streaming/error states, and permission approval
//! overlay. Wraps an `AcpThread` entity for the Tab AI surface.

use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, ParentElement, Render, SharedString, Window,
};

use crate::prompts::markdown::render_markdown_with_scope;
use crate::theme::{self, PromptColors};

use super::thread::{AcpThread, AcpThreadMessage, AcpThreadMessageRole, AcpThreadStatus};
use super::AcpApprovalRequest;

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
        cx: &mut Context<Self>,
    ) -> bool {
        cx.notify();
        true
    }

    pub(crate) fn set_input(&mut self, value: String, cx: &mut Context<Self>) {
        self.thread.update(cx, |thread, cx| thread.set_input(value, cx));
    }

    // ── Rendering helpers ─────────────────────────────────────────

    fn prompt_colors() -> PromptColors {
        PromptColors::from_theme(&theme::get_cached_theme())
    }

    fn role_title(role: AcpThreadMessageRole) -> SharedString {
        match role {
            AcpThreadMessageRole::User => "You".into(),
            AcpThreadMessageRole::Assistant => "Claude Code".into(),
            AcpThreadMessageRole::Thought => "Thinking".into(),
            AcpThreadMessageRole::Tool => "Tool".into(),
            AcpThreadMessageRole::System => "System".into(),
            AcpThreadMessageRole::Error => "Error".into(),
        }
    }

    fn render_message_card(msg: &AcpThreadMessage, colors: &PromptColors) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        let (surface, border, title_opacity) = match msg.role {
            AcpThreadMessageRole::User => (
                rgba((theme.colors.accent.selected << 8) | 0x16),
                rgba((theme.colors.accent.selected << 8) | 0x44),
                0.82,
            ),
            AcpThreadMessageRole::Assistant => (
                rgba((theme.colors.background.search_box << 8) | 0xF2),
                rgba((theme.colors.ui.border << 8) | 0x70),
                0.72,
            ),
            AcpThreadMessageRole::Thought => (
                rgba((theme.colors.text.primary << 8) | 0x08),
                rgba((theme.colors.text.primary << 8) | 0x22),
                0.62,
            ),
            AcpThreadMessageRole::Tool => (
                rgba((theme.colors.accent.selected << 8) | 0x10),
                rgba((theme.colors.accent.selected << 8) | 0x38),
                0.72,
            ),
            AcpThreadMessageRole::System => (
                rgba((theme.colors.text.primary << 8) | 0x08),
                rgba((theme.colors.ui.border << 8) | 0x60),
                0.62,
            ),
            AcpThreadMessageRole::Error => (
                rgba(0xEF444420),
                rgba(0xEF444480),
                0.86,
            ),
        };

        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(10.0))
            .rounded(px(10.0))
            .bg(surface)
            .border_1()
            .border_color(border)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(title_opacity)
                    .pb(px(6.0))
                    .child(Self::role_title(msg.role)),
            )
            .child(
                render_markdown_with_scope(
                    &msg.body,
                    colors,
                    Some(&scope_id),
                )
                .w_full(),
            )
            .into_any_element()
    }

    fn render_empty_state() -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px(px(24.0))
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.9)
                    .child("Tab AI is ready"),
            )
            .child(
                div()
                    .pt(px(8.0))
                    .text_sm()
                    .opacity(0.65)
                    .child("Context is assembling in the background. Type a request and press Enter."),
            )
            .child(
                div()
                    .pt(px(14.0))
                    .text_xs()
                    .text_color(rgb(theme.colors.accent.selected))
                    .opacity(0.8)
                    .child("Claude Code over ACP"),
            )
            .into_any_element()
    }

    fn render_streaming_hint() -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .size(px(6.0))
                    .rounded_full()
                    .bg(rgb(theme.colors.accent.selected)),
            )
            .child(
                div()
                    .text_xs()
                    .opacity(0.7)
                    .child("Streaming response\u{2026}"),
            )
            .into_any_element()
    }

    fn render_status_badge(
        status: AcpThreadStatus,
        has_pending_permission: bool,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        let (label, bg) = match status {
            AcpThreadStatus::Idle if has_pending_permission => (
                "Permission required",
                rgba((theme.colors.accent.selected << 8) | 0x18),
            ),
            AcpThreadStatus::Idle => (
                "Ready",
                rgba((theme.colors.text.primary << 8) | 0x08),
            ),
            AcpThreadStatus::Streaming => (
                "Streaming",
                rgba((theme.colors.accent.selected << 8) | 0x16),
            ),
            AcpThreadStatus::WaitingForPermission => (
                "Permission required",
                rgba((theme.colors.accent.selected << 8) | 0x18),
            ),
            AcpThreadStatus::Error => (
                "Error",
                rgba(0xEF444420),
            ),
        };

        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(999.0))
            .bg(bg)
            .text_xs()
            .opacity(0.8)
            .child(label)
            .into_any_element()
    }

    fn render_permission_overlay(
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        let title = request.title.clone();
        let body = request.body.clone();
        let options = request.options.clone();

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(theme::modal_overlay_bg(&theme, 0x80))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(520.0))
                    .max_w_full()
                    .mx_4()
                    .p_4()
                    .rounded(px(14.0))
                    .bg(rgb(theme.colors.background.search_box))
                    .border_1()
                    .border_color(rgba((theme.colors.ui.border << 8) | 0x99))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(
                        div()
                            .pt(px(8.0))
                            .pb(px(12.0))
                            .text_sm()
                            .opacity(0.76)
                            .child(body),
                    )
                    .children(options.into_iter().enumerate().map(|(i, option)| {
                        let option_id = option.option_id.clone();
                        let label = format!("{} \u{00b7} {}", option.name, option.kind);

                        div()
                            .id(SharedString::from(format!("perm-opt-{i}")))
                            .mt(px(8.0))
                            .px(px(10.0))
                            .py(px(9.0))
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .bg(rgba((theme.colors.text.primary << 8) | 0x06))
                            .hover(|d| {
                                d.bg(rgba((theme.colors.text.primary << 8) | 0x12))
                            })
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.thread.update(cx, |thread, cx| {
                                    thread.approve_pending_permission(
                                        Some(option_id.clone()),
                                        cx,
                                    );
                                });
                            }))
                            .child(label)
                    }))
                    .child(
                        div()
                            .id("perm-cancel")
                            .mt(px(12.0))
                            .text_sm()
                            .opacity(0.62)
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.thread.update(cx, |thread, cx| {
                                    thread.approve_pending_permission(None, cx);
                                });
                            }))
                            .child("Cancel"),
                    ),
            )
            .into_any_element()
    }

    fn render_plan_strip(entries: &[String]) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba((theme.colors.accent.selected << 8) | 0x0C))
            .border_1()
            .border_color(rgba((theme.colors.accent.selected << 8) | 0x28))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.7)
                    .pb(px(4.0))
                    .child("Plan"),
            )
            .children(entries.iter().enumerate().map(|(i, entry)| {
                div()
                    .text_xs()
                    .opacity(0.65)
                    .py(px(1.0))
                    .child(format!("{}. {}", i + 1, entry))
            }))
            .into_any_element()
    }

    // ── Key handling ──────────────────────────────────────────────

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

        // Shift+Enter inserts a newline.
        if crate::ui_foundation::is_key_enter(key) && modifiers.shift {
            self.thread.update(cx, |thread, cx| {
                let mut text = thread.input.to_string();
                text.push('\n');
                thread.set_input(text, cx);
            });
            cx.stop_propagation();
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
        let is_empty = thread.messages.is_empty();
        let input_text = thread.input.clone();
        let has_pending_permission = thread.pending_permission.is_some();
        let pending_permission = thread.pending_permission.clone();
        let plan_entries = thread.active_plan_entries().to_vec();
        let messages: Vec<AcpThreadMessage> = thread.messages.clone();
        let colors = Self::prompt_colors();

        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                this.handle_key_down(event, cx);
            }))
            // ── Message list ──────────────────────────────────
            .child(
                div()
                    .id("acp-message-list")
                    .flex_grow()
                    .overflow_y_scroll()
                    .min_h(gpui::px(0.))
                    .when(is_empty, |d| d.child(Self::render_empty_state()))
                    .when(!is_empty, |d| {
                        d.p_2()
                            .gap_2()
                            .flex()
                            .flex_col()
                            .children(messages.iter().map(|msg| {
                                div()
                                    .w_full()
                                    .pb(px(4.0))
                                    .child(Self::render_message_card(msg, &colors))
                            }))
                    }),
            )
            // ── Plan strip ────────────────────────────────────
            .when(!plan_entries.is_empty(), |d| {
                d.child(
                    div()
                        .w_full()
                        .px(px(8.0))
                        .pb(px(4.0))
                        .child(Self::render_plan_strip(&plan_entries)),
                )
            })
            // ── Streaming hint ────────────────────────────────
            .when(
                matches!(status, AcpThreadStatus::Streaming),
                |d| {
                    d.child(
                        div()
                            .w_full()
                            .px(px(12.0))
                            .pb(px(6.0))
                            .child(Self::render_streaming_hint()),
                    )
                },
            )
            // ── Footer: input + status ────────────────────────
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
                            .opacity(0.45)
                            .child("Enter to send"),
                    )
                    .child(Self::render_status_badge(status, has_pending_permission)),
            )
            // ── Permission overlay ────────────────────────────
            .when_some(pending_permission, |d, request| {
                d.child(Self::render_permission_overlay(&request, cx))
            })
    }
}
