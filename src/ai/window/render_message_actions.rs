use super::*;
use crate::theme::opacity::{
    OPACITY_BORDER, OPACITY_HOVER, OPACITY_ICON_MUTED, OPACITY_SELECTED, OPACITY_STRONG,
    OPACITY_TEXT_MUTED,
};

fn ai_message_actions_can_copy_chat_transcript(message_count: usize, is_streaming: bool) -> bool {
    message_count > 0 && !is_streaming
}

impl AiApp {
    pub(super) fn render_message_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted_fg = cx.theme().muted_foreground;

        // Show "Generated in Xs · ~N words" for 8 seconds after streaming completes
        let completion_label: Option<String> =
            self.last_streaming_completed_at.and_then(|completed_at| {
                if completed_at.elapsed().as_secs() < 8 {
                    self.last_streaming_duration.map(|dur| {
                        let time_label = {
                            let secs = dur.as_secs();
                            if secs < 1 {
                                format!("{}ms", dur.as_millis())
                            } else {
                                format!("{}s", secs)
                            }
                        };
                        // Count words in the last assistant message
                        let word_count = self
                            .current_messages
                            .last()
                            .filter(|m| m.role == MessageRole::Assistant)
                            .map(|m| m.content.split_whitespace().count())
                            .unwrap_or(0);
                        if word_count > 0 {
                            let secs = dur.as_secs_f64();
                            if secs > 0.5 {
                                format!(
                                    "{} \u{00b7} ~{} words \u{00b7} {:.0} words/s",
                                    time_label,
                                    word_count,
                                    word_count as f64 / secs
                                )
                            } else {
                                format!("{} \u{00b7} ~{} words", time_label, word_count)
                            }
                        } else {
                            time_label
                        }
                    })
                } else {
                    None
                }
            });

        div()
            .id("message-actions")
            .flex()
            .items_center()
            .gap(S2)
            .pl(MSG_PX)
            .mt(S1)
            .mb(S2)
            .child(
                div()
                    .id("regenerate-btn")
                    .flex()
                    .items_center()
                    .gap(S1)
                    .px(S2)
                    .py(S1)
                    .rounded(R_SM)
                    .cursor_pointer()
                    .text_xs()
                    .text_color(muted_fg.opacity(OPACITY_TEXT_MUTED))
                    .hover(|s| {
                        s.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                            .text_color(muted_fg)
                    })
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.regenerate_response(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(LocalIconName::Refresh.external_path())
                            .size(ICON_XS)
                            .text_color(muted_fg.opacity(OPACITY_ICON_MUTED)),
                    )
                    .child("Regenerate"),
            )
            // Copy response button
            .child({
                let last_assistant_id = self
                    .current_messages
                    .last()
                    .filter(|m| m.role == MessageRole::Assistant)
                    .map(|m| m.id.clone());
                let is_copied = last_assistant_id
                    .as_ref()
                    .map(|id| self.is_message_copied(id))
                    .unwrap_or(false);
                let (icon, label, icon_color) = if is_copied {
                    (
                        LocalIconName::Check,
                        "Copied!",
                        cx.theme().success.opacity(OPACITY_STRONG),
                    )
                } else {
                    (
                        LocalIconName::Copy,
                        "Copy",
                        muted_fg.opacity(OPACITY_SELECTED),
                    )
                };
                div()
                    .id("copy-response-btn")
                    .flex()
                    .items_center()
                    .gap(S1)
                    .px(S2)
                    .py(S1)
                    .rounded(R_SM)
                    .cursor_pointer()
                    .text_xs()
                    .text_color(if is_copied {
                        cx.theme().success.opacity(OPACITY_STRONG)
                    } else {
                        muted_fg.opacity(OPACITY_TEXT_MUTED)
                    })
                    .hover(|s| {
                        s.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                            .text_color(muted_fg)
                    })
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.copy_last_assistant_response(cx);
                    }))
                    .child(
                        svg()
                            .external_path(icon.external_path())
                            .size(ICON_XS)
                            .text_color(icon_color),
                    )
                    .child(label)
            })
            // Copy full chat transcript button
            .child({
                let can_copy_chat = ai_message_actions_can_copy_chat_transcript(
                    self.current_messages.len(),
                    self.is_streaming,
                );
                let is_copied = self.is_showing_chat_transcript_copied_feedback();
                let (icon, label, icon_color) = if is_copied {
                    (
                        LocalIconName::Check,
                        "Copied!",
                        cx.theme().success.opacity(OPACITY_STRONG),
                    )
                } else {
                    (
                        LocalIconName::Copy,
                        "Copy chat",
                        if can_copy_chat {
                            muted_fg.opacity(OPACITY_SELECTED)
                        } else {
                            muted_fg.opacity(OPACITY_HOVER)
                        },
                    )
                };

                div()
                    .id("copy-chat-btn")
                    .flex()
                    .items_center()
                    .gap(S1)
                    .px(S2)
                    .py(S1)
                    .rounded(R_SM)
                    .when(can_copy_chat, |d| {
                        d.cursor_pointer()
                            .hover(|s| {
                                s.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                    .text_color(muted_fg)
                            })
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.copy_chat_transcript(cx);
                            }))
                    })
                    .text_xs()
                    .text_color(if is_copied {
                        cx.theme().success.opacity(OPACITY_STRONG)
                    } else if can_copy_chat {
                        muted_fg.opacity(OPACITY_TEXT_MUTED)
                    } else {
                        muted_fg.opacity(OPACITY_BORDER)
                    })
                    .child(
                        svg()
                            .external_path(icon.external_path())
                            .size(ICON_XS)
                            .text_color(icon_color),
                    )
                    .child(label)
            })
            // "Generated in Xs" completion feedback (fades after 5 seconds)
            .when_some(completion_label, |el, label| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(S1)
                        .text_xs()
                        .text_color(cx.theme().success.opacity(OPACITY_STRONG))
                        .child(
                            svg()
                                .external_path(LocalIconName::Check.external_path())
                                .size(ICON_XS)
                                .text_color(cx.theme().success.opacity(OPACITY_SELECTED)),
                        )
                        .child(format!("Generated in {}", label)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::ai_message_actions_can_copy_chat_transcript;

    #[test]
    fn test_ai_message_actions_can_copy_chat_transcript_requires_messages() {
        assert!(
            !ai_message_actions_can_copy_chat_transcript(0, false),
            "Copy chat should stay disabled when there are no messages"
        );
        assert!(
            ai_message_actions_can_copy_chat_transcript(1, false),
            "Copy chat should be enabled with at least one message and no stream"
        );
    }

    #[test]
    fn test_ai_message_actions_can_copy_chat_transcript_disables_during_streaming() {
        assert!(
            !ai_message_actions_can_copy_chat_transcript(3, true),
            "Copy chat should stay disabled while streaming"
        );
    }
}
