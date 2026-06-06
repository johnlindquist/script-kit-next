use gpui::{
    div, list, prelude::*, px, rems, rgb, rgba, App, Context, Entity, FontWeight, ListAlignment,
    ListOffset, ListSizingBehavior, ListState, Render, Rgba, SharedString, StyleRefinement, Window,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::text::{TextView, TextViewState, TextViewStyle};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::super::thread::{AcpThread, AcpThreadMessage, AcpThreadMessageRole};
use super::super::ui_variant::{AcpChatUiVariant, AcpTranscriptPresentation};
use crate::dev_style_tool::agent_chat_catalog::AgentChatStyleDef;
use crate::theme::{self, PromptColors};

pub enum AcpTranscriptEvent {
    ToggleMessage(u64),
}

impl gpui::EventEmitter<AcpTranscriptEvent> for AcpTranscript {}

pub struct AcpTranscript {
    list_state: ListState,
    messages: Vec<AcpThreadMessage>,
    collapsed_ids: HashSet<u64>,
    message_views: HashMap<u64, gpui::Entity<TextViewState>>,
    message_texts: HashMap<u64, String>,
    ui_variant: AcpChatUiVariant,
}

impl AcpTranscript {
    pub fn new(messages: Vec<AcpThreadMessage>, _cx: &mut Context<Self>) -> Self {
        let total = messages.len();
        let list_state = ListState::new(total, ListAlignment::Bottom, px(200.0));
        list_state.set_follow_tail(true);

        Self {
            list_state,
            messages,
            collapsed_ids: HashSet::new(),
            message_views: HashMap::new(),
            message_texts: HashMap::new(),
            ui_variant: AcpChatUiVariant::Standard,
        }
    }

    pub fn with_ui_variant(mut self, ui_variant: AcpChatUiVariant) -> Self {
        self.ui_variant = ui_variant;
        self
    }

    pub fn set_ui_variant(&mut self, ui_variant: AcpChatUiVariant, cx: &mut Context<Self>) {
        if self.ui_variant != ui_variant {
            self.ui_variant = ui_variant;
            cx.notify();
        }
    }

    pub fn list_state(&self) -> ListState {
        self.list_state.clone()
    }

    fn messages_match_current(&self, messages: &[AcpThreadMessage]) -> bool {
        self.messages.len() == messages.len()
            && self
                .messages
                .iter()
                .zip(messages.iter())
                .all(|(current, incoming)| {
                    current.id == incoming.id
                        && current.role == incoming.role
                        && current.body == incoming.body
                        && current.tool_call_id == incoming.tool_call_id
                })
    }

    pub fn set_messages(&mut self, messages: Vec<AcpThreadMessage>, cx: &mut Context<Self>) {
        if self.messages_match_current(&messages) {
            return;
        }

        let old_count = self.messages.len();
        self.messages = messages;
        let new_count = self.messages.len();

        if new_count != old_count {
            self.list_state.reset(new_count);
        }

        // Clean up message inputs for deleted messages
        let active_ids: HashSet<u64> = self.messages.iter().map(|m| m.id).collect();
        self.message_views.retain(|id, _| active_ids.contains(id));
        self.message_texts.retain(|id, _| active_ids.contains(id));

        cx.notify();
    }

    pub fn set_show_activity_row(&mut self, _show: bool, cx: &mut Context<Self>) {
        // Streaming/loading activity is surfaced by the footer status, not as a
        // synthetic transcript row. Keep this method as a narrow compatibility
        // shim for the thread observer.
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

    pub fn expand_ids(&mut self, ids: Vec<u64>, cx: &mut Context<Self>) {
        self.collapsed_ids.extend(ids);
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

    fn transcript_text_style(
        _theme: &crate::theme::Theme,
        colors: &PromptColors,
        style_def: &AgentChatStyleDef,
    ) -> TextViewStyle {
        let code_bg =
            rgba((colors.code_bg << 8) | style_def.markdown.code_block_bg_alpha.round() as u32);
        let code_border = rgba(
            (colors.quote_border << 8) | style_def.markdown.code_block_border_alpha.round() as u32,
        );
        let heading_1_font_size = style_def.markdown.heading_1_font_size;
        let heading_2_font_size = style_def.markdown.heading_2_font_size;
        let heading_3_font_size = style_def.markdown.heading_3_font_size;
        let body_font_size = style_def.markdown.body_font_size;
        let mut style = TextViewStyle::default()
            .paragraph_gap(rems(style_def.markdown.paragraph_gap))
            .heading_font_size(move |level, _base_size| match level {
                1 => px(heading_1_font_size),
                2 => px(heading_2_font_size),
                3 => px(heading_3_font_size),
                _ => px(body_font_size),
            })
            .code_block(
                StyleRefinement::default()
                    .bg(code_bg)
                    .border_1()
                    .border_color(code_border)
                    .rounded(px(style_def.markdown.code_block_radius))
                    .px(px(style_def.markdown.code_block_padding_x))
                    .py(px(style_def.markdown.code_block_padding_y))
                    .text_size(px(style_def.markdown.code_block_font_size)),
            );

        style.highlight_theme = Arc::new(
            crate::theme::gpui_integration::build_markdown_highlight_theme(
                _theme,
                _theme.is_dark_mode(),
            ),
        );
        style.is_dark = _theme.is_dark_mode();
        style
    }

    fn selectable_markdown_view(
        text_view_state: &gpui::Entity<TextViewState>,
        theme: &crate::theme::Theme,
        colors: &PromptColors,
        text_color: Rgba,
        style_def: &AgentChatStyleDef,
    ) -> TextView {
        TextView::new(text_view_state)
            .style(Self::transcript_text_style(theme, colors, style_def))
            .selectable(true)
            .w_full()
            .text_size(px(style_def.markdown.body_font_size))
            .text_color(text_color)
    }

    fn render_message(
        ui_variant: AcpChatUiVariant,
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        is_collapsed: bool,
        text_view_state: &gpui::Entity<TextViewState>,
        style_def: &AgentChatStyleDef,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let presentation = ui_variant.config().transcript;

        match msg.role {
            AcpThreadMessageRole::User => Self::render_user_message(
                msg,
                colors,
                &theme,
                text_view_state,
                presentation,
                style_def,
            ),
            AcpThreadMessageRole::Assistant => Self::render_assistant_message(
                msg,
                colors,
                &theme,
                text_view_state,
                presentation,
                style_def,
            ),
            AcpThreadMessageRole::Thought => Self::render_collapsible_block(
                msg,
                colors,
                &theme,
                is_collapsed,
                false,
                text_view_state,
                style_def,
            ),
            AcpThreadMessageRole::Tool => Self::render_collapsible_block(
                msg,
                colors,
                &theme,
                is_collapsed,
                true,
                text_view_state,
                style_def,
            ),
            AcpThreadMessageRole::Error => {
                Self::render_error_message(msg, colors, text_view_state, style_def)
            }
            AcpThreadMessageRole::System => {
                Self::render_system_message(msg, colors, &theme, text_view_state, style_def)
            }
        }
    }

    fn render_user_message(
        _msg: &AcpThreadMessage,
        _colors: &PromptColors,
        theme: &crate::theme::Theme,
        text_view_state: &gpui::Entity<TextViewState>,
        presentation: AcpTranscriptPresentation,
        style_def: &AgentChatStyleDef,
    ) -> gpui::AnyElement {
        let user_style = style_def.user_message;
        let bubble = div()
            .w_full()
            .px(px(user_style.padding_x))
            .py(
                if matches!(presentation, AcpTranscriptPresentation::DenseLog) {
                    px(user_style.dense_padding_y)
                } else {
                    px(user_style.padding_y)
                },
            )
            .rounded(px(user_style.radius))
            .bg(rgba(
                (theme.colors.text.primary << 8) | user_style.bg_alpha.round() as u32,
            ))
            .when(
                matches!(presentation, AcpTranscriptPresentation::UserBold),
                |d| d.font_weight(FontWeight::BOLD),
            )
            .child(Self::selectable_markdown_view(
                text_view_state,
                theme,
                _colors,
                rgb(_colors.text_primary),
                style_def,
            ));

        if matches!(presentation, AcpTranscriptPresentation::RoleSplit) {
            div()
                .w_full()
                .flex()
                .justify_end()
                .child(div().max_w(px(user_style.max_width)).child(bubble))
                .into_any_element()
        } else {
            bubble.into_any_element()
        }
    }

    fn render_assistant_message(
        _msg: &AcpThreadMessage,
        _colors: &PromptColors,
        _theme: &crate::theme::Theme,
        text_view_state: &gpui::Entity<TextViewState>,
        presentation: AcpTranscriptPresentation,
        style_def: &AgentChatStyleDef,
    ) -> gpui::AnyElement {
        let assistant_style = style_def.assistant_message;
        let message = div()
            .w_full()
            .px(px(assistant_style.padding_x))
            .py(
                if matches!(presentation, AcpTranscriptPresentation::DenseLog) {
                    px(assistant_style.dense_padding_y)
                } else {
                    px(assistant_style.padding_y)
                },
            )
            .rounded(px(assistant_style.radius))
            .when(assistant_style.bg_alpha > 0.0, |d| {
                d.bg(rgba(
                    (_theme.colors.text.primary << 8) | assistant_style.bg_alpha.round() as u32,
                ))
            })
            .child(Self::selectable_markdown_view(
                text_view_state,
                _theme,
                _colors,
                rgb(_colors.text_primary),
                style_def,
            ));

        if matches!(presentation, AcpTranscriptPresentation::RoleSplit) {
            div()
                .w_full()
                .flex()
                .justify_start()
                .child(div().max_w(px(assistant_style.max_width)).child(message))
                .into_any_element()
        } else {
            message.into_any_element()
        }
    }

    fn render_collapsible_block(
        msg: &AcpThreadMessage,
        _colors: &PromptColors,
        theme: &crate::theme::Theme,
        is_collapsed: bool,
        is_tool: bool,
        text_view_state: &gpui::Entity<TextViewState>,
        style_def: &AgentChatStyleDef,
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
        let block_style = style_def.collapsible;
        let header_opacity = if is_tool {
            block_style.tool_header_opacity
        } else {
            block_style.thought_header_opacity
        };
        let left_border_color = if is_tool {
            rgba((theme.colors.accent.selected << 8) | block_style.tool_border_alpha.round() as u32)
        } else {
            rgba((theme.colors.text.primary << 8) | block_style.thought_border_alpha.round() as u32)
        };

        let mut container = div()
            .w_full()
            .pl(px(block_style.padding_x))
            .pr(px(block_style.padding_x))
            .py(px(block_style.padding_y))
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
                    .text_sm()
                    .opacity(header_opacity)
                    .when(is_tool, |d| d.text_color(rgb(_colors.accent_color)))
                    .child(chevron.to_string()),
            )
            .child(
                div()
                    .text_sm()
                    .opacity(header_opacity)
                    .when(is_tool, |d| d.text_color(rgb(_colors.accent_color)))
                    .child(label),
            )
            .when_some(status_hint.clone(), |d, status| {
                d.child(
                    div()
                        .text_sm()
                        .opacity(block_style.status_opacity)
                        .when(is_tool, |d| d.text_color(rgb(_colors.accent_color)))
                        .child(status),
                )
            })
            .when(
                is_collapsed && line_count > 1 && status_hint.is_none(),
                |d| {
                    d.child(
                        div()
                            .text_sm()
                            .opacity(block_style.status_opacity)
                            .when(is_tool, |d| d.text_color(rgb(_colors.accent_color)))
                            .child(format!("{line_count} lines")),
                    )
                },
            );

        container = container.child(header);

        if !is_collapsed {
            let body_color = if is_tool {
                rgb(_colors.accent_color)
            } else {
                rgb(_colors.text_primary)
            };

            let body = div()
                .pt(px(block_style.body_padding_top))
                .max_h(px(block_style.max_body_height))
                .overflow_y_hidden()
                .child(Self::selectable_markdown_view(
                    text_view_state,
                    theme,
                    _colors,
                    body_color,
                    style_def,
                ));

            container = container.child(body);
        }

        container.into_any_element()
    }

    fn render_error_message(
        _msg: &AcpThreadMessage,
        _colors: &PromptColors,
        text_view_state: &gpui::Entity<TextViewState>,
        style_def: &AgentChatStyleDef,
    ) -> gpui::AnyElement {
        let error_style = style_def.error;
        div()
            .w_full()
            .px(px(error_style.padding_x))
            .py(px(error_style.padding_y))
            .rounded(px(error_style.radius))
            .bg(rgba(0xEF444400 | error_style.bg_alpha.round() as u32))
            .border_l_2()
            .border_color(rgba(0xEF444400 | error_style.border_alpha.round() as u32))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .pb(px(4.0))
                    .child(
                        div()
                            .text_sm()
                            .opacity(error_style.label_opacity)
                            .child("\u{26A0}"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .opacity(error_style.label_opacity)
                            .child("Error"),
                    ),
            )
            .child(Self::selectable_markdown_view(
                text_view_state,
                &theme::get_cached_theme(),
                _colors,
                rgb(_colors.text_primary),
                style_def,
            ))
            .child(
                div()
                    .pt(px(4.0))
                    .text_sm()
                    .opacity(error_style.hint_opacity)
                    .child(
                        "Try sending your message again or use \u{2318}N for a new conversation",
                    ),
            )
            .into_any_element()
    }

    fn render_system_message(
        _msg: &AcpThreadMessage,
        _colors: &PromptColors,
        theme: &crate::theme::Theme,
        text_view_state: &gpui::Entity<TextViewState>,
        style_def: &AgentChatStyleDef,
    ) -> gpui::AnyElement {
        let system_style = style_def.system;
        div()
            .w_full()
            .px(px(system_style.padding_x))
            .py(px(system_style.padding_y))
            .opacity(system_style.opacity)
            .border_l_2()
            .border_color(rgba(
                (theme.colors.ui.border << 8) | system_style.border_alpha.round() as u32,
            ))
            .child(Self::selectable_markdown_view(
                text_view_state,
                theme,
                _colors,
                rgb(_colors.text_primary),
                style_def,
            ))
            .into_any_element()
    }
}

impl Render for AcpTranscript {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let colors = PromptColors::from_theme(&theme);
        let style_def = crate::dev_style_tool::runtime_overrides::effective_agent_chat_style();

        let focused_text_preview = matches!(
            self.ui_variant.config().transcript,
            AcpTranscriptPresentation::FocusedTextPreview
        );
        let messages_snapshot = self.messages.clone();
        let collapsed_ids = self.collapsed_ids.clone();
        let visible_indices: Vec<usize> = if focused_text_preview {
            messages_snapshot
                .iter()
                .enumerate()
                .filter_map(|(ix, msg)| {
                    (matches!(msg.role, AcpThreadMessageRole::Assistant)
                        && !msg.body.trim().is_empty())
                    .then_some(ix)
                })
                .collect()
        } else {
            (0..messages_snapshot.len()).collect()
        };

        // Reconcile/sync TextViewState entities for each message
        for msg in &messages_snapshot {
            let text_view_state = self
                .message_views
                .entry(msg.id)
                .or_insert_with(|| cx.new(|cx| TextViewState::markdown(&msg.body, cx)));

            // Update text buffer if it's changing (e.g. streaming assistant output)
            let last_text = self.message_texts.get(&msg.id).cloned().unwrap_or_default();
            if last_text != msg.body.as_ref() {
                text_view_state.update(cx, |state, cx| {
                    state.set_text(&msg.body, cx);
                });
                self.message_texts.insert(msg.id, msg.body.to_string());
            }
        }

        let message_views_snapshot = self.message_views.clone();
        let ui_variant = self.ui_variant;
        let transcript_content = if focused_text_preview {
            let mut preview = div().size_full().flex().flex_col().overflow_hidden();

            for message_ix in visible_indices.iter().copied() {
                let Some(msg) = messages_snapshot.get(message_ix) else {
                    continue;
                };
                let Some(text_view_state) = message_views_snapshot.get(&msg.id) else {
                    continue;
                };

                let is_collapsible = matches!(
                    msg.role,
                    AcpThreadMessageRole::Thought | AcpThreadMessageRole::Tool
                );
                let is_collapsed = is_collapsible && !collapsed_ids.contains(&msg.id);
                preview = preview.child(
                    div()
                        .w_full()
                        .px(px(style_def.transcript.focused_preview_padding_x))
                        .pb(px(style_def.transcript.focused_preview_padding_bottom))
                        .child(Self::render_message(
                            ui_variant,
                            msg,
                            &colors,
                            is_collapsed,
                            text_view_state,
                            &style_def,
                        )),
                );
            }

            preview.into_any_element()
        } else {
            list(self.list_state.clone(), move |ix, _window, _cx| {
                let Some(&message_ix) = visible_indices.get(ix) else {
                    return div().into_any();
                };
                let msg = &messages_snapshot[message_ix];

                let is_collapsible = matches!(
                    msg.role,
                    AcpThreadMessageRole::Thought | AcpThreadMessageRole::Tool
                );
                let is_collapsed = is_collapsible && !collapsed_ids.contains(&msg.id);

                let prev_was_user = message_ix > 0
                    && matches!(
                        messages_snapshot[message_ix - 1].role,
                        AcpThreadMessageRole::User
                    );
                let is_response_start =
                    prev_was_user && !matches!(msg.role, AcpThreadMessageRole::User);
                let is_new_turn = message_ix > 0
                    && matches!(msg.role, AcpThreadMessageRole::User)
                    && !matches!(
                        messages_snapshot[message_ix - 1].role,
                        AcpThreadMessageRole::User
                    );

                let Some(text_view_state) = message_views_snapshot.get(&msg.id) else {
                    return div().into_any();
                };

                div()
                    .w_full()
                    .px(px(style_def.transcript.row_padding_x))
                    .pb(px(style_def.transcript.row_padding_bottom))
                    .when(is_response_start, |d| {
                        d.mt(px(style_def.transcript.response_start_margin_top))
                    })
                    .when(is_new_turn, |d| {
                        d.mt(px(style_def.transcript.turn_margin_top))
                            .pt(px(style_def.transcript.turn_padding_top))
                            .border_t_1()
                            .border_color(rgba(
                                (theme.colors.ui.border << 8)
                                    | style_def.transcript.turn_divider_alpha.round() as u32,
                            ))
                    })
                    .when(
                        matches!(
                            ui_variant.config().transcript,
                            AcpTranscriptPresentation::DenseLog
                        ),
                        |d| d.pb(px(style_def.transcript.dense_row_padding_bottom)),
                    )
                    .child(Self::render_message(
                        ui_variant,
                        msg,
                        &colors,
                        is_collapsed,
                        text_view_state,
                        &style_def,
                    ))
                    .into_any()
            })
            .size_full()
            .with_sizing_behavior(ListSizingBehavior::Auto)
            .into_any_element()
        };

        div()
            .relative()
            .flex_1()
            .min_h(px(0.))
            .overflow_hidden()
            .child(transcript_content)
            .vertical_scrollbar(&self.list_state)
    }
}
