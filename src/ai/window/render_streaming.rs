use super::*;

impl AiApp {
    pub(super) fn render_streaming_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::PromptColors::from_theme(&crate::theme::get_cached_theme());
        let streaming_bg = cx.theme().muted.opacity(OP_ASSISTANT_MSG_BG);

        let elapsed_label: SharedString = self
            .streaming_started_at
            .map(|started| {
                let secs = started.elapsed().as_secs();
                if secs < 1 {
                    String::new()
                } else {
                    format!("{}s", secs)
                }
            })
            .unwrap_or_default()
            .into();
        let show_elapsed = !elapsed_label.is_empty();

        let content_element = if self.streaming_content.is_empty() {
            // "Thinking" state with model name and elapsed time
            let thinking_label: SharedString = self
                .selected_model
                .as_ref()
                .map(|m| format!("Thinking with {}", m.display_name))
                .unwrap_or_else(|| "Thinking".to_string())
                .into();
            div()
                .flex()
                .items_center()
                .gap(SP_3)
                .py_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(thinking_label),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(SP_1)
                        .child(
                            div()
                                .size(SP_2)
                                .rounded_full()
                                .bg(cx.theme().accent.opacity(OP_NEAR_FULL)),
                        )
                        .child(
                            div()
                                .size(SP_2)
                                .rounded_full()
                                .bg(cx.theme().accent.opacity(OP_MEDIUM)),
                        )
                        .child(
                            div()
                                .size(SP_2)
                                .rounded_full()
                                .bg(cx.theme().accent.opacity(OP_MUTED)),
                        ),
                )
                .when(show_elapsed, |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground.opacity(OP_MEDIUM))
                            .child(elapsed_label.clone()),
                    )
                })
                .into_any_element()
        } else {
            // Render markdown separately from cursor to avoid invalidating
            // the markdown cache on every frame during streaming
            div()
                .w_full()
                .min_w_0()
                .overflow_x_hidden()
                .child(render_markdown(&self.streaming_content, &colors))
                .child(div().text_sm().text_color(cx.theme().accent).child("▌"))
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .mb_3()
            .child({
                // Model name for display in streaming header
                let model_label: Option<SharedString> = self
                    .selected_model
                    .as_ref()
                    .map(|m| SharedString::from(m.display_name.clone()));

                // Role label matching render_message style
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
                                    .external_path(LocalIconName::MessageCircle.external_path())
                                    .size(ICON_SM)
                                    .text_color(cx.theme().muted_foreground.opacity(OP_STRONG)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().muted_foreground.opacity(OP_NEAR_FULL))
                                    .child("Assistant"),
                            )
                            .child(
                                div()
                                    .size(DOT_SIZE)
                                    .rounded_full()
                                    .bg(cx.theme().muted_foreground.opacity(OP_MUTED)),
                            )
                            .when_some(model_label, |d, label| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(OP_MEDIUM))
                                        .child(label),
                                )
                            })
                            .when(show_elapsed, |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(OP_MEDIUM))
                                        .child(elapsed_label),
                                )
                            })
                            // Words/sec indicator during active streaming
                            .when(!self.streaming_content.is_empty(), |d| {
                                let word_count = self.streaming_content.split_whitespace().count();
                                let wps = self
                                    .streaming_started_at
                                    .map(|started| {
                                        let secs = started.elapsed().as_secs_f64();
                                        if secs > 0.5 {
                                            format!("~{:.0} words/s", word_count as f64 / secs)
                                        } else {
                                            String::new()
                                        }
                                    })
                                    .unwrap_or_default();
                                if wps.is_empty() {
                                    d
                                } else {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .text_color(
                                                cx.theme().muted_foreground.opacity(OP_MUTED),
                                            )
                                            .child(wps),
                                    )
                                }
                            }),
                    )
                    // Escape hint to stop streaming
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(SP_2)
                            .child(
                                div()
                                    .px(SP_2)
                                    .py(SP_1)
                                    .rounded(RADIUS_XS)
                                    .bg(cx.theme().muted.opacity(OP_MUTED))
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OP_MEDIUM))
                                    .child("Esc"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OP_MUTED))
                                    .child("to stop"),
                            ),
                    )
            })
            .child(
                div()
                    .w_full()
                    .px(MSG_PX)
                    .py(MSG_PY)
                    .rounded(MSG_RADIUS)
                    .bg(streaming_bg)
                    .child(content_element),
            )
    }

    /// Render a streaming error row with a retry button and contextual help.
    pub(super) fn render_streaming_error(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let err_msg = self
            .streaming_error
            .clone()
            .unwrap_or_else(|| "Unknown error".to_string());
        let danger = cx.theme().danger;

        // Classify error and provide contextual help
        let err_lower = err_msg.to_lowercase();
        let help_hint: Option<&str> = if err_lower.contains("401")
            || err_lower.contains("unauthorized")
            || err_lower.contains("invalid api key")
            || err_lower.contains("authentication")
        {
            Some("Check your API key in settings")
        } else if err_lower.contains("403") || err_lower.contains("forbidden") {
            Some("Your API key may lack required permissions")
        } else if err_lower.contains("429")
            || err_lower.contains("rate limit")
            || err_lower.contains("too many requests")
        {
            Some("Rate limited \u{2014} wait a moment and retry")
        } else if err_lower.contains("timeout")
            || err_lower.contains("timed out")
            || err_lower.contains("deadline")
        {
            Some("Request timed out \u{2014} check your connection")
        } else if err_lower.contains("network")
            || err_lower.contains("connection")
            || err_lower.contains("dns")
            || err_lower.contains("resolve")
        {
            Some("Network error \u{2014} check your internet connection")
        } else if err_lower.contains("500")
            || err_lower.contains("502")
            || err_lower.contains("503")
            || err_lower.contains("server error")
        {
            Some("Server error \u{2014} the provider may be experiencing issues")
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .gap_1()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(danger.opacity(0.1))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        svg()
                            .external_path(LocalIconName::Warning.external_path())
                            .size_4()
                            .text_color(danger),
                    )
                    .child(div().flex_1().text_sm().text_color(danger).child(err_msg))
                    .child(
                        div()
                            .id("retry-btn")
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(danger.opacity(0.2))
                            .text_sm()
                            .text_color(danger)
                            .cursor_pointer()
                            .hover(|s| s.bg(danger.opacity(0.3)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.retry_after_error(window, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Refresh.external_path())
                                    .size(px(12.))
                                    .text_color(danger),
                            )
                            .child("Retry"),
                    ),
            )
            .when_some(help_hint, |el, hint| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground.opacity(0.7))
                        .pl(px(24.))
                        .child(hint),
                )
            })
    }

    /// Render the editing indicator bar above the input.
    pub(super) fn render_editing_indicator(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = cx.theme().accent;
        let muted_fg = cx.theme().muted_foreground;
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_1()
            .bg(accent.opacity(0.1))
            .rounded_t_md()
            .child(
                svg()
                    .external_path(LocalIconName::Pencil.external_path())
                    .size_3()
                    .text_color(accent),
            )
            .child(div().text_xs().text_color(accent).child("Editing message"))
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(muted_fg)
                    .child("Esc to cancel  \u{00b7}  Enter to save"),
            )
    }
}
