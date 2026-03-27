use super::*;
use crate::theme::opacity::{
    OPACITY_HIDDEN, OPACITY_HOVER, OPACITY_MESSAGE_ASSISTANT_BACKGROUND,
    OPACITY_MESSAGE_USER_BACKGROUND, OPACITY_MUTED, OPACITY_SELECTED, OPACITY_STRONG,
    OPACITY_SUBTLE, OPACITY_TEXT_MUTED,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MessageCueTone {
    Accent,
    Muted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MessageBubbleCue {
    pub(super) background_tone: MessageCueTone,
    pub(super) background_opacity: f32,
    pub(super) italic: bool,
}

pub(super) fn message_bubble_cue(role: MessageRole) -> MessageBubbleCue {
    match role {
        MessageRole::User => MessageBubbleCue {
            background_tone: MessageCueTone::Muted,
            background_opacity: OPACITY_MESSAGE_USER_BACKGROUND,
            italic: false,
        },
        MessageRole::Assistant => MessageBubbleCue {
            background_tone: MessageCueTone::Muted,
            background_opacity: OPACITY_MESSAGE_ASSISTANT_BACKGROUND,
            italic: false,
        },
        MessageRole::System => MessageBubbleCue {
            background_tone: MessageCueTone::Muted,
            background_opacity: OPACITY_MESSAGE_ASSISTANT_BACKGROUND,
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
        let is_mini = self.window_mode.is_mini();
        let mini_style = mini_ai_chat_style();
        let cue = message_bubble_cue(message.role);
        let colors = theme::PromptColors::from_theme(&crate::theme::get_cached_theme());

        // Mini mode uses whisper-level opacity; full mode keeps the original values.
        let bubble_opacity = if is_mini {
            match message.role {
                MessageRole::User => mini_style.message_user_bg_opacity,
                MessageRole::Assistant | MessageRole::System => {
                    mini_style.message_assistant_bg_opacity
                }
            }
        } else {
            cue.background_opacity
        };
        let bubble_bg = match cue.background_tone {
            MessageCueTone::Accent => cx.theme().accent.opacity(bubble_opacity),
            MessageCueTone::Muted => cx.theme().muted.opacity(bubble_opacity),
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

        let role_label = if is_user {
            "You"
        } else if is_system {
            "System"
        } else {
            "Assistant"
        };

        // Pre-clone values for mini hover-reveal overlay (avoids borrow-after-move
        // when the full-mode role-row closure consumes the originals).
        let mini_full_timestamp = full_timestamp.clone();
        let mini_timestamp = timestamp.clone();
        let mini_msg_id = msg_id.clone();
        let mini_msg_id_for_click = msg_id_for_click.clone();
        let mini_content_for_copy = content_for_copy.clone();
        let mini_msg_id_for_edit = msg_id_for_edit.clone();
        let mini_content_for_edit = content_for_edit.clone();

        div()
            .id(SharedString::from(format!("msg-{}", msg_id)))
            .group("message")
            .flex()
            .flex_col()
            .w_full()
            .when(uses_continuation_spacing_after, |d| d.mb(S2))
            .when(!uses_continuation_spacing_after, |d| d.mb(MSG_GAP))
            // Role label row - hidden in mini mode and for continuation messages
            .when(!is_mini && !is_continuation, |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .mb(S1)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(S2)
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_STRONG),
                                        )
                                        .child(role_label),
                                )
                                .child({
                                    let tooltip_text = full_timestamp.clone();
                                    div()
                                        .id(SharedString::from(format!("ts-{}", msg_id)))
                                        .text_xs()
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_MUTED),
                                        )
                                        .tooltip(move |window, cx| {
                                            Tooltip::new(tooltip_text.clone()).build(window, cx)
                                        })
                                        .child(timestamp)
                                }),
                        )
                        // Hover-revealed action buttons
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(S1)
                                .opacity(OPACITY_HIDDEN)
                                .group_hover("message", |s| s.opacity(1.0))
                                // Edit button for user messages
                                .when(is_user, |el| {
                                    el.child(
                                        div()
                                            .id(SharedString::from(format!(
                                                "edit-{}",
                                                msg_id_for_edit
                                            )))
                                            .flex()
                                            .items_center()
                                            .px(S1)
                                            .py(S1)
                                            .rounded(R_SM)
                                            .cursor_pointer()
                                            .hover(|s| {
                                                s.bg(cx.theme().muted.opacity(OPACITY_SELECTED))
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
                                                    .external_path(
                                                        LocalIconName::Pencil.external_path(),
                                                    )
                                                    .size(ICON_XS)
                                                    .text_color(
                                                        cx.theme()
                                                            .muted_foreground
                                                            .opacity(OPACITY_MUTED),
                                                    ),
                                            ),
                                    )
                                })
                                // Copy button
                                .child(
                                    div()
                                        .id(SharedString::from(format!("copy-{}", msg_id)))
                                        .flex()
                                        .items_center()
                                        .gap(S1)
                                        .px(S1)
                                        .py(S1)
                                        .rounded(R_SM)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(cx.theme().muted.opacity(OPACITY_SELECTED)))
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            this.copy_message(
                                                msg_id_for_click.clone(),
                                                content_for_copy.clone(),
                                                cx,
                                            );
                                        }))
                                        .when(is_copied, |d| {
                                            d.child(
                                                svg()
                                                    .external_path(
                                                        LocalIconName::Check.external_path(),
                                                    )
                                                    .size(ICON_XS)
                                                    .text_color(cx.theme().success),
                                            )
                                        })
                                        .when(!is_copied, |d| {
                                            d.child(
                                                svg()
                                                    .external_path(
                                                        LocalIconName::Copy.external_path(),
                                                    )
                                                    .size(ICON_XS)
                                                    .text_color(
                                                        cx.theme()
                                                            .muted_foreground
                                                            .opacity(OPACITY_MUTED),
                                                    ),
                                            )
                                        }),
                                ),
                        ),
                )
            })
            .child(
                // Message content — mini: borderless floating bubble with whisper bg.
                // Full: accent/muted left-border for visual differentiation.
                div()
                    .relative()
                    .w_full()
                    .when(is_mini, |d| {
                        d.px(px(mini_style.message_padding_x))
                            .py(px(mini_style.message_padding_y))
                    })
                    .when(!is_mini, |d| {
                        d.px(MSG_PX)
                            .py(MSG_PY)
                            .border_l(px(3.0))
                            .when(is_user, |d| d.border_color(cx.theme().accent))
                            .when(!is_user, |d| {
                                d.border_color(cx.theme().muted.opacity(OPACITY_SUBTLE))
                            })
                    })
                    .bg(bubble_bg)
                    .when(cue.italic, |d| d.italic())
                    .when(has_images, |el| {
                        el.child(
                            div().flex().flex_wrap().gap(S2).mb(S2).children(
                                image_thumbnails
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, render_img)| {
                                        div()
                                            .id(SharedString::from(format!("msg-img-{}", i)))
                                            .rounded(R_MD)
                                            .overflow_hidden()
                                            .border_1()
                                            .border_color(
                                                cx.theme().border.opacity(OPACITY_SELECTED),
                                            )
                                            .child(
                                                img(move |_window: &mut Window, _cx: &mut App| {
                                                    Some(Ok(render_img.clone()))
                                                })
                                                .w(IMG_THUMBNAIL_SIZE)
                                                .h(IMG_THUMBNAIL_SIZE)
                                                .object_fit(gpui::ObjectFit::Cover),
                                            )
                                    }),
                            ),
                        )
                    })
                    // Mini hover-reveal: timestamp + edit/copy icons appear on hover
                    .when(is_mini, |el| {
                        let tooltip_text = mini_full_timestamp;
                        let ts = mini_timestamp;
                        let copy_id = mini_msg_id.clone();
                        let click_id = mini_msg_id_for_click;
                        let copy_content = mini_content_for_copy;
                        let edit_id = mini_msg_id_for_edit.clone();
                        let edit_content = mini_content_for_edit;
                        el.child(
                            div()
                                .absolute()
                                .top(S1)
                                .right(S2)
                                .flex()
                                .items_center()
                                .gap(S1)
                                .opacity(OPACITY_HIDDEN)
                                .group_hover("message", |s| s.opacity(MINI_MESSAGE_META_OPACITY))
                                .child(
                                    div()
                                        .id(SharedString::from(format!(
                                            "mini-ts-{}",
                                            copy_id.clone()
                                        )))
                                        .text_xs()
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_TEXT_MUTED),
                                        )
                                        .tooltip(move |window, cx| {
                                            Tooltip::new(tooltip_text.clone()).build(window, cx)
                                        })
                                        .child(ts),
                                )
                                .when(is_user, |el| {
                                    el.child(
                                        div()
                                            .id(SharedString::from(format!(
                                                "mini-edit-{}",
                                                edit_id.clone()
                                            )))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(MINI_BTN_SIZE)
                                            .rounded_full()
                                            .cursor_pointer()
                                            .hover(|s| {
                                                s.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                            })
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.start_editing_message(
                                                    edit_id.clone(),
                                                    edit_content.clone(),
                                                    window,
                                                    cx,
                                                );
                                            }))
                                            .child(
                                                svg()
                                                    .external_path(
                                                        LocalIconName::Pencil.external_path(),
                                                    )
                                                    .size(ICON_XS)
                                                    .text_color(
                                                        cx.theme()
                                                            .muted_foreground
                                                            .opacity(OPACITY_TEXT_MUTED),
                                                    ),
                                            ),
                                    )
                                })
                                .child(
                                    div()
                                        .id(SharedString::from(format!("mini-copy-{}", copy_id)))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(MINI_BTN_SIZE)
                                        .rounded_full()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(cx.theme().muted.opacity(OPACITY_HOVER)))
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            this.copy_message(
                                                click_id.clone(),
                                                copy_content.clone(),
                                                cx,
                                            );
                                        }))
                                        .child(
                                            svg()
                                                .external_path(
                                                    if is_copied {
                                                        LocalIconName::Check
                                                    } else {
                                                        LocalIconName::Copy
                                                    }
                                                    .external_path(),
                                                )
                                                .size(ICON_XS)
                                                .text_color(if is_copied {
                                                    cx.theme().success
                                                } else {
                                                    cx.theme()
                                                        .muted_foreground
                                                        .opacity(OPACITY_TEXT_MUTED)
                                                }),
                                        ),
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
                        let should_show_toggle =
                            message.content.len() > MSG_COLLAPSE_CHAR_THRESHOLD;
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
                                        .gap(S1)
                                        .mt(S2)
                                        .px(S2)
                                        .py(S1)
                                        .rounded(R_MD)
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(cx.theme().accent.opacity(OPACITY_STRONG))
                                        .hover(|s| {
                                            s.text_color(cx.theme().accent)
                                                .bg(cx.theme().accent.opacity(OPACITY_SUBTLE))
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
                                                .text_color(
                                                    cx.theme().accent.opacity(OPACITY_SELECTED),
                                                ),
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
    fn test_message_bubble_cue_assigns_background_to_each_role() {
        let user_cue = message_bubble_cue(MessageRole::User);
        let assistant_cue = message_bubble_cue(MessageRole::Assistant);
        let system_cue = message_bubble_cue(MessageRole::System);

        assert!(user_cue.background_opacity > 0.0);
        // Assistant messages have a subtle background tint
        assert!(assistant_cue.background_opacity > 0.0);
        assert!(system_cue.background_opacity > 0.0);
        // User bubble should be more prominent than assistant
        assert!(user_cue.background_opacity > assistant_cue.background_opacity);
        assert!(!assistant_cue.italic);
        assert!(system_cue.italic);
    }
}
