use super::*;

impl AiApp {
    pub(super) fn render_sidebar_group_header(
        &self,
        group: DateGroup,
        is_first_group: bool,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let group_label: SharedString = group.label().to_uppercase().into();
        div()
            .flex()
            .w_full()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(cx.theme().muted_foreground.opacity(0.6))
            .px_1()
            .pt(px(if is_first_group { 4. } else { 12. }))
            .pb(px(6.))
            .child(group_label)
    }

    /// Render a single chat item with title, relative time, and hover-revealed delete button
    pub(super) fn render_chat_item(
        &self,
        chat: &Chat,
        selected_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let chat_id = chat.id;
        let is_selected = selected_id == Some(chat_id);

        let title: SharedString = if chat.title.is_empty() {
            "New Chat".into()
        } else {
            chat.title.clone().into()
        };

        let preview = self.message_previews.get(&chat_id).cloned();
        let msg_count = self.message_counts.get(&chat_id).copied().unwrap_or(0);
        let is_external_source = matches!(chat.source, ChatSource::ChatPrompt | ChatSource::Script);

        // Derive short model label from model_id (e.g., "claude-3-5-sonnet..." → "Sonnet")
        let model_badge: Option<SharedString> = if !chat.model_id.is_empty() {
            let short = Self::abbreviate_model_name(&chat.model_id);
            Some(short.into())
        } else {
            None
        };

        // Relative time for this chat
        let relative_time: SharedString = {
            let now = Utc::now();
            let diff = now - chat.updated_at;
            if diff.num_minutes() < 1 {
                "now".into()
            } else if diff.num_minutes() < 60 {
                format!("{}m", diff.num_minutes()).into()
            } else if diff.num_hours() < 24 {
                format!("{}h", diff.num_hours()).into()
            } else if diff.num_days() < 7 {
                format!("{}d", diff.num_days()).into()
            } else {
                chat.updated_at.format("%b %d").to_string().into()
            }
        };
        let full_chat_time: SharedString = chat
            .updated_at
            .format("%B %-d, %Y at %-I:%M %p")
            .to_string()
            .into();

        let selected_bg = cx.theme().muted.opacity(0.7);
        let hover_bg = cx.theme().muted.opacity(0.5);

        let title_color = if is_selected {
            cx.theme().foreground
        } else {
            cx.theme().sidebar_foreground
        };
        let description_color = if is_selected {
            cx.theme().sidebar_foreground
        } else {
            cx.theme().muted_foreground
        };

        let muted_fg = cx.theme().muted_foreground;
        let is_renaming = self.renaming_chat_id == Some(chat_id);
        div()
            .id(SharedString::from(format!("chat-{}", chat_id)))
            .group("chat-item")
            .flex()
            .flex_col()
            .w_full()
            .px_2()
            .py(px(6.))
            .rounded(px(8.))
            .cursor_pointer()
            .when(is_selected, |d| d.bg(selected_bg))
            .when(!is_selected, |d| d.hover(|d| d.bg(hover_bg)))
            .on_click(
                cx.listener(move |this, event: &gpui::ClickEvent, window, cx| {
                    if event.click_count() == 2 {
                        this.start_rename(chat_id, window, cx);
                    } else {
                        this.select_chat(chat_id, window, cx);
                    }
                }),
            )
            .child(
                // Title row with relative time and hover-revealed delete
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .gap(px(4.))
                    .when(is_renaming, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(self.rename_input_state.clone()),
                        )
                    })
                    .when(!is_renaming, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(title_color)
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(title),
                        )
                    })
                    // Relative time - hidden on hover to make room for delete
                    .child({
                        let tooltip_text = full_chat_time.clone();
                        div()
                            .id(SharedString::from(format!("time-{}", chat_id)))
                            .flex_shrink_0()
                            .text_xs()
                            .text_color(description_color.opacity(0.6))
                            .group_hover("chat-item", |s| s.opacity(0.))
                            .tooltip(move |window, cx| {
                                Tooltip::new(tooltip_text.clone()).build(window, cx)
                            })
                            .child(relative_time)
                    })
                    // Delete button - visible on hover only.
                    // Two-step: first click shows "Delete?", second click confirms.
                    .child({
                        let is_confirming = self.pending_delete_chat_id == Some(chat_id);
                        if is_confirming {
                            // Confirmation state: show red "Delete?" label
                            div()
                                .id(SharedString::from(format!("del-{}", chat_id)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .px(px(4.))
                                .py(px(1.))
                                .rounded(px(4.))
                                .flex_shrink_0()
                                .cursor_pointer()
                                .bg(cx.theme().danger.opacity(0.15))
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .hover(|s| s.bg(cx.theme().danger.opacity(0.25)))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        this.pending_delete_chat_id = None;
                                        this.delete_chat_by_id(chat_id, cx);
                                    }),
                                )
                                .child("Delete?")
                        } else {
                            div()
                                .id(SharedString::from(format!("del-{}", chat_id)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(18.))
                                .rounded(px(4.))
                                .flex_shrink_0()
                                .cursor_pointer()
                                .opacity(0.)
                                .group_hover("chat-item", |s| s.opacity(1.0))
                                .hover(|s| s.bg(cx.theme().danger.opacity(0.19)))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        this.pending_delete_chat_id = Some(chat_id);
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    svg()
                                        .external_path(LocalIconName::Trash.external_path())
                                        .size(px(12.))
                                        .text_color(muted_fg.opacity(0.5)),
                                )
                        }
                    }),
            )
            .when_some(preview, |d, preview_text| {
                let clean_preview: String = preview_text
                    .lines()
                    .map(|line| line.trim())
                    .find(|line| {
                        !line.is_empty()
                            && !line.starts_with('#')
                            && !line.starts_with("```")
                            && !line.chars().all(|c| c == '-' || c == '*' || c == '_')
                    })
                    .unwrap_or("")
                    .chars()
                    .take(50)
                    .collect();

                d.child(
                    div()
                        .text_xs()
                        .text_color(description_color)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(clean_preview),
                )
            })
            // Bottom row: model badge + message count + source indicator
            .when(
                model_badge.is_some() || msg_count > 0 || is_external_source,
                |d| {
                    d.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .when(is_external_source, |d| {
                                let source_label: &str = match chat.source {
                                    ChatSource::ChatPrompt => "Prompt",
                                    ChatSource::Script => "Script",
                                    _ => "",
                                };
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(2.))
                                        .child(
                                            svg()
                                                .external_path(
                                                    LocalIconName::Terminal.external_path(),
                                                )
                                                .size(px(10.))
                                                .text_color(cx.theme().accent.opacity(0.4)),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().accent.opacity(0.4))
                                                .child(source_label),
                                        ),
                                )
                            })
                            .when_some(model_badge, |d, badge| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(0.45))
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .child(badge),
                                )
                            })
                            .when(msg_count > 0, |d| {
                                let count_label = if msg_count == 1 {
                                    "1 msg".to_string()
                                } else {
                                    format!("{} msgs", msg_count)
                                };
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(0.35))
                                        .child(count_label),
                                )
                            }),
                    )
                },
            )
    }

    /// Abbreviate a model ID to a short display label for sidebar badges.
    /// e.g., "claude-3-5-sonnet-20241022" → "Sonnet"
    /// e.g., "gpt-4o-mini" → "GPT-4o Mini"
    /// e.g., "claude-3-5-haiku-20241022" → "Haiku"
    pub(super) fn abbreviate_model_name(model_id: &str) -> String {
        let lower = model_id.to_lowercase();
        if lower.contains("sonnet") {
            "Sonnet".to_string()
        } else if lower.contains("haiku") {
            "Haiku".to_string()
        } else if lower.contains("opus") {
            "Opus".to_string()
        } else if lower.contains("gpt-4o-mini") {
            "GPT-4o Mini".to_string()
        } else if lower.contains("gpt-4o") {
            "GPT-4o".to_string()
        } else if lower.contains("gpt-4") {
            "GPT-4".to_string()
        } else if lower.contains("gpt-3") {
            "GPT-3.5".to_string()
        } else if lower.contains("o1") || lower.contains("o3") {
            // OpenAI reasoning models
            let parts: Vec<&str> = model_id.split('-').collect();
            parts.first().unwrap_or(&model_id).to_uppercase()
        } else {
            // Fallback: take the most descriptive part
            let parts: Vec<&str> = model_id.split('-').collect();
            if parts.len() > 1 {
                // Skip version-like parts (dates, numbers)
                parts
                    .iter()
                    .find(|p| p.len() > 2 && !p.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(&parts[0])
                    .to_string()
            } else {
                model_id.to_string()
            }
        }
    }
}
