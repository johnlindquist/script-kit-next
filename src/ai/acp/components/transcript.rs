use gpui::{
    div, list, prelude::*, px, rgb, rgba, App, Context, Entity, FontWeight, ListAlignment,
    ListOffset, ListState, Render, SharedString, Window,
};
use std::collections::HashSet;
use std::time::Duration;

use super::super::thread::{AcpThread, AcpThreadMessage, AcpThreadMessageRole};
use crate::prompts::markdown::render_markdown_with_scope;
use crate::theme::{self, PromptColors};

pub enum AcpTranscriptEvent {
    ToggleMessage(u64),
}

impl gpui::EventEmitter<AcpTranscriptEvent> for AcpTranscript {}

type ToggleHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;

pub struct AcpTranscript {
    list_state: ListState,
    messages: Vec<AcpThreadMessage>,
    collapsed_ids: HashSet<u64>,
    show_activity_row: bool,
}

impl AcpTranscript {
    pub fn new(messages: Vec<AcpThreadMessage>, cx: &mut Context<Self>) -> Self {
        let list_state = ListState::new(0, ListAlignment::Bottom, px(200.0));
        list_state.set_follow_tail(true);

        Self {
            list_state,
            messages,
            collapsed_ids: HashSet::new(),
            show_activity_row: false,
        }
    }

    pub fn list_state(&self) -> ListState {
        self.list_state.clone()
    }

    pub fn set_messages(&mut self, messages: Vec<AcpThreadMessage>, cx: &mut Context<Self>) {
        let old_count = self.messages.len() + usize::from(self.show_activity_row);
        self.messages = messages;
        let new_count = self.messages.len() + usize::from(self.show_activity_row);

        if new_count != old_count {
            self.list_state.reset(new_count);
        }
        cx.notify();
    }

    pub fn set_show_activity_row(&mut self, show: bool, cx: &mut Context<Self>) {
        if self.show_activity_row != show {
            self.show_activity_row = show;
            let count = self.messages.len() + usize::from(self.show_activity_row);
            self.list_state.reset(count);
        }
        cx.notify();
    }

    pub fn toggle_collapsed(&mut self, id: u64, cx: &mut Context<Self>) {
        if self.collapsed_ids.contains(&id) {
            self.collapsed_ids.remove(&id);
        } else {
            self.collapsed_ids.insert(id);
        }
        cx.notify();
    }

    pub fn clear_collapsed_ids(&mut self, cx: &mut Context<Self>) {
        self.collapsed_ids.clear();
        cx.notify();
    }

    pub fn scroll_to_reveal_item(&self, index: usize) {
        self.list_state.scroll_to_reveal_item(index);
    }

    pub fn logical_scroll_top(&self) -> ListOffset {
        self.list_state.logical_scroll_top()
    }

    pub fn scroll_to(&self, offset: ListOffset) {
        self.list_state.scroll_to(offset);
    }

    pub fn scroll_to_end(&self) {
        self.list_state.scroll_to_end();
    }

    fn render_message_static(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        is_collapsed: bool,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        match msg.role {
            AcpThreadMessageRole::User => Self::render_user_message_static(msg, colors, &theme),
            AcpThreadMessageRole::Assistant => {
                Self::render_assistant_message_static(msg, colors, &theme)
            }
            AcpThreadMessageRole::Thought => {
                Self::render_collapsible_block_static(msg, colors, &theme, is_collapsed, false)
            }
            AcpThreadMessageRole::Tool => {
                Self::render_collapsible_block_static(msg, colors, &theme, is_collapsed, true)
            }
            AcpThreadMessageRole::Error => Self::render_error_message_static(msg, colors),
            AcpThreadMessageRole::System => Self::render_system_message_static(msg, colors, &theme),
        }
    }

    fn render_user_message_static(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        theme: &crate::theme::Theme,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba((theme.colors.text.primary << 8) | 0x06))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    fn render_assistant_message_static(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        _theme: &crate::theme::Theme,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(4.0))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    fn render_collapsible_block_static(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        theme: &crate::theme::Theme,
        is_collapsed: bool,
        is_tool: bool,
    ) -> gpui::AnyElement {
        let (label, status_hint) = if is_tool {
            let mut lines = msg.body.lines();
            let title = lines
                .next()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() < 80)
                .unwrap_or_else(|| "Tool".to_string());
            let status = lines
                .next()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() < 40);
            (title, status)
        } else {
            ("Thinking".to_string(), None)
        };

        let chevron = if is_collapsed {
            "\u{25B8}" // ▸
        } else {
            "\u{25BE}" // ▾
        };

        let line_count = msg.body.lines().count();
        let header_opacity = if is_tool { 0.55 } else { 0.50 };
        let left_border_color = if is_tool {
            rgba((theme.colors.accent.selected << 8) | 0x30)
        } else {
            rgba((theme.colors.text.primary << 8) | 0x18)
        };

        let scope_id = format!("acp-msg-{}", msg.id);

        let mut container = div()
            .w_full()
            .pl(px(12.0))
            .pr(px(12.0))
            .py(px(2.0))
            .border_l_2()
            .border_color(left_border_color);

        // Header row — clickable toggle uses element ID only (no cx.listener in static context).
        let header = div()
            .id(SharedString::from(format!("acp-toggle-{}", msg.id)))
            .flex()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .child(
                div()
                    .text_xs()
                    .opacity(header_opacity)
                    .child(chevron.to_string()),
            )
            .child(div().text_xs().opacity(header_opacity).child(label))
            .when_some(status_hint.clone(), |d, status| {
                d.child(div().text_xs().opacity(0.35).child(status))
            })
            .when(
                is_collapsed && line_count > 1 && status_hint.is_none(),
                |d| {
                    d.child(
                        div()
                            .text_xs()
                            .opacity(0.35)
                            .child(format!("{line_count} lines")),
                    )
                },
            );

        container = container.child(header);

        if !is_collapsed {
            let body = div()
                .pt(px(4.0))
                .max_h(px(200.0))
                .overflow_y_hidden()
                .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full());

            container = container.child(body);
        }

        container.into_any_element()
    }

    fn render_error_message_static(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba(0xEF444410))
            .border_l_2()
            .border_color(rgba(0xEF444480))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .pb(px(4.0))
                    .child(div().text_xs().opacity(0.75).child("\u{26A0}"))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .opacity(0.75)
                            .child("Error"),
                    ),
            )
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .child(
                div().pt(px(4.0)).text_xs().opacity(0.40).child(
                    "Try sending your message again or use \u{2318}N for a new conversation",
                ),
            )
            .into_any_element()
    }

    fn render_system_message_static(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        theme: &crate::theme::Theme,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(4.0))
            .opacity(0.60)
            .border_l_2()
            .border_color(rgba((theme.colors.ui.border << 8) | 0x30))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    fn render_assistant_activity_row_static() -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let accent = rgb(theme.colors.accent.selected);
        let pulse_duration = Duration::from_millis(1100);
        let dot = div().size(px(6.0)).rounded_full().bg(accent).opacity(0.7);

        div()
            .id("acp-assistant-activity-row")
            .w_full()
            .px(px(8.0))
            .pb(px(4.0))
            .child(
                div()
                    .w_full()
                    .px(px(12.0))
                    .py(px(8.0))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgba((theme.colors.text.muted << 8) | 0x99))
                            .child("Assistant"),
                    )
                    .child(
                        div()
                            .pt(px(4.0))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .text_sm()
                            .text_color(rgba((theme.colors.text.secondary << 8) | 0xCC))
                            .child(dot)
                            .child("Working..."),
                    ),
            )
            .into_any_element()
    }
}

impl Render for AcpTranscript {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let colors = PromptColors::from_theme(&theme);

        let total = self.messages.len() + usize::from(self.show_activity_row);
        self.list_state.reset(total);

        let messages_snapshot = self.messages.clone();
        let collapsed_ids = self.collapsed_ids.clone();
        let show_activity_row = self.show_activity_row;

        div().flex_1().min_h(px(0.)).child(list(
            self.list_state.clone(),
            move |ix, _window, _cx| {
                if show_activity_row && ix == messages_snapshot.len() {
                    return Self::render_assistant_activity_row_static();
                }

                let msg = &messages_snapshot[ix];
                let is_collapsible = matches!(
                    msg.role,
                    AcpThreadMessageRole::Thought | AcpThreadMessageRole::Tool
                );
                let is_collapsed = is_collapsible && !collapsed_ids.contains(&msg.id);

                let prev_was_user =
                    ix > 0 && matches!(messages_snapshot[ix - 1].role, AcpThreadMessageRole::User);
                let is_response_start =
                    prev_was_user && !matches!(msg.role, AcpThreadMessageRole::User);
                let is_new_turn = ix > 0
                    && matches!(msg.role, AcpThreadMessageRole::User)
                    && !matches!(messages_snapshot[ix - 1].role, AcpThreadMessageRole::User);

                div()
                    .w_full()
                    .px(px(8.0))
                    .pb(px(4.0))
                    .when(is_response_start, |d| d.mt(px(4.0)))
                    .when(is_new_turn, |d| {
                        d.mt(px(8.0))
                            .pt(px(8.0))
                            .border_t_1()
                            .border_color(rgba((theme.colors.ui.border << 8) | 0x18))
                    })
                    .child(Self::render_message_static(msg, &colors, is_collapsed))
                    .into_any()
            },
        ))
    }
}
