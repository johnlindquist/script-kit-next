use super::*;

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
            .gap(SP_3)
            .pl_1()
            .mt(SP_2)
            .mb(SP_3)
            .child(
                div()
                    .id("regenerate-btn")
                    .flex()
                    .items_center()
                    .gap(SP_2)
                    .px(SP_4)
                    .py(SP_2)
                    .rounded(RADIUS_MD)
                    .cursor_pointer()
                    .text_xs()
                    .text_color(muted_fg.opacity(0.65))
                    .hover(|s| s.bg(cx.theme().muted.opacity(0.3)).text_color(muted_fg))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.regenerate_response(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(LocalIconName::Refresh.external_path())
                            .size(ICON_XS)
                            .text_color(muted_fg.opacity(0.55)),
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
                        cx.theme().success.opacity(0.7),
                    )
                } else {
                    (LocalIconName::Copy, "Copy", muted_fg.opacity(0.5))
                };
                div()
                    .id("copy-response-btn")
                    .flex()
                    .items_center()
                    .gap(SP_2)
                    .px(SP_4)
                    .py(SP_2)
                    .rounded(RADIUS_MD)
                    .cursor_pointer()
                    .text_xs()
                    .text_color(if is_copied {
                        cx.theme().success.opacity(0.7)
                    } else {
                        muted_fg.opacity(0.65)
                    })
                    .hover(|s| s.bg(cx.theme().muted.opacity(0.3)).text_color(muted_fg))
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
            // "Generated in Xs" completion feedback (fades after 5 seconds)
            .when_some(completion_label, |el, label| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(SP_2)
                        .text_xs()
                        .text_color(cx.theme().success.opacity(0.7))
                        .child(
                            svg()
                                .external_path(LocalIconName::Check.external_path())
                                .size(ICON_XS)
                                .text_color(cx.theme().success.opacity(0.5)),
                        )
                        .child(format!("Generated in {}", label)),
                )
            })
    }
}
