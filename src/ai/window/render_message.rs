use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MessageCueTone {
    Accent,
    Muted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MessageBubbleCue {
    pub(super) background_tone: MessageCueTone,
    pub(super) background_opacity: f32,
    pub(super) border_tone: MessageCueTone,
    pub(super) border_opacity: f32,
    pub(super) italic: bool,
}

pub(super) fn message_bubble_cue(role: MessageRole) -> MessageBubbleCue {
    match role {
        MessageRole::User => MessageBubbleCue {
            background_tone: MessageCueTone::Accent,
            background_opacity: OP_USER_MSG_BG,
            border_tone: MessageCueTone::Accent,
            border_opacity: OP_MSG_BORDER,
            italic: false,
        },
        MessageRole::Assistant => MessageBubbleCue {
            background_tone: MessageCueTone::Muted,
            background_opacity: OP_ASSISTANT_MSG_BG,
            border_tone: MessageCueTone::Muted,
            border_opacity: OP_MUTED,
            italic: false,
        },
        MessageRole::System => MessageBubbleCue {
            background_tone: MessageCueTone::Muted,
            background_opacity: OP_ASSISTANT_MSG_BG,
            border_tone: MessageCueTone::Muted,
            border_opacity: OP_MEDIUM,
            italic: true,
        },
    }
}

impl AiApp {
    pub(super) fn render_message(
        &self,
        message: &Message,
        is_continuation: bool,
        uses_continuation_spacing_after: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_user = message.role == MessageRole::User;
        let is_system = message.role == MessageRole::System;
        let cue = message_bubble_cue(message.role);
        let colors = theme::PromptColors::from_theme(&crate::theme::get_cached_theme());

        let bubble_bg = match cue.background_tone {
            MessageCueTone::Accent => cx.theme().accent.opacity(cue.background_opacity),
            MessageCueTone::Muted => cx.theme().muted.opacity(cue.background_opacity),
        };
        let bubble_border_color = match cue.border_tone {
            MessageCueTone::Accent => cx.theme().accent.opacity(cue.border_opacity),
            MessageCueTone::Muted => cx.theme().muted_foreground.opacity(cue.border_opacity),
        };

        // Collect cached thumbnails for this message's images
        let image_thumbnails: Vec<std::sync::Arc<RenderImage>> = message
            .images
            .iter()
            .filter_map(|attachment| self.get_cached_image(&attachment.data))
            .collect();
        let has_images = !image_thumbnails.is_empty();

        let content_for_copy = message.content.clone();
        let content_for_edit = message.content.clone();
        let msg_id = message.id.clone();
        let msg_id_for_edit = msg_id.clone();
        let msg_id_for_click = msg_id.clone();
        let is_copied = self.is_message_copied(&msg_id);

        // Relative timestamp + full datetime for tooltip
        let timestamp: SharedString = {
            let now = Utc::now();
            let diff = now - message.created_at;
            if diff.num_minutes() < 1 {
                "just now".into()
            } else if diff.num_minutes() < 60 {
                format!("{}m ago", diff.num_minutes()).into()
            } else if diff.num_hours() < 24 {
                format!("{}h ago", diff.num_hours()).into()
            } else {
                message.created_at.format("%b %d").to_string().into()
            }
        };
        let full_timestamp: SharedString = message
            .created_at
            .format("%B %-d, %Y at %-I:%M %p")
            .to_string()
            .into();

        let role_icon = if is_user {
            LocalIconName::Terminal
        } else if is_system {
            LocalIconName::Settings
        } else {
            LocalIconName::MessageCircle
        };
        let role_label = if is_user {
            "You"
        } else if is_system {
            "System"
        } else {
            "Assistant"
        };

        div()
            .id(SharedString::from(format!("msg-{}", msg_id)))
            .group("message")
            .flex()
            .flex_col()
            .w_full()
            .when(uses_continuation_spacing_after, |d| {
                d.mb(MSG_GAP_CONTINUATION)
            })
            .when(!uses_continuation_spacing_after, |d| d.mb(MSG_GAP))
            // Role label row - hidden for continuation messages from same sender
            .when(!is_continuation, |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .mb(SP_3)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(SP_3)
                                .child(
                                    svg()
                                        .external_path(role_icon.external_path())
                                        .size(ICON_SM)
                                        .text_color(if is_user {
                                            cx.theme().accent
                                        } else {
                                            cx.theme().muted_foreground.opacity(OP_STRONG)
                                        }),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(if is_user {
                                            cx.theme().foreground
                                        } else {
                                            cx.theme().muted_foreground.opacity(OP_NEAR_FULL)
                                        })
                                        .child(role_label),
                                )
                                .child(
                                    div()
                                        .size(DOT_SIZE)
                                        .rounded_full()
                                        .bg(cx.theme().muted_foreground.opacity(OP_MUTED)),
                                )
                                .child({
                                    let tooltip_text = full_timestamp.clone();
                                    div()
                                        .id(SharedString::from(format!("ts-{}", msg_id)))
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(OP_MEDIUM))
                                        .tooltip(move |window, cx| {
                                            Tooltip::new(tooltip_text.clone()).build(window, cx)
                                        })
                                        .child(timestamp)
                                }),
                        )
                        // Edit button for user messages (hover-revealed)
                        .when(is_user, |el| {
                            el.child(
                                div()
                                    .id(SharedString::from(format!("edit-{}", msg_id_for_edit)))
                                    .flex()
                                    .items_center()
                                    .px(SP_3)
                                    .py(SP_1)
                                    .rounded(RADIUS_SM)
                                    .cursor_pointer()
                                    .opacity(0.0)
                                    .group_hover("message", |s| s.opacity(0.6))
                                    .hover(|s| {
                                        s.bg(cx.theme().muted.opacity(OP_MEDIUM)).opacity(1.0)
                                    })
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.start_editing_message(
                                            msg_id_for_edit.clone(),
                                            content_for_edit.clone(),
                                            window,
                                            cx,
                                        );
                                    }))
                                    .child(
                                        svg()
                                            .external_path(LocalIconName::Pencil.external_path())
                                            .size(ICON_XS)
                                            .text_color(cx.theme().muted_foreground.opacity(0.6)),
                                    ),
                            )
                        })
                        // Copy button - shows checkmark when recently copied, hidden until hover
                        .child(
                            div()
                                .id(SharedString::from(format!("copy-{}", msg_id)))
                                .flex()
                                .items_center()
                                .gap(SP_2)
                                .px(SP_3)
                                .py(SP_1)
                                .rounded(RADIUS_SM)
                                .cursor_pointer()
                                .when(!is_copied, |d| {
                                    d.opacity(0.0).group_hover("message", |s| s.opacity(0.6))
                                })
                                .hover(|s| s.bg(cx.theme().muted.opacity(OP_MEDIUM)).opacity(1.0))
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    this.copy_message(
                                        msg_id_for_click.clone(),
                                        content_for_copy.clone(),
                                        cx,
                                    );
                                }))
                                .when(is_copied, |d| {
                                    d.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(SP_1)
                                            .child(
                                                svg()
                                                    .external_path(
                                                        LocalIconName::Check.external_path(),
                                                    )
                                                    .size(ICON_XS)
                                                    .text_color(cx.theme().success),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().success)
                                                    .child("Copied"),
                                            ),
                                    )
                                })
                                .when(!is_copied, |d| {
                                    d.child(
                                        svg()
                                            .external_path(LocalIconName::Copy.external_path())
                                            .size(ICON_XS)
                                            .text_color(
                                                cx.theme().muted_foreground.opacity(OP_MEDIUM),
                                            ),
                                    )
                                }),
                        ),
                )
            })
            .child(
                // Message content - differentiated backgrounds
                div()
                    .w_full()
                    .px(MSG_PX)
                    .py(MSG_PY)
                    .rounded(MSG_RADIUS)
                    .bg(bubble_bg)
                    .border_l_2()
                    .border_color(bubble_border_color)
                    .when(cue.italic, |d| d.italic())
                    .when(has_images, |el| {
                        el.child(
                            div().flex().flex_wrap().gap_2().mb_2().children(
                                image_thumbnails
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, render_img)| {
                                        div()
                                            .id(SharedString::from(format!("msg-img-{}", i)))
                                            .rounded(RADIUS_MD)
                                            .overflow_hidden()
                                            .border_1()
                                            .border_color(cx.theme().border.opacity(OP_MEDIUM))
                                            .child(
                                                img(move |_window: &mut Window, _cx: &mut App| {
                                                    Some(Ok(render_img.clone()))
                                                })
                                                .w(px(120.))
                                                .h(px(120.))
                                                .object_fit(gpui::ObjectFit::Cover),
                                            )
                                    }),
                            ),
                        )
                    })
                    .child({
                        let is_collapsed =
                            self.is_message_collapsed(&msg_id, message.content.len());
                        let display_content = if is_collapsed {
                            // Truncate to ~300 chars at a word boundary
                            let truncated: String = message.content.chars().take(300).collect();
                            let truncated = match truncated.rfind(' ') {
                                Some(pos) if pos > 200 => truncated[..pos].to_string(),
                                _ => truncated,
                            };
                            format!("{}...", truncated)
                        } else {
                            Self::message_body_content(&message.content)
                        };
                        let should_show_toggle = message.content.len() > 800;
                        let toggle_msg_id = msg_id.clone();
                        let total_words = message.content.split_whitespace().count();
                        let hidden_words = if is_collapsed {
                            let shown: String = message.content.chars().take(300).collect();
                            let shown_words = shown.split_whitespace().count();
                            total_words.saturating_sub(shown_words)
                        } else {
                            0
                        };
                        div()
                            .w_full()
                            .min_w_0()
                            .overflow_x_hidden()
                            .child(render_markdown(&display_content, &colors))
                            .when(should_show_toggle, |el| {
                                let toggle_label: SharedString = if is_collapsed {
                                    if hidden_words > 0 {
                                        format!(
                                            "Show more ({} more {})",
                                            hidden_words,
                                            if hidden_words == 1 { "word" } else { "words" }
                                        )
                                        .into()
                                    } else {
                                        "Show more".into()
                                    }
                                } else {
                                    format!("Show less ({} words)", total_words).into()
                                };
                                el.child(
                                    div()
                                        .id(SharedString::from(format!(
                                            "collapse-toggle-{}",
                                            toggle_msg_id
                                        )))
                                        .flex()
                                        .items_center()
                                        .gap(SP_2)
                                        .mt(SP_3)
                                        .px(SP_3)
                                        .py(SP_2)
                                        .rounded(RADIUS_MD)
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(cx.theme().accent.opacity(OP_STRONG))
                                        .hover(|s| {
                                            s.text_color(cx.theme().accent)
                                                .bg(cx.theme().accent.opacity(OP_SUBTLE))
                                        })
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.toggle_message_collapse(toggle_msg_id.clone(), cx);
                                        }))
                                        .child(
                                            svg()
                                                .external_path(
                                                    if is_collapsed {
                                                        LocalIconName::ChevronDown
                                                    } else {
                                                        LocalIconName::ArrowUp
                                                    }
                                                    .external_path(),
                                                )
                                                .size(ICON_XS)
                                                .text_color(cx.theme().accent.opacity(OP_MEDIUM)),
                                        )
                                        .child(toggle_label),
                                )
                            })
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_bubble_cue_assigns_persistent_border_to_each_role() {
        let user_cue = message_bubble_cue(MessageRole::User);
        let assistant_cue = message_bubble_cue(MessageRole::Assistant);
        let system_cue = message_bubble_cue(MessageRole::System);

        assert!(user_cue.border_opacity > 0.0);
        assert!(assistant_cue.border_opacity > 0.0);
        assert!(system_cue.border_opacity > 0.0);
        assert!(!assistant_cue.italic);
        assert!(system_cue.italic);
    }
}
