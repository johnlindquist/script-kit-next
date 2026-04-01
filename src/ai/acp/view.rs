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
    /// Index of the currently highlighted permission option in the overlay.
    permission_index: usize,
}

impl AcpChatView {
    pub(crate) fn new(thread: Entity<AcpThread>, cx: &mut Context<Self>) -> Self {
        Self {
            thread,
            focus_handle: cx.focus_handle(),
            permission_index: 0,
        }
    }

    /// Consume Tab / Shift+Tab. When the permission overlay is open,
    /// cycle the highlighted option; otherwise just swallow the key so
    /// the global interceptors do not re-open a fresh ACP chat.
    pub(crate) fn handle_tab_key(
        &mut self,
        has_shift: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let option_count = self
            .thread
            .read(cx)
            .pending_permission
            .as_ref()
            .map(|r| r.options.len())
            .unwrap_or(0);

        if option_count > 0 {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, has_shift);
            cx.notify();
            return true;
        }

        cx.notify();
        true
    }

    fn approve_permission(&mut self, option_id: Option<String>, cx: &mut Context<Self>) {
        self.permission_index = 0;
        self.thread.update(cx, |thread, cx| {
            thread.approve_pending_permission(option_id, cx);
        });
    }

    fn normalized_permission_index(&self, option_count: usize) -> usize {
        if option_count == 0 {
            0
        } else {
            self.permission_index.min(option_count - 1)
        }
    }

    fn step_permission_index(current: usize, option_count: usize, reverse: bool) -> usize {
        if option_count == 0 {
            return 0;
        }

        if reverse {
            if current == 0 {
                option_count - 1
            } else {
                current - 1
            }
        } else {
            (current + 1) % option_count
        }
    }

    /// Handle key events when the permission overlay is displayed.
    /// Returns `true` if the key was consumed.
    fn handle_permission_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = event.keystroke.key.as_str();
        let option_count = request.options.len();
        self.permission_index = self.normalized_permission_index(option_count);

        if crate::ui_foundation::is_key_up(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, true);
            cx.notify();
            return true;
        }

        if crate::ui_foundation::is_key_down(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, false);
            cx.notify();
            return true;
        }

        // J/K navigation (vim-style, unmodified only)
        match key {
            "j" | "J" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, false);
                cx.notify();
                return true;
            }
            "k" | "K" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, true);
                cx.notify();
                return true;
            }
            _ => {}
        }

        if crate::ui_foundation::is_key_escape(key) {
            self.approve_permission(None, cx);
            return true;
        }

        if crate::ui_foundation::is_key_enter(key) {
            if let Some(option) = request.options.get(self.normalized_permission_index(option_count))
            {
                self.approve_permission(Some(option.option_id.clone()), cx);
            } else {
                self.approve_permission(None, cx);
            }
            return true;
        }

        // 1-9 instant pick
        if let Ok(digit) = key.parse::<usize>() {
            if digit >= 1 {
                let idx = digit - 1;
                if let Some(option) = request.options.get(idx) {
                    self.permission_index = idx;
                    self.approve_permission(Some(option.option_id.clone()), cx);
                    return true;
                }
            }
        }

        false
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

    fn render_permission_section(
        title: &'static str,
        text: String,
    ) -> gpui::AnyElement {
        div()
            .pt(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.64)
                    .child(title),
            )
            .child(
                div()
                    .mt(px(4.0))
                    .max_h(px(140.0))
                    .overflow_y_hidden()
                    .rounded(px(8.0))
                    .bg(rgba(0x00000018))
                    .px(px(10.0))
                    .py(px(8.0))
                    .text_xs()
                    .child(text),
            )
            .into_any_element()
    }

    fn render_permission_overlay(
        request: &AcpApprovalRequest,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let preview = request.preview.clone();
        let selected_index = selected_index.min(request.options.len().saturating_sub(1));

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
                    .w(px(640.0))
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
                            .child(request.title.clone()),
                    )
                    // ── Structured preview sections ──────────────
                    .when_some(preview.clone(), |d, preview| {
                        d.child(
                            div()
                                .pt(px(8.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(preview.tool_title),
                                )
                                .child(
                                    div()
                                        .pt(px(2.0))
                                        .text_xs()
                                        .opacity(0.62)
                                        .child(format!("Tool call ID: {}", preview.tool_call_id)),
                                )
                                .when_some(preview.subject, |d, subject| {
                                    d.child(
                                        div()
                                            .pt(px(2.0))
                                            .text_xs()
                                            .opacity(0.72)
                                            .child(subject),
                                    )
                                })
                                .when_some(preview.summary, |d, summary| {
                                    d.child(Self::render_permission_section("Summary", summary))
                                })
                                .when_some(preview.input_preview, |d, input| {
                                    d.child(Self::render_permission_section("Input", input))
                                })
                                .when_some(preview.output_preview, |d, output| {
                                    d.child(Self::render_permission_section("Output", output))
                                })
                                .when(!preview.option_summary.is_empty(), |d| {
                                    d.child(
                                        div()
                                            .pt(px(8.0))
                                            .text_xs()
                                            .opacity(0.52)
                                            .child(format!(
                                                "Available options: {}",
                                                preview.option_summary.join(" \u{00b7} ")
                                            )),
                                    )
                                }),
                        )
                    })
                    // ── Fallback to body when no preview ─────────
                    .when(preview.is_none(), |d| {
                        d.child(
                            div()
                                .pt(px(8.0))
                                .pb(px(12.0))
                                .text_sm()
                                .opacity(0.76)
                                .child(request.body.clone()),
                        )
                    })
                    // ── Option list with keyboard highlight ──────
                    .children(request.options.iter().enumerate().map(|(i, option)| {
                        let option_id = option.option_id.clone();
                        let is_selected = i == selected_index;
                        let label = format!(
                            "{} \u{00b7} {} \u{00b7} {}",
                            i + 1,
                            option.name,
                            option.kind,
                        );

                        div()
                            .id(SharedString::from(format!("perm-opt-{i}")))
                            .mt(px(8.0))
                            .px(px(10.0))
                            .py(px(9.0))
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgba((theme.colors.accent.selected << 8) | 0x18)
                            } else {
                                rgba((theme.colors.text.primary << 8) | 0x06)
                            })
                            .border_1()
                            .border_color(if is_selected {
                                rgba((theme.colors.accent.selected << 8) | 0x55)
                            } else {
                                rgba((theme.colors.ui.border << 8) | 0x30)
                            })
                            .hover(|d| {
                                d.bg(rgba((theme.colors.text.primary << 8) | 0x12))
                            })
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.approve_permission(Some(option_id.clone()), cx);
                            }))
                            .child(label)
                    }))
                    // ── Keyboard hint strip ──────────────────────
                    .child(
                        div()
                            .pt(px(12.0))
                            .text_xs()
                            .opacity(0.56)
                            .child(
                                "Tab/\u{21e7}Tab or \u{2191}\u{2193} or J/K to move \u{00b7} 1\u{2013}9 to choose \u{00b7} Enter to confirm \u{00b7} Esc to cancel",
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_mode_badge(mode_id: &str) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .px(px(8.0))
            .py(px(3.0))
            .rounded(px(999.0))
            .bg(rgba((theme.colors.accent.selected << 8) | 0x14))
            .border_1()
            .border_color(rgba((theme.colors.accent.selected << 8) | 0x30))
            .text_xs()
            .opacity(0.78)
            .child(format!("Mode: {mode_id}"))
            .into_any_element()
    }

    fn render_commands_strip(commands: &[String]) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .w_full()
            .px(px(12.0))
            .py(px(6.0))
            .rounded(px(8.0))
            .bg(rgba((theme.colors.text.primary << 8) | 0x06))
            .border_1()
            .border_color(rgba((theme.colors.ui.border << 8) | 0x20))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.62)
                    .pb(px(3.0))
                    .child("Commands"),
            )
            .child(
                div()
                    .text_xs()
                    .opacity(0.58)
                    .child(commands.join(" \u{00b7} ")),
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
        // ── Permission overlay intercept ─────────────────────────
        let pending_permission = self.thread.read(cx).pending_permission.clone();
        if let Some(ref request) = pending_permission {
            if self.handle_permission_key_down(event, request, cx) {
                cx.stop_propagation();
                return;
            }
            // Block composer typing behind the modal, but still allow
            // platform/control/alt shortcuts to propagate.
            if !event.keystroke.modifiers.platform
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt
            {
                cx.stop_propagation();
                return;
            }
        }

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
        let active_mode = thread.active_mode_id().map(String::from);
        let available_commands = thread.available_commands().to_vec();
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
            // ── Commands strip ───────────────────────────────
            .when(!available_commands.is_empty(), |d| {
                d.child(
                    div()
                        .w_full()
                        .px(px(8.0))
                        .pb(px(4.0))
                        .child(Self::render_commands_strip(&available_commands)),
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
                    .child(Self::render_status_badge(status, has_pending_permission))
                    .when_some(active_mode.clone(), |d, mode_id| {
                        d.child(Self::render_mode_badge(&mode_id))
                    }),
            )
            // ── Permission overlay ────────────────────────────
            .when_some(pending_permission, |d, request| {
                d.child(Self::render_permission_overlay(&request, self.permission_index, cx))
            })
    }
}
